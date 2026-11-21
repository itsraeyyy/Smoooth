use smoooth::error::SmooothError;
use smoooth::storage::postgres::{Migration, MigrationRunner};
use std::process::Command;
use std::sync::OnceLock;
use tokio_postgres::{Client, NoTls};

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

/// Discovers candidate database connection strings, prioritizing WSL IP on Windows.
async fn get_test_client() -> Option<Client> {
    // 1. Explicit DATABASE_URL
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if let Ok((client, connection)) = tokio_postgres::connect(&url, NoTls).await {
            tokio::spawn(async move {
                let _ = connection.await;
            });
            return Some(client);
        }
    }

    // 2. Discovered candidate hosts
    for host in get_candidate_hosts() {
        let conn_str = format!(
            "host={} port=5432 user=postgres password=postgres dbname=smoooth_test",
            host
        );
        if let Ok((client, connection)) = tokio_postgres::connect(&conn_str, NoTls).await {
            tokio::spawn(async move {
                let _ = connection.await;
            });
            return Some(client);
        }
    }

    None
}

/// Hermetic test harness providing an isolated schema per test.
struct TestContext {
    client: Client,
    schema_name: String,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let client = get_test_client().await?;
        let random_id = uuid::Uuid::new_v4().simple().to_string();
        let schema_name = format!("test_{}", random_id);

        let setup_sql = format!(
            "CREATE SCHEMA \"{}\"; SET search_path TO \"{}\", public;",
            schema_name, schema_name
        );
        client
            .batch_execute(&setup_sql)
            .await
            .expect("Failed to initialize test schema");

        Some(Self {
            client,
            schema_name,
        })
    }

    async fn cleanup(&mut self) {
        let drop_sql = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE;", self.schema_name);
        let _ = self.client.batch_execute(&drop_sql).await;
    }
}

#[tokio::test]
async fn test_migration_runner_clean_database() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: PostgreSQL instance unavailable");
            return;
        }
    };

    // Run embedded migrations
    let mut runner = MigrationRunner::new(&mut ctx.client);
    let report = runner.run().await.expect("Migration run failed");

    assert_eq!(report.current_version, 1);
    assert_eq!(report.already_applied_count, 0);
    assert_eq!(report.applied_migrations.len(), 1);
    assert_eq!(report.applied_migrations[0].version, 1);
    assert_eq!(report.applied_migrations[0].name, "initial_control_plane");
    assert!(!report.applied_migrations[0].checksum.is_empty());
    assert!(report.applied_migrations[0].execution_time_ms >= 0);

    // Verify schema_migrations ledger directly
    let applied = runner.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].version, 1);
    assert_eq!(applied[0].name, "initial_control_plane");
    assert_eq!(applied[0].checksum, report.applied_migrations[0].checksum);

    // Verify all 11 required domain tables exist in isolated schema
    let expected_tables = vec![
        "workspaces",
        "agents",
        "plans",
        "operations",
        "idempotency_records",
        "resources",
        "service_bindings",
        "audit_events",
        "capability_revocations",
        "used_nonces",
        "fencing_epochs",
    ];

    for table in expected_tables {
        let row = ctx
            .client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.tables 
                    WHERE table_schema = current_schema() AND table_name = $1
                )",
                &[&table],
            )
            .await
            .expect("Table check query failed");
        let exists: bool = row.get(0);
        assert!(
            exists,
            "Expected table '{}' to exist after migration",
            table
        );
    }

    // Verify key indexes exist in isolated schema
    let expected_indexes = vec![
        "idx_operations_status_lease",
        "idx_operations_workspace",
        "idx_resources_workspace",
        "idx_resources_status",
        "idx_bindings_workspace",
        "idx_bindings_target",
        "idx_audit_events_workspace",
        "idx_agents_workspace",
        "idx_agents_token_hash",
        "idx_used_nonces_expires_at",
        "idx_capability_revocations_expires_at",
    ];

    for index_name in expected_indexes {
        let row = ctx
            .client
            .query_one(
                "SELECT EXISTS (
                    SELECT 1 FROM pg_indexes 
                    WHERE schemaname = current_schema() AND indexname = $1
                )",
                &[&index_name],
            )
            .await
            .expect("Index check query failed");
        let exists: bool = row.get(0);
        assert!(
            exists,
            "Expected index '{}' to exist after migration",
            index_name
        );
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_migration_runner_idempotency() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => return,
    };

    // First run applies migration
    {
        let mut runner = MigrationRunner::new(&mut ctx.client);
        let report = runner.run().await.unwrap();
        assert_eq!(report.current_version, 1);
        assert_eq!(report.applied_migrations.len(), 1);
        assert_eq!(report.already_applied_count, 0);
    }

    // Second run must be completely idempotent
    {
        let mut runner = MigrationRunner::new(&mut ctx.client);
        let report = runner.run().await.unwrap();
        assert_eq!(report.current_version, 1);
        assert_eq!(report.already_applied_count, 1);
        assert_eq!(report.applied_migrations.len(), 0);
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_checksum_tamper_fails_closed() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => return,
    };

    // Apply standard migration 1
    {
        let mut runner = MigrationRunner::new(&mut ctx.client);
        runner.run().await.unwrap();
    }

    // Present tampered migration 1
    let tampered_sql = "-- TAMPERED SQL CONTENT --\nCREATE TABLE malicious (id INT PRIMARY KEY);";
    let tampered_migrations = vec![Migration::new(1, "initial_control_plane", tampered_sql)];

    let mut runner = MigrationRunner::with_migrations(&mut ctx.client, tampered_migrations);
    let result = runner.run().await;

    assert!(
        result.is_err(),
        "Expected runner to fail on checksum mismatch"
    );
    match result.unwrap_err() {
        SmooothError::Storage(msg) => {
            assert!(
                msg.contains("checksum mismatch"),
                "Expected error message to contain 'checksum mismatch', got: {}",
                msg
            );
            assert!(
                msg.contains("Migration 1"),
                "Expected error message to reference Migration 1, got: {}",
                msg
            );
        }
        other => panic!("Unexpected error type: {:?}", other),
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_out_of_order_migration_rejected() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => return,
    };

    // Apply migration 2 directly
    let mig2 = Migration::new(
        2,
        "future_migration",
        "CREATE TABLE future_table (id INT PRIMARY KEY);",
    );
    {
        let mut runner = MigrationRunner::with_migrations(&mut ctx.client, vec![mig2.clone()]);
        runner.run().await.unwrap();
    }

    // Now try to run with migration 1 and migration 2 (where migration 1 was skipped)
    let mig1 = Migration::new(
        1,
        "retroactive_migration",
        "CREATE TABLE skipped_table (id INT PRIMARY KEY);",
    );
    let mut runner = MigrationRunner::with_migrations(&mut ctx.client, vec![mig1, mig2]);
    let result = runner.run().await;

    assert!(
        result.is_err(),
        "Expected runner to reject out-of-order migration"
    );
    match result.unwrap_err() {
        SmooothError::Storage(msg) => {
            assert!(
                msg.contains("out-of-order pending migration 1"),
                "Expected out-of-order error, got: {}",
                msg
            );
        }
        other => panic!("Unexpected error type: {:?}", other),
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_transactional_rollback_on_ddl_failure() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => return,
    };

    // Apply migration 1
    {
        let mut runner = MigrationRunner::new(&mut ctx.client);
        runner.run().await.unwrap();
    }

    // Attempt migration 2 with DDL that causes a syntax/execution error mid-script
    let broken_sql = r#"
        CREATE TABLE canary_table (id INT PRIMARY KEY);
        THIS IS INVALID SQL SYNTAX AND WILL FAIL;
    "#;
    let migrations = vec![
        MigrationRunner::embedded_migrations()[0].clone(),
        Migration::new(2, "broken_migration", broken_sql),
    ];

    let mut runner = MigrationRunner::with_migrations(&mut ctx.client, migrations);
    let result = runner.run().await;

    assert!(result.is_err(), "Expected broken migration to fail");

    // Verify version 2 is NOT recorded in schema_migrations
    let applied = runner.get_applied_migrations().await.unwrap();
    assert_eq!(
        applied.len(),
        1,
        "Only migration 1 should be recorded in schema_migrations"
    );
    assert_eq!(applied[0].version, 1);

    drop(runner);

    // Verify canary_table does NOT exist (transaction rolled back!)
    let row = ctx
        .client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_schema = current_schema() AND table_name = 'canary_table'
            )",
            &[],
        )
        .await
        .unwrap();
    let canary_exists: bool = row.get(0);
    assert!(
        !canary_exists,
        "canary_table must NOT exist after failed migration rollback"
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_relational_contracts_and_isolation() {
    let mut ctx = match TestContext::new().await {
        Some(c) => c,
        None => return,
    };

    // Initialize schema
    let mut runner = MigrationRunner::new(&mut ctx.client);
    runner.run().await.unwrap();

    // 1. Create two isolated workspaces
    ctx.client
        .execute(
            "INSERT INTO workspaces (id, name) VALUES ('ws-alpha', 'Alpha'), ('ws-beta', 'Beta')",
            &[],
        )
        .await
        .unwrap();

    // 2. Insert resources in both workspaces
    ctx.client
        .execute(
            "INSERT INTO resources (id, workspace_id, name, service_type, provider) 
             VALUES ('res_alpha_web', 'ws-alpha', 'web', 'container', 'docker')",
            &[],
        )
        .await
        .unwrap();

    ctx.client
        .execute(
            "INSERT INTO resources (id, workspace_id, name, service_type, provider) 
             VALUES ('res_alpha_db', 'ws-alpha', 'db', 'postgres', 'docker')",
            &[],
        )
        .await
        .unwrap();

    ctx.client
        .execute(
            "INSERT INTO resources (id, workspace_id, name, service_type, provider) 
             VALUES ('res_beta_db', 'ws-beta', 'db', 'postgres', 'docker')",
            &[],
        )
        .await
        .unwrap();

    // 3. Test canonical resource uniqueness within workspace
    let dup_res = ctx
        .client
        .execute(
            "INSERT INTO resources (id, workspace_id, name, service_type, provider) 
             VALUES ('res_alpha_db2', 'ws-alpha', 'db', 'postgres', 'docker')",
            &[],
        )
        .await;
    assert!(
        dup_res.is_err(),
        "Duplicate resource name in same workspace must be rejected"
    );

    // 4. Test Cross-Workspace Isolation in Service Bindings
    // Attempt to bind ws-alpha web container to ws-beta db
    let cross_ws_binding = ctx
        .client
        .execute(
            "INSERT INTO service_bindings (id, workspace_id, source_resource_id, target_resource_id, binding_type, encrypted_credentials) 
             VALUES ('bind-cross', 'ws-alpha', 'res_alpha_web', 'res_beta_db', 'db_conn', decode('', 'hex'))",
            &[],
        )
        .await;
    assert!(
        cross_ws_binding.is_err(),
        "Cross-workspace binding (alpha source -> beta target) must be rejected by foreign key constraint"
    );

    // 5. Test Self-Binding Prevention
    let self_binding = ctx
        .client
        .execute(
            "INSERT INTO service_bindings (id, workspace_id, source_resource_id, target_resource_id, binding_type, encrypted_credentials) 
             VALUES ('bind-self', 'ws-alpha', 'res_alpha_web', 'res_alpha_web', 'loopback', decode('', 'hex'))",
            &[],
        )
        .await;
    assert!(
        self_binding.is_err(),
        "Self-binding (source == target) must be rejected by CHECK constraint"
    );

    // 6. Test Valid Intra-Workspace Binding
    let valid_binding = ctx
        .client
        .execute(
            "INSERT INTO service_bindings (id, workspace_id, source_resource_id, target_resource_id, binding_type, encrypted_credentials) 
             VALUES ('bind-valid', 'ws-alpha', 'res_alpha_web', 'res_alpha_db', 'db_conn', decode('deadbeef', 'hex'))",
            &[],
        )
        .await;
    assert!(
        valid_binding.is_ok(),
        "Intra-workspace binding must succeed"
    );

    // 7. Test Idempotency Record Orphan Prevention
    // Attempt to insert idempotency record pointing to non-existent operation
    let orphan_idempotency = ctx
        .client
        .execute(
            "INSERT INTO idempotency_records (workspace_id, operation_type, client_key, request_hash, operation_id) 
             VALUES ('ws-alpha', 'provision', 'client-key-1', 'req-hash-1', 'nonexistent-op')",
            &[],
        )
        .await;
    assert!(
        orphan_idempotency.is_err(),
        "Idempotency record referencing nonexistent operation must be rejected by foreign key"
    );

    // 8. Test Plan Risk Tier Validation
    let invalid_risk_tier = ctx
        .client
        .execute(
            "INSERT INTO plans (id, workspace_id, plan_hash, canonical_spec, risk_tier) 
             VALUES ('plan-1', 'ws-alpha', 'hash-1', '{}'::jsonb, 'extreme_danger')",
            &[],
        )
        .await;
    assert!(
        invalid_risk_tier.is_err(),
        "Plan with invalid risk_tier must be rejected by CHECK constraint"
    );

    ctx.cleanup().await;
}
