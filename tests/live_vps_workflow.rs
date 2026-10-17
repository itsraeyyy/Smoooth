use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::ContainerManager;
use smoooth::adapters::docker::networks::NetworkManager;
use smoooth::adapters::docker::volumes::VolumeManager;
use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::postgres::PostgresAdapter;
use smoooth::adapters::ProviderAdapter;
use smoooth::engine::Orchestrator;
use smoooth::models::{ResourceStatus, ServiceBinding};
use smoooth::security::SecretStore;
use smoooth::storage::{LocalStorage, Storage};

#[tokio::test]
async fn test_live_vps_docker_and_postgres_workflow_and_failure_drills() {
    println!("\n=== Starting Smoooth v2 Live VPS Docker & PostgreSQL Proof ===");

    // Verify Docker daemon connectivity
    let client = match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => c,
            Err(e) => {
                if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                    panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but Docker ping failed: {e}");
                }
                println!("Docker daemon unreachable on host: {}", e);
                return;
            }
        },
        Err(e) => {
            if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but DockerClient::try_new() failed: {e}");
            }
            println!("Docker client initialization failed: {}", e);
            return;
        }
    };

    let docker_version = client.version_info().await.unwrap();
    println!(">>> Connected to live Docker Engine: v{}", docker_version);

    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::with_client(client.clone()));
    let postgres_adapter = Arc::new(PostgresAdapter::new(docker_adapter.clone()));
    orchestrator.register_adapter(docker_adapter.clone());
    orchestrator.register_adapter(postgres_adapter.clone());

    let workspace_id = "ws-live-vps";
    let net_name = "smoooth_live_vps_net";
    let pg_vol_name = "smoooth_live_vps_pg_data";
    let pg_service_name = "smoooth_live_vps_pg";
    let app_service_name = "smoooth_live_vps_app";

    // Pre-cleanup in case of previous interrupted runs
    let _ = ContainerManager::stop_and_remove(&client, app_service_name).await;
    let _ = ContainerManager::stop_and_remove(&client, pg_service_name).await;
    let _ = VolumeManager::remove_volume(&client, pg_vol_name, true).await;

    // -------------------------------------------------------------------------
    // 1. Create an isolated application network
    // -------------------------------------------------------------------------
    println!(
        "\n[Step 1] Creating isolated application network: {}",
        net_name
    );
    let net_res = NetworkManager::ensure_network(&client, net_name, workspace_id).await;
    assert!(
        net_res.is_ok(),
        "Isolated application network creation must succeed"
    );
    println!(">>> Isolated network '{}' created/verified.", net_name);

    // -------------------------------------------------------------------------
    // 2. Create PostgreSQL container with persistent named volume (NO host port)
    // -------------------------------------------------------------------------
    println!(
        "\n[Step 2] Creating PostgreSQL container with persistent named volume: {}",
        pg_vol_name
    );
    let pg_config = serde_json::json!({
        "database": "vps_app_db",
        "user": "vps_app_user",
        "password": "vps_secret_password_777",
        "volume_name": pg_vol_name,
        "network": net_name,
        "workspace_id": workspace_id
        // NOTE: "port" omitted -> NO public host port binding!
    });

    let pg_res = postgres_adapter
        .provision(pg_service_name, &pg_config, true)
        .await;
    assert!(
        pg_res.is_ok(),
        "PostgreSQL provisioning must succeed: {:?}",
        pg_res.err()
    );
    let pg_resource = pg_res.unwrap();
    assert_eq!(pg_resource.status, ResourceStatus::Active);
    assert!(pg_resource.is_persistent, "Database must be persistent");
    storage.save_resource(pg_resource.clone()).await.unwrap();
    println!(
        ">>> PostgreSQL container '{}' provisioned and verified active.",
        pg_service_name
    );

    // -------------------------------------------------------------------------
    // 3. Confirm PostgreSQL has NO public host port binding
    // -------------------------------------------------------------------------
    println!("\n[Step 3] Confirming PostgreSQL has NO public host port binding...");
    let has_public_ports = ContainerManager::has_host_port_binding(&client, pg_service_name)
        .await
        .unwrap();
    assert!(
        !has_public_ports,
        "SECURITY VIOLATION: PostgreSQL must NOT have public host port bindings!"
    );
    println!(">>> Confirmed: PostgreSQL has 0 public host port bindings (Private container networking enforced).");

    // -------------------------------------------------------------------------
    // 4. Generate application-scoped credentials & ServiceBinding
    // -------------------------------------------------------------------------
    println!("\n[Step 4] Generating application-scoped credentials & encrypted ServiceBinding...");
    let secret_store = SecretStore::default_local();
    let pg_ip = docker_adapter
        .get_container_ip(pg_service_name)
        .await
        .unwrap();
    println!(">>> Discovered private PostgreSQL container IP: {}", pg_ip);

    let raw_conn = format!(
        "postgresql://vps_app_user:vps_secret_password_777@{}:5432/vps_app_db",
        pg_ip
    );
    let binding = ServiceBinding::new_with_secret(
        "bind_vps_pg_app".to_string(),
        app_service_name.to_string(),
        pg_resource.id.clone(),
        "postgres_connection".to_string(),
        raw_conn.as_bytes(),
        &secret_store,
        vec!["DATABASE_URL".to_string()],
    )
    .unwrap();

    // Verify secret is not in serialized JSON
    let serialized_binding = serde_json::to_string(&binding).unwrap();
    assert!(
        !serialized_binding.contains("vps_secret_password_777"),
        "Credentials must never leak into serialized binding"
    );
    println!(">>> Encrypted ServiceBinding created with zero secret leakage.");

    // -------------------------------------------------------------------------
    // 5 & 6. Connect using generated binding and execute authenticated SELECT 1
    // -------------------------------------------------------------------------
    println!("\n[Step 5 & 6] Connecting with authenticated credentials and executing SELECT 1...");
    let decrypted_bytes = binding.decrypt_credentials(&secret_store).unwrap();
    let decrypted_conn_str = String::from_utf8(decrypted_bytes).unwrap();

    let (pg_client, connection) =
        tokio_postgres::connect(&decrypted_conn_str, tokio_postgres::NoTls)
            .await
            .expect("Failed to connect to PostgreSQL using decrypted credentials");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    let row = pg_client
        .query_one("SELECT 1 as test_val", &[])
        .await
        .unwrap();
    let val: i32 = row.get("test_val");
    assert_eq!(val, 1, "SELECT 1 must return 1");
    println!(">>> Authenticated SELECT 1 verified successfully.");

    // -------------------------------------------------------------------------
    // 7. Write a test record and verify it can be read
    // -------------------------------------------------------------------------
    println!("\n[Step 7] Writing test record to database...");
    pg_client
        .execute(
            "CREATE TABLE IF NOT EXISTS smoooth_persistence_test (id serial primary key, proof_text text, created_at timestamp default now());",
            &[],
        )
        .await
        .unwrap();

    let test_token = "smoooth_v2_zero_data_loss_verified";
    pg_client
        .execute(
            "INSERT INTO smoooth_persistence_test (proof_text) VALUES ($1);",
            &[&test_token],
        )
        .await
        .unwrap();

    let verify_row = pg_client
        .query_one(
            "SELECT proof_text FROM smoooth_persistence_test WHERE proof_text = $1;",
            &[&test_token],
        )
        .await
        .unwrap();
    let read_proof: &str = verify_row.get(0);
    assert_eq!(
        read_proof, test_token,
        "Read record must match written test token"
    );
    println!(">>> Test record '{}' persisted and verified.", read_proof);

    // -------------------------------------------------------------------------
    // 8 & 9. Deploy real application container & verify via actual HTTP readiness check
    // -------------------------------------------------------------------------
    println!("\n[Step 8 & 9] Deploying application container with HTTP endpoint...");
    let app_config = serde_json::json!({
        "image": "alpine:latest",
        "network": net_name,
        "workspace_id": workspace_id,
        "cmd": [
            "sh", "-c",
            "while true; do printf 'HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: 26\\r\\n\\r\\n{\"status\":\"ok\",\"app\":\"v1\"}' | nc -l -p 14040; done"
        ],
        "ports": {
            "14040": 14040
        }
    });

    let app_res = docker_adapter
        .provision(app_service_name, &app_config, false)
        .await;
    assert!(
        app_res.is_ok(),
        "Application container deployment must succeed: {:?}",
        app_res.err()
    );
    let app_resource = app_res.unwrap();
    storage.save_resource(app_resource.clone()).await.unwrap();
    println!(">>> Application container '{}' started.", app_service_name);

    // HTTP probe against loopback port 14040
    println!(">>> Executing real HTTP readiness check against 127.0.0.1:14040...");
    let mut http_ready = false;
    for attempt in 1..=10 {
        sleep(Duration::from_millis(500)).await;
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:14040").await {
            let req = "GET /health HTTP/1.1\r\nHost: 127.0.0.1:14040\r\nConnection: close\r\n\r\n";
            if stream.write_all(req.as_bytes()).await.is_ok() {
                let mut buf = vec![0u8; 1024];
                if let Ok(n) = stream.read(&mut buf).await {
                    let resp = String::from_utf8_lossy(&buf[..n]);
                    if resp.contains("200 OK")
                        && resp.contains("{\"status\":\"ok\",\"app\":\"v1\"}")
                    {
                        println!(
                            ">>> HTTP readiness verified on attempt {}: {}",
                            attempt,
                            resp.trim()
                        );
                        http_ready = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(
        http_ready,
        "Application must pass real HTTP readiness probe on 127.0.0.1:14040"
    );

    // -------------------------------------------------------------------------
    // 10. Stop the application container and verify autonomous recovery
    // -------------------------------------------------------------------------
    println!("\n[Step 10] Stopping application container to simulate unexpected crash...");
    let docker_client = client.inner();
    docker_client
        .stop_container(
            app_service_name,
            Some(bollard::container::StopContainerOptions { t: 2 }),
        )
        .await
        .unwrap();

    let inspected_status = docker_adapter.inspect(app_service_name).await.unwrap();
    assert_eq!(
        inspected_status,
        ResourceStatus::Stopped,
        "Container must be Stopped after crash"
    );
    println!(">>> Confirmed container stopped. Running orchestrator reconciliation...");

    let recon_outcome = orchestrator
        .reconcile_resource(&app_resource.id)
        .await
        .unwrap();
    assert!(
        recon_outcome.drift_corrected,
        "Reconciliation must correct drift"
    );
    assert_eq!(recon_outcome.observed_status, ResourceStatus::Active);
    println!(
        ">>> Autonomous recovery succeeded: {}",
        recon_outcome.message
    );

    // -------------------------------------------------------------------------
    // 11. Deploy intentionally broken release and exercise failure path
    // -------------------------------------------------------------------------
    println!("\n[Step 11] Deploying intentionally broken release to exercise failure path...");
    let _ = ContainerManager::stop_and_remove(&client, app_service_name).await;

    let broken_config = serde_json::json!({
        "image": "alpine:latest",
        "network": net_name,
        "workspace_id": workspace_id,
        "cmd": ["sh", "-c", "exit 1"], // Intentionally broken command: exits with error immediately
        "ports": {
            "14040": 14040
        }
    });

    // Deploy the broken container
    let broken_res = docker_adapter
        .provision(app_service_name, &broken_config, false)
        .await;
    assert!(
        broken_res.is_ok(),
        "Provisioning call succeeds, but container will crash immediately on exit 1"
    );

    // Give the container a brief moment to execute and exit
    sleep(Duration::from_millis(800)).await;

    // Active health check fails!
    let health = docker_adapter.health_check(app_service_name).await.unwrap();
    assert!(
        !health.is_healthy,
        "Health probe for broken release must report failure!"
    );
    println!(
        ">>> Expected failure verified: health probe message: '{}'",
        health.message
    );

    // Verify container inspected status is Stopped / Degraded
    let inspected = docker_adapter.inspect(app_service_name).await.unwrap();
    assert!(
        inspected == ResourceStatus::Stopped
            || inspected == ResourceStatus::Degraded
            || inspected == ResourceStatus::Terminated,
        "Inspected status must reflect container failure: {:?}",
        inspected
    );
    println!(
        ">>> Broken container inspected state confirmed: {:?}",
        inspected
    );

    // -------------------------------------------------------------------------
    // 12. Verify that database data survived failure intact
    // -------------------------------------------------------------------------
    println!(
        "\n[Step 12] Verifying database and test record remain intact after application failure..."
    );
    let (post_fail_client, post_fail_conn) =
        tokio_postgres::connect(&decrypted_conn_str, tokio_postgres::NoTls)
            .await
            .expect("Database must remain connected and responsive");

    tokio::spawn(async move {
        let _ = post_fail_conn.await;
    });

    let survive_row = post_fail_client
        .query_one(
            "SELECT proof_text, created_at FROM smoooth_persistence_test WHERE proof_text = $1;",
            &[&test_token],
        )
        .await
        .unwrap();
    let verified_proof: &str = survive_row.get(0);
    assert_eq!(
        verified_proof, test_token,
        "Database data MUST be intact after application failure!"
    );
    println!(
        ">>> DATA SURVIVAL PROVEN: Database record '{}' is intact!",
        verified_proof
    );

    // -------------------------------------------------------------------------
    // 13. Restart Smoooth process & reconcile persisted state against actual Docker resources
    // -------------------------------------------------------------------------
    println!("\n[Step 13] Simulating Smoooth process restart & startup reconciliation...");
    // Create new orchestrator instance simulating fresh process startup
    let mut restarted_orchestrator = Orchestrator::new(storage.clone());
    restarted_orchestrator.register_adapter(docker_adapter.clone());
    restarted_orchestrator.register_adapter(postgres_adapter.clone());

    let startup_reconciled = restarted_orchestrator.reconcile_on_startup().await.unwrap();
    println!(
        ">>> Startup reconciliation completed. Reconciled {} resources against live engine.",
        startup_reconciled
    );

    // Verify PostgreSQL is still healthy after restart
    let pg_health = restarted_orchestrator
        .health_check(&pg_resource.id)
        .await
        .unwrap();
    assert!(
        pg_health.is_healthy,
        "PostgreSQL must be healthy after Smoooth restart"
    );
    println!(
        ">>> PostgreSQL health check after Smoooth restart: {}",
        pg_health.message
    );

    // -------------------------------------------------------------------------
    // Teardown test resources cleanly
    // -------------------------------------------------------------------------
    println!("\n[Teardown] Cleaning up test containers, volume, and network...");
    let _ = ContainerManager::stop_and_remove(&client, app_service_name).await;
    let _ = ContainerManager::stop_and_remove(&client, pg_service_name).await;
    let _ = VolumeManager::remove_volume(&client, pg_vol_name, true).await;
    let _ = NetworkManager::remove_network(&client, net_name).await;
    println!(">>> Teardown completed cleanly. All test resources pruned.\n");
    println!("=== LIVE VPS DOCKER & POSTGRESQL PROOF PASSED (13/13 STEPS) ===");
}
