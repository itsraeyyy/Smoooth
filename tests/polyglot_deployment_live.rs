use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::{ContainerManager, ContainerSpec};
use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::redis::RedisAdapter;
use smoooth::engine::Orchestrator;
use smoooth::models::plan::{
    CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, Plan, PolicyConstraints, RiskTier,
};
use smoooth::models::OperationStatus;
use smoooth::security::SecretStore;
use smoooth::storage::LocalStorage;

#[tokio::test]
async fn test_live_docker_polyglot_redis_application_deployment() {
    println!("\n=== Starting Smoooth Milestone 6C Live Polyglot (Redis + App) Deployment Test ===");

    let client = match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => c,
            Err(e) => {
                if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                    panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but Docker ping failed: {e}");
                }
                println!(
                    "Docker daemon not reachable: {}. Skipping live integration test.",
                    e
                );
                return;
            }
        },
        Err(e) => {
            if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but DockerClient::try_new() failed: {e}");
            }
            println!(
                "Docker client initialization failed: {}. Skipping live integration test.",
                e
            );
            return;
        }
    };

    let redis_name = "smoooth_it_poly_redis";
    let app_name = "smoooth_it_poly_app";
    let redis_vol = "smoooth_it_poly_redis_vol";

    // 0. Pre-cleanup any stale disposable test resources
    let _ = ContainerManager::stop_and_remove(&client, redis_name).await;
    let _ = ContainerManager::stop_and_remove(&client, app_name).await;
    let _ = client.inner().remove_volume(redis_vol, None).await;

    let storage = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());
    let mut orchestrator =
        Orchestrator::new(storage.clone()).with_secret_store(secret_store.clone());

    let docker_adapter = Arc::new(DockerAdapter::with_client(client.clone()));
    let redis_adapter = Arc::new(RedisAdapter::new(docker_adapter.clone()));

    orchestrator.register_adapter(docker_adapter.clone());
    orchestrator.register_adapter(redis_adapter.clone());

    // -------------------------------------------------------------------------
    // Phase 1 & 2: Plan with Redis Cache Dependency & Out-of-Order Declaration
    // -------------------------------------------------------------------------
    println!("\n[Phase 1] Proposing plan with Redis Cache and Polyglot Web Application...");

    // Python script acting as microservice:
    // Connects to Redis via raw TCP socket using RESP protocol, authenticates, writes counter, and serves HTTP health check
    let app_python_script = r#"
import socket
import sys
import os
import time

host = os.environ.get("REDIS_HOST", "127.0.0.1")
port = int(os.environ.get("REDIS_PORT", "6379"))
password = os.environ.get("REDIS_PASSWORD", "")

print(f"Connecting to Redis at {host}:{port}...", flush=True)

# Connect to Redis
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(10)
s.connect((host, port))

# Authenticate if password provided
if password:
    auth_cmd = f"*2\r\n$4\r\nAUTH\r\n${len(password)}\r\n{password}\r\n".encode()
    s.sendall(auth_cmd)
    resp = s.recv(1024).decode()
    if not resp.startswith("+OK"):
        print(f"Redis AUTH failed: {resp}", flush=True)
        sys.exit(1)

# Write initial counter or increment
incr_cmd = b"*2\r\n$4\r\nINCR\r\n$7\r\ncounter\r\n"
s.sendall(incr_cmd)
val_resp = s.recv(1024).decode()
print(f"Redis INCR response: {val_resp.strip()}", flush=True)

s.close()

# Start HTTP Health Server on port 8080
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(('0.0.0.0', 8080))
server.listen(5)
print("HTTP health server listening on port 8080...", flush=True)

while True:
    client_sock, _ = server.accept()
    req = client_sock.recv(1024)
    body = '{"status":"healthy","service":"polyglot_app","redis_status":"verified"}'
    response = f"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {len(body)}\r\n\r\n{body}"
    client_sock.sendall(response.encode())
    client_sock.close()
"#;

    let plan = Plan::new(
        "plan-polyglot-redis".to_string(),
        CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: "ws-polyglot".to_string(),
            environment: "development".to_string(),
            target_id: "node-vps".to_string(),
            actions: vec![
                // Intentionally list dependent APP first (index 0) to verify DAG reordering!
                CanonicalActionSpec {
                    resource_name: app_name.to_string(),
                    provider: "docker".to_string(),
                    service_type: "web".to_string(),
                    action_type: "create".to_string(),
                    config: serde_json::json!({
                        "image": "python:3.11-alpine",
                        "cmd": ["python", "-c", app_python_script],
                        "network": "smoooth_poly_net",
                        "ports": { "8080": 0 }, // Ephemeral port
                        "bindings": [
                            {
                                "resource": redis_name,
                                "env_prefix": "REDIS",
                                "preferred_endpoint": "internal"
                            }
                        ],
                        "env": {
                            "APP_ENV": "polyglot_demo"
                        }
                    }),
                    is_persistent: false,
                    exposes_network_publicly: false,
                },
                // Prerequisite REDIS declared second (index 1)
                CanonicalActionSpec {
                    resource_name: redis_name.to_string(),
                    provider: "redis".to_string(),
                    service_type: "cache".to_string(),
                    action_type: "create".to_string(),
                    config: serde_json::json!({
                        "network": "smoooth_poly_net",
                        "port": 0, // Ephemeral host port
                        "password": "smoooth_secure_redis_pwd_2026!",
                        "volume_name": redis_vol,
                        "workspace_id": "ws-polyglot"
                    }),
                    is_persistent: true,
                    exposes_network_publicly: false,
                },
            ],
            dependencies: vec![DependencyEdge {
                from: app_name.to_string(),
                to: redis_name.to_string(),
            }],
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: false,
                allow_data_destruction: false,
                max_monthly_budget_usd: None,
            },
        },
        RiskTier::Low,
        None,
    );

    println!("[Phase 1] Executing plan via Orchestrator DAG engine...");
    let op = orchestrator
        .execute_plan_sync(plan, "idemp-polyglot-1".to_string())
        .await
        .expect("Plan execution must succeed");

    if op.status != OperationStatus::Succeeded {
        eprintln!(">>> EXECUTION FAILED WITH ERROR: {:?}", op.error_message);
    }
    assert_eq!(
        op.status,
        OperationStatus::Succeeded,
        "Plan execution should succeed: {:?}",
        op.error_message
    );
    println!(">>> Plan succeeded! Verifying Redis and Application live containers...");

    // -------------------------------------------------------------------------
    // Phase 2: Active Verification of Deployed Services
    // -------------------------------------------------------------------------
    let redis_inspect = client
        .inner()
        .inspect_container(redis_name, None)
        .await
        .expect("Inspect redis container");
    let is_redis_running = redis_inspect.state.and_then(|s| s.running).unwrap_or(false);
    assert!(is_redis_running, "Redis container must be running");

    let app_inspect = client
        .inner()
        .inspect_container(app_name, None)
        .await
        .expect("Inspect app container");
    let is_app_running = app_inspect.state.and_then(|s| s.running).unwrap_or(false);
    assert!(is_app_running, "Application container must be running");

    // Verify injected bindings inside application container
    let app_env = app_inspect.config.and_then(|c| c.env).unwrap_or_default();
    assert!(
        app_env
            .iter()
            .any(|e| e == &format!("REDIS_HOST={}", redis_name)),
        "Must have REDIS_HOST pointing to redis resource"
    );
    assert!(
        app_env.iter().any(|e| e == "REDIS_PORT=6379"),
        "Must have internal REDIS_PORT 6379"
    );
    assert!(
        app_env
            .iter()
            .any(|e| e == "REDIS_PASSWORD=smoooth_secure_redis_pwd_2026!"),
        "Must have REDIS_PASSWORD"
    );
    println!(">>> Injected bindings verified in running application container!");

    // Verify application's HTTP health server is responding
    let app_port = ContainerManager::get_host_port(&client, app_name, 8080)
        .await
        .expect("Failed to get app host port");
    println!("App listening on host loopback port: {}", app_port);

    sleep(Duration::from_millis(500)).await;

    // Test health endpoint using raw TCP HTTP request
    let mut http_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", app_port))
        .await
        .expect("Should connect to app HTTP port");
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    http_stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut http_buf = [0u8; 1024];
    let n = http_stream.read(&mut http_buf).await.unwrap();
    let http_resp = String::from_utf8_lossy(&http_buf[..n]);
    assert!(
        http_resp.contains("200 OK"),
        "App health endpoint must return 200 OK"
    );
    assert!(
        http_resp.contains("\"redis_status\":\"verified\""),
        "App response must confirm Redis interaction"
    );
    println!(
        ">>> App HTTP probe verified: {}",
        http_resp.lines().next().unwrap_or("")
    );

    // -------------------------------------------------------------------------
    // Phase 3: Workload Replacement Preserving Persistent Data
    // -------------------------------------------------------------------------
    println!("\n[Phase 3] Testing safe container replacement with updated configuration...");
    let updated_script = r#"
import socket
import os
import sys

host = os.environ.get("REDIS_HOST", "smoooth_it_poly_redis")
port = int(os.environ.get("REDIS_PORT", "6379"))
password = os.environ.get("REDIS_PASSWORD", "")

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(10)
s.connect((host, port))
if password:
    auth_cmd = f"*2\r\n$4\r\nAUTH\r\n${len(password)}\r\n{password}\r\n".encode()
    s.sendall(auth_cmd)
    s.recv(1024)

# Increment counter again: must be 2!
incr_cmd = b"*2\r\n$4\r\nINCR\r\n$7\r\ncounter\r\n"
s.sendall(incr_cmd)
val = s.recv(1024).decode()
print(f"Counter incremented on replacement: {val.strip()}", flush=True)

if not ":2" in val:
    print(f"FATAL: counter was not preserved! Got {val}", flush=True)
    sys.exit(1)

s.close()
sys.exit(0)
"#;

    let replacement_spec = ContainerSpec {
        name: app_name.to_string(),
        image: "python:3.11-alpine".to_string(),
        cmd: Some(vec![
            "python".to_string(),
            "-c".to_string(),
            updated_script.to_string(),
        ]),
        env: vec![
            "APP_ENV=polyglot_demo_v2".to_string(), // Drift triggers replacement
            format!("REDIS_HOST={}", redis_name),
            "REDIS_PORT=6379".to_string(),
            "REDIS_PASSWORD=smoooth_secure_redis_pwd_2026!".to_string(),
        ],
        port_bindings: HashMap::new(),
        volume_mounts: vec![],
        network_name: Some("smoooth_poly_net".to_string()),
        restart_policy: Some("no".to_string()),
        labels: {
            let mut l = HashMap::new();
            l.insert("smoooth.fencing_epoch".to_string(), "2".to_string());
            l
        },
        memory_bytes: Some(64 * 1024 * 1024),
        cpu_shares: Some(256),
        nano_cpus: None,
        health_spec: None,
    };

    let repl_id = ContainerManager::create_and_start(&client, &replacement_spec)
        .await
        .expect("Safe replacement must succeed");
    println!(
        ">>> Replacement container executed successfully (ID: {})",
        &repl_id[..12]
    );
    sleep(Duration::from_millis(1500)).await;

    let repl_inspect = client
        .inner()
        .inspect_container(&repl_id, None)
        .await
        .expect("inspect replacement container");
    let exit_code = repl_inspect.state.and_then(|s| s.exit_code).unwrap_or(-1);
    assert_eq!(
        exit_code, 0,
        "Replacement container should have exited with code 0 (confirming counter reached 2)"
    );
    println!(">>> Persistent counter 2 verified after container replacement!");

    // -------------------------------------------------------------------------
    // Phase 4: Replacement Failure & Automatic Rollback Guarantee
    // -------------------------------------------------------------------------
    println!("\n[Phase 4] Testing replacement failure handling with automatic rollback...");
    // Let's create a known-working container first
    let stable_name = "smoooth_it_rollback_test";
    let _ = ContainerManager::stop_and_remove(&client, stable_name).await;

    let stable_spec = ContainerSpec {
        name: stable_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "echo 'stable_v1'; sleep 3600".to_string(),
        ]),
        env: vec!["STAGE=stable_v1".to_string()],
        port_bindings: HashMap::new(),
        volume_mounts: vec![],
        network_name: None,
        restart_policy: Some("no".to_string()),
        labels: HashMap::new(),
        memory_bytes: Some(32 * 1024 * 1024),
        cpu_shares: Some(128),
        nano_cpus: None,
        health_spec: None,
    };

    let original_id = ContainerManager::create_and_start(&client, &stable_spec)
        .await
        .expect("Stable container start failed");
    println!(
        "Original stable container started (ID: {})",
        &original_id[..12]
    );

    // Now attempt a replacement with an invalid image that fails creation
    let failing_replacement_spec = ContainerSpec {
        name: stable_name.to_string(),
        image: "this-image-definitely-does-not-exist-smoooth-fail-test:9999".to_string(),
        cmd: Some(vec!["echo".to_string(), "fail".to_string()]),
        env: vec!["STAGE=failing_v2".to_string()], // Environment drift
        port_bindings: HashMap::new(),
        volume_mounts: vec![],
        network_name: None,
        restart_policy: Some("no".to_string()),
        labels: HashMap::new(),
        memory_bytes: Some(32 * 1024 * 1024),
        cpu_shares: Some(128),
        nano_cpus: None,
        health_spec: None,
    };

    let fail_result = ContainerManager::create_and_start(&client, &failing_replacement_spec).await;
    assert!(
        fail_result.is_err(),
        "Replacement with nonexistent image must fail"
    );
    let err_msg = fail_result.err().unwrap().to_string();
    assert!(
        err_msg.contains("automatically rolled back"),
        "Error message must confirm automatic rollback: {}",
        err_msg
    );
    println!(
        ">>> Verified error confirmed automatic rollback: {}",
        err_msg
    );

    // Verify original stable container is still present, running, and restored!
    let restored_inspect = client
        .inner()
        .inspect_container(stable_name, None)
        .await
        .expect("Original container must exist after rollback");
    let is_restored_running = restored_inspect
        .state
        .and_then(|s| s.running)
        .unwrap_or(false);
    assert!(
        is_restored_running,
        "Rolled-back container must be running!"
    );
    let restored_env = restored_inspect
        .config
        .and_then(|c| c.env)
        .unwrap_or_default();
    assert!(
        restored_env.iter().any(|e| e == "STAGE=stable_v1"),
        "Restored container must retain original env"
    );
    println!(">>> Automatic rollback verified: original container is running and healthy!");

    // -------------------------------------------------------------------------
    // Teardown
    // -------------------------------------------------------------------------
    println!("\nCleaning up disposable test resources...");
    let _ = ContainerManager::stop_and_remove(&client, redis_name).await;
    let _ = ContainerManager::stop_and_remove(&client, app_name).await;
    let _ = ContainerManager::stop_and_remove(&client, stable_name).await;
    let _ = client.inner().remove_volume(redis_vol, None).await;

    println!("\n=== Milestone 6C Live Polyglot Deployment Test Passed 100% ===");
}
