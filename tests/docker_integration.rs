use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::{ContainerManager, ContainerSpec};
use smoooth::adapters::docker::images::ImageManager;
use smoooth::adapters::docker::volumes::VolumeManager;
use std::collections::HashMap;

#[tokio::test]
async fn test_docker_daemon_connectivity_or_graceful_diagnostics() {
    match DockerClient::try_new() {
        Ok(client) => match client.ping().await {
            Ok(_) => {
                let version = client.version_info().await.unwrap();
                println!("Docker daemon is connected: version {}", version);
                assert!(!version.is_empty());
            }
            Err(e) => {
                println!("Docker daemon unreachable (ping error): {}", e);
            }
        },
        Err(e) => {
            println!("Docker client connection unavailable: {}", e);
        }
    }
}

#[tokio::test]
async fn test_docker_container_lifecycle_when_daemon_available() {
    let client = match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => c,
            Err(_) => return, // Skip live test if daemon is not running in current environment
        },
        Err(_) => return,
    };

    // 1. Pull lightweight alpine image
    let image = "alpine:latest";
    let pull_res = ImageManager::ensure_image(&client, image, None).await;
    assert!(pull_res.is_ok(), "Image pull must succeed");

    // 2. Ensure volume with persistence metadata
    let vol_name = "smoooth_it_vol_test";
    let vol_res = VolumeManager::ensure_volume(&client, vol_name, "test_ws", true).await;
    assert!(vol_res.is_ok(), "Volume creation must succeed");

    // 3. Start disposable test container
    let container_name = "smoooth_it_alpine_test";
    let _ = ContainerManager::stop_and_remove(&client, container_name).await;
    let spec = ContainerSpec {
        name: container_name.to_string(),
        image: image.to_string(),
        cmd: Some(vec!["sleep".to_string(), "30".to_string()]),
        env: vec!["TEST_VAR=hello_smoooth".to_string()],
        port_bindings: HashMap::new(),
        volume_mounts: vec![smoooth::models::VolumeMount::new(
            vol_name.to_string(),
            "/data".to_string(),
            false,
        )],
        network_name: None,
        labels: HashMap::new(),
        restart_policy: None,
        memory_bytes: None,
        cpu_shares: None,
        nano_cpus: None,
        health_spec: None,
    };

    let container_id = ContainerManager::create_and_start(&client, &spec)
        .await
        .unwrap();
    assert!(!container_id.is_empty());

    // 4. Verify running inspection
    let status = ContainerManager::inspect(&client, container_name)
        .await
        .unwrap();
    assert_eq!(status, smoooth::models::ResourceStatus::Active);

    // 5. Cleanup container without deleting volume
    let cleanup = ContainerManager::stop_and_remove(&client, container_name).await;
    assert!(cleanup.is_ok());

    // 6. Verify persistent volume was preserved
    let vol_remove_blocked = VolumeManager::remove_volume(&client, vol_name, false).await;
    assert!(
        vol_remove_blocked.is_err(),
        "Persistent volume deletion without force must be blocked"
    );

    // 7. Explicit cleanup of test volume with force
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;
}
