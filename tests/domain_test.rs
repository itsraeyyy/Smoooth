use std::collections::HashMap;
use std::sync::Arc;

use smoooth::engine::Orchestrator;
use smoooth::models::{
    CanonicalActionSpec, CanonicalExecutionSpec, Plan, PolicyConstraints, RiskTier,
    ServiceBinding,
};
use smoooth::storage::LocalStorage;

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
