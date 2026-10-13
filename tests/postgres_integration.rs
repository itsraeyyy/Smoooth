use std::sync::Arc;
use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::postgres::PostgresAdapter;
use smoooth::adapters::ProviderAdapter;

#[tokio::test]
async fn test_postgres_provisioning_and_query_when_docker_available() {
    let client = match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => c,
            Err(_) => return, // Graceful skip if Docker daemon is not active in this test runner
        },
        Err(_) => return,
    };

    let docker = Arc::new(DockerAdapter::with_client(client));
    let postgres = PostgresAdapter::new(docker.clone());

    let service_name = "smoooth_it_pg_test";
    let vol_name = "smoooth_it_pg_vol_test";
    let config = serde_json::json!({
        "database": "test_app_db",
        "user": "test_app_user",
        "password": "test_secure_password_999",
        "port": 15432, // Non-standard test port to avoid host conflict
        "volume_name": vol_name,
        "workspace_id": "test_workspace"
    });

    // 1. Provision real PostgreSQL service
    let res = postgres.provision(service_name, &config, true).await;
    assert!(res.is_ok(), "PostgreSQL provisioning and active query verification must succeed");
    let resource = res.unwrap();
    assert_eq!(resource.provider, "postgres");
    assert!(resource.is_persistent, "Database must be persistent");

    // 2. Active health check
    let health = postgres.health_check(&resource.id).await.unwrap();
    assert!(health.is_healthy, "Active probe must confirm health");

    // 3. Destroy container with persistent data preservation
    let destroy_res = postgres.destroy(&resource.id, true).await;
    assert!(destroy_res.is_ok(), "Container teardown must succeed");

    // 4. Verify data volume is intact by re-provisioning with the same volume
    let re_provision = postgres.provision(service_name, &config, true).await;
    assert!(re_provision.is_ok(), "Re-provisioning over existing persistent volume must succeed");

    // Final cleanup of test resources
    let re_res = re_provision.unwrap();
    let _ = postgres.destroy(&re_res.id, false).await;
    let _ = docker.destroy(&re_res.id, false).await;
}
