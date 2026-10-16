use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::{ContainerManager, ContainerSpec};
use smoooth::adapters::docker::DockerAdapter;
use smoooth::engine::Orchestrator;
use smoooth::models::connection::VolumeMount;
use smoooth::models::plan::{CanonicalActionSpec, CanonicalExecutionSpec, Plan, PolicyConstraints, RiskTier};
use smoooth::models::{OperationStatus, Resource, ResourceOwnership, ResourceStatus};
use smoooth::security::SecretStore;
use smoooth::storage::{LocalStorage, Storage};

#[tokio::test]
async fn test_live_docker_connectivity_and_dynamic_bindings() {
    println!("\n=== Starting Smoooth Milestone 6B Live Docker Connectivity & Dynamic Bindings Integration Test ===");

    let client = match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => c,
            Err(e) => {
                println!("Docker daemon not reachable: {}. Skipping live integration test.", e);
                return;
            }
        },
        Err(e) => {
            println!("Docker client initialization failed: {}. Skipping live integration test.", e);
            return;
        }
    };

    let dep_name = "smoooth_it_conn_dep";
    let consumer_name = "smoooth_it_conn_consumer";
    let repl_name = "smoooth_it_conn_repl";
    let test_vol = "smoooth_it_conn_volume";

    // 0. Pre-cleanup any stale disposable test resources
    let _ = ContainerManager::stop_and_remove(&client, dep_name).await;
    let _ = ContainerManager::stop_and_remove(&client, consumer_name).await;
    let _ = ContainerManager::stop_and_remove(&client, repl_name).await;
    let _ = client.inner().remove_volume(test_vol, None).await;

    let storage = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());
    let mut orchestrator = Orchestrator::new(storage.clone()).with_secret_store(secret_store.clone());
    let docker_adapter = Arc::new(DockerAdapter::with_client(client.clone()));
    orchestrator.register_adapter(docker_adapter.clone());

    // -------------------------------------------------------------------------
    // Scenario 1: Ephemeral Port Allocation (Host Port 0 -> Dynamic OS Port)
    // -------------------------------------------------------------------------
    println!("\n[Scenario 1] Testing dynamic ephemeral port allocation (port 0)...");
    let netcat_http = "while true; do { echo -e 'HTTP/1.1 200 OK\\r\\nContent-Type: text/plain\\r\\nContent-Length: 6\\r\\n\\r\\nONLINE'; } | nc -l -p 8080; done";

    let mut port_bindings = HashMap::new();
    port_bindings.insert(8080u16, 0u16); // Request ephemeral port

    let mut dep_labels = HashMap::new();
    dep_labels.insert("smoooth.workspace".to_string(), "ws-conn-test".to_string());
    dep_labels.insert("db_password".to_string(), "super_secret_dont_leak".to_string());
    dep_labels.insert("api_token".to_string(), "bearer_token_xyz".to_string());

    let dep_spec = ContainerSpec {
        name: dep_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), netcat_http.to_string()]),
        env: vec!["SERVICE_ROLE=backend".to_string()],
        port_bindings,
        volume_mounts: vec![],
        network_name: None,
        restart_policy: Some("no".to_string()),
        labels: dep_labels,
        memory_bytes: Some(64 * 1024 * 1024),
        cpu_shares: Some(256),
        nano_cpus: None,
        health_spec: None,
    };

    let dep_id = ContainerManager::create_and_start(&client, &dep_spec).await
        .expect("Failed to create and start dependency container");

    sleep(Duration::from_millis(500)).await;

    // Resolve dynamically assigned host port
    let dynamic_port = ContainerManager::get_host_port(&client, dep_name, 8080).await
        .expect("Failed to resolve dynamic host port");
    assert!(dynamic_port > 0, "Dynamic port must be allocated (> 0)");
    println!("Dynamic host port allocated by Docker: {}", dynamic_port);

    // Verify TCP connectivity to the dynamically allocated port on 127.0.0.1
    let probe_conn = TcpStream::connect(format!("127.0.0.1:{}", dynamic_port)).await;
    assert!(probe_conn.is_ok(), "Should successfully connect to dynamic loopback port {}", dynamic_port);
    println!("Verified TCP connectivity to dynamic port {}", dynamic_port);

    // Verify label scrubbing on Docker daemon: secret and token labels MUST NOT exist on container
    let inspected = client.inner().inspect_container(dep_name, None).await.expect("Inspect dep");
    if let Some(config) = inspected.config {
        if let Some(labels) = config.labels {
            assert!(!labels.contains_key("db_password"), "db_password MUST be scrubbed from Docker labels");
            assert!(!labels.contains_key("api_token"), "api_token MUST be scrubbed from Docker labels");
            assert_eq!(labels.get("smoooth.workspace").unwrap(), "ws-conn-test");
            println!("Verified zero-credential label scrubbing on Docker daemon");
        }
    }

    // -------------------------------------------------------------------------
    // Scenario 2: Dynamic Binding Injection & Inter-Service Dependency Resolution
    // -------------------------------------------------------------------------
    println!("\n[Scenario 2] Testing dynamic binding injection into consumer workload...");

    // Register dependency resource in orchestrator storage
    let dep_res = Resource {
        id: dep_id.clone(),
        name: dep_name.to_string(),
        service_type: "http_backend".to_string(),
        provider: "docker".to_string(),
        workspace_id: "ws-conn-test".to_string(),
        status: ResourceStatus::Active,
        ownership: ResourceOwnership::SmooothManaged,
        configuration: serde_json::json!({
            "protocol": "http",
            "host": "127.0.0.1",
            "port": dynamic_port,
            "ports": { "8080": dynamic_port }
        }),
        is_persistent: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.save_resource(dep_res).await.unwrap();

    // Consumer plan depending on dep_name and consuming its dynamic bindings
    let consumer_script = r#"
        echo "Validating injected bindings:"
        echo "BACKEND_HOST=$BACKEND_HOST"
        echo "BACKEND_PORT=$BACKEND_PORT"
        echo "BACKEND_URL=$BACKEND_URL"
        if [ -z "$BACKEND_PORT" ]; then
            echo "ERROR: BACKEND_PORT is missing"
            exit 1
        fi
        sleep 3600
    "#;

    let plan = Plan::new(
        "plan-conn-test".to_string(),
        CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: "ws-conn-test".to_string(),
            environment: "development".to_string(),
            target_id: "node-1".to_string(),
            actions: vec![
                CanonicalActionSpec {
                    resource_name: consumer_name.to_string(),
                    provider: "docker".to_string(),
                    service_type: "web".to_string(),
                    action_type: "create".to_string(),
                    config: serde_json::json!({
                        "image": "alpine:latest",
                        "cmd": ["/bin/sh", "-c", consumer_script],
                        "bindings": [
                            {
                                "resource": dep_name,
                                "env_prefix": "BACKEND",
                                "preferred_endpoint": "host"
                            }
                        ],
                        "env": {
                            "CONSUMER_MODE": "test"
                        }
                    }),
                    is_persistent: false,
                    exposes_network_publicly: false,
                }
            ],
            dependencies: vec![],
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

    let op = orchestrator.execute_plan_sync(plan, "idemp-conn-consumer".to_string()).await
        .expect("execute_plan_sync must succeed");

    assert_eq!(op.status, OperationStatus::Succeeded);
    println!("Consumer plan executed successfully. Verifying injected env vars inside container...");

    let consumer_inspect = client.inner().inspect_container(consumer_name, None).await.expect("Inspect consumer");
    let consumer_env = consumer_inspect.config.and_then(|c| c.env).unwrap_or_default();

    let has_host = consumer_env.iter().any(|e| e == "BACKEND_HOST=127.0.0.1");
    let has_port = consumer_env.iter().any(|e| e == &format!("BACKEND_PORT={}", dynamic_port));
    let has_url = consumer_env.iter().any(|e| e.starts_with("BACKEND_URL=http://"));
    let has_mode = consumer_env.iter().any(|e| e == "CONSUMER_MODE=test");

    assert!(has_host, "Consumer must have BACKEND_HOST=127.0.0.1");
    assert!(has_port, "Consumer must have dynamically injected BACKEND_PORT={}", dynamic_port);
    assert!(has_url, "Consumer must have BACKEND_URL");
    assert!(has_mode, "Consumer must preserve existing CONSUMER_MODE=test");
    println!("Verified dynamic env injection in live consumer container!");

    // -------------------------------------------------------------------------
    // Scenario 3: Safe Container Replacement on Environment Drift Preserving Named Volume
    // -------------------------------------------------------------------------
    println!("\n[Scenario 3] Testing safe container replacement preserving persistent named volumes...");

    // Create volume mount
    let v_mount = VolumeMount::new(test_vol.to_string(), "/data".to_string(), false);

    // Initial container: writes a persistent marker to the volume
    let init_script = "echo 'smoooth_milestone_6b_preserved' > /data/persistence_marker.txt; sleep 3600";
    let init_spec = ContainerSpec {
        name: repl_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), init_script.to_string()]),
        env: vec!["CONFIG_VERSION=v1".to_string()],
        port_bindings: HashMap::new(),
        volume_mounts: vec![v_mount.clone()],
        network_name: None,
        restart_policy: Some("no".to_string()),
        labels: HashMap::new(),
        memory_bytes: Some(64 * 1024 * 1024),
        cpu_shares: Some(256),
        nano_cpus: None,
        health_spec: None,
    };

    let first_id = ContainerManager::create_and_start(&client, &init_spec).await
        .expect("Initial container creation failed");
    println!("First container started (ID: {}). Writing persistent marker...", &first_id[..12]);
    sleep(Duration::from_secs(1)).await;

    // Updated container spec: environment changes to CONFIG_VERSION=v2, command verifies marker
    let verify_script = r#"
        if [ ! -f /data/persistence_marker.txt ]; then
            echo "FATAL: persistent marker missing!"
            exit 1
        fi
        CONTENT=$(cat /data/persistence_marker.txt)
        if [ "$CONTENT" != "smoooth_milestone_6b_preserved" ]; then
            echo "FATAL: unexpected content: $CONTENT"
            exit 2
        fi
        echo "SUCCESS: persistent marker verified: $CONTENT"
        sleep 3600
    "#;

    let updated_spec = ContainerSpec {
        name: repl_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), verify_script.to_string()]),
        env: vec!["CONFIG_VERSION=v2".to_string()], // Environment drift!
        port_bindings: HashMap::new(),
        volume_mounts: vec![v_mount],
        network_name: None,
        restart_policy: Some("no".to_string()),
        labels: HashMap::new(),
        memory_bytes: Some(64 * 1024 * 1024),
        cpu_shares: Some(256),
        nano_cpus: None,
        health_spec: None,
    };

    // ContainerManager::create_and_start detects drift, stops & removes old container, creates new one
    let second_id = ContainerManager::create_and_start(&client, &updated_spec).await
        .expect("Safe replacement container failed");
    println!("Replacement container started (ID: {}). Verifying container was recreated...", &second_id[..12]);
    assert_ne!(first_id, second_id, "Container must be cleanly replaced with a new instance");

    sleep(Duration::from_secs(1)).await;

    // Verify replacement container is running and healthy
    let repl_inspect = client.inner().inspect_container(repl_name, None).await.expect("Inspect replacement");
    let is_running = repl_inspect.state.and_then(|s| s.running).unwrap_or(false);
    assert!(is_running, "Replacement container must be running");

    let repl_env = repl_inspect.config.and_then(|c| c.env).unwrap_or_default();
    assert!(repl_env.iter().any(|e| e == "CONFIG_VERSION=v2"), "Replacement container must have CONFIG_VERSION=v2");
    println!("Verified persistent volume preservation across container replacement!");

    // -------------------------------------------------------------------------
    // Teardown: Clean up test containers and volume
    // -------------------------------------------------------------------------
    println!("\nCleaning up disposable test containers...");
    let _ = ContainerManager::stop_and_remove(&client, dep_name).await;
    let _ = ContainerManager::stop_and_remove(&client, consumer_name).await;
    let _ = ContainerManager::stop_and_remove(&client, repl_name).await;
    let _ = client.inner().remove_volume(test_vol, None).await;

    println!("\n=== Milestone 6B Live Docker Connectivity Integration Test Passed 100% ===");
}
