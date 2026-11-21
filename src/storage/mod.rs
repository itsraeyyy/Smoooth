use crate::error::Result;
use crate::models::{
    AgentIdentity, IdempotencyKey, IdempotencyRecord, Operation, Plan, Resource, ServiceBinding,
};
use async_trait::async_trait;

pub mod local;
pub mod postgres;

pub use local::{FileLock, LocalStorage, RevocationEntry, StateSnapshot};
pub use postgres::{
    AppliedMigration, AppliedMigrationSummary, Migration, MigrationReport, MigrationRunner,
    MIGRATION_ADVISORY_LOCK_ID,
};

#[derive(Debug, Clone, PartialEq)]
pub enum OperationInitOutcome {
    Created(Operation),
    Existing(Operation),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    pub id: u64,
    pub workspace_id: String,
    pub actor: String,
    pub action: String,
    pub target_resource_id: Option<String>,
    pub plan_hash: Option<String>,
    pub outcome: String,
    pub redacted_evidence: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    // Agents & Access Security
    async fn save_agent(&self, agent: AgentIdentity) -> Result<()>;
    async fn get_agent(&self, id: &str) -> Result<Option<AgentIdentity>>;
    async fn get_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<AgentIdentity>>;
    async fn list_agents(&self, workspace_id: Option<&str>) -> Result<Vec<AgentIdentity>>;
    async fn revoke_agent(&self, id: &str) -> Result<()>;

    // Plans
    async fn save_plan(&self, plan: Plan) -> Result<()>;
    async fn get_plan(&self, id: &str) -> Result<Option<Plan>>;
    async fn list_plans(&self) -> Result<Vec<Plan>>;
    async fn delete_plan(&self, id: &str) -> Result<()>;

    // Idempotency & Operation Atomic Reservation
    async fn check_idempotency(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>>;
    async fn record_idempotency(&self, record: IdempotencyRecord) -> Result<()>;
    async fn check_or_record_idempotency(
        &self,
        record: IdempotencyRecord,
    ) -> Result<Option<IdempotencyRecord>>;
    async fn reserve_or_get_operation(
        &self,
        idempotency_record: IdempotencyRecord,
        operation: Operation,
    ) -> Result<OperationInitOutcome>;

    // Operations & Fencing Leases
    async fn save_operation(&self, operation: Operation) -> Result<()>;
    async fn get_operation(&self, id: &str) -> Result<Option<Operation>>;
    async fn list_operations(&self, workspace_id: Option<&str>) -> Result<Vec<Operation>>;
    async fn update_operation(&self, operation: Operation) -> Result<()>;

    // Resources
    async fn save_resource(&self, resource: Resource) -> Result<Resource>;
    async fn get_resource(&self, id: &str) -> Result<Option<Resource>>;
    async fn list_resources(&self, workspace_id: &str) -> Result<Vec<Resource>>;
    async fn delete_resource(&self, id: &str) -> Result<()>;

    // Service Bindings & Batch Rotation
    async fn save_binding(&self, binding: ServiceBinding) -> Result<()>;
    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>>;
    async fn list_bindings(&self) -> Result<Vec<ServiceBinding>>;
    async fn rotate_service_bindings_batch(
        &self,
        expected_old_fingerprint: &str,
        new_fingerprint: &str,
        updated_bindings: Vec<ServiceBinding>,
        audit_event: AuditEvent,
    ) -> Result<usize>;

    // Audit Log
    async fn record_audit(&self, event: AuditEvent) -> Result<()>;
    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>>;

    // Capability Revocation & Nonce Tracking
    async fn record_revocation(
        &self,
        capability_id: &uuid::Uuid,
        reason: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()>;
    async fn is_revoked(&self, capability_id: &uuid::Uuid) -> Result<bool>;
    async fn record_used_nonce(
        &self,
        nonce: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()>;
    async fn is_nonce_used(&self, nonce: &str) -> Result<bool>;
    async fn consume_nonce(
        &self,
        nonce: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool>;
    async fn get_fencing_epoch(&self, workspace_id: &str) -> Result<u64>;
    async fn increment_fencing_epoch(&self, workspace_id: &str) -> Result<u64>;

    // Backup & Disaster Recovery
    async fn export_snapshot(&self, workspace_id: &str) -> Result<StateSnapshot>;
    async fn import_snapshot(&self, snapshot: StateSnapshot) -> Result<()>;
}
