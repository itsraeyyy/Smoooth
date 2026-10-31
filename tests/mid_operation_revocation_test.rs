use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use smoooth::adapters::{AdapterGuarantees, HealthEvidence, ProviderAdapter};
use smoooth::engine::Orchestrator;
use smoooth::error::Result;
use smoooth::models::plan::{CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, PolicyConstraints, RiskTier};
use smoooth::models::{Capability, OperationStatus, Plan, Resource, ResourceOwnership, ResourceStatus};
use smoooth::storage::{LocalStorage, Storage};

/// Controllable mock adapter allowing controlled delays and tracking executed actions.
struct ControllableMockAdapter {
    step_1_executed: AtomicBool,
    step_2_executed: AtomicBool,
    step_3_executed: AtomicBool,
    step_1_destroyed: AtomicBool,
    execution_delay: Duration,
}

impl ControllableMockAdapter {
    fn new(delay: Duration) -> Self {
        Self {
            step_1_executed: AtomicBool::new(false),
            step_2_executed: AtomicBool::new(false),
            step_3_executed: AtomicBool::new(false),
            step_1_destroyed: AtomicBool::new(false),
            execution_delay: delay,
        }
    }
}

#[async_trait]
impl ProviderAdapter for ControllableMockAdapter {
    fn provider_name(&self) -> &str {
        "controllable_mock"
    }

    fn guarantees(&self) -> AdapterGuarantees {
        AdapterGuarantees {
            provider_name: "controllable_mock".into(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![]
    }

    async fn provision(
        &self,
        name: &str,
        _config: &serde_json::Value,
        is_persistent: bool,
    ) -> Result<Resource> {
        if name == "step_1" {
            self.step_1_executed.store(true, Ordering::SeqCst);
        } else if name == "step_2" {
            self.step_2_executed.store(true, Ordering::SeqCst);
            // Simulate slow external I/O (e.g. database migration or container pull)
            tokio::time::sleep(self.execution_delay).await;
        } else if name == "step_3" {
            self.step_3_executed.store(true, Ordering::SeqCst);
        }

        Ok(Resource {
            id: name.to_string(),
            name: name.to_string(),
            workspace_id: "test_ws".to_string(),
            service_type: "mock_svc".to_string(),
            provider: "controllable_mock".to_string(),
            is_persistent,
            ownership: ResourceOwnership::SmooothManaged,
            status: ResourceStatus::Active,
            configuration: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn inspect(&self, _resource_id: &str) -> Result<ResourceStatus> {
        Ok(ResourceStatus::Active)
    }

    async fn health_check(&self, _resource_id: &str) -> Result<HealthEvidence> {
        Ok(HealthEvidence {
            is_healthy: true,
            latency_ms: Some(1),
            message: "Healthy".into(),
            details: serde_json::json!({}),
        })
    }

    async fn reconcile(
        &self,
        _resource_id: &str,
        _intended_config: &serde_json::Value,
    ) -> Result<ResourceStatus> {
        Ok(ResourceStatus::Active)
    }

    async fn destroy(&self, resource_id: &str, _preserve_persistent_data: bool) -> Result<()> {
        if resource_id == "step_1" {
            self.step_1_destroyed.store(true, Ordering::SeqCst);
        }
        Ok(())
    }
}

fn build_test_plan(id: &str) -> Plan {
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "test_ws".into(),
        environment: "development".into(),
        target_id: "test_target".into(),
        actions: vec![
            CanonicalActionSpec {
                resource_name: "step_1".into(),
                provider: "controllable_mock".into(),
                service_type: "mock_svc".into(),
                action_type: "create".into(),
                config: serde_json::json!({}),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            CanonicalActionSpec {
                resource_name: "step_2".into(),
                provider: "controllable_mock".into(),
                service_type: "mock_svc".into(),
                action_type: "create".into(),
                config: serde_json::json!({}),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            CanonicalActionSpec {
                resource_name: "step_3".into(),
                provider: "controllable_mock".into(),
                service_type: "mock_svc".into(),
                action_type: "create".into(),
                config: serde_json::json!({}),
                is_persistent: false,
                exposes_network_publicly: false,
            },
        ],
        dependencies: vec![
            DependencyEdge {
                from: "step_2".into(),
                to: "step_1".into(),
            },
            DependencyEdge {
                from: "step_3".into(),
                to: "step_2".into(),
            },
        ],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    Plan::new(id.to_string(), spec, RiskTier::Low, None)
}

#[tokio::test]
async fn test_adversarial_revocation_during_active_multistep_operation() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mock_adapter = Arc::new(ControllableMockAdapter::new(Duration::from_millis(150)));

    let mut orchestrator = Orchestrator::new(storage.clone());
    orchestrator.register_adapter(mock_adapter.clone());

    let plan = build_test_plan("plan_adversarial_multistep");

    // 1. Launch multi-step operation asynchronously
    let op = orchestrator
        .execute_plan(plan, "idemp_client_key_1".into())
        .await
        .expect("Operation dispatch must succeed");

    let op_id = op.id.clone();

    // 2. Allow Step 1 to complete and Step 2 to enter slow provider execution
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(mock_adapter.step_1_executed.load(Ordering::SeqCst), "Step 1 must have executed");
    assert!(mock_adapter.step_2_executed.load(Ordering::SeqCst), "Step 2 must be running");

    // 3. ADVERSARIAL ACTION: Operator / Security Guard revokes lease mid-flight!
    // Bumps lease epoch from 1 to 2 in storage
    let mut current_op = storage.get_operation(&op_id).await.unwrap().unwrap();
    current_op.lease_epoch += 1; // Revoke lease epoch
    storage.update_operation(current_op).await.unwrap();

    // 4. Wait for Step 2 to yield and worker to arrive at checkpoint before Step 3
    tokio::time::sleep(Duration::from_millis(250)).await;

    // 5. SECURITY INVARIANT ASSERTIONS:
    // Step 3 must NEVER have executed
    assert!(
        !mock_adapter.step_3_executed.load(Ordering::SeqCst),
        "CRITICAL SECURITY FAILURE: Step 3 must not execute after lease epoch revocation!"
    );

    // Ephemeral resources from Step 1 must have been safely rolled back
    assert!(
        mock_adapter.step_1_destroyed.load(Ordering::SeqCst),
        "Ephemeral resources from Step 1 must be cleaned up on aborted lease"
    );

    // The operation status in storage must be Failed with explicit lease loss diagnostics
    let final_op = storage.get_operation(&op_id).await.unwrap().unwrap();
    assert_eq!(final_op.status, OperationStatus::Failed);
    assert!(
        final_op.error_message.as_ref().map(|s| s.contains("Worker lease lost")).unwrap_or(false),
        "Operation error must indicate Worker lease lost: {:?}",
        final_op.error_message
    );
}

#[tokio::test]
async fn test_adversarial_lease_expiry_during_slow_provider_aborts_subsequent_steps() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mock_adapter = Arc::new(ControllableMockAdapter::new(Duration::from_millis(150)));

    let mut orchestrator = Orchestrator::new(storage.clone());
    orchestrator.register_adapter(mock_adapter.clone());

    let plan = build_test_plan("plan_lease_expiry_test");

    let op = orchestrator
        .execute_plan(plan, "idemp_client_key_expiry".into())
        .await
        .unwrap();

    let op_id = op.id.clone();

    // Allow Step 1 to execute and Step 2 to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate expired lease by rewinding lease_expires_at into the past
    let mut current_op = storage.get_operation(&op_id).await.unwrap().unwrap();
    current_op.lease_expires_at = Some(Utc::now() - chrono::Duration::seconds(10));
    storage.update_operation(current_op).await.unwrap();

    // Wait for Step 2 to finish and checkpoint evaluation to run
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Step 3 must never execute
    assert!(
        !mock_adapter.step_3_executed.load(Ordering::SeqCst),
        "Step 3 must not execute when lease has expired"
    );

    let final_op = storage.get_operation(&op_id).await.unwrap().unwrap();
    assert_eq!(final_op.status, OperationStatus::Failed);
    assert!(
        final_op.error_message.as_ref().map(|s| s.contains("Worker lease expired")).unwrap_or(false),
        "Operation error must indicate lease expiration: {:?}",
        final_op.error_message
    );
}
