use crate::error::{Result, SmooothError};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::time::Instant;
use tokio_postgres::Client;
use tracing::info;

/// Advisory lock ID for serializing migrations: "SMOOOTH" in hex (0x534D4F4F4F5448).
pub const MIGRATION_ADVISORY_LOCK_ID: i64 = 0x534D4F4F4F5448;

/// Represents a single versioned SQL migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub sql: String,
}

impl Migration {
    pub fn new(version: i64, name: impl Into<String>, sql: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            sql: sql.into(),
        }
    }

    /// Computes the deterministic SHA-256 hex checksum of the migration SQL.
    pub fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.sql.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Recorded entry in the `schema_migrations` tracking table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub checksum: String,
    pub applied_at: DateTime<Utc>,
    pub execution_time_ms: i64,
}

/// Summary of an executed migration within a migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigrationSummary {
    pub version: i64,
    pub name: String,
    pub checksum: String,
    pub execution_time_ms: i64,
}

/// Overall report returned by a migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub current_version: i64,
    pub applied_migrations: Vec<AppliedMigrationSummary>,
    pub already_applied_count: usize,
}

/// Embedded PostgreSQL Migration Runner.
///
/// Guarantees:
/// 1. Concurrency control via PostgreSQL session-level advisory locks (`pg_advisory_lock`).
/// 2. Checksum validation: fails closed if any previously applied migration SQL has been altered.
/// 3. Out-of-order detection: rejects retroactive migrations with lower version than applied.
/// 4. Transactional atomicity: each migration runs in its own transaction (`BEGIN` -> DDL -> ledger insert -> `COMMIT`),
///    ensuring automatic rollback if DDL execution fails.
pub struct MigrationRunner<'a> {
    client: &'a mut Client,
    migrations: Vec<Migration>,
}

impl<'a> MigrationRunner<'a> {
    /// Creates a runner configured with the canonical embedded migrations.
    pub fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            migrations: Self::embedded_migrations(),
        }
    }

    /// Creates a runner with custom migrations (primarily for testing error/rollback paths).
    pub fn with_migrations(client: &'a mut Client, migrations: Vec<Migration>) -> Self {
        Self { client, migrations }
    }

    /// Returns the embedded migration suite for Smoooth control-plane storage.
    pub fn embedded_migrations() -> Vec<Migration> {
        vec![Migration::new(
            1,
            "initial_control_plane",
            include_str!("../../../migrations/0001_initial_control_plane.sql"),
        )]
    }

    /// Ensures the `schema_migrations` tracking ledger table exists.
    pub async fn ensure_ledger_table(&mut self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                checksum VARCHAR(64) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                execution_time_ms BIGINT NOT NULL
            );
        "#;
        self.client.batch_execute(sql).await.map_err(|e| {
            SmooothError::PostgresFailed(format!("Failed to create schema_migrations table: {}", e))
        })?;
        Ok(())
    }

    /// Acquires the PostgreSQL session-level advisory lock.
    pub async fn acquire_advisory_lock(&mut self) -> Result<()> {
        self.client
            .execute(
                "SELECT pg_advisory_lock($1)",
                &[&MIGRATION_ADVISORY_LOCK_ID],
            )
            .await
            .map_err(|e| {
                SmooothError::PostgresFailed(format!(
                    "Failed to acquire migration advisory lock: {}",
                    e
                ))
            })?;
        Ok(())
    }

    /// Releases the PostgreSQL session-level advisory lock.
    pub async fn release_advisory_lock(&mut self) -> Result<()> {
        self.client
            .execute(
                "SELECT pg_advisory_unlock($1)",
                &[&MIGRATION_ADVISORY_LOCK_ID],
            )
            .await
            .map_err(|e| {
                SmooothError::PostgresFailed(format!(
                    "Failed to release migration advisory lock: {}",
                    e
                ))
            })?;
        Ok(())
    }

    /// Retrieves all applied migrations from `schema_migrations`, ordered by version ascending.
    pub async fn get_applied_migrations(&mut self) -> Result<Vec<AppliedMigration>> {
        self.ensure_ledger_table().await?;

        let rows = self
            .client
            .query(
                "SELECT version, name, checksum, applied_at, execution_time_ms FROM schema_migrations ORDER BY version ASC",
                &[],
            )
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed to query schema_migrations: {}", e)))?;

        let mut applied = Vec::new();
        for row in rows {
            applied.push(AppliedMigration {
                version: row.get(0),
                name: row.get(1),
                checksum: row.get(2),
                applied_at: row.get(3),
                execution_time_ms: row.get(4),
            });
        }
        Ok(applied)
    }

    /// Inspects migration status without applying pending migrations.
    pub async fn status(&mut self) -> Result<(i64, Vec<AppliedMigration>, Vec<Migration>)> {
        let applied = self.get_applied_migrations().await?;
        let applied_versions: BTreeSet<i64> = applied.iter().map(|m| m.version).collect();

        let mut pending = Vec::new();
        for m in &self.migrations {
            if !applied_versions.contains(&m.version) {
                pending.push(m.clone());
            }
        }

        let current_version = applied.last().map(|m| m.version).unwrap_or(0);
        Ok((current_version, applied, pending))
    }

    /// Runs all pending migrations under the advisory lock and returns a summary report.
    pub async fn run(&mut self) -> Result<MigrationReport> {
        self.acquire_advisory_lock().await?;

        let run_result = self.run_internal().await;

        // Ensure advisory lock is always released
        let unlock_result = self.release_advisory_lock().await;

        match (run_result, unlock_result) {
            (Err(e), _) => Err(e),
            (Ok(_), Err(e)) => Err(e),
            (Ok(report), Ok(_)) => Ok(report),
        }
    }

    async fn run_internal(&mut self) -> Result<MigrationReport> {
        self.ensure_ledger_table().await?;

        let applied = self.get_applied_migrations().await?;
        let applied_map: std::collections::HashMap<i64, &AppliedMigration> =
            applied.iter().map(|m| (m.version, m)).collect();

        // 1. Validate previously applied migrations against current migration definitions
        for mig in &self.migrations {
            if let Some(applied_entry) = applied_map.get(&mig.version) {
                let current_checksum = mig.checksum();
                if applied_entry.checksum != current_checksum {
                    return Err(SmooothError::Storage(format!(
                        "Migration {} ('{}') checksum mismatch: recorded checksum '{}', code checksum '{}'. Database schema has been modified or corrupted.",
                        mig.version, mig.name, applied_entry.checksum, current_checksum
                    )));
                }
            }
        }

        // 2. Detect missing migrations that were previously applied
        let code_versions: BTreeSet<i64> = self.migrations.iter().map(|m| m.version).collect();
        for applied_entry in &applied {
            if !code_versions.contains(&applied_entry.version) {
                return Err(SmooothError::Storage(format!(
                    "Migration {} ('{}') recorded in database is missing from migration runner code.",
                    applied_entry.version, applied_entry.name
                )));
            }
        }

        // 3. Detect out-of-order pending migrations
        let max_applied_version = applied.last().map(|m| m.version).unwrap_or(0);
        for mig in &self.migrations {
            if !applied_map.contains_key(&mig.version) && mig.version < max_applied_version {
                return Err(SmooothError::Storage(format!(
                    "Detected out-of-order pending migration {} ('{}'); latest applied version is {}.",
                    mig.version, mig.name, max_applied_version
                )));
            }
        }

        // 4. Sort migrations to apply pending in ascending order
        let mut sorted_migrations = self.migrations.clone();
        sorted_migrations.sort_by_key(|m| m.version);

        let mut applied_summaries = Vec::new();

        for mig in sorted_migrations {
            if applied_map.contains_key(&mig.version) {
                continue;
            }

            info!(
                version = mig.version,
                name = %mig.name,
                "Applying database migration"
            );

            let start = Instant::now();
            let checksum = mig.checksum();

            // Execute migration in an isolated database transaction
            let tx = self.client.transaction().await.map_err(|e| {
                SmooothError::PostgresFailed(format!(
                    "Failed to start transaction for migration {}: {}",
                    mig.version, e
                ))
            })?;

            // Execute the DDL batch
            tx.batch_execute(&mig.sql).await.map_err(|e| {
                SmooothError::PostgresFailed(format!(
                    "Migration {} ('{}') execution failed: {}",
                    mig.version, mig.name, e
                ))
            })?;

            let elapsed_ms = start.elapsed().as_millis() as i64;

            // Record in schema_migrations ledger
            tx.execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, execution_time_ms) VALUES ($1, $2, $3, NOW(), $4)",
                &[&mig.version, &mig.name, &checksum, &elapsed_ms],
            )
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed to record migration {} in schema_migrations: {}", mig.version, e)))?;

            // Commit transaction
            tx.commit().await.map_err(|e| {
                SmooothError::PostgresFailed(format!(
                    "Failed to commit transaction for migration {}: {}",
                    mig.version, e
                ))
            })?;

            info!(
                version = mig.version,
                name = %mig.name,
                elapsed_ms = elapsed_ms,
                "Migration successfully applied and committed"
            );

            applied_summaries.push(AppliedMigrationSummary {
                version: mig.version,
                name: mig.name,
                checksum,
                execution_time_ms: elapsed_ms,
            });
        }

        let current_version = applied
            .last()
            .map(|m| m.version)
            .into_iter()
            .chain(applied_summaries.iter().map(|s| s.version))
            .max()
            .unwrap_or(0);

        Ok(MigrationReport {
            current_version,
            applied_migrations: applied_summaries,
            already_applied_count: applied.len(),
        })
    }
}
