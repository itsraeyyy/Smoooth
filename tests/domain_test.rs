use std::collections::HashMap;
use std::sync::Arc;

use smoooth::engine::Orchestrator;
use smoooth::models::{
    CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, HealthSpec, Plan,
    PolicyConstraints, RiskTier, ServiceBinding,
};
use smoooth::storage::{LocalStorage, Storage};

#[tokio::test]
async fn test_canonical_plan_hash_determinism() {
    let spec1 = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-1".to_string(),
        environment: "production".to_string(),
        target_id: "cluster-a".to_string(),
        actions: vec![CanonicalActionSpec {
            resource_name: "db".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            action_type: "create".to_string(),
            config: serde_json::json!({ "port": 5432 }),
            is_persistent: true,
            exposes_network_publicly: false,
        }],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: Some(100),
        },
    };

    let hash1 = Plan::compute_canonical_hash(&spec1);
    let hash2 = Plan::compute_canonical_hash(&spec1);
    assert_eq!(hash1, hash2, "Canonical hash must be deterministic");

    // Mutation alters hash
    let mut spec2 = spec1.clone();
    spec2.actions[0].exposes_network_publicly = true;
    let hash3 = Plan::compute_canonical_hash(&spec2);
    assert_ne!(hash1, hash3, "Altering spec must change canonical hash");
}

#[tokio::test]
async fn test_idempotency_key_mismatch_rejection() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Orchestrator::new(storage);

    let spec1 = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-test".to_string(),
        environment: "dev".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let plan1 = Plan::new("plan-1".to_string(), spec1.clone(), RiskTier::Low, None);
    let key = "client-idemp-101".to_string();

    // Execute first plan
    let op1 = orchestrator.execute_plan(plan1, key.clone()).await.unwrap();
    assert_eq!(op1.idempotency_client_key, key);

    // Re-execute with different spec using same idempotency key -> Must return conflict
    let mut spec2 = spec1.clone();
    spec2.environment = "staging".to_string();
    let plan2 = Plan::new("plan-2".to_string(), spec2, RiskTier::Low, None);

    let err = orchestrator.execute_plan(plan2, key).await.unwrap_err();
    match err {
        smoooth::error::SmooothError::IdempotencyConflict(_) => (),
        other => panic!("Expected IdempotencyConflict, got {:?}", other),
    }
}

#[test]
fn test_sensitive_credential_masking() {
    let raw = "postgresql://admin:super_secret_password_123!@db.host.internal:5432/production_db";
    let masked = ServiceBinding::mask_connection_string(raw);
    assert!(!masked.contains("super_secret_password_123!"));
    assert!(masked.contains("[REDACTED]"));
    assert!(masked.contains("@db.host.internal:5432/production_db"));
}

#[tokio::test]
async fn test_plan_approval_binding_and_expiry() {
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-prod".to_string(),
        environment: "production".to_string(),
        target_id: "vps-node-1".to_string(),
        actions: vec![CanonicalActionSpec {
            resource_name: "prod_db".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            action_type: "create".to_string(),
            config: serde_json::json!({ "port": 5432 }),
            is_persistent: true,
            exposes_network_publicly: false,
        }],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: Some(200),
        },
    };

    let mut plan = Plan::new(
        "plan-approval-test".to_string(),
        spec.clone(),
        RiskTier::High,
        None,
    );
    assert!(
        !plan.is_approval_valid(),
        "Unapproved high risk plan must not be valid"
    );

    // Approve with standard 1 hour TTL
    plan.approve("lead_architect".to_string());
    assert!(plan.is_approval_valid(), "Fresh approval must be valid");

    // Test 1: Modified target environment invalidates approval
    let mut modified_env_plan = plan.clone();
    modified_env_plan.spec.environment = "staging".to_string();
    assert!(
        !modified_env_plan.is_approval_valid(),
        "Approval must not be valid if environment changed"
    );

    // Test 2: Modified workspace invalidates approval
    let mut modified_ws_plan = plan.clone();
    modified_ws_plan.spec.workspace_id = "ws-other".to_string();
    assert!(
        !modified_ws_plan.is_approval_valid(),
        "Approval must not be valid if workspace changed"
    );

    // Test 3: Expired approval is rejected
    let mut expired_plan = Plan::new(
        "plan-expired-test".to_string(),
        spec.clone(),
        RiskTier::High,
        None,
    );
    expired_plan.approve_with_ttl("lead_architect".to_string(), -10); // Expired 10 seconds ago
    assert!(
        !expired_plan.is_approval_valid(),
        "Expired approval must be rejected"
    );
    assert_eq!(
        expired_plan.approval_rejection_reason(),
        Some("Approval has expired")
    );
}

#[tokio::test]
async fn test_secret_leak_scanner_across_serialized_surfaces() {
    let secret_password = "super_classified_pg_password_999!";
    let secret_store = smoooth::security::SecretStore::default_local();
    let raw_conn = format!(
        "postgresql://app_user:{}@smoooth-postgres:5432/app_db",
        secret_password
    );

    // Create binding with encrypted credentials
    let binding = ServiceBinding::new_with_secret(
        "bind-secret-test".to_string(),
        "app_backend".to_string(),
        "app_db".to_string(),
        "postgres_connection".to_string(),
        raw_conn.as_bytes(),
        &secret_store,
        vec!["DATABASE_URL".to_string()],
    )
    .unwrap();

    // 1. Scan JSON serialization of ServiceBinding
    let serialized_binding = serde_json::to_string(&binding).unwrap();
    assert!(
        !serialized_binding.contains(secret_password),
        "Plaintext secret must NEVER appear in serialized ServiceBinding"
    );

    // 2. Scan masked connection string
    let masked = ServiceBinding::mask_connection_string(&raw_conn);
    assert!(
        !masked.contains(secret_password),
        "Plaintext secret must NEVER appear in masked connection string"
    );
    assert!(masked.contains("[REDACTED]"));

    // 3. Scan decrypted credentials in memory (decrypted only when injected)
    let decrypted = binding.decrypt_credentials(&secret_store).unwrap();
    assert_eq!(String::from_utf8(decrypted).unwrap(), raw_conn);
}

struct MockCrashAdapter;

#[async_trait::async_trait]
impl smoooth::adapters::ProviderAdapter for MockCrashAdapter {
    fn provider_name(&self) -> &str {
        "docker"
    }
    fn guarantees(&self) -> smoooth::adapters::AdapterGuarantees {
        smoooth::adapters::AdapterGuarantees {
            provider_name: "docker".to_string(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: false,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }
    fn supported_capabilities(&self) -> Vec<smoooth::models::Capability> {
        vec![]
    }
    async fn provision(
        &self,
        _name: &str,
        _config: &serde_json::Value,
        _is_persistent: bool,
    ) -> smoooth::error::Result<smoooth::models::Resource> {
        Err(smoooth::error::SmooothError::ExecutionFailed(
            "Simulated container crash".to_string(),
        ))
    }
    async fn inspect(
        &self,
        _resource_id: &str,
    ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Stopped)
    }
    async fn health_check(
        &self,
        _resource_id: &str,
    ) -> smoooth::error::Result<smoooth::adapters::HealthEvidence> {
        Ok(smoooth::adapters::HealthEvidence {
            is_healthy: false,
            latency_ms: None,
            message: "Container stopped".to_string(),
            details: serde_json::json!({}),
        })
    }
    async fn reconcile(
        &self,
        _resource_id: &str,
        _intended_config: &serde_json::Value,
    ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Stopped)
    }
    async fn destroy(
        &self,
        _resource_id: &str,
        _preserve_persistent_data: bool,
    ) -> smoooth::error::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_bounded_reconciliation_stops_infinite_loops() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    orchestrator.register_adapter(Arc::new(MockCrashAdapter));

    // Create a resource simulating a crash loop with 3 existing attempts
    let resource_id = "res-crash-loop-app";
    let crashed_app = smoooth::models::Resource {
        id: resource_id.to_string(),
        name: "crash_loop_app".to_string(),
        workspace_id: "ws-prod".to_string(),
        service_type: "compute".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: smoooth::models::ResourceOwnership::SmooothManaged,
        status: smoooth::models::ResourceStatus::Active, // intended
        configuration: serde_json::json!({
            "image": "broken_app:v1",
            "smoooth_recovery_attempts": 3 // already hit 3 attempts
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.save_resource(crashed_app).await.unwrap();

    // Reconcile resource should refuse to restart and mark Degraded
    let outcome = orchestrator.reconcile_resource(resource_id).await.unwrap();
    assert!(
        !outcome.drift_corrected,
        "Must NOT attempt further restart when bound is hit"
    );
    assert_eq!(
        outcome.intended_status,
        smoooth::models::ResourceStatus::Degraded
    );
    assert!(outcome.message.contains("Bounded recovery limit reached"));

    // Verify stored resource status was transitioned to Degraded
    let updated = storage.get_resource(resource_id).await.unwrap().unwrap();
    assert_eq!(updated.status, smoooth::models::ResourceStatus::Degraded);
}

#[tokio::test]
async fn test_data_safety_guard_blocks_destructive_deletion() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Orchestrator::new(storage.clone());

    // Spec attempting to delete a persistent database with allow_data_destruction = false
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-prod".to_string(),
        environment: "production".to_string(),
        target_id: "vps-node-1".to_string(),
        actions: vec![CanonicalActionSpec {
            resource_name: "prod_db".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            action_type: "delete".to_string(), // Destructive action!
            config: serde_json::json!({ "database": "prod_db" }),
            is_persistent: true,
            exposes_network_publicly: false,
        }],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false, // Disallowed!
            max_monthly_budget_usd: None,
        },
    };

    let mut plan = Plan::new(
        "plan-delete-safety".to_string(),
        spec,
        RiskTier::Critical,
        None,
    );
    plan.approve("admin".to_string());

    let op = orchestrator
        .execute_plan_sync(plan, "key-safety-01".to_string())
        .await
        .unwrap();
    assert_eq!(op.status, smoooth::models::OperationStatus::Failed);
    assert!(op
        .error_message
        .unwrap()
        .contains("Data safety guard blocked deletion of persistent resource"));
}

#[tokio::test]
async fn test_policy_schema_version_mismatch_rejection() {
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-prod".to_string(),
        environment: "production".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let mut plan = Plan::new("plan-version-test".to_string(), spec, RiskTier::High, None);
    plan.approve("approver".to_string());

    // Tamper with policy schema version
    if let Some(approval) = &mut plan.approval {
        approval.policy_version = 999; // Mismatched policy version
    }

    assert!(!plan.is_approval_valid());
    assert_eq!(
        plan.approval_rejection_reason(),
        Some("Policy version mismatch in approval binding")
    );
}

struct MockPreemptionAdapter {
    storage: Arc<LocalStorage>,
}

#[async_trait::async_trait]
impl smoooth::adapters::ProviderAdapter for MockPreemptionAdapter {
    fn provider_name(&self) -> &str {
        "preemption_provider"
    }
    fn guarantees(&self) -> smoooth::adapters::AdapterGuarantees {
        smoooth::adapters::AdapterGuarantees {
            provider_name: "preemption_provider".to_string(),
            supports_persistent_volumes: false,
            supports_staged_dual_credentials: false,
            supports_safe_rollback: true,
            supports_drift_detection: false,
            is_experimental: false,
        }
    }
    fn supported_capabilities(&self) -> Vec<smoooth::models::Capability> {
        vec![]
    }
    async fn provision(
        &self,
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
    ) -> smoooth::error::Result<smoooth::models::Resource> {
        // When step 1 executes, another worker steals the lease in durable storage!
        if name == "res_step1" {
            if let Some(op_id) = config.get("operation_id").and_then(|v| v.as_str()) {
                if let Ok(Some(mut op)) = self.storage.get_operation(op_id).await {
                    op.lease_epoch = 5; // Lease stolen / bumped by competing worker
                    let _ = self.storage.update_operation(op).await;
                }
            }
        }
        Ok(smoooth::models::Resource {
            id: format!("res_{}", name),
            name: name.to_string(),
            workspace_id: "ws-fencing".to_string(),
            service_type: "compute".to_string(),
            provider: "preemption_provider".to_string(),
            is_persistent,
            ownership: smoooth::models::ResourceOwnership::SmooothManaged,
            status: smoooth::models::ResourceStatus::Active,
            configuration: config.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn inspect(
        &self,
        _resource_id: &str,
    ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Active)
    }
    async fn health_check(
        &self,
        resource_id: &str,
    ) -> smoooth::error::Result<smoooth::adapters::HealthEvidence> {
        Ok(smoooth::adapters::HealthEvidence {
            is_healthy: true,
            latency_ms: Some(1),
            message: format!("Resource {} verified healthy", resource_id),
            details: serde_json::json!({}),
        })
    }
    async fn reconcile(
        &self,
        _resource_id: &str,
        _intended_config: &serde_json::Value,
    ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Active)
    }
    async fn destroy(
        &self,
        _resource_id: &str,
        _preserve_persistent_data: bool,
    ) -> smoooth::error::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_stale_worker_lease_preemption_halts_mutations() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    orchestrator.register_adapter(Arc::new(MockPreemptionAdapter {
        storage: storage.clone(),
    }));

    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-fencing".to_string(),
        environment: "dev".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![
            CanonicalActionSpec {
                resource_name: "res_step1".to_string(),
                provider: "preemption_provider".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({ "image": "alpine:latest" }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            CanonicalActionSpec {
                resource_name: "res_step2".to_string(),
                provider: "preemption_provider".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({ "image": "alpine:latest" }),
                is_persistent: false,
                exposes_network_publicly: false,
            },
        ],
        dependencies: vec![],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let plan = Plan::new("plan-lease-test".to_string(), spec, RiskTier::Low, None);

    // Worker executes plan synchronously.
    // res_step1 succeeds, but while executing, lease is bumped in storage to epoch 5.
    // Before executing res_step2, stale worker check fires and halts execution!
    let res_op = orchestrator
        .execute_plan_sync(plan, "fencing-key-01".to_string())
        .await
        .unwrap();
    assert_eq!(res_op.status, smoooth::models::OperationStatus::Failed);
    assert!(res_op.error_message.unwrap().contains("Worker lease lost"));
}

#[tokio::test]
async fn test_crash_consistent_atomic_snapshot_persistence() {
    let temp_dir =
        std::env::temp_dir().join(format!("smoooth_test_{}", uuid::Uuid::new_v4().simple()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let state_file = temp_dir.join("state.json");

    // 1. Initial file-backed storage
    let storage = LocalStorage::new_file_backed(&state_file);
    let resource = smoooth::models::Resource {
        id: "res-durable-01".to_string(),
        name: "durable_app".to_string(),
        workspace_id: "ws-durable".to_string(),
        service_type: "compute".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: smoooth::models::ResourceOwnership::SmooothManaged,
        status: smoooth::models::ResourceStatus::Active,
        configuration: serde_json::json!({ "image": "nginx:alpine" }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.save_resource(resource.clone()).await.unwrap();

    // Verify snapshot file exists and is valid JSON
    assert!(state_file.exists());
    let content = std::fs::read_to_string(&state_file).unwrap();
    assert!(content.contains("res-durable-01"));

    // 2. Simulate daemon restart: new instance loads existing file
    let storage_restarted = LocalStorage::new_file_backed(&state_file);
    let loaded = storage_restarted
        .get_resource("res-durable-01")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.name, "durable_app");

    // 3. Mutate to generate backup (.bak)
    let mut updated_resource = resource.clone();
    updated_resource.status = smoooth::models::ResourceStatus::Degraded;
    storage_restarted
        .save_resource(updated_resource)
        .await
        .unwrap();
    assert!(state_file.with_extension("bak").exists());

    // 4. Simulate corruption of primary file: write garbage to state.json
    std::fs::write(&state_file, b"MALFORMED_GARBAGE_JSON_DATA{{{[[[").unwrap();

    // 5. Restart must automatically fall back to .bak file and recover intact state
    let storage_recovered = LocalStorage::new_file_backed(&state_file);
    let recovered = storage_recovered
        .get_resource("res-durable-01")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.id, "res-durable-01");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dag_topological_sort_resolves_out_of_order_actions() {
    let make_action = |name: &str| CanonicalActionSpec {
        resource_name: name.to_string(),
        provider: "docker".to_string(),
        service_type: "compute".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({ "image": "alpine:latest" }),
        is_persistent: false,
        exposes_network_publicly: false,
    };

    // Actions declared intentionally in reverse order: [frontend, worker, api, db]
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-dag".to_string(),
        environment: "production".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![
            make_action("frontend"),
            make_action("worker"),
            make_action("api"),
            make_action("db"),
        ],
        dependencies: vec![
            DependencyEdge {
                from: "frontend".to_string(),
                to: "api".to_string(),
            },
            DependencyEdge {
                from: "worker".to_string(),
                to: "api".to_string(),
            },
            DependencyEdge {
                from: "api".to_string(),
                to: "db".to_string(),
            },
        ],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let resolved = spec.resolve_execution_order().unwrap();
    let resolved_names: Vec<String> = resolved.into_iter().map(|a| a.resource_name).collect();

    // Verification: 'db' must come first, followed by 'api', followed by 'frontend' & 'worker' (deterministic)
    assert_eq!(resolved_names, vec!["db", "api", "frontend", "worker"]);
}

#[test]
fn test_dag_cycle_rejection_and_error_reporting() {
    let make_action = |name: &str| CanonicalActionSpec {
        resource_name: name.to_string(),
        provider: "docker".to_string(),
        service_type: "compute".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({}),
        is_persistent: false,
        exposes_network_publicly: false,
    };

    // 2-node cycle: A -> B and B -> A
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-cycle".to_string(),
        environment: "development".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![make_action("service_a"), make_action("service_b")],
        dependencies: vec![
            DependencyEdge {
                from: "service_a".to_string(),
                to: "service_b".to_string(),
            },
            DependencyEdge {
                from: "service_b".to_string(),
                to: "service_a".to_string(),
            },
        ],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let err = spec.resolve_execution_order().unwrap_err();
    assert!(err.to_string().contains("Cyclic dependency detected"));
    assert!(err.to_string().contains("service_a"));
    assert!(err.to_string().contains("service_b"));
}

#[test]
fn test_dag_missing_and_self_dependency_validation() {
    let make_action = |name: &str| CanonicalActionSpec {
        resource_name: name.to_string(),
        provider: "docker".to_string(),
        service_type: "compute".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({}),
        is_persistent: false,
        exposes_network_publicly: false,
    };

    // 1. Missing prerequisite reference
    let spec_missing = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-val".to_string(),
        environment: "dev".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![make_action("api")],
        dependencies: vec![DependencyEdge {
            from: "api".to_string(),
            to: "ghost_database".to_string(),
        }],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };
    let err_missing = spec_missing.resolve_execution_order().unwrap_err();
    assert!(err_missing.to_string().contains("ghost_database"));

    // 2. Self-dependency
    let spec_self = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-val".to_string(),
        environment: "dev".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![make_action("api")],
        dependencies: vec![DependencyEdge {
            from: "api".to_string(),
            to: "api".to_string(),
        }],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };
    let err_self = spec_self.resolve_execution_order().unwrap_err();
    assert!(err_self.to_string().contains("Self-dependency detected"));
}

#[tokio::test]
async fn test_prerequisite_failure_halts_execution_and_protects_dependent() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());

    // Mock failing adapter
    struct FailingAdapter;
    #[async_trait::async_trait]
    impl smoooth::adapters::ProviderAdapter for FailingAdapter {
        fn provider_name(&self) -> &str {
            "mock_failing"
        }
        fn guarantees(&self) -> smoooth::adapters::AdapterGuarantees {
            smoooth::adapters::AdapterGuarantees {
                provider_name: "mock_failing".to_string(),
                supports_persistent_volumes: false,
                supports_staged_dual_credentials: false,
                supports_safe_rollback: true,
                supports_drift_detection: true,
                is_experimental: true,
            }
        }
        fn supported_capabilities(&self) -> Vec<smoooth::models::Capability> {
            vec![]
        }
        async fn provision(
            &self,
            name: &str,
            config: &serde_json::Value,
            is_persistent: bool,
        ) -> smoooth::error::Result<smoooth::models::Resource> {
            Ok(smoooth::models::Resource {
                id: format!("res_{}", name),
                name: name.to_string(),
                workspace_id: "ws-fail".to_string(),
                service_type: "compute".to_string(),
                provider: "mock_failing".to_string(),
                is_persistent,
                ownership: smoooth::models::ResourceOwnership::SmooothManaged,
                status: smoooth::models::ResourceStatus::Active,
                configuration: config.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn inspect(
            &self,
            _id: &str,
        ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
            Ok(smoooth::models::ResourceStatus::Active)
        }
        async fn health_check(
            &self,
            id: &str,
        ) -> smoooth::error::Result<smoooth::adapters::HealthEvidence> {
            if id.contains("failing_db") {
                Ok(smoooth::adapters::HealthEvidence {
                    is_healthy: false,
                    latency_ms: Some(10),
                    message: "Database port 5432 connection refused".to_string(),
                    details: serde_json::json!({ "error": "connection refused" }),
                })
            } else {
                Ok(smoooth::adapters::HealthEvidence {
                    is_healthy: true,
                    latency_ms: Some(1),
                    message: "Healthy".to_string(),
                    details: serde_json::json!({}),
                })
            }
        }
        async fn reconcile(
            &self,
            _id: &str,
            _c: &serde_json::Value,
        ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
            Ok(smoooth::models::ResourceStatus::Active)
        }
        async fn destroy(&self, _id: &str, _p: bool) -> smoooth::error::Result<()> {
            Ok(())
        }
    }

    orchestrator.register_adapter(Arc::new(FailingAdapter));

    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "ws-fail".to_string(),
        environment: "development".to_string(),
        target_id: "node-1".to_string(),
        actions: vec![
            CanonicalActionSpec {
                resource_name: "dependent_app".to_string(),
                provider: "mock_failing".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({}),
                is_persistent: false,
                exposes_network_publicly: false,
            },
            CanonicalActionSpec {
                resource_name: "failing_db".to_string(),
                provider: "mock_failing".to_string(),
                service_type: "database".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({}),
                is_persistent: true,
                exposes_network_publicly: false,
            },
        ],
        dependencies: vec![DependencyEdge {
            from: "dependent_app".to_string(),
            to: "failing_db".to_string(),
        }],
        immutable_artifacts: HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let plan = Plan::new("plan-prereq-fail".to_string(), spec, RiskTier::Low, None);
    let op = orchestrator
        .execute_plan_sync(plan, "key-fail-001".to_string())
        .await
        .unwrap();

    assert_eq!(op.status, smoooth::models::OperationStatus::Failed);
    assert!(op
        .error_message
        .unwrap()
        .contains("Health probe failed for failing_db"));

    // Critical assertion: dependent_app MUST NOT exist in storage!
    let dep_res = storage.get_resource("res_dependent_app").await.unwrap();
    assert!(
        dep_res.is_none(),
        "Dependent resource must NEVER be created when prerequisite fails!"
    );
}

#[test]
fn test_health_spec_and_container_options_serde() {
    let http_spec = HealthSpec::Http {
        path: "/healthz".to_string(),
        port: 8080,
        expected_status: 200,
    };
    let json_val = serde_json::to_value(&http_spec).unwrap();
    assert_eq!(json_val["type"], "http");
    assert_eq!(json_val["path"], "/healthz");
    assert_eq!(json_val["port"], 8080);
    assert_eq!(json_val["expected_status"], 200);

    let roundtrip: HealthSpec = serde_json::from_value(json_val).unwrap();
    assert_eq!(roundtrip, http_spec);

    let exec_spec = HealthSpec::Exec {
        cmd: vec!["redis-cli".to_string(), "ping".to_string()],
    };
    let json_exec = serde_json::to_value(&exec_spec).unwrap();
    assert_eq!(json_exec["type"], "exec");
    assert_eq!(json_exec["cmd"][0], "redis-cli");

    let liveness_spec = HealthSpec::Liveness;
    let json_live = serde_json::to_value(&liveness_spec).unwrap();
    assert_eq!(json_live["type"], "liveness");
}
