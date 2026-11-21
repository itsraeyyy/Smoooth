use chrono::{Duration, Utc};
use smoooth::error::SmooothError;
use smoooth::models::{
    AgentIdentity, AgentScope, CanonicalActionSpec, CanonicalExecutionSpec, IdempotencyKey,
    IdempotencyRecord, Operation, OperationStatus, Plan, PolicyConstraints, Resource,
    ResourceOwnership, ResourceStatus, RiskTier, RotationStage, ServiceBinding,
};
use smoooth::storage::{AuditEvent, LocalStorage, OperationInitOutcome, PostgresStorage, Storage};
use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;

static CANDIDATE_HOSTS: OnceLock<Vec<String>> = OnceLock::new();

fn get_candidate_hosts() -> &'static [String] {
    CANDIDATE_HOSTS.get_or_init(|| {
        let mut hosts = Vec::new();
        if cfg!(windows) {
            if let Ok(output) = Command::new("wsl").arg("hostname").arg("-I").output() {
                if output.status.success() {
                    let ip_str = String::from_utf8_lossy(&output.stdout);
                    for ip in ip_str.split_whitespace() {
                        hosts.push(ip.trim().to_string());
                    }
                }
            }
        }
        hosts.push("127.0.0.1".to_string());
        hosts.push("localhost".to_string());
        hosts
    })
}

async fn get_test_postgres_storage() -> (PostgresStorage, String) {
    let schema_name = format!("test_pg_contract_{}", uuid::Uuid::new_v4().simple());

    if let Ok(url) = std::env::var("DATABASE_URL") {
        match PostgresStorage::new_with_schema(&url, Some(&schema_name)).await {
            Ok(storage) => return (storage, schema_name),
            Err(e) => eprintln!("Error connecting with DATABASE_URL: {:?}", e),
        }
    }

    let mut last_err = None;
    for host in get_candidate_hosts() {
        let conn_str = format!(
            "host={} port=5432 user=postgres password=postgres dbname=smoooth_test",
            host
        );
        match PostgresStorage::new_with_schema(&conn_str, Some(&schema_name)).await {
            Ok(storage) => return (storage, schema_name),
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    panic!(
        "Failed to connect to test PostgreSQL database for storage contract tests: {:?}",
        last_err
    );
}

async fn cleanup_postgres_schema(storage: &PostgresStorage, schema_name: &str) {
    if let Ok(client) = storage.get_client().await {
        let _ = client
            .batch_execute(&format!(
                "DROP SCHEMA IF EXISTS \"{}\" CASCADE;",
                schema_name
            ))
            .await;
    }
}

// =============================================================================
// Generic Behavioral Contract Tests (Exercised against both backends)
// =============================================================================

async fn contract_workspace_isolation(storage: &dyn Storage) {
    let ws_a = "ws-isolation-alpha";
    let ws_b = "ws-isolation-beta";

    // 1. Agents Isolation
    let agent_a = AgentIdentity {
        agent_id: "agent_a_001".to_string(),
        name: "agent alpha".to_string(),
        token_hash: "hash_alpha_001".to_string(),
        token_prefix: "smoooth_ag_a".to_string(),
        scopes: vec![AgentScope::Deploy],
        workspace_id: ws_a.to_string(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };
    storage.save_agent(agent_a.clone()).await.unwrap();

    let agent_b = AgentIdentity {
        agent_id: "agent_b_001".to_string(),
        name: "agent beta".to_string(),
        token_hash: "hash_beta_001".to_string(),
        token_prefix: "smoooth_ag_b".to_string(),
        scopes: vec![AgentScope::ReadOnly],
        workspace_id: ws_b.to_string(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };
    storage.save_agent(agent_b.clone()).await.unwrap();

    let listed_a = storage.list_agents(Some(ws_a)).await.unwrap();
    assert_eq!(listed_a.len(), 1);
    assert_eq!(listed_a[0].agent_id, "agent_a_001");

    let listed_b = storage.list_agents(Some(ws_b)).await.unwrap();
    assert_eq!(listed_b.len(), 1);
    assert_eq!(listed_b[0].agent_id, "agent_b_001");

    // 2. Resources Isolation
    let res_a = Resource {
        id: format!("res_{}_db", ws_a),
        name: "database".to_string(),
        workspace_id: ws_a.to_string(),
        service_type: "database".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({ "port": 5432 }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(res_a).await.unwrap();

    let res_b = Resource {
        id: format!("res_{}_db", ws_b),
        name: "database".to_string(),
        workspace_id: ws_b.to_string(),
        service_type: "database".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({ "port": 5433 }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(res_b).await.unwrap();

    let resources_a = storage.list_resources(ws_a).await.unwrap();
    assert_eq!(resources_a.len(), 1);
    assert_eq!(resources_a[0].id, format!("res_{}_db", ws_a));

    let resources_b = storage.list_resources(ws_b).await.unwrap();
    assert_eq!(resources_b.len(), 1);
    assert_eq!(resources_b[0].id, format!("res_{}_db", ws_b));

    // 3. Audit Events Isolation
    storage
        .record_audit(AuditEvent {
            id: 0,
            workspace_id: ws_a.to_string(),
            actor: "operator".to_string(),
            action: "deploy".to_string(),
            target_resource_id: Some("res_a".to_string()),
            plan_hash: None,
            outcome: "success".to_string(),
            redacted_evidence: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();

    storage
        .record_audit(AuditEvent {
            id: 0,
            workspace_id: ws_b.to_string(),
            actor: "operator".to_string(),
            action: "destroy".to_string(),
            target_resource_id: Some("res_b".to_string()),
            plan_hash: None,
            outcome: "success".to_string(),
            redacted_evidence: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();

    let audit_a = storage.list_audit_events(ws_a).await.unwrap();
    assert_eq!(audit_a.len(), 1);
    assert_eq!(audit_a[0].workspace_id, ws_a);

    let audit_b = storage.list_audit_events(ws_b).await.unwrap();
    assert_eq!(audit_b.len(), 1);
    assert_eq!(audit_b[0].workspace_id, ws_b);
}

async fn contract_canonical_resource_identity_retention(storage: &dyn Storage) {
    let ws = "ws-canonical-id-test";
    let canonical_id = format!("res_{}_postgres", ws);

    // Initial provision of postgres resource
    let r1 = Resource {
        id: canonical_id.clone(),
        name: "postgres".to_string(),
        workspace_id: ws.to_string(),
        service_type: "database".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({ "version": "16" }),
        created_at: Utc::now() - Duration::hours(1),
        updated_at: Utc::now() - Duration::hours(1),
    };

    let saved1 = storage.save_resource(r1.clone()).await.unwrap();
    assert_eq!(saved1.id, canonical_id);

    // Second provision with a different random/synthesized ID but matching (workspace_id, name)
    let r2 = Resource {
        id: "res_fabricated_random_id_999".to_string(),
        name: "postgres".to_string(),
        workspace_id: ws.to_string(),
        service_type: "database".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({ "version": "17" }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let saved2 = storage.save_resource(r2).await.unwrap();

    // MUST retain the canonical ID and original creation timestamp
    assert_eq!(
        saved2.id, canonical_id,
        "Storage backend must retain canonical resource ID on (workspace_id, name) collision"
    );
    assert_eq!(
        saved2.created_at.timestamp_micros(),
        saved1.created_at.timestamp_micros(),
        "Original created_at timestamp must be preserved across updates"
    );

    // Verify lookup by canonical ID reflects updated configuration
    let retrieved = storage.get_resource(&canonical_id).await.unwrap().unwrap();
    assert_eq!(
        retrieved
            .configuration
            .get("version")
            .and_then(|v| v.as_str()),
        Some("17")
    );
}

async fn contract_cross_workspace_binding_rejection(storage: &dyn Storage) {
    let ws_a = "ws-bind-src";
    let ws_b = "ws-bind-target";

    let src = Resource {
        id: format!("res_{}_app", ws_a),
        name: "app".to_string(),
        workspace_id: ws_a.to_string(),
        service_type: "service".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(src.clone()).await.unwrap();

    let target = Resource {
        id: format!("res_{}_db", ws_b),
        name: "db".to_string(),
        workspace_id: ws_b.to_string(),
        service_type: "database".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(target.clone()).await.unwrap();

    // Attempt to bind across workspace boundaries: source in ws_a, target in ws_b
    let binding = ServiceBinding {
        id: "binding_cross_001".to_string(),
        workspace_id: ws_a.to_string(),
        source_resource_id: src.id.clone(),
        target_resource_id: target.id.clone(),
        binding_type: "postgres_connection".to_string(),
        rotation_stage: RotationStage::Active,
        ownership: ResourceOwnership::SmooothManaged,
        binding_version: 1,
        key_fingerprint: "fp_test_123".to_string(),
        resolved_endpoint: None,
        safe_connection_details: None,
        encrypted_credentials: vec![1, 2, 3, 4],
        exposed_env_keys: vec!["DATABASE_URL".to_string()],
        env_mappings: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let result = storage.save_binding(binding).await;
    assert!(
        result.is_err(),
        "Storage must reject cross-workspace service bindings"
    );
}

async fn contract_atomic_operation_reservation_and_idempotency(storage: &dyn Storage) {
    let ws = "ws-atomic-idemp";
    let key = IdempotencyKey {
        workspace_id: ws.to_string(),
        operation_type: "execute_plan".to_string(),
        client_key: "idemp_client_key_42".to_string(),
    };

    let op = Operation {
        id: "op_reservation_001".to_string(),
        workspace_id: ws.to_string(),
        operation_type: "execute_plan".to_string(),
        plan_id: "plan_42".to_string(),
        plan_hash: "hash_plan_42".to_string(),
        request_hash: "req_hash_alpha".to_string(),
        idempotency_client_key: "idemp_client_key_42".to_string(),
        status: OperationStatus::Running,
        progress_percent: 0,
        lease_epoch: 1,
        lease_owner: Some("worker-1".to_string()),
        lease_expires_at: Some(Utc::now() + Duration::minutes(5)),
        intended_state: serde_json::json!({}),
        observed_state: serde_json::json!({}),
        logs: vec![],
        evidence: None,
        error_message: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let record = IdempotencyRecord {
        key: key.clone(),
        request_hash: "req_hash_alpha".to_string(),
        operation_id: op.id.clone(),
        created_at: Utc::now(),
    };

    // 1. Initial reservation -> Created
    let outcome1 = storage
        .reserve_or_get_operation(record.clone(), op.clone())
        .await
        .unwrap();
    assert_eq!(outcome1, OperationInitOutcome::Created(op.clone()));

    // 2. Re-invoking with matching request hash -> Existing
    let outcome2 = storage
        .reserve_or_get_operation(record.clone(), op.clone())
        .await
        .unwrap();
    match outcome2 {
        OperationInitOutcome::Existing(existing_op) => {
            assert_eq!(existing_op.id, op.id);
            assert_eq!(existing_op.workspace_id, op.workspace_id);
            assert_eq!(existing_op.operation_type, op.operation_type);
            assert_eq!(existing_op.request_hash, op.request_hash);
            assert_eq!(
                existing_op.idempotency_client_key,
                op.idempotency_client_key
            );
            assert_eq!(
                existing_op.created_at.timestamp_micros(),
                op.created_at.timestamp_micros()
            );
        }
        other => panic!("Expected OperationInitOutcome::Existing, got {:?}", other),
    }

    // 3. Re-invoking with mismatched request hash -> IdempotencyConflict
    let conflicting_record = IdempotencyRecord {
        key,
        request_hash: "req_hash_DIFFERENT".to_string(),
        operation_id: "op_another_999".to_string(),
        created_at: Utc::now(),
    };

    let conflict_err = storage
        .reserve_or_get_operation(conflicting_record, op.clone())
        .await
        .unwrap_err();

    match conflict_err {
        SmooothError::IdempotencyConflict(_) => (),
        other => panic!("Expected IdempotencyConflict, got {:?}", other),
    }
}

async fn contract_atomic_nonce_consumption(storage: &dyn Storage) {
    let nonce = format!("nonce_{}", uuid::Uuid::new_v4().simple());
    let future_exp = Utc::now() + Duration::minutes(10);
    let past_exp = Utc::now() - Duration::seconds(5);

    // 1. Expired nonce must fail consumption immediately
    let expired_result = storage.consume_nonce(&nonce, past_exp).await.unwrap();
    assert!(!expired_result, "Expired nonce must be rejected");

    // 2. First consumption of valid nonce must succeed
    let consumed_first = storage.consume_nonce(&nonce, future_exp).await.unwrap();
    assert!(consumed_first, "First consumption of nonce must succeed");

    // 3. Second consumption of same nonce must fail (replay attack prevented)
    let consumed_second = storage.consume_nonce(&nonce, future_exp).await.unwrap();
    assert!(
        !consumed_second,
        "Replay of already consumed nonce must return false"
    );

    // 4. is_nonce_used check must return true
    let is_used = storage.is_nonce_used(&nonce).await.unwrap();
    assert!(is_used, "Consumed nonce must be reported as used");
}

async fn contract_fencing_epoch_monotonicity(storage: &dyn Storage) {
    let ws = "ws-fencing-epoch-contract";

    // Default epoch for fresh workspace must be 1
    let initial_epoch = storage.get_fencing_epoch(ws).await.unwrap();
    assert_eq!(initial_epoch, 1);

    // Monotonic increments
    let epoch2 = storage.increment_fencing_epoch(ws).await.unwrap();
    assert_eq!(epoch2, 2);

    let epoch3 = storage.increment_fencing_epoch(ws).await.unwrap();
    assert_eq!(epoch3, 3);

    let current = storage.get_fencing_epoch(ws).await.unwrap();
    assert_eq!(current, 3);
}

async fn contract_batch_binding_rotation_fingerprint_verification(storage: &dyn Storage) {
    let ws = "ws-rotation-contract";
    let src = Resource {
        id: format!("res_{}_web", ws),
        name: "web".to_string(),
        workspace_id: ws.to_string(),
        service_type: "service".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(src.clone()).await.unwrap();

    let target = Resource {
        id: format!("res_{}_cache", ws),
        name: "cache".to_string(),
        workspace_id: ws.to_string(),
        service_type: "cache".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(target.clone()).await.unwrap();

    let binding_id = format!("binding_{}_001", ws);
    let initial_binding = ServiceBinding {
        id: binding_id.clone(),
        workspace_id: ws.to_string(),
        source_resource_id: src.id.clone(),
        target_resource_id: target.id.clone(),
        binding_type: "redis_connection".to_string(),
        rotation_stage: RotationStage::Active,
        ownership: ResourceOwnership::SmooothManaged,
        binding_version: 1,
        key_fingerprint: "fingerprint_v1".to_string(),
        resolved_endpoint: None,
        safe_connection_details: None,
        encrypted_credentials: vec![10, 20, 30],
        exposed_env_keys: vec!["REDIS_URL".to_string()],
        env_mappings: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_binding(initial_binding.clone()).await.unwrap();

    // 1. Mismatched expected old fingerprint -> Must fail closed
    let mut updated_binding_bad = initial_binding.clone();
    updated_binding_bad.key_fingerprint = "fingerprint_v2".to_string();

    let audit_event = AuditEvent {
        id: 0,
        workspace_id: ws.to_string(),
        actor: "key_rotator".to_string(),
        action: "rotate_keys".to_string(),
        target_resource_id: Some(binding_id.clone()),
        plan_hash: None,
        outcome: "success".to_string(),
        redacted_evidence: None,
        timestamp: Utc::now(),
    };

    let mismatch_err = storage
        .rotate_service_bindings_batch(
            "WRONG_OLD_FINGERPRINT",
            "fingerprint_v2",
            vec![updated_binding_bad],
            audit_event.clone(),
        )
        .await;
    assert!(
        mismatch_err.is_err(),
        "Batch rotation must fail closed if old fingerprint does not match"
    );

    // 2. Matching old fingerprint and correct new fingerprint -> Must succeed
    let mut updated_binding_good = initial_binding;
    updated_binding_good.key_fingerprint = "fingerprint_v2".to_string();
    updated_binding_good.encrypted_credentials = vec![40, 50, 60];

    let success_count = storage
        .rotate_service_bindings_batch(
            "fingerprint_v1",
            "fingerprint_v2",
            vec![updated_binding_good],
            audit_event,
        )
        .await
        .unwrap();

    assert_eq!(success_count, 1);

    // Verify updated fingerprint and payload in storage
    let fetched = storage.get_binding(&binding_id).await.unwrap().unwrap();
    assert_eq!(fetched.key_fingerprint, "fingerprint_v2");
    assert_eq!(fetched.encrypted_credentials, vec![40, 50, 60]);
}

async fn contract_capability_revocation_registry(storage: &dyn Storage) {
    let cap_id = uuid::Uuid::new_v4();
    let unrevoked_id = uuid::Uuid::new_v4();

    assert!(!storage.is_revoked(&cap_id).await.unwrap());
    assert!(!storage.is_revoked(&unrevoked_id).await.unwrap());

    storage
        .record_revocation(
            &cap_id,
            "compromised token",
            Utc::now() + Duration::hours(24),
        )
        .await
        .unwrap();

    assert!(storage.is_revoked(&cap_id).await.unwrap());
    assert!(!storage.is_revoked(&unrevoked_id).await.unwrap());
}

async fn contract_snapshot_export_import_roundtrip(
    source: &dyn Storage,
    destination: &dyn Storage,
) {
    let ws = "ws-snapshot-roundtrip";

    let agent = AgentIdentity {
        agent_id: "agent_snap_01".to_string(),
        name: "snapshot agent".to_string(),
        token_hash: "hash_snap_01".to_string(),
        token_prefix: "smoooth_ag_s".to_string(),
        scopes: vec![AgentScope::Plan, AgentScope::Deploy],
        workspace_id: ws.to_string(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };
    source.save_agent(agent).await.unwrap();

    let plan = Plan::new(
        "plan_snap_01".to_string(),
        CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: ws.to_string(),
            environment: "development".to_string(),
            target_id: "cluster-1".to_string(),
            actions: vec![CanonicalActionSpec {
                resource_name: "queue".to_string(),
                provider: "docker".to_string(),
                service_type: "queue".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({}),
                is_persistent: true,
                exposes_network_publicly: false,
            }],
            dependencies: vec![],
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: false,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(50),
            },
        },
        RiskTier::Low,
        Some(10.0),
    );
    source.save_plan(plan).await.unwrap();

    let res = Resource {
        id: format!("res_{}_queue", ws),
        name: "queue".to_string(),
        workspace_id: ws.to_string(),
        service_type: "queue".to_string(),
        provider: "docker".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    source.save_resource(res).await.unwrap();

    let initial_epoch = source.get_fencing_epoch(ws).await.unwrap();
    let expected_epoch = source.increment_fencing_epoch(ws).await.unwrap();
    assert_eq!(expected_epoch, initial_epoch + 1);

    // Export snapshot from source
    let snapshot = source.export_snapshot(ws).await.unwrap();
    assert_eq!(snapshot.agents.len(), 1);
    assert_eq!(snapshot.plans.len(), 1);
    assert_eq!(snapshot.resources.len(), 1);

    // Import into destination
    destination.import_snapshot(snapshot).await.unwrap();

    // Verify entities are present in destination
    let dest_agents = destination.list_agents(Some(ws)).await.unwrap();
    assert_eq!(dest_agents.len(), 1);
    assert_eq!(dest_agents[0].agent_id, "agent_snap_01");

    let dest_plans = destination.list_plans().await.unwrap();
    assert!(dest_plans.iter().any(|p| p.id == "plan_snap_01"));

    let dest_res = destination.list_resources(ws).await.unwrap();
    assert_eq!(dest_res.len(), 1);
    assert_eq!(dest_res[0].name, "queue");

    let dest_epoch = destination.get_fencing_epoch(ws).await.unwrap();
    assert_eq!(dest_epoch, expected_epoch);
}

// =============================================================================
// LocalStorage Contract Test Harness
// =============================================================================

#[tokio::test]
async fn test_local_storage_workspace_isolation() {
    let storage = LocalStorage::new_in_memory();
    contract_workspace_isolation(&storage).await;
}

#[tokio::test]
async fn test_local_storage_canonical_resource_identity_retention() {
    let storage = LocalStorage::new_in_memory();
    contract_canonical_resource_identity_retention(&storage).await;
}

#[tokio::test]
async fn test_local_storage_cross_workspace_binding_rejection() {
    let storage = LocalStorage::new_in_memory();
    contract_cross_workspace_binding_rejection(&storage).await;
}

#[tokio::test]
async fn test_local_storage_atomic_operation_reservation_and_idempotency() {
    let storage = LocalStorage::new_in_memory();
    contract_atomic_operation_reservation_and_idempotency(&storage).await;
}

#[tokio::test]
async fn test_local_storage_atomic_nonce_consumption() {
    let storage = LocalStorage::new_in_memory();
    contract_atomic_nonce_consumption(&storage).await;
}

#[tokio::test]
async fn test_local_storage_fencing_epoch_monotonicity() {
    let storage = LocalStorage::new_in_memory();
    contract_fencing_epoch_monotonicity(&storage).await;
}

#[tokio::test]
async fn test_local_storage_batch_binding_rotation() {
    let storage = LocalStorage::new_in_memory();
    contract_batch_binding_rotation_fingerprint_verification(&storage).await;
}

#[tokio::test]
async fn test_local_storage_capability_revocation_registry() {
    let storage = LocalStorage::new_in_memory();
    contract_capability_revocation_registry(&storage).await;
}

#[tokio::test]
async fn test_local_storage_snapshot_roundtrip() {
    let src = LocalStorage::new_in_memory();
    let dest = LocalStorage::new_in_memory();
    contract_snapshot_export_import_roundtrip(&src, &dest).await;
}

// =============================================================================
// PostgresStorage Contract Test Harness
// =============================================================================

#[tokio::test]
async fn test_postgres_storage_workspace_isolation() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_workspace_isolation(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_canonical_resource_identity_retention() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_canonical_resource_identity_retention(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_cross_workspace_binding_rejection() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_cross_workspace_binding_rejection(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_atomic_operation_reservation_and_idempotency() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_atomic_operation_reservation_and_idempotency(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_atomic_nonce_consumption() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_atomic_nonce_consumption(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_fencing_epoch_monotonicity() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_fencing_epoch_monotonicity(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_batch_binding_rotation() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_batch_binding_rotation_fingerprint_verification(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_capability_revocation_registry() {
    let (storage, schema) = get_test_postgres_storage().await;
    contract_capability_revocation_registry(&storage).await;
    cleanup_postgres_schema(&storage, &schema).await;
}

#[tokio::test]
async fn test_postgres_storage_snapshot_roundtrip() {
    let (src, schema_src) = get_test_postgres_storage().await;
    let (dest, schema_dest) = get_test_postgres_storage().await;
    contract_snapshot_export_import_roundtrip(&src, &dest).await;
    cleanup_postgres_schema(&src, &schema_src).await;
    cleanup_postgres_schema(&dest, &schema_dest).await;
}

// Cross-Backend Snapshot Interoperability: Local -> Postgres and Postgres -> Local
#[tokio::test]
async fn test_cross_backend_snapshot_interoperability() {
    let local = LocalStorage::new_in_memory();
    let (pg, schema_pg) = get_test_postgres_storage().await;

    // 1. LocalStorage -> PostgresStorage
    contract_snapshot_export_import_roundtrip(&local, &pg).await;

    // 2. PostgresStorage -> LocalStorage
    let local_dest = LocalStorage::new_in_memory();
    contract_snapshot_export_import_roundtrip(&pg, &local_dest).await;

    cleanup_postgres_schema(&pg, &schema_pg).await;
}
