use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

use smoooth::adapters::docker::containers::ContainerManager;
use smoooth::engine::Orchestrator;
use smoooth::models::{
    CanonicalActionSpec, CanonicalExecutionSpec, ConnectionDetails, Endpoint, OperationStatus,
    Plan, PolicyConstraints, Resource, ResourceOwnership, ResourceStatus, RiskTier, ServiceBinding,
    VolumeMount,
};
use smoooth::security::SecretStore;
use smoooth::storage::{LocalStorage, Storage};

#[test]
fn test_connection_details_uri_generation() {
    // 1. PostgreSQL URI
    let mut pg_params = HashMap::new();
    pg_params.insert("sslmode".to_string(), "disable".to_string());
    let pg = ConnectionDetails {
        protocol: "postgresql".to_string(),
        host: "db.internal".to_string(),
        port: 5432,
        path_or_database: Some("production_db".to_string()),
        username: Some("smoooth_admin".to_string()),
        password: Some("super_secret_pg_pass".to_string()),
        extra_params: pg_params,
    };
    let pg_unmasked = pg.to_uri(false);
    assert_eq!(
        pg_unmasked,
        "postgresql://smoooth_admin:super_secret_pg_pass@db.internal:5432/production_db?sslmode=disable"
    );
    let pg_masked = pg.to_uri(true);
    assert!(pg_masked.contains("[REDACTED]"));
    assert!(!pg_masked.contains("super_secret_pg_pass"));

    // 2. Redis URI
    let redis = ConnectionDetails {
        protocol: "redis".to_string(),
        host: "redis-cluster.local".to_string(),
        port: 6379,
        path_or_database: Some("0".to_string()),
        username: None,
        password: Some("redis_auth_token".to_string()),
        extra_params: HashMap::new(),
    };
    let redis_masked = redis.to_uri(true);
    assert_eq!(redis_masked, "redis://redis-cluster.local:6379/0");

    // 3. HTTP / REST API endpoint
    let http = ConnectionDetails {
        protocol: "http".to_string(),
        host: "127.0.0.1".to_string(),
        port: 8080,
        path_or_database: Some("api/v1".to_string()),
        username: None,
        password: None,
        extra_params: HashMap::new(),
    };
    assert_eq!(http.to_uri(false), "http://127.0.0.1:8080/api/v1");

    // 4. MongoDB URI
    let mongo = ConnectionDetails {
        protocol: "mongodb".to_string(),
        host: "mongo.svc".to_string(),
        port: 27017,
        path_or_database: Some("analytics".to_string()),
        username: Some("app".to_string()),
        password: Some("secret123".to_string()),
        extra_params: HashMap::new(),
    };
    assert_eq!(
        mongo.to_uri(false),
        "mongodb://app:secret123@mongo.svc:27017/analytics"
    );
}

#[test]
fn test_connection_details_to_env_map() {
    let conn = ConnectionDetails {
        protocol: "postgres".to_string(),
        host: "db-node-1".to_string(),
        port: 5432,
        path_or_database: Some("smoooth_app".to_string()),
        username: Some("app_user".to_string()),
        password: Some("secret_p@ss".to_string()),
        extra_params: HashMap::new(),
    };

    // Unprefixed mapping
    let env_unprefixed = conn.to_env_map(None, false);
    assert_eq!(env_unprefixed.get("HOST").unwrap(), "db-node-1");
    assert_eq!(env_unprefixed.get("PORT").unwrap(), "5432");
    assert_eq!(env_unprefixed.get("DATABASE").unwrap(), "smoooth_app");
    assert_eq!(env_unprefixed.get("USER").unwrap(), "app_user");
    assert_eq!(env_unprefixed.get("PASSWORD").unwrap(), "secret_p@ss");
    assert_eq!(
        env_unprefixed.get("URL").unwrap(),
        "postgres://app_user:secret_p@ss@db-node-1:5432/smoooth_app"
    );

    // Prefixed mapping with masked password
    let env_prefixed = conn.to_env_map(Some("PG"), true);
    assert_eq!(env_prefixed.get("PG_HOST").unwrap(), "db-node-1");
    assert_eq!(env_prefixed.get("PG_PORT").unwrap(), "5432");
    assert_eq!(env_prefixed.get("PG_PASSWORD").unwrap(), "[REDACTED]");
    assert!(env_prefixed.get("PG_URL").unwrap().contains("[REDACTED]"));
}

#[test]
fn test_volume_mount_read_only_semantics() {
    let rw_mount = VolumeMount {
        volume_name: "smoooth_data_vol".to_string(),
        container_path: "/var/lib/data".to_string(),
        read_only: false,
    };
    assert_eq!(rw_mount.to_docker_bind(), "smoooth_data_vol:/var/lib/data");

    let ro_mount = VolumeMount {
        volume_name: "smoooth_config_vol".to_string(),
        container_path: "/etc/app/config".to_string(),
        read_only: true,
    };
    assert_eq!(
        ro_mount.to_docker_bind(),
        "smoooth_config_vol:/etc/app/config:ro"
    );
}

#[test]
fn test_label_scrubbing_redaction() {
    let mut labels = HashMap::new();
    labels.insert("smoooth.workspace".to_string(), "prod-app".to_string());
    labels.insert("smoooth.service".to_string(), "web-api".to_string());
    labels.insert(
        "smoooth.db_password".to_string(),
        "dont_leak_me".to_string(),
    );
    labels.insert("app.secret_key".to_string(), "jwt_secret_token".to_string());
    labels.insert("api_token".to_string(), "bearer_abc_123".to_string());
    labels.insert("safe_env_flag".to_string(), "enabled".to_string());

    let scrubbed = ContainerManager::filter_safe_labels(&labels);
    assert_eq!(scrubbed.len(), 3);
    assert_eq!(scrubbed.get("smoooth.workspace").unwrap(), "prod-app");
    assert_eq!(scrubbed.get("smoooth.service").unwrap(), "web-api");
    assert_eq!(scrubbed.get("safe_env_flag").unwrap(), "enabled");
    assert!(!scrubbed.contains_key("smoooth.db_password"));
    assert!(!scrubbed.contains_key("app.secret_key"));
    assert!(!scrubbed.contains_key("api_token"));
}

#[tokio::test]
async fn test_safe_credential_encryption_and_decryption() {
    let secret_store = SecretStore::default_local();
    let conn = ConnectionDetails {
        protocol: "postgresql".to_string(),
        host: "postgres-host".to_string(),
        port: 5432,
        path_or_database: Some("customer_db".to_string()),
        username: Some("admin".to_string()),
        password: Some("super_classified_pass_999!".to_string()),
        extra_params: HashMap::new(),
    };

    let mut env_map = HashMap::new();
    env_map.insert("DATABASE_URL".to_string(), conn.to_uri(false));

    let binding = ServiceBinding::new_dynamic(
        "bind-dyn-1".to_string(),
        "app-service".to_string(),
        "postgres-service".to_string(),
        "postgres_connection".to_string(),
        ResourceOwnership::SmooothManaged,
        Some(Endpoint::Internal {
            host: "postgres-host".to_string(),
            port: 5432,
        }),
        conn.clone(),
        &secret_store,
        env_map,
    )
    .expect("Failed to create dynamic service binding");

    // Safe details has stripped password
    assert!(binding.safe_connection_details.is_some());
    assert_eq!(
        binding.safe_connection_details.as_ref().unwrap().password,
        None
    );

    // Decrypt credentials
    let decrypted = binding
        .decrypt_connection_details(&secret_store)
        .expect("Decryption must succeed");
    assert_eq!(
        decrypted.password,
        Some("super_classified_pass_999!".to_string())
    );
    assert_eq!(decrypted.host, "postgres-host");
    assert_eq!(decrypted.port, 5432);
}

#[tokio::test]
async fn test_dynamic_env_injection_bindings() {
    let storage: Arc<dyn Storage> = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());

    // Register a dependency resource: redis cache
    let redis_res = Resource {
        id: "redis-cache-1".to_string(),
        name: "redis_cache".to_string(),
        service_type: "cache".to_string(),
        provider: "docker".to_string(),
        workspace_id: "ws-demo".to_string(),
        status: ResourceStatus::Active,
        ownership: ResourceOwnership::SmooothManaged,
        configuration: serde_json::json!({
            "protocol": "redis",
            "host": "redis-cache-1.internal",
            "port": 6379,
            "password": "redis_secure_pass"
        }),
        is_persistent: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(redis_res).await.unwrap();

    // Consumer action with declared binding requirement
    let mut action = CanonicalActionSpec {
        resource_name: "web-app".to_string(),
        provider: "docker".to_string(),
        service_type: "web".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({
            "image": "my-app:1.0",
            "env": {
                "APP_ENV": "staging",
                "PORT": "3000"
            },
            "bindings": [
                {
                    "resource": "redis_cache",
                    "env_prefix": "CACHE"
                }
            ]
        }),
        is_persistent: false,
        exposes_network_publicly: true,
    };

    Orchestrator::inject_bindings(
        &storage,
        &secret_store,
        &mut action,
        &["redis_cache".to_string()],
        "ws-demo",
    )
    .await
    .expect("Binding injection must succeed");

    let env = action.config.get("env").expect("Config must have env");
    let env_obj = env.as_object().expect("Env must be JSON object");

    // Pre-existing environment preserved
    assert_eq!(env_obj.get("APP_ENV").unwrap().as_str().unwrap(), "staging");
    assert_eq!(env_obj.get("PORT").unwrap().as_str().unwrap(), "3000");

    // Dynamic bindings injected
    assert_eq!(
        env_obj.get("CACHE_HOST").unwrap().as_str().unwrap(),
        "redis-cache-1.internal"
    );
    assert_eq!(env_obj.get("CACHE_PORT").unwrap().as_str().unwrap(), "6379");
    assert_eq!(
        env_obj.get("CACHE_PASSWORD").unwrap().as_str().unwrap(),
        "redis_secure_pass"
    );
    assert!(env_obj
        .get("CACHE_URL")
        .unwrap()
        .as_str()
        .unwrap()
        .starts_with("redis://"));
}

#[tokio::test]
async fn test_custom_env_mappings_injection() {
    let storage: Arc<dyn Storage> = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());

    let db_res = Resource {
        id: "db-res-custom".to_string(),
        name: "custom_postgres".to_string(),
        service_type: "database".to_string(),
        provider: "postgres".to_string(),
        workspace_id: "ws-custom".to_string(),
        status: ResourceStatus::Active,
        ownership: ResourceOwnership::SmooothManaged,
        configuration: serde_json::json!({
            "protocol": "postgresql",
            "host": "pg.internal",
            "port": 5432,
            "database": "myapp_db",
            "user": "appuser",
            "password": "apppassword"
        }),
        is_persistent: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(db_res).await.unwrap();

    let mut action = CanonicalActionSpec {
        resource_name: "custom-consumer".to_string(),
        provider: "docker".to_string(),
        service_type: "web".to_string(),
        action_type: "create".to_string(),
        config: serde_json::json!({
            "image": "node:18",
            "bindings": [
                {
                    "resource": "custom_postgres",
                    "env_mappings": {
                        "url": "DATABASE_CONNECTION_STRING",
                        "host": "SQL_HOST",
                        "database": "SQL_DB"
                    }
                }
            ]
        }),
        is_persistent: false,
        exposes_network_publicly: false,
    };

    Orchestrator::inject_bindings(
        &storage,
        &secret_store,
        &mut action,
        &["custom_postgres".to_string()],
        "ws-custom",
    )
    .await
    .expect("Binding injection must succeed");

    let env = action.config.get("env").expect("Config must have env");
    let env_obj = env.as_object().unwrap();

    assert_eq!(
        env_obj
            .get("DATABASE_CONNECTION_STRING")
            .unwrap()
            .as_str()
            .unwrap(),
        "postgresql://appuser:apppassword@pg.internal:5432/myapp_db"
    );
    assert_eq!(
        env_obj.get("SQL_HOST").unwrap().as_str().unwrap(),
        "pg.internal"
    );
    assert_eq!(env_obj.get("SQL_DB").unwrap().as_str().unwrap(), "myapp_db");
}

#[tokio::test]
async fn test_cross_workspace_isolation_rejection() {
    let storage: Arc<dyn Storage> = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());

    // Resource belongs to ws-tenant-alpha
    let private_db = Resource {
        id: "alpha-db-id".to_string(),
        name: "alpha_db".to_string(),
        service_type: "database".to_string(),
        provider: "postgres".to_string(),
        workspace_id: "ws-tenant-alpha".to_string(),
        status: ResourceStatus::Active,
        ownership: ResourceOwnership::SmooothManaged,
        configuration: serde_json::json!({
            "protocol": "postgresql",
            "host": "alpha-db.lan",
            "port": 5432
        }),
        is_persistent: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(private_db).await.unwrap();

    // Consumer in ws-tenant-beta attempts to resolve alpha_db
    let result = Orchestrator::resolve_connection_for_resource(
        &storage,
        &secret_store,
        "alpha_db",
        None,
        "ws-tenant-beta",
    )
    .await;

    assert!(
        result.is_err(),
        "Cross-workspace resolution must be rejected"
    );
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("Cross-workspace boundary violation") || err_str.contains("not found"),
        "Error message must indicate workspace isolation violation: {}",
        err_str
    );
}

#[tokio::test]
async fn test_external_resource_safe_cross_workspace_and_delete_protection() {
    let storage: Arc<dyn Storage> = Arc::new(LocalStorage::new_in_memory());
    let secret_store = Arc::new(SecretStore::default_local());
    let orchestrator = Orchestrator::new(storage.clone()).with_secret_store(secret_store.clone());

    // External database (e.g. AWS RDS or external host database)
    let external_db = Resource {
        id: "external-rds-pg".to_string(),
        name: "external_rds".to_string(),
        service_type: "database".to_string(),
        provider: "postgres".to_string(),
        workspace_id: "shared-infra".to_string(),
        status: ResourceStatus::Active,
        ownership: ResourceOwnership::External, // Marked External!
        configuration: serde_json::json!({
            "protocol": "postgresql",
            "host": "rds.us-east-1.amazonaws.com",
            "port": 5432,
            "database": "rds_prod_db",
            "user": "rds_user",
            "password": "rds_password_123"
        }),
        is_persistent: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage.save_resource(external_db).await.unwrap();

    // 1. External resource CAN be safely bound across workspaces
    let resolved = Orchestrator::resolve_connection_for_resource(
        &storage,
        &secret_store,
        "external_rds",
        None,
        "app-workspace",
    )
    .await
    .expect("External resource resolution across workspaces must succeed");
    assert_eq!(resolved.host, "rds.us-east-1.amazonaws.com");

    // 2. Reconciliation MUST bypass external resources without modifying state
    let rec_outcome = orchestrator
        .reconcile_resource("external-rds-pg")
        .await
        .expect("Reconcile should succeed with bypass");
    assert!(!rec_outcome.is_drifted);
    assert!(!rec_outcome.drift_corrected);
    assert!(rec_outcome.message.contains("externally managed"));

    // 3. Attempting to DELETE an external resource MUST fail with PolicyViolation
    let mut delete_plan = Plan::new(
        "plan-delete-external".to_string(),
        CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: "shared-infra".to_string(),
            environment: "staging".to_string(),
            target_id: "default".to_string(),
            actions: vec![CanonicalActionSpec {
                resource_name: "external-rds-pg".to_string(),
                provider: "postgres".to_string(),
                service_type: "database".to_string(),
                action_type: "delete".to_string(),
                config: serde_json::json!({}),
                is_persistent: true,
                exposes_network_publicly: false,
            }],
            dependencies: vec![],
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: false,
                allow_data_destruction: true, // Even if data destruction is true, external resources are protected!
                max_monthly_budget_usd: None,
            },
        },
        RiskTier::Low,
        None,
    );
    delete_plan.approval = Some(smoooth::models::PlanApproval {
        plan_hash: delete_plan.plan_hash.clone(),
        workspace_id: "shared-infra".to_string(),
        environment: "staging".to_string(),
        policy_version: 1,
        approved_by: "sec_admin".to_string(),
        approved_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    });
    delete_plan.approved_plan_hash = Some(delete_plan.plan_hash.clone());
    delete_plan.approved_by = Some("sec_admin".to_string());

    let op = orchestrator
        .execute_plan_sync(delete_plan, "idemp-ext-del".to_string())
        .await
        .expect("execute_plan_sync must return operation");

    assert_eq!(op.status, OperationStatus::Failed);
    assert!(
        op.error_message
            .unwrap()
            .contains("External resource 'external-rds-pg' cannot be deleted by Smoooth"),
        "External resource must be strictly protected from deletion"
    );

    // Verify external resource still exists intact in storage
    let still_exists = storage.get_resource("external-rds-pg").await.unwrap();
    assert!(
        still_exists.is_some(),
        "External resource must NOT be removed from storage"
    );
}

#[test]
fn test_explicit_label_allowlist_and_value_sanitization() {
    let mut labels = HashMap::new();
    labels.insert("smoooth.workspace".to_string(), "prod-ws".to_string());
    labels.insert("app.name".to_string(), "my-billing-service".to_string());
    labels.insert("app.version".to_string(), "2.4.1".to_string());
    labels.insert(
        "org.opencontainers.image.title".to_string(),
        "billing-app".to_string(),
    );

    // Non-allowlisted arbitrary labels (must be dropped)
    labels.insert("random_tag".to_string(), "random_value".to_string());
    labels.insert("internal_data".to_string(), "cluster-123".to_string());

    // Allowlisted keys with sensitive value patterns (must be dropped)
    labels.insert(
        "app.service".to_string(),
        "super_secret_password_here".to_string(),
    );

    let filtered = ContainerManager::filter_safe_labels(&labels);
    assert_eq!(filtered.len(), 4);
    assert_eq!(filtered.get("smoooth.workspace").unwrap(), "prod-ws");
    assert_eq!(filtered.get("app.name").unwrap(), "my-billing-service");
    assert_eq!(filtered.get("app.version").unwrap(), "2.4.1");
    assert_eq!(
        filtered.get("org.opencontainers.image.title").unwrap(),
        "billing-app"
    );

    assert!(!filtered.contains_key("random_tag"));
    assert!(!filtered.contains_key("internal_data"));
    assert!(
        !filtered.contains_key("app.service"),
        "Sensitive value must trigger label omission"
    );
}

#[test]
fn test_external_endpoint_ssrf_and_protocol_validation() {
    // 1. AWS EC2 link-local metadata address (SSRF vector) must be rejected
    let ssrf_aws = Orchestrator::validate_external_endpoint("postgres", "169.254.169.254", 5432);
    assert!(ssrf_aws.is_err());
    assert!(ssrf_aws.unwrap_err().to_string().contains("SSRF blocked"));

    // 2. GCP metadata service address must be rejected
    let ssrf_gcp =
        Orchestrator::validate_external_endpoint("redis", "metadata.google.internal", 6379);
    assert!(ssrf_gcp.is_err());
    assert!(ssrf_gcp.unwrap_err().to_string().contains("SSRF blocked"));

    // 3. Port 0 must be rejected
    let zero_port = Orchestrator::validate_external_endpoint("postgres", "rds.amazonaws.com", 0);
    assert!(zero_port.is_err());
    assert!(zero_port
        .unwrap_err()
        .to_string()
        .contains("invalid external port 0"));

    // 4. Unsupported protocol must be rejected
    let bad_proto = Orchestrator::validate_external_endpoint("ftp", "ftp.example.com", 21);
    assert!(bad_proto.is_err());
    assert!(bad_proto
        .unwrap_err()
        .to_string()
        .contains("unsupported external protocol"));

    // 5. Valid external cloud database must be accepted
    let valid_rds = Orchestrator::validate_external_endpoint(
        "postgres",
        "mydb.c78abc123.us-east-1.rds.amazonaws.com",
        5432,
    );
    assert!(valid_rds.is_ok());

    // 6. Valid external Redis cache must be accepted
    let valid_redis =
        Orchestrator::validate_external_endpoint("redis", "my-cache.cache.amazonaws.com", 6379);
    assert!(valid_redis.is_ok());
}

#[test]
fn test_redis_connection_details_and_env_mapping() {
    let conn = ConnectionDetails {
        protocol: "redis".to_string(),
        host: "redis-cluster.local".to_string(),
        port: 6379,
        path_or_database: Some("0".to_string()),
        username: None,
        password: Some("redis_p@ssword_999!".to_string()),
        extra_params: HashMap::new(),
    };

    let env_unmasked = conn.to_env_map(Some("CACHE"), false);
    assert_eq!(
        env_unmasked.get("CACHE_HOST").unwrap(),
        "redis-cluster.local"
    );
    assert_eq!(env_unmasked.get("CACHE_PORT").unwrap(), "6379");
    assert_eq!(
        env_unmasked.get("CACHE_PASSWORD").unwrap(),
        "redis_p@ssword_999!"
    );
    assert_eq!(
        env_unmasked.get("CACHE_URL").unwrap(),
        "redis://redis-cluster.local:6379/0"
    );

    let env_masked = conn.to_env_map(Some("CACHE"), true);
    assert_eq!(env_masked.get("CACHE_PASSWORD").unwrap(), "[REDACTED]");
}
