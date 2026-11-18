use smoooth::models::workload::{WorkloadResourceLimits, WorkloadSpec, WorkloadVolumeMount};
use smoooth::runtime::mock::MockRuntimeProvider;
use smoooth::runtime::RuntimeProvider;
use std::collections::HashMap;

#[tokio::test]
async fn test_mock_runtime_lifecycle() {
    let runtime = MockRuntimeProvider::new();
    assert_eq!(runtime.runtime_name(), "mock");
    assert!(runtime.is_available().await.unwrap());

    // 1. Valid workload spec creation
    let spec = WorkloadSpec {
        name: "test-service".to_string(),
        image: "redis:7-alpine".to_string(),
        digest: None,
        cmd: None,
        env: HashMap::new(),
        ports: vec![],
        volumes: vec![],
        network: None,
        restart_policy: "unless-stopped".to_string(),
        limits: WorkloadResourceLimits::default(),
        health_check: None,
    };

    let id = runtime.create_and_start(&spec).await.unwrap();
    assert_eq!(id, "mock_test-service");

    // 2. Inspect running state
    let inspection = runtime.inspect(&id).await.unwrap();
    assert!(inspection.is_running);
    assert_eq!(inspection.name, "test-service");
    assert_eq!(inspection.image, "redis:7-alpine");

    // 3. Health check on running container
    let health = runtime.health_check(&id).await.unwrap();
    assert!(health.is_healthy);

    // 4. Logs fetch
    let logs = runtime.logs(&id, 100).await.unwrap();
    assert!(logs.contains("test-service"));

    // 5. Restart container
    runtime.restart(&id, 5).await.unwrap();
    let inspection_after_restart = runtime.inspect(&id).await.unwrap();
    assert!(inspection_after_restart.is_running);

    // 6. Stop container
    runtime.stop(&id, 5).await.unwrap();
    let inspection_after_stop = runtime.inspect(&id).await.unwrap();
    assert!(!inspection_after_stop.is_running);

    // Health check on stopped container
    let health_stopped = runtime.health_check(&id).await.unwrap();
    assert!(!health_stopped.is_healthy);

    // 7. Delete container
    runtime.delete(&id, true).await.unwrap();
    assert!(runtime.inspect(&id).await.is_err());
}

#[tokio::test]
async fn test_runtime_rejects_invalid_workload_spec_before_execution() {
    let runtime = MockRuntimeProvider::new();

    // Workload with host bind mount escape attempt
    let malicious_spec = WorkloadSpec {
        name: "escape-artist".to_string(),
        image: "alpine:latest".to_string(),
        digest: None,
        cmd: None,
        env: HashMap::new(),
        ports: vec![],
        volumes: vec![WorkloadVolumeMount {
            volume_name: "/etc".to_string(),
            container_path: "/host_etc".to_string(),
            read_only: false,
        }],
        network: None,
        restart_policy: "unless-stopped".to_string(),
        limits: WorkloadResourceLimits::default(),
        health_check: None,
    };

    let result = runtime.create_and_start(&malicious_spec).await;
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("Host path bind mount"));
}
