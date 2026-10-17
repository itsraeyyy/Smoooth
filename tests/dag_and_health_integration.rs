use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::ContainerManager;
use smoooth::adapters::docker::DockerAdapter;
use smoooth::engine::Orchestrator;
use smoooth::models::plan::{
    CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, Plan, PolicyConstraints, RiskTier,
};
use smoooth::models::ResourceStatus;
use smoooth::storage::LocalStorage;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_live_docker_dag_and_health_probes() {
    println!("\n=== Starting Smoooth Milestone 6A Live Docker DAG & Health Probing Test ===");

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

    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::with_client(client.clone()));
    orchestrator.register_adapter(docker_adapter.clone());

    let http_name = "smoooth_it_dag_http";
    let worker_name = "smoooth_it_dag_worker";
    let fail_http_name = "smoooth_it_fail_http";
    let fail_worker_name = "smoooth_it_fail_worker";

    // 0. Pre-cleanup any stale disposable test containers
    let _ = ContainerManager::stop_and_remove(&client, http_name).await;
    let _ = ContainerManager::stop_and_remove(&client, worker_name).await;
    let _ = ContainerManager::stop_and_remove(&client, fail_http_name).await;
    let _ = ContainerManager::stop_and_remove(&client, fail_worker_name).await;

    // -------------------------------------------------------------------------
    // Scenario 1: Reverse-order Actions with HTTP Prerequisite Probe
    // -------------------------------------------------------------------------
    println!("\n[Scenario 1] Testing out-of-order execution with live HTTP health probe and dependent worker...");

    // Lightweight busybox HTTP responder script in alpine
    let http_cmd = "while true; do { echo -e 'HTTP/1.1 200 OK\\r\\nContent-Type: text/plain\\r\\nContent-Length: 2\\r\\n\\r\\nOK'; } | nc -l -p 8080; done";

    let spec_success = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-dag-test".to_string(),
        environment: "development".to_string(),
        target_id: "node-vps".to_string(),
        actions: vec![
            // Deliberately declare worker FIRST (index 0)
            CanonicalActionSpec {
                resource_name: worker_name.to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({
                    "image": "alpine:latest",
                    "cmd": ["sleep", "60"],
                    "restart_policy": "no",
                    "health_check": {
                        "type": "liveness"
                    }
                }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            // Declare prerequisite http_app SECOND (index 1)
            CanonicalActionSpec {
                resource_name: http_name.to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({
                    "image": "alpine:latest",
                    "cmd": ["sh", "-c", http_cmd],
                    "ports": {
                        "8080": 18088
                    },
                    "restart_policy": "on-failure",
                    "health_check": {
                        "type": "http",
                        "path": "/healthz",
                        "port": 8080,
                        "expected_status": 200
                    }
                }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
        ],
        // worker depends on http
        dependencies: vec![DependencyEdge {
            from: worker_name.to_string(),
            to: http_name.to_string(),
        }],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let plan_success = Plan::new(
        "plan-dag-success".to_string(),
        spec_success,
        RiskTier::Low,
        None,
    );
    let op_res = orchestrator
        .execute_plan_sync(plan_success, "idemp-dag-001".to_string())
        .await;
    assert!(
        op_res.is_ok(),
        "Plan execution must succeed: {:?}",
        op_res.err()
    );
    let op = op_res.unwrap();
    assert_eq!(op.status, smoooth::models::OperationStatus::Succeeded);

    // Verify both containers are running in Docker
    let http_status = ContainerManager::inspect(&client, http_name).await.unwrap();
    assert_eq!(http_status, ResourceStatus::Active);

    let worker_status = ContainerManager::inspect(&client, worker_name)
        .await
        .unwrap();
    assert_eq!(worker_status, ResourceStatus::Active);

    // Verify worker has no published ports, while http has loopback-only port
    let worker_has_ports = ContainerManager::has_host_port_binding(&client, worker_name)
        .await
        .unwrap();
    assert!(
        !worker_has_ports,
        "Worker must NOT have any host port bindings"
    );

    println!(">>> Scenario 1 PASSED: Out-of-order DAG resolved and verified with live HTTP health probe.");

    // -------------------------------------------------------------------------
    // Scenario 2: Prerequisite Failure Halts Execution & Protects Dependent
    // -------------------------------------------------------------------------
    println!("\n[Scenario 2] Testing prerequisite failure halting dependent container creation...");

    // Container returns 500 error, so expected 200 check fails
    let fail_http_cmd = "while true; do { echo -e 'HTTP/1.1 500 Internal Server Error\\r\\nContent-Type: text/plain\\r\\nContent-Length: 5\\r\\n\\r\\nERROR'; } | nc -l -p 8081; done";

    let spec_fail = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-dag-test".to_string(),
        environment: "development".to_string(),
        target_id: "node-vps".to_string(),
        actions: vec![
            // Deliberately declare dependent worker first
            CanonicalActionSpec {
                resource_name: fail_worker_name.to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({
                    "image": "alpine:latest",
                    "cmd": ["sleep", "60"]
                }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            // Prerequisite that fails HTTP health probe
            CanonicalActionSpec {
                resource_name: fail_http_name.to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({
                    "image": "alpine:latest",
                    "cmd": ["sh", "-c", fail_http_cmd],
                    "ports": {
                        "8081": 18089
                    },
                    "health_check": {
                        "type": "http",
                        "path": "/healthz",
                        "port": 8081,
                        "expected_status": 200
                    }
                }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
        ],
        dependencies: vec![DependencyEdge {
            from: fail_worker_name.to_string(),
            to: fail_http_name.to_string(),
        }],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let plan_fail = Plan::new("plan-dag-fail".to_string(), spec_fail, RiskTier::Low, None);
    let op_fail = orchestrator
        .execute_plan_sync(plan_fail, "idemp-dag-002".to_string())
        .await
        .unwrap();
    assert_eq!(op_fail.status, smoooth::models::OperationStatus::Failed);
    assert!(
        op_fail
            .error_message
            .unwrap()
            .contains("Health probe failed for smoooth_it_fail_http"),
        "Failure must clearly identify prerequisite health failure"
    );

    // CRITICAL SECURITY ASSERTION: fail_worker_name must NEVER exist in Docker!
    let fail_worker_inspect = ContainerManager::inspect(&client, fail_worker_name)
        .await
        .unwrap();
    assert_eq!(
        fail_worker_inspect,
        ResourceStatus::Terminated,
        "CRITICAL: Dependent worker container was provisioned despite prerequisite health check failure!"
    );
    let raw_docker_inspect = client
        .inner()
        .inspect_container(
            fail_worker_name,
            None::<bollard::container::InspectContainerOptions>,
        )
        .await;
    assert!(
        raw_docker_inspect.is_err(),
        "Docker engine must confirm dependent worker container does not exist"
    );
    println!(">>> Scenario 2 PASSED: Prerequisite failure aborted pipeline and 0 dependent containers were created.");

    // -------------------------------------------------------------------------
    // Teardown / Cleanup
    // -------------------------------------------------------------------------
    println!("\n[Teardown] Cleaning up disposable test containers...");
    let _ = ContainerManager::stop_and_remove(&client, http_name).await;
    let _ = ContainerManager::stop_and_remove(&client, worker_name).await;
    let _ = ContainerManager::stop_and_remove(&client, fail_http_name).await;
    let _ = ContainerManager::stop_and_remove(&client, fail_worker_name).await;
    println!(">>> Teardown complete.");
}
