use crate::error::{Result, SmooothError};
use crate::models::{
    AgentIdentity, AgentScope, CanonicalExecutionSpec, IdempotencyKey, IdempotencyRecord,
    Operation, OperationLogEntry, OperationStatus, Plan, PlanApproval, Resource, ResourceOwnership,
    ResourceStatus, RiskTier, RotationStage, ServiceBinding,
};
use crate::storage::postgres::pool::{PooledClient, PostgresPool};
use crate::storage::postgres::MigrationRunner;
use crate::storage::{AuditEvent, OperationInitOutcome, RevocationEntry, StateSnapshot, Storage};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::GenericClient;

/// PostgreSQL implementation of the control-plane Storage trait.
#[derive(Clone)]
pub struct PostgresStorage {
    pool: Arc<PostgresPool>,
}

impl PostgresStorage {
    /// Creates a new PostgresStorage, executing embedded migrations to ensure schema readiness.
    pub async fn new(config_str: &str) -> Result<Self> {
        Self::new_with_schema(config_str, None).await
    }

    /// Creates a new PostgresStorage scoped to a specific schema, applying migrations within it.
    pub async fn new_with_schema(config_str: &str, schema: Option<&str>) -> Result<Self> {
        let pool = Arc::new(PostgresPool::new_with_schema(
            config_str,
            16,
            schema.map(|s| s.to_string()),
        ));

        // Initialize schema and run migration runner under advisory lock
        {
            let mut client = pool.get().await?;

            if let Some(s) = schema {
                client
                    .batch_execute(&format!(
                        "CREATE SCHEMA IF NOT EXISTS \"{}\"; SET search_path TO \"{}\", public;",
                        s, s
                    ))
                    .await
                    .map_err(|e| {
                        SmooothError::Storage(format!("Failed to initialize schema '{}': {}", s, e))
                    })?;
            }

            let mut runner = MigrationRunner::new(&mut client);
            runner.run().await?;
        }

        Ok(Self { pool })
    }

    /// Returns a direct connection from the pool.
    pub async fn get_client(&self) -> Result<PooledClient> {
        self.pool.get().await
    }

    // --- Helper methods for workspace and placeholder management ---

    async fn ensure_workspace<C: GenericClient>(client: &C, workspace_id: &str) -> Result<()> {
        client
            .execute(
                "INSERT INTO workspaces (id, name) VALUES ($1, $1) ON CONFLICT (id) DO NOTHING;",
                &[&workspace_id],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!(
                    "Failed to ensure workspace '{}': {}",
                    workspace_id, e
                ))
            })?;
        Ok(())
    }

    async fn ensure_plan_placeholder<C: GenericClient>(
        client: &C,
        workspace_id: &str,
        plan_id: &str,
        plan_hash: &str,
    ) -> Result<()> {
        Self::ensure_workspace(client, workspace_id).await?;
        let empty_spec = serde_json::json!({
            "schema_version": 1,
            "workspace_id": workspace_id,
            "environment": "development",
            "target_id": "auto",
            "actions": [],
            "dependencies": [],
            "immutable_artifacts": {},
            "policy_envelope": {
                "allow_public_network": false,
                "allow_data_destruction": false,
                "max_monthly_budget_usd": null
            }
        });
        client
            .execute(
                "INSERT INTO plans (id, workspace_id, schema_version, environment, plan_hash, canonical_spec, risk_tier, created_by)
                 VALUES ($1, $2, 1, 'development', $3, $4, 'low', 'system')
                 ON CONFLICT (id) DO NOTHING;",
                &[&plan_id, &workspace_id, &plan_hash, &empty_spec],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to ensure plan placeholder '{}': {}", plan_id, e)))?;
        Ok(())
    }

    async fn ensure_operation_placeholder<C: GenericClient>(
        client: &C,
        workspace_id: &str,
        operation_id: &str,
    ) -> Result<()> {
        Self::ensure_plan_placeholder(client, workspace_id, "placeholder_plan", "placeholder_hash")
            .await?;
        client
            .execute(
                "INSERT INTO operations (
                    id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                    idempotency_client_key, status, progress_percent, lease_epoch,
                    intended_state, observed_state, logs
                ) VALUES ($1, $2, 'unknown', 'placeholder_plan', 'placeholder_hash', 'placeholder_req',
                          'placeholder_key', 'pending', 0, 1, '{}'::jsonb, '{}'::jsonb, '[]'::jsonb)
                ON CONFLICT (id) DO NOTHING;",
                &[&operation_id, &workspace_id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to ensure operation placeholder '{}': {}", operation_id, e)))?;
        Ok(())
    }

    // --- Entity Row Mappers ---

    fn map_agent_row(row: &tokio_postgres::Row) -> Result<AgentIdentity> {
        let scopes_val: serde_json::Value = row.get("scopes");
        let scopes: Vec<AgentScope> = serde_json::from_value(scopes_val)
            .map_err(|e| SmooothError::Storage(format!("Corrupt agent scopes JSON: {}", e)))?;
        Ok(AgentIdentity {
            agent_id: row.get("agent_id"),
            name: row.get("name"),
            token_hash: row.get("token_hash"),
            token_prefix: row.get("token_prefix"),
            scopes,
            workspace_id: row.get("workspace_id"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            revoked: row.get("revoked"),
            revoked_at: row.get("revoked_at"),
            last_used_at: row.get("last_used_at"),
        })
    }

    fn map_plan_row(row: &tokio_postgres::Row) -> Result<Plan> {
        let canonical_spec: serde_json::Value = row.get("canonical_spec");
        let spec: CanonicalExecutionSpec = serde_json::from_value(canonical_spec).map_err(|e| {
            SmooothError::Storage(format!("Corrupt plan canonical_spec JSON: {}", e))
        })?;
        let rt_str: &str = row.get("risk_tier");
        let risk_tier = match rt_str {
            "moderate" => RiskTier::Moderate,
            "high" => RiskTier::High,
            "critical" => RiskTier::Critical,
            _ => RiskTier::Low,
        };
        let approval_val: Option<serde_json::Value> = row.get("approval");
        let approval: Option<PlanApproval> =
            approval_val.and_then(|v| serde_json::from_value(v).ok());
        let cost: Option<f64> = row.get("cost");

        Ok(Plan {
            id: row.get("id"),
            spec,
            risk_tier,
            estimated_monthly_cost_usd: cost,
            plan_hash: row.get("plan_hash"),
            approved_plan_hash: row.get("approved_plan_hash"),
            approved_by: row.get("approved_by"),
            approval,
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        })
    }

    fn map_operation_row(row: &tokio_postgres::Row) -> Result<Operation> {
        let status_str: &str = row.get("status");
        let status = match status_str {
            "running" => OperationStatus::Running,
            "succeeded" => OperationStatus::Succeeded,
            "failed" => OperationStatus::Failed,
            "cancelled" => OperationStatus::Cancelled,
            _ => OperationStatus::Pending,
        };
        let progress_i16: i16 = row.get("progress_percent");
        let progress_percent = progress_i16.clamp(0, 100) as u8;
        let epoch_i64: i64 = row.get("lease_epoch");
        let lease_epoch = epoch_i64.max(1) as u64;

        let logs_val: serde_json::Value = row.get("logs");
        let logs: Vec<OperationLogEntry> = serde_json::from_value(logs_val).unwrap_or_default();

        Ok(Operation {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            operation_type: row.get("operation_type"),
            plan_id: row.get("plan_id"),
            plan_hash: row.get("plan_hash"),
            request_hash: row.get("request_hash"),
            idempotency_client_key: row.get("idempotency_client_key"),
            status,
            progress_percent,
            lease_epoch,
            lease_owner: row.get("lease_owner"),
            lease_expires_at: row.get("lease_expires_at"),
            intended_state: row.get("intended_state"),
            observed_state: row.get("observed_state"),
            logs,
            evidence: row.get("evidence"),
            error_message: row.get("error_message"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn map_resource_row(row: &tokio_postgres::Row) -> Result<Resource> {
        let status_str: &str = row.get("status");
        let status = match status_str {
            "active" => ResourceStatus::Active,
            "degraded" => ResourceStatus::Degraded,
            "stopping" => ResourceStatus::Stopping,
            "stopped" => ResourceStatus::Stopped,
            "terminated" => ResourceStatus::Terminated,
            _ => ResourceStatus::Provisioning,
        };
        let ownership_str: &str = row.get("ownership");
        let ownership = match ownership_str {
            "external" => ResourceOwnership::External,
            "discovered" | "custom_workload" => ResourceOwnership::CustomWorkload,
            _ => ResourceOwnership::SmooothManaged,
        };

        Ok(Resource {
            id: row.get("id"),
            name: row.get("name"),
            workspace_id: row.get("workspace_id"),
            service_type: row.get("service_type"),
            provider: row.get("provider"),
            is_persistent: row.get("is_persistent"),
            ownership,
            status,
            configuration: row.get("configuration"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn map_binding_row(row: &tokio_postgres::Row) -> Result<ServiceBinding> {
        let stage_str: &str = row.get("rotation_stage");
        let rotation_stage = match stage_str {
            "overlapping" => RotationStage::Overlapping,
            "verifying" => RotationStage::Verifying,
            "retired" => RotationStage::Retired,
            "rolled_back" => RotationStage::RolledBack,
            _ => RotationStage::Active,
        };
        let ownership_str: &str = row.get("ownership");
        let ownership = match ownership_str {
            "external" => ResourceOwnership::External,
            "discovered" | "custom_workload" => ResourceOwnership::CustomWorkload,
            _ => ResourceOwnership::SmooothManaged,
        };
        let version_i32: i32 = row.get("binding_version");
        let binding_version = version_i32.max(1) as u32;

        let endpoint_val: Option<serde_json::Value> = row.get("resolved_endpoint");
        let resolved_endpoint = endpoint_val.and_then(|v| serde_json::from_value(v).ok());

        let details_val: Option<serde_json::Value> = row.get("safe_connection_details");
        let safe_connection_details = details_val.and_then(|v| serde_json::from_value(v).ok());

        let env_keys_val: serde_json::Value = row.get("exposed_env_keys");
        let exposed_env_keys: Vec<String> =
            serde_json::from_value(env_keys_val).unwrap_or_default();

        let env_map_val: serde_json::Value = row.get("env_mappings");
        let env_mappings: HashMap<String, String> =
            serde_json::from_value(env_map_val).unwrap_or_default();

        let encrypted_credentials: Vec<u8> = row.get("encrypted_credentials");

        Ok(ServiceBinding {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            source_resource_id: row.get("source_resource_id"),
            target_resource_id: row.get("target_resource_id"),
            binding_type: row.get("binding_type"),
            rotation_stage,
            ownership,
            binding_version,
            key_fingerprint: row.get("key_fingerprint"),
            resolved_endpoint,
            safe_connection_details,
            encrypted_credentials,
            exposed_env_keys,
            env_mappings,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn map_idempotency_row(row: &tokio_postgres::Row) -> IdempotencyRecord {
        IdempotencyRecord {
            key: IdempotencyKey {
                workspace_id: row.get("workspace_id"),
                operation_type: row.get("operation_type"),
                client_key: row.get("client_key"),
            },
            request_hash: row.get("request_hash"),
            operation_id: row.get("operation_id"),
            created_at: row.get("created_at"),
        }
    }

    fn map_audit_row(row: &tokio_postgres::Row) -> AuditEvent {
        let id_i64: i64 = row.get("id");
        AuditEvent {
            id: id_i64 as u64,
            workspace_id: row.get("workspace_id"),
            actor: row.get("actor"),
            action: row.get("action"),
            target_resource_id: row.get("target_resource_id"),
            plan_hash: row.get("plan_hash"),
            outcome: row.get("outcome"),
            redacted_evidence: row.get("redacted_evidence"),
            timestamp: row.get("created_at"),
        }
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    // =========================================================================
    // Agents & Access Security
    // =========================================================================

    async fn save_agent(&self, agent: AgentIdentity) -> Result<()> {
        let client = self.pool.get().await?;
        Self::ensure_workspace(&*client, &agent.workspace_id).await?;

        let scopes_json = serde_json::to_value(&agent.scopes).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize agent scopes: {}", e))
        })?;

        client
            .execute(
                "INSERT INTO agents (
                    agent_id, name, token_hash, token_prefix, scopes, workspace_id,
                    created_at, expires_at, revoked, revoked_at, last_used_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (agent_id) DO UPDATE SET
                    name = EXCLUDED.name,
                    token_hash = EXCLUDED.token_hash,
                    token_prefix = EXCLUDED.token_prefix,
                    scopes = EXCLUDED.scopes,
                    workspace_id = EXCLUDED.workspace_id,
                    expires_at = EXCLUDED.expires_at,
                    revoked = EXCLUDED.revoked,
                    revoked_at = EXCLUDED.revoked_at,
                    last_used_at = EXCLUDED.last_used_at;",
                &[
                    &agent.agent_id,
                    &agent.name,
                    &agent.token_hash,
                    &agent.token_prefix,
                    &scopes_json,
                    &agent.workspace_id,
                    &agent.created_at,
                    &agent.expires_at,
                    &agent.revoked,
                    &agent.revoked_at,
                    &agent.last_used_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to save agent: {}", e)))?;

        Ok(())
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentIdentity>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT agent_id, name, token_hash, token_prefix, scopes, workspace_id,
                        created_at, expires_at, revoked, revoked_at, last_used_at
                 FROM agents WHERE agent_id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get agent: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_agent_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn get_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<AgentIdentity>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT agent_id, name, token_hash, token_prefix, scopes, workspace_id,
                        created_at, expires_at, revoked, revoked_at, last_used_at
                 FROM agents WHERE token_hash = $1;",
                &[&token_hash],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to get agent by token hash: {}", e))
            })?;

        match row {
            Some(r) => Ok(Some(Self::map_agent_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_agents(&self, workspace_id: Option<&str>) -> Result<Vec<AgentIdentity>> {
        let client = self.pool.get().await?;
        let rows = match workspace_id {
            Some(ws) if ws != "*" && !ws.is_empty() => {
                client
                    .query(
                        "SELECT agent_id, name, token_hash, token_prefix, scopes, workspace_id,
                                created_at, expires_at, revoked, revoked_at, last_used_at
                         FROM agents WHERE workspace_id = '*' OR workspace_id = $1
                         ORDER BY created_at ASC;",
                        &[&ws],
                    )
                    .await
            }
            _ => {
                client
                    .query(
                        "SELECT agent_id, name, token_hash, token_prefix, scopes, workspace_id,
                                created_at, expires_at, revoked, revoked_at, last_used_at
                         FROM agents ORDER BY created_at ASC;",
                        &[],
                    )
                    .await
            }
        }
        .map_err(|e| SmooothError::Storage(format!("Failed to list agents: {}", e)))?;

        let mut agents = Vec::with_capacity(rows.len());
        for row in rows {
            agents.push(Self::map_agent_row(&row)?);
        }
        Ok(agents)
    }

    async fn revoke_agent(&self, id: &str) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE agents SET revoked = TRUE, revoked_at = NOW() WHERE agent_id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to revoke agent: {}", e)))?;
        Ok(())
    }

    // =========================================================================
    // Plans
    // =========================================================================

    async fn save_plan(&self, plan: Plan) -> Result<()> {
        let client = self.pool.get().await?;
        Self::ensure_workspace(&*client, &plan.spec.workspace_id).await?;

        let canonical_spec_json = serde_json::to_value(&plan.spec)
            .map_err(|e| SmooothError::Storage(format!("Failed to serialize plan spec: {}", e)))?;
        let approval_json = plan
            .approval
            .as_ref()
            .map(|a| serde_json::to_value(a).map_err(|e| SmooothError::Storage(e.to_string())))
            .transpose()?;

        let risk_tier_str = match plan.risk_tier {
            RiskTier::Low => "low",
            RiskTier::Moderate => "moderate",
            RiskTier::High => "high",
            RiskTier::Critical => "critical",
        };
        let schema_version = plan.spec.schema_version as i32;
        let cost_str = plan.estimated_monthly_cost_usd.map(|c| format!("{:.2}", c));

        client
            .execute(
                "INSERT INTO plans (
                    id, workspace_id, schema_version, environment, plan_hash,
                    canonical_spec, risk_tier, estimated_monthly_cost_usd,
                    approved_plan_hash, approved_by, approval, created_by, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::text::numeric(10,2), $9, $10, $11, $12, $13)
                ON CONFLICT (id) DO UPDATE SET
                    workspace_id = EXCLUDED.workspace_id,
                    schema_version = EXCLUDED.schema_version,
                    environment = EXCLUDED.environment,
                    plan_hash = EXCLUDED.plan_hash,
                    canonical_spec = EXCLUDED.canonical_spec,
                    risk_tier = EXCLUDED.risk_tier,
                    estimated_monthly_cost_usd = EXCLUDED.estimated_monthly_cost_usd,
                    approved_plan_hash = EXCLUDED.approved_plan_hash,
                    approved_by = EXCLUDED.approved_by,
                    approval = EXCLUDED.approval,
                    created_by = EXCLUDED.created_by;",
                &[
                    &plan.id,
                    &plan.spec.workspace_id,
                    &schema_version,
                    &plan.spec.environment,
                    &plan.plan_hash,
                    &canonical_spec_json,
                    &risk_tier_str,
                    &cost_str,
                    &plan.approved_plan_hash,
                    &plan.approved_by,
                    &approval_json,
                    &plan.created_by,
                    &plan.created_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to save plan: {}", e)))?;

        Ok(())
    }

    async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, workspace_id, schema_version, environment, plan_hash,
                        canonical_spec, risk_tier, estimated_monthly_cost_usd::float8 AS cost,
                        approved_plan_hash, approved_by, approval, created_by, created_at
                 FROM plans WHERE id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get plan: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_plan_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_plans(&self) -> Result<Vec<Plan>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, workspace_id, schema_version, environment, plan_hash,
                        canonical_spec, risk_tier, estimated_monthly_cost_usd::float8 AS cost,
                        approved_plan_hash, approved_by, approval, created_by, created_at
                 FROM plans ORDER BY created_at ASC;",
                &[],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to list plans: {}", e)))?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            plans.push(Self::map_plan_row(&row)?);
        }
        Ok(plans)
    }

    async fn delete_plan(&self, id: &str) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute("DELETE FROM plans WHERE id = $1;", &[&id])
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to delete plan: {}", e)))?;
        Ok(())
    }

    // =========================================================================
    // Idempotency & Operation Atomic Reservation
    // =========================================================================

    async fn check_idempotency(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT workspace_id, operation_type, client_key, request_hash, operation_id, created_at
                 FROM idempotency_records
                 WHERE workspace_id = $1 AND operation_type = $2 AND client_key = $3;",
                &[&key.workspace_id, &key.operation_type, &key.client_key],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to check idempotency: {}", e)))?;

        Ok(row.as_ref().map(Self::map_idempotency_row))
    }

    async fn record_idempotency(&self, record: IdempotencyRecord) -> Result<()> {
        let mut client = self.pool.get().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction begin error: {}", e)))?;

        Self::ensure_workspace(&tx, &record.key.workspace_id).await?;
        Self::ensure_operation_placeholder(&tx, &record.key.workspace_id, &record.operation_id)
            .await?;

        tx.execute(
            "INSERT INTO idempotency_records (
                workspace_id, operation_type, client_key, request_hash, operation_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (workspace_id, operation_type, client_key) DO UPDATE SET
                request_hash = EXCLUDED.request_hash,
                operation_id = EXCLUDED.operation_id,
                created_at = EXCLUDED.created_at;",
            &[
                &record.key.workspace_id,
                &record.key.operation_type,
                &record.key.client_key,
                &record.request_hash,
                &record.operation_id,
                &record.created_at,
            ],
        )
        .await
        .map_err(|e| SmooothError::Storage(format!("Failed to record idempotency: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }

    async fn check_or_record_idempotency(
        &self,
        record: IdempotencyRecord,
    ) -> Result<Option<IdempotencyRecord>> {
        let mut client = self.pool.get().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction begin error: {}", e)))?;

        Self::ensure_workspace(&tx, &record.key.workspace_id).await?;

        let existing_row = tx
            .query_opt(
                "SELECT workspace_id, operation_type, client_key, request_hash, operation_id, created_at
                 FROM idempotency_records
                 WHERE workspace_id = $1 AND operation_type = $2 AND client_key = $3
                 FOR UPDATE;",
                &[&record.key.workspace_id, &record.key.operation_type, &record.key.client_key],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to query idempotency lock: {}", e)))?;

        if let Some(row) = existing_row {
            let existing = Self::map_idempotency_row(&row);
            if existing.request_hash != record.request_hash {
                return Err(SmooothError::IdempotencyConflict(format!(
                    "Idempotency key '{}' reused with mismatched request hash. Prior: {}, Current: {}",
                    record.key.client_key, existing.request_hash, record.request_hash
                )));
            }
            tx.rollback()
                .await
                .map_err(|e| SmooothError::Storage(format!("Transaction rollback error: {}", e)))?;
            return Ok(Some(existing));
        }

        Self::ensure_operation_placeholder(&tx, &record.key.workspace_id, &record.operation_id)
            .await?;

        tx.execute(
            "INSERT INTO idempotency_records (
                workspace_id, operation_type, client_key, request_hash, operation_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6);",
            &[
                &record.key.workspace_id,
                &record.key.operation_type,
                &record.key.client_key,
                &record.request_hash,
                &record.operation_id,
                &record.created_at,
            ],
        )
        .await
        .map_err(|e| {
            SmooothError::Storage(format!("Failed to insert idempotency record: {}", e))
        })?;

        tx.commit()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(None)
    }

    async fn reserve_or_get_operation(
        &self,
        idempotency_record: IdempotencyRecord,
        operation: Operation,
    ) -> Result<OperationInitOutcome> {
        let mut client = self.pool.get().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction begin error: {}", e)))?;

        Self::ensure_workspace(&tx, &idempotency_record.key.workspace_id).await?;

        // 1. Lock and check idempotency record
        let existing_idemp = tx
            .query_opt(
                "SELECT workspace_id, operation_type, client_key, request_hash, operation_id, created_at
                 FROM idempotency_records
                 WHERE workspace_id = $1 AND operation_type = $2 AND client_key = $3
                 FOR UPDATE;",
                &[
                    &idempotency_record.key.workspace_id,
                    &idempotency_record.key.operation_type,
                    &idempotency_record.key.client_key,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to lock idempotency record: {}", e)))?;

        if let Some(row) = existing_idemp {
            let existing = Self::map_idempotency_row(&row);
            if existing.request_hash != idempotency_record.request_hash {
                return Err(SmooothError::IdempotencyConflict(format!(
                    "Idempotency key '{}' reused with mismatched request hash. Prior: {}, Current: {}",
                    idempotency_record.key.client_key, existing.request_hash, idempotency_record.request_hash
                )));
            }

            // Retrieve existing operation
            let op_row = tx
                .query_opt(
                    "SELECT id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                            idempotency_client_key, status, progress_percent, lease_epoch,
                            lease_owner, lease_expires_at, intended_state, observed_state,
                            logs, evidence, error_message, created_at, updated_at
                     FROM operations WHERE id = $1;",
                    &[&existing.operation_id],
                )
                .await
                .map_err(|e| {
                    SmooothError::Storage(format!("Failed to query existing operation: {}", e))
                })?;

            tx.rollback()
                .await
                .map_err(|e| SmooothError::Storage(format!("Transaction rollback error: {}", e)))?;

            match op_row {
                Some(r) => Ok(OperationInitOutcome::Existing(Self::map_operation_row(&r)?)),
                None => Err(SmooothError::Storage(format!(
                    "Inconsistent state: idempotency record references nonexistent operation '{}'",
                    existing.operation_id
                ))),
            }
        } else {
            // 2. Ensure plan and foreign keys exist before operation insertion
            Self::ensure_plan_placeholder(
                &tx,
                &operation.workspace_id,
                &operation.plan_id,
                &operation.plan_hash,
            )
            .await?;

            let status_str = match operation.status {
                OperationStatus::Pending => "pending",
                OperationStatus::Running => "running",
                OperationStatus::Succeeded => "succeeded",
                OperationStatus::Failed => "failed",
                OperationStatus::Cancelled => "cancelled",
            };
            let progress_i16 = operation.progress_percent as i16;
            let lease_epoch_i64 = operation.lease_epoch as i64;
            let logs_json = serde_json::to_value(&operation.logs).map_err(|e| {
                SmooothError::Storage(format!("Failed to serialize operation logs: {}", e))
            })?;

            // Insert operation
            tx.execute(
                "INSERT INTO operations (
                    id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                    idempotency_client_key, status, progress_percent, lease_epoch,
                    lease_owner, lease_expires_at, intended_state, observed_state,
                    logs, evidence, error_message, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
                );",
                &[
                    &operation.id,
                    &operation.workspace_id,
                    &operation.operation_type,
                    &operation.plan_id,
                    &operation.plan_hash,
                    &operation.request_hash,
                    &operation.idempotency_client_key,
                    &status_str,
                    &progress_i16,
                    &lease_epoch_i64,
                    &operation.lease_owner,
                    &operation.lease_expires_at,
                    &operation.intended_state,
                    &operation.observed_state,
                    &logs_json,
                    &operation.evidence,
                    &operation.error_message,
                    &operation.created_at,
                    &operation.updated_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to insert operation: {}", e)))?;

            // Insert idempotency record bound to this operation
            tx.execute(
                "INSERT INTO idempotency_records (
                    workspace_id, operation_type, client_key, request_hash, operation_id, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6);",
                &[
                    &idempotency_record.key.workspace_id,
                    &idempotency_record.key.operation_type,
                    &idempotency_record.key.client_key,
                    &idempotency_record.request_hash,
                    &operation.id,
                    &idempotency_record.created_at,
                ],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to insert idempotency record: {}", e))
            })?;

            tx.commit()
                .await
                .map_err(|e| SmooothError::Storage(format!("Transaction commit error: {}", e)))?;

            Ok(OperationInitOutcome::Created(operation))
        }
    }

    // =========================================================================
    // Operations & Fencing Leases
    // =========================================================================

    async fn save_operation(&self, operation: Operation) -> Result<()> {
        let client = self.pool.get().await?;
        Self::ensure_workspace(&*client, &operation.workspace_id).await?;
        Self::ensure_plan_placeholder(
            &*client,
            &operation.workspace_id,
            &operation.plan_id,
            &operation.plan_hash,
        )
        .await?;

        let status_str = match operation.status {
            OperationStatus::Pending => "pending",
            OperationStatus::Running => "running",
            OperationStatus::Succeeded => "succeeded",
            OperationStatus::Failed => "failed",
            OperationStatus::Cancelled => "cancelled",
        };
        let progress_i16 = operation.progress_percent as i16;
        let lease_epoch_i64 = operation.lease_epoch as i64;
        let logs_json = serde_json::to_value(&operation.logs).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize operation logs: {}", e))
        })?;

        client
            .execute(
                "INSERT INTO operations (
                    id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                    idempotency_client_key, status, progress_percent, lease_epoch,
                    lease_owner, lease_expires_at, intended_state, observed_state,
                    logs, evidence, error_message, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
                )
                ON CONFLICT (id) DO UPDATE SET
                    workspace_id = EXCLUDED.workspace_id,
                    operation_type = EXCLUDED.operation_type,
                    plan_id = EXCLUDED.plan_id,
                    plan_hash = EXCLUDED.plan_hash,
                    request_hash = EXCLUDED.request_hash,
                    idempotency_client_key = EXCLUDED.idempotency_client_key,
                    status = EXCLUDED.status,
                    progress_percent = EXCLUDED.progress_percent,
                    lease_epoch = EXCLUDED.lease_epoch,
                    lease_owner = EXCLUDED.lease_owner,
                    lease_expires_at = EXCLUDED.lease_expires_at,
                    intended_state = EXCLUDED.intended_state,
                    observed_state = EXCLUDED.observed_state,
                    logs = EXCLUDED.logs,
                    evidence = EXCLUDED.evidence,
                    error_message = EXCLUDED.error_message,
                    updated_at = NOW();",
                &[
                    &operation.id,
                    &operation.workspace_id,
                    &operation.operation_type,
                    &operation.plan_id,
                    &operation.plan_hash,
                    &operation.request_hash,
                    &operation.idempotency_client_key,
                    &status_str,
                    &progress_i16,
                    &lease_epoch_i64,
                    &operation.lease_owner,
                    &operation.lease_expires_at,
                    &operation.intended_state,
                    &operation.observed_state,
                    &logs_json,
                    &operation.evidence,
                    &operation.error_message,
                    &operation.created_at,
                    &operation.updated_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to save operation: {}", e)))?;

        Ok(())
    }

    async fn get_operation(&self, id: &str) -> Result<Option<Operation>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                        idempotency_client_key, status, progress_percent, lease_epoch,
                        lease_owner, lease_expires_at, intended_state, observed_state,
                        logs, evidence, error_message, created_at, updated_at
                 FROM operations WHERE id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get operation: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_operation_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_operations(&self, workspace_id: Option<&str>) -> Result<Vec<Operation>> {
        let client = self.pool.get().await?;
        let rows =
            match workspace_id {
                Some(ws) if ws != "*" && !ws.is_empty() => client
                    .query(
                        "SELECT id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                                idempotency_client_key, status, progress_percent, lease_epoch,
                                lease_owner, lease_expires_at, intended_state, observed_state,
                                logs, evidence, error_message, created_at, updated_at
                         FROM operations WHERE workspace_id = '*' OR workspace_id = $1
                         ORDER BY created_at ASC;",
                        &[&ws],
                    )
                    .await,
                _ => client
                    .query(
                        "SELECT id, workspace_id, operation_type, plan_id, plan_hash, request_hash,
                                idempotency_client_key, status, progress_percent, lease_epoch,
                                lease_owner, lease_expires_at, intended_state, observed_state,
                                logs, evidence, error_message, created_at, updated_at
                         FROM operations ORDER BY created_at ASC;",
                        &[],
                    )
                    .await,
            }
            .map_err(|e| SmooothError::Storage(format!("Failed to list operations: {}", e)))?;

        let mut ops = Vec::with_capacity(rows.len());
        for row in rows {
            ops.push(Self::map_operation_row(&row)?);
        }
        Ok(ops)
    }

    async fn update_operation(&self, operation: Operation) -> Result<()> {
        self.save_operation(operation).await
    }

    // =========================================================================
    // Resources & Canonical Identities
    // =========================================================================

    async fn save_resource(&self, mut resource: Resource) -> Result<Resource> {
        let mut client = self.pool.get().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction begin error: {}", e)))?;

        Self::ensure_workspace(&tx, &resource.workspace_id).await?;

        // 1. Check for canonical identity by (workspace_id, name)
        let existing_by_name = tx
            .query_opt(
                "SELECT id, created_at FROM resources WHERE workspace_id = $1 AND name = $2 FOR UPDATE;",
                &[&resource.workspace_id, &resource.name],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to query resource by name: {}", e)))?;

        // 2. Check for resource by id if not found by name
        let existing_by_id = if existing_by_name.is_none() {
            tx.query_opt(
                "SELECT id, created_at FROM resources WHERE id = $1 FOR UPDATE;",
                &[&resource.id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to query resource by id: {}", e)))?
        } else {
            None
        };

        let status_str = match resource.status {
            ResourceStatus::Active => "active",
            ResourceStatus::Degraded => "degraded",
            ResourceStatus::Stopping => "stopping",
            ResourceStatus::Stopped => "stopped",
            ResourceStatus::Terminated => "terminated",
            _ => "provisioning",
        };
        let ownership_str = match resource.ownership {
            ResourceOwnership::External => "external",
            ResourceOwnership::CustomWorkload => "discovered",
            ResourceOwnership::SmooothManaged => "smoooth_managed",
        };

        if let Some(row) = existing_by_name.or(existing_by_id) {
            let canonical_id: String = row.get("id");
            let created_at: DateTime<Utc> = row.get("created_at");

            resource.id = canonical_id.clone();
            resource.created_at = created_at;
            resource.updated_at = Utc::now();

            tx.execute(
                "UPDATE resources SET
                    service_type = $2,
                    provider = $3,
                    is_persistent = $4,
                    ownership = $5,
                    status = $6,
                    configuration = $7,
                    updated_at = $8
                 WHERE id = $1;",
                &[
                    &canonical_id,
                    &resource.service_type,
                    &resource.provider,
                    &resource.is_persistent,
                    &ownership_str,
                    &status_str,
                    &resource.configuration,
                    &resource.updated_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to update resource: {}", e)))?;
        } else {
            if resource.created_at == DateTime::<Utc>::default() {
                resource.created_at = Utc::now();
            }
            resource.updated_at = Utc::now();

            tx.execute(
                "INSERT INTO resources (
                    id, workspace_id, name, service_type, provider, is_persistent,
                    ownership, status, configuration, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);",
                &[
                    &resource.id,
                    &resource.workspace_id,
                    &resource.name,
                    &resource.service_type,
                    &resource.provider,
                    &resource.is_persistent,
                    &ownership_str,
                    &status_str,
                    &resource.configuration,
                    &resource.created_at,
                    &resource.updated_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to insert resource: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(resource)
    }

    async fn get_resource(&self, id: &str) -> Result<Option<Resource>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, name, workspace_id, service_type, provider, is_persistent,
                        ownership, status, configuration, created_at, updated_at
                 FROM resources WHERE id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get resource: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_resource_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_resources(&self, workspace_id: &str) -> Result<Vec<Resource>> {
        let client = self.pool.get().await?;
        let rows = if workspace_id == "*" || workspace_id.is_empty() {
            client
                .query(
                    "SELECT id, name, workspace_id, service_type, provider, is_persistent,
                            ownership, status, configuration, created_at, updated_at
                     FROM resources ORDER BY created_at ASC;",
                    &[],
                )
                .await
        } else {
            client
                .query(
                    "SELECT id, name, workspace_id, service_type, provider, is_persistent,
                            ownership, status, configuration, created_at, updated_at
                     FROM resources WHERE workspace_id = $1 ORDER BY created_at ASC;",
                    &[&workspace_id],
                )
                .await
        }
        .map_err(|e| SmooothError::Storage(format!("Failed to list resources: {}", e)))?;

        let mut resources = Vec::with_capacity(rows.len());
        for row in rows {
            resources.push(Self::map_resource_row(&row)?);
        }
        Ok(resources)
    }

    async fn delete_resource(&self, id: &str) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute("DELETE FROM resources WHERE id = $1;", &[&id])
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to delete resource: {}", e)))?;
        Ok(())
    }

    // =========================================================================
    // Service Bindings & Batch Rotation
    // =========================================================================

    async fn save_binding(&self, mut binding: ServiceBinding) -> Result<()> {
        let client = self.pool.get().await?;

        if binding.source_resource_id == binding.target_resource_id {
            return Err(SmooothError::Validation(
                "Self-binding prohibited: source and target resources must differ".to_string(),
            ));
        }

        // Resolve workspace if omitted
        if binding.workspace_id.is_empty() {
            let src_row = client
                .query_opt(
                    "SELECT workspace_id FROM resources WHERE id = $1;",
                    &[&binding.source_resource_id],
                )
                .await
                .map_err(|e| {
                    SmooothError::Storage(format!("Failed to query source resource: {}", e))
                })?;

            match src_row {
                Some(r) => binding.workspace_id = r.get("workspace_id"),
                None => {
                    return Err(SmooothError::Storage(format!(
                        "Cannot save service binding: source resource '{}' not found",
                        binding.source_resource_id
                    )))
                }
            }
        }

        Self::ensure_workspace(&*client, &binding.workspace_id).await?;

        // Fail-closed validation: verify target resource belongs to the identical workspace
        let target_row = client
            .query_opt(
                "SELECT workspace_id FROM resources WHERE id = $1;",
                &[&binding.target_resource_id],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to query target resource: {}", e))
            })?;

        match target_row {
            Some(r) => {
                let target_ws: String = r.get("workspace_id");
                if target_ws != binding.workspace_id {
                    return Err(SmooothError::Security(format!(
                        "Cross-workspace binding prohibited: source workspace '{}' vs target workspace '{}'",
                        binding.workspace_id, target_ws
                    )));
                }
            }
            None => {
                return Err(SmooothError::Storage(format!(
                    "Cannot save service binding: target resource '{}' not found in workspace '{}'",
                    binding.target_resource_id, binding.workspace_id
                )));
            }
        }

        let stage_str = match binding.rotation_stage {
            RotationStage::Overlapping => "overlapping",
            RotationStage::Verifying => "verifying",
            RotationStage::Retired => "retired",
            RotationStage::RolledBack => "rolled_back",
            _ => "active",
        };
        let ownership_str = match binding.ownership {
            ResourceOwnership::External => "external",
            ResourceOwnership::CustomWorkload => "discovered",
            ResourceOwnership::SmooothManaged => "smoooth_managed",
        };
        let version_i32 = binding.binding_version as i32;
        let endpoint_json = binding
            .resolved_endpoint
            .as_ref()
            .map(|ep| serde_json::to_value(ep).map_err(|e| SmooothError::Storage(e.to_string())))
            .transpose()?;
        let details_json = binding
            .safe_connection_details
            .as_ref()
            .map(|d| serde_json::to_value(d).map_err(|e| SmooothError::Storage(e.to_string())))
            .transpose()?;
        let env_keys_json = serde_json::to_value(&binding.exposed_env_keys).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize exposed env keys: {}", e))
        })?;
        let env_map_json = serde_json::to_value(&binding.env_mappings).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize env mappings: {}", e))
        })?;

        client
            .execute(
                "INSERT INTO service_bindings (
                    id, workspace_id, source_resource_id, target_resource_id, binding_type,
                    rotation_stage, ownership, binding_version, key_fingerprint,
                    resolved_endpoint, safe_connection_details, encrypted_credentials,
                    exposed_env_keys, env_mappings, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12,
                    $13, $14, $15, $16
                )
                ON CONFLICT (id) DO UPDATE SET
                    workspace_id = EXCLUDED.workspace_id,
                    source_resource_id = EXCLUDED.source_resource_id,
                    target_resource_id = EXCLUDED.target_resource_id,
                    binding_type = EXCLUDED.binding_type,
                    rotation_stage = EXCLUDED.rotation_stage,
                    ownership = EXCLUDED.ownership,
                    binding_version = EXCLUDED.binding_version,
                    key_fingerprint = EXCLUDED.key_fingerprint,
                    resolved_endpoint = EXCLUDED.resolved_endpoint,
                    safe_connection_details = EXCLUDED.safe_connection_details,
                    encrypted_credentials = EXCLUDED.encrypted_credentials,
                    exposed_env_keys = EXCLUDED.exposed_env_keys,
                    env_mappings = EXCLUDED.env_mappings,
                    updated_at = EXCLUDED.updated_at;",
                &[
                    &binding.id,
                    &binding.workspace_id,
                    &binding.source_resource_id,
                    &binding.target_resource_id,
                    &binding.binding_type,
                    &stage_str,
                    &ownership_str,
                    &version_i32,
                    &binding.key_fingerprint,
                    &endpoint_json,
                    &details_json,
                    &binding.encrypted_credentials,
                    &env_keys_json,
                    &env_map_json,
                    &binding.created_at,
                    &binding.updated_at,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to save service binding: {}", e)))?;

        Ok(())
    }

    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, workspace_id, source_resource_id, target_resource_id, binding_type,
                        rotation_stage, ownership, binding_version, key_fingerprint,
                        resolved_endpoint, safe_connection_details, encrypted_credentials,
                        exposed_env_keys, env_mappings, created_at, updated_at
                 FROM service_bindings WHERE id = $1;",
                &[&id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get binding: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_binding_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn list_bindings(&self) -> Result<Vec<ServiceBinding>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, workspace_id, source_resource_id, target_resource_id, binding_type,
                        rotation_stage, ownership, binding_version, key_fingerprint,
                        resolved_endpoint, safe_connection_details, encrypted_credentials,
                        exposed_env_keys, env_mappings, created_at, updated_at
                 FROM service_bindings ORDER BY created_at ASC;",
                &[],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to list bindings: {}", e)))?;

        let mut bindings = Vec::with_capacity(rows.len());
        for row in rows {
            bindings.push(Self::map_binding_row(&row)?);
        }
        Ok(bindings)
    }

    async fn rotate_service_bindings_batch(
        &self,
        expected_old_fingerprint: &str,
        new_fingerprint: &str,
        updated_bindings: Vec<ServiceBinding>,
        audit_event: AuditEvent,
    ) -> Result<usize> {
        let mut client = self.pool.get().await?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction begin error: {}", e)))?;

        // 1. Transactional validation of current fingerprints
        for binding in &updated_bindings {
            let row = tx
                .query_opt(
                    "SELECT key_fingerprint FROM service_bindings WHERE id = $1 FOR UPDATE;",
                    &[&binding.id],
                )
                .await
                .map_err(|e| {
                    SmooothError::Storage(format!("Failed to query binding fingerprint: {}", e))
                })?;

            if let Some(r) = row {
                let existing_fp: String = r.get("key_fingerprint");
                if !existing_fp.is_empty()
                    && !expected_old_fingerprint.is_empty()
                    && existing_fp != expected_old_fingerprint
                {
                    return Err(SmooothError::Storage(format!(
                        "Binding '{}' fingerprint mismatch: expected '{}', found '{}'",
                        binding.id, expected_old_fingerprint, existing_fp
                    )));
                }
            }

            if binding.key_fingerprint != new_fingerprint {
                return Err(SmooothError::Storage(format!(
                    "Updated binding '{}' has incorrect new fingerprint: expected '{}', found '{}'",
                    binding.id, new_fingerprint, binding.key_fingerprint
                )));
            }
        }

        // 2. Persist updated bindings
        let count = updated_bindings.len();
        for binding in updated_bindings {
            let stage_str = match binding.rotation_stage {
                RotationStage::Overlapping => "overlapping",
                RotationStage::Verifying => "verifying",
                RotationStage::Retired => "retired",
                RotationStage::RolledBack => "rolled_back",
                _ => "active",
            };
            let ownership_str = match binding.ownership {
                ResourceOwnership::External => "external",
                ResourceOwnership::CustomWorkload => "discovered",
                ResourceOwnership::SmooothManaged => "smoooth_managed",
            };
            let version_i32 = binding.binding_version as i32;
            let endpoint_json = binding
                .resolved_endpoint
                .as_ref()
                .map(|ep| {
                    serde_json::to_value(ep).map_err(|e| SmooothError::Storage(e.to_string()))
                })
                .transpose()?;
            let details_json = binding
                .safe_connection_details
                .as_ref()
                .map(|d| serde_json::to_value(d).map_err(|e| SmooothError::Storage(e.to_string())))
                .transpose()?;
            let env_keys_json = serde_json::to_value(&binding.exposed_env_keys).map_err(|e| {
                SmooothError::Storage(format!("Failed to serialize exposed env keys: {}", e))
            })?;
            let env_map_json = serde_json::to_value(&binding.env_mappings).map_err(|e| {
                SmooothError::Storage(format!("Failed to serialize env mappings: {}", e))
            })?;

            tx.execute(
                "INSERT INTO service_bindings (
                    id, workspace_id, source_resource_id, target_resource_id, binding_type,
                    rotation_stage, ownership, binding_version, key_fingerprint,
                    resolved_endpoint, safe_connection_details, encrypted_credentials,
                    exposed_env_keys, env_mappings, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12,
                    $13, $14, $15, $16
                )
                ON CONFLICT (id) DO UPDATE SET
                    rotation_stage = EXCLUDED.rotation_stage,
                    ownership = EXCLUDED.ownership,
                    binding_version = EXCLUDED.binding_version,
                    key_fingerprint = EXCLUDED.key_fingerprint,
                    resolved_endpoint = EXCLUDED.resolved_endpoint,
                    safe_connection_details = EXCLUDED.safe_connection_details,
                    encrypted_credentials = EXCLUDED.encrypted_credentials,
                    exposed_env_keys = EXCLUDED.exposed_env_keys,
                    env_mappings = EXCLUDED.env_mappings,
                    updated_at = NOW();",
                &[
                    &binding.id,
                    &binding.workspace_id,
                    &binding.source_resource_id,
                    &binding.target_resource_id,
                    &binding.binding_type,
                    &stage_str,
                    &ownership_str,
                    &version_i32,
                    &binding.key_fingerprint,
                    &endpoint_json,
                    &details_json,
                    &binding.encrypted_credentials,
                    &env_keys_json,
                    &env_map_json,
                    &binding.created_at,
                    &binding.updated_at,
                ],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to update binding in batch: {}", e))
            })?;
        }

        // 3. Atomically record rotation audit event
        Self::ensure_workspace(&tx, &audit_event.workspace_id).await?;
        tx.execute(
            "INSERT INTO audit_events (
                workspace_id, actor, action, target_resource_id, plan_hash, outcome, redacted_evidence, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8);",
            &[
                &audit_event.workspace_id,
                &audit_event.actor,
                &audit_event.action,
                &audit_event.target_resource_id,
                &audit_event.plan_hash,
                &audit_event.outcome,
                &audit_event.redacted_evidence,
                &audit_event.timestamp,
            ],
        )
        .await
        .map_err(|e| SmooothError::Storage(format!("Failed to record audit event in batch: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| SmooothError::Storage(format!("Transaction commit error: {}", e)))?;

        Ok(count)
    }

    // =========================================================================
    // Audit Log
    // =========================================================================

    async fn record_audit(&self, event: AuditEvent) -> Result<()> {
        let client = self.pool.get().await?;
        Self::ensure_workspace(&*client, &event.workspace_id).await?;

        client
            .execute(
                "INSERT INTO audit_events (
                    workspace_id, actor, action, target_resource_id, plan_hash, outcome, redacted_evidence, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8);",
                &[
                    &event.workspace_id,
                    &event.actor,
                    &event.action,
                    &event.target_resource_id,
                    &event.plan_hash,
                    &event.outcome,
                    &event.redacted_evidence,
                    &event.timestamp,
                ],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to record audit event: {}", e)))?;

        Ok(())
    }

    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>> {
        let client = self.pool.get().await?;
        let rows = if workspace_id == "*" || workspace_id.is_empty() {
            client
                .query(
                    "SELECT id, workspace_id, actor, action, target_resource_id,
                            plan_hash, outcome, redacted_evidence, created_at
                     FROM audit_events ORDER BY id ASC;",
                    &[],
                )
                .await
        } else {
            client
                .query(
                    "SELECT id, workspace_id, actor, action, target_resource_id,
                            plan_hash, outcome, redacted_evidence, created_at
                     FROM audit_events WHERE workspace_id = $1 ORDER BY id ASC;",
                    &[&workspace_id],
                )
                .await
        }
        .map_err(|e| SmooothError::Storage(format!("Failed to list audit events: {}", e)))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(Self::map_audit_row(&row));
        }
        Ok(events)
    }

    // =========================================================================
    // Capability Revocation & Nonce Tracking
    // =========================================================================

    async fn record_revocation(
        &self,
        capability_id: &uuid::Uuid,
        reason: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO capability_revocations (capability_id, reason, revoked_at, expires_at)
                 VALUES ($1, $2, NOW(), $3)
                 ON CONFLICT (capability_id) DO UPDATE SET
                    reason = EXCLUDED.reason,
                    expires_at = EXCLUDED.expires_at;",
                &[&capability_id, &reason, &expires_at],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to record revocation: {}", e)))?;
        Ok(())
    }

    async fn is_revoked(&self, capability_id: &uuid::Uuid) -> Result<bool> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT 1 FROM capability_revocations WHERE capability_id = $1;",
                &[&capability_id],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to query revocation status: {}", e))
            })?;

        Ok(row.is_some())
    }

    async fn record_used_nonce(&self, nonce: &str, expires_at: DateTime<Utc>) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO used_nonces (nonce, expires_at, consumed_at)
                 VALUES ($1, $2, NOW())
                 ON CONFLICT (nonce) DO UPDATE SET expires_at = EXCLUDED.expires_at;",
                &[&nonce, &expires_at],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to record used nonce: {}", e)))?;
        Ok(())
    }

    async fn is_nonce_used(&self, nonce: &str) -> Result<bool> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT 1 FROM used_nonces WHERE nonce = $1 AND expires_at > NOW();",
                &[&nonce],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to check if nonce is used: {}", e))
            })?;

        Ok(row.is_some())
    }

    async fn consume_nonce(&self, nonce: &str, expires_at: DateTime<Utc>) -> Result<bool> {
        if expires_at <= Utc::now() {
            return Ok(false);
        }

        let client = self.pool.get().await?;
        let rows_affected = client
            .execute(
                "INSERT INTO used_nonces (nonce, expires_at, consumed_at)
                 VALUES ($1, $2, NOW())
                 ON CONFLICT (nonce) DO NOTHING;",
                &[&nonce, &expires_at],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to atomically consume nonce: {}", e))
            })?;

        Ok(rows_affected == 1)
    }

    // =========================================================================
    // Fencing Generation Tokens
    // =========================================================================

    async fn get_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT epoch FROM fencing_epochs WHERE workspace_id = $1;",
                &[&workspace_id],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to get fencing epoch: {}", e)))?;

        match row {
            Some(r) => {
                let epoch_i64: i64 = r.get(0);
                Ok(epoch_i64.max(1) as u64)
            }
            None => Ok(1),
        }
    }

    async fn increment_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        let client = self.pool.get().await?;
        Self::ensure_workspace(&*client, workspace_id).await?;

        let row = client
            .query_one(
                "INSERT INTO fencing_epochs (workspace_id, epoch, updated_at)
                 VALUES ($1, 2, NOW())
                 ON CONFLICT (workspace_id) DO UPDATE SET
                    epoch = fencing_epochs.epoch + 1,
                    updated_at = NOW()
                 RETURNING epoch;",
                &[&workspace_id],
            )
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to increment fencing epoch: {}", e))
            })?;

        let epoch_i64: i64 = row.get(0);
        Ok(epoch_i64 as u64)
    }

    // =========================================================================
    // Backup & Disaster Recovery
    // =========================================================================

    async fn export_snapshot(&self, workspace_id: &str) -> Result<StateSnapshot> {
        let is_global = workspace_id == "*" || workspace_id.is_empty();

        let agents = self
            .list_agents(if is_global { None } else { Some(workspace_id) })
            .await?
            .into_iter()
            .map(|a| (a.agent_id.clone(), a))
            .collect();

        let plans = self
            .list_plans()
            .await?
            .into_iter()
            .filter(|p| is_global || p.spec.workspace_id == workspace_id)
            .map(|p| (p.id.clone(), p))
            .collect();

        let operations = self
            .list_operations(if is_global { None } else { Some(workspace_id) })
            .await?
            .into_iter()
            .map(|o| (o.id.clone(), o))
            .collect();

        let resources = self
            .list_resources(workspace_id)
            .await?
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();

        let bindings = self
            .list_bindings()
            .await?
            .into_iter()
            .filter(|b| is_global || b.workspace_id == workspace_id)
            .map(|b| (b.id.clone(), b))
            .collect();

        let audit_events = self.list_audit_events(workspace_id).await?;

        let client = self.pool.get().await?;

        // Fencing epochs
        let mut fencing_epochs = HashMap::new();
        if is_global {
            let epoch_rows = client
                .query("SELECT workspace_id, epoch FROM fencing_epochs;", &[])
                .await
                .map_err(|e| {
                    SmooothError::Storage(format!("Failed to query fencing epochs: {}", e))
                })?;
            for row in epoch_rows {
                let ws: String = row.get(0);
                let ep: i64 = row.get(1);
                fencing_epochs.insert(ws, ep as u64);
            }
        } else {
            let epoch = self.get_fencing_epoch(workspace_id).await?;
            fencing_epochs.insert(workspace_id.to_string(), epoch);
        }

        // Revocations
        let mut revocations = HashMap::new();
        let rev_rows = client
            .query(
                "SELECT capability_id, reason, revoked_at, expires_at FROM capability_revocations;",
                &[],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to export revocations: {}", e)))?;
        for row in rev_rows {
            let cap_id: uuid::Uuid = row.get(0);
            let reason: String = row.get(1);
            let revoked_at: DateTime<Utc> = row.get(2);
            let expires_at: DateTime<Utc> = row.get(3);
            revocations.insert(
                cap_id.to_string(),
                RevocationEntry {
                    capability_id: cap_id.to_string(),
                    reason,
                    revoked_at,
                    expires_at,
                },
            );
        }

        // Used nonces
        let mut used_nonces = HashMap::new();
        let nonce_rows = client
            .query(
                "SELECT nonce, expires_at FROM used_nonces WHERE expires_at > NOW();",
                &[],
            )
            .await
            .map_err(|e| SmooothError::Storage(format!("Failed to export used nonces: {}", e)))?;
        for row in nonce_rows {
            let nonce: String = row.get(0);
            let exp: DateTime<Utc> = row.get(1);
            used_nonces.insert(nonce, exp);
        }

        Ok(StateSnapshot {
            version: 1,
            agents,
            plans,
            idempotency: HashMap::new(),
            operations,
            resources,
            bindings,
            audit_events,
            revocations,
            used_nonces,
            fencing_epochs,
        })
    }

    async fn import_snapshot(&self, snapshot: StateSnapshot) -> Result<()> {
        for agent in snapshot.agents.into_values() {
            self.save_agent(agent).await?;
        }

        for plan in snapshot.plans.into_values() {
            self.save_plan(plan).await?;
        }

        for resource in snapshot.resources.into_values() {
            self.save_resource(resource).await?;
        }

        for operation in snapshot.operations.into_values() {
            self.save_operation(operation).await?;
        }

        for binding in snapshot.bindings.into_values() {
            self.save_binding(binding).await?;
        }

        for event in snapshot.audit_events {
            self.record_audit(event).await?;
        }

        for entry in snapshot.revocations.into_values() {
            if let Ok(cap_uuid) = uuid::Uuid::parse_str(&entry.capability_id) {
                self.record_revocation(&cap_uuid, &entry.reason, entry.expires_at)
                    .await?;
            }
        }

        for (nonce, exp) in snapshot.used_nonces {
            self.record_used_nonce(&nonce, exp).await?;
        }

        let client = self.pool.get().await?;
        for (ws, epoch) in snapshot.fencing_epochs {
            Self::ensure_workspace(&*client, &ws).await?;
            let epoch_i64 = epoch as i64;
            client
                .execute(
                    "INSERT INTO fencing_epochs (workspace_id, epoch, updated_at)
                     VALUES ($1, $2, NOW())
                     ON CONFLICT (workspace_id) DO UPDATE SET epoch = EXCLUDED.epoch, updated_at = NOW();",
                    &[&ws, &epoch_i64],
                )
                .await
                .map_err(|e| SmooothError::Storage(format!("Failed to import fencing epoch: {}", e)))?;
        }

        Ok(())
    }
}
