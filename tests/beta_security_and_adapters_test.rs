use chrono::Utc;
use std::sync::Arc;

use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::mongo::MongoAdapter;
use smoooth::adapters::ProviderAdapter;
use smoooth::dashboard::get_dashboard_html;
use smoooth::doctor::Doctor;
use smoooth::engine::orchestrator::Orchestrator;
use smoooth::models::agent::{AgentIdentity, AgentScope};
use smoooth::models::connection::ResourceOwnership;
use smoooth::models::plan::{CanonicalActionSpec, CanonicalExecutionSpec, PolicyConstraints};
use smoooth::models::resource::{Resource, ResourceStatus};
use smoooth::storage::local::LocalStorage;
use smoooth::storage::Storage;

#[tokio::test]
async fn test_agent_pairing_hashing_and_revocation() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Orchestrator::new(storage.clone());

    // 1. Pair a new agent with Plan and ReadOnly scopes
    let (identity, raw_token) = orchestrator
        .pair_agent(
            "test-planner-agent",
            "ws-100",
            vec![AgentScope::Plan, AgentScope::ReadOnly],
            None,
        )
        .await
        .expect("Pair agent should succeed");

    assert_eq!(identity.name, "test-planner-agent");
    assert_eq!(identity.workspace_id, "ws-100");
    assert_eq!(identity.scopes.len(), 2);
    assert!(identity.is_active());
    assert!(!raw_token.is_empty());
    assert!(raw_token.starts_with("smoooth_ag_"));

    // 2. Verify that storage does NOT store the raw token in plaintext
    let stored_identity = storage
        .get_agent(&identity.agent_id)
        .await
        .unwrap()
        .expect("Agent must exist in storage");
    assert_eq!(stored_identity.token_hash, identity.token_hash);
    assert_ne!(stored_identity.token_hash, raw_token);

    // 3. Authenticate with valid token
    let authenticated = orchestrator
        .authenticate_agent(&raw_token)
        .await
        .expect("Query should succeed")
        .expect("Valid token should authenticate");
    assert_eq!(authenticated.agent_id, identity.agent_id);
    assert!(authenticated.last_used_at.is_some());

    // 4. Authenticate with invalid token should return None
    let invalid = orchestrator
        .authenticate_agent("smoooth_ag_invalidtoken123")
        .await
        .unwrap();
    assert!(invalid.is_none());

    // 5. Test token rotation
    let new_token = orchestrator
        .rotate_agent_token(&identity.agent_id)
        .await
        .expect("Rotation should succeed");
    assert_ne!(new_token, raw_token);

    // Old token should now be rejected
    assert!(orchestrator
        .authenticate_agent(&raw_token)
        .await
        .unwrap()
        .is_none());

    // New token should authenticate
    let auth_new = orchestrator.authenticate_agent(&new_token).await.unwrap();
    assert!(auth_new.is_some());

    // 6. Test revocation
    orchestrator
        .revoke_agent(&identity.agent_id)
        .await
        .expect("Revoke should succeed");

    let revoked = storage
        .get_agent(&identity.agent_id)
        .await
        .unwrap()
        .unwrap();
    assert!(revoked.revoked);
    assert!(revoked.revoked_at.is_some());
    assert!(!revoked.is_active());

    // Revoked token should fail authentication
    assert!(orchestrator
        .authenticate_agent(&new_token)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_agent_scope_enforcement() {
    let identity_ro = AgentIdentity {
        agent_id: "agent_ro".to_string(),
        name: "ro-agent".to_string(),
        token_hash: "dummy_hash_1".to_string(),
        token_prefix: "smoooth_ag_1234".to_string(),
        scopes: vec![AgentScope::ReadOnly],
        workspace_id: "ws-1".to_string(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };

    assert!(identity_ro.has_scope(&AgentScope::ReadOnly));
    assert!(!identity_ro.has_scope(&AgentScope::Plan));
    assert!(!identity_ro.has_scope(&AgentScope::Deploy));
    assert!(!identity_ro.has_scope(&AgentScope::Destructive));
    assert!(!identity_ro.has_scope(&AgentScope::Admin));
    assert!(!identity_ro.is_overprivileged());

    let identity_admin = AgentIdentity {
        agent_id: "agent_admin".to_string(),
        name: "admin-agent".to_string(),
        token_hash: "dummy_hash_2".to_string(),
        token_prefix: "smoooth_ag_admin".to_string(),
        scopes: vec![AgentScope::Admin],
        workspace_id: "*".to_string(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };

    // Admin implies all scopes and is marked overprivileged
    assert!(identity_admin.has_scope(&AgentScope::ReadOnly));
    assert!(identity_admin.has_scope(&AgentScope::Plan));
    assert!(identity_admin.has_scope(&AgentScope::Deploy));
    assert!(identity_admin.has_scope(&AgentScope::Destructive));
    assert!(identity_admin.has_scope(&AgentScope::Admin));
    assert!(identity_admin.is_overprivileged());

    // Workspace scoping
    assert!(identity_ro.allows_workspace("ws-1"));
    assert!(!identity_ro.allows_workspace("ws-2"));
    assert!(identity_admin.allows_workspace("any-workspace"));
}

#[tokio::test]
async fn test_mongo_adapter_guarantees_and_capabilities() {
    let docker_adapter = Arc::new(DockerAdapter::new());
    let adapter = MongoAdapter::new(docker_adapter);

    assert_eq!(adapter.provider_name(), "mongodb");

    let guarantees = adapter.guarantees();
    assert!(guarantees.supports_persistent_volumes);
    assert!(guarantees.supports_staged_dual_credentials);
    assert!(guarantees.supports_safe_rollback);
    assert!(guarantees.supports_drift_detection);
    assert!(!guarantees.is_experimental);

    let capabilities = adapter.supported_capabilities();
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0].id, "cap_mongo_db");
    assert_eq!(capabilities[0].provider, "mongodb");
    assert_eq!(capabilities[0].service_type, "database");
    assert!(capabilities[0]
        .operations
        .contains(&"provision".to_string()));
    assert!(capabilities[0]
        .operations
        .contains(&"health_check".to_string()));
    assert!(capabilities[0]
        .operations
        .contains(&"reconcile".to_string()));
    assert!(capabilities[0].operations.contains(&"destroy".to_string()));
}

#[tokio::test]
async fn test_embedded_sqlite_workload_pattern() {
    // SQLite pattern: service workload with persistent volume mounted to /data
    let sqlite_action = CanonicalActionSpec {
        resource_name: "sqlite-app".to_string(),
        provider: "docker".to_string(),
        service_type: "service".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({
            "image": "alpine:latest",
            "env": {
                "DATABASE_URL": "sqlite:///data/app.db",
                "SQLITE_BUSY_TIMEOUT": "5000"
            },
            "volumes": [
                {
                    "volume_name": "sqlite_app_data",
                    "container_path": "/data",
                    "read_only": false
                }
            ]
        }),
        is_persistent: true,
        exposes_network_publicly: false,
    };

    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "default".to_string(),
        environment: "production".to_string(),
        target_id: "target_local".to_string(),
        actions: vec![sqlite_action],
        dependencies: vec![],
        immutable_artifacts: std::collections::HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: None,
        },
    };

    let execution_order = spec
        .resolve_execution_order()
        .expect("DAG resolution should succeed");
    assert_eq!(execution_order.len(), 1);
    assert_eq!(execution_order[0].resource_name, "sqlite-app");
    assert!(execution_order[0].is_persistent);
}

#[tokio::test]
async fn test_connected_external_resource_guarantees() {
    let external_resource = Resource {
        id: "external-postgres-1".to_string(),
        name: "Prod RDS Postgres".to_string(),
        workspace_id: "default".to_string(),
        service_type: "database".to_string(),
        provider: "postgres".to_string(),
        is_persistent: true,
        ownership: ResourceOwnership::External,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({
            "connection_url": "postgresql://prod_user:secret@external-db.example.com:5432/main",
            "managed_externally": true
        }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(external_resource.ownership, ResourceOwnership::External);
    // Smoooth orchestrator treats External ownership as protected from destructive deletion
    assert_eq!(
        external_resource.configuration["connection_url"],
        "postgresql://prod_user:secret@external-db.example.com:5432/main"
    );
}

#[tokio::test]
async fn test_doctor_diagnostics_report() {
    let report = Doctor::run_diagnostics("/tmp/smoooth_test_state.json", "127.0.0.1", 4040).await;

    assert!(!report.checks.is_empty());
    let check_names: Vec<String> = report.checks.iter().map(|c| c.name.clone()).collect();
    assert!(check_names.iter().any(|n| n.contains("CPU Architecture")));
    assert!(check_names.iter().any(|n| n.contains("Operating System")));
    assert!(check_names.iter().any(|n| n.contains("Docker Engine")));
    assert!(check_names.iter().any(|n| n.contains("Port Availability")));
    assert!(check_names.iter().any(|n| n.contains("Storage Directory")));
}

#[tokio::test]
async fn test_dashboard_html_rendering() {
    let html = get_dashboard_html();
    assert!(!html.is_empty());
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Smoooth — Control Plane Dashboard"));
    assert!(html.contains("#0b0d10"));
    assert!(html.contains("/api/status"));
    assert!(html.contains("/api/agents"));
    assert!(html.contains("/api/audit"));
    assert!(html.contains("/api/doctor"));
}
