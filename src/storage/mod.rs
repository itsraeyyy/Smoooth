use async_trait::async_trait;
use crate::error::Result;
use crate::models::{
    IdempotencyKey, IdempotencyRecord, Operation, Plan, Resource, ServiceBinding,
};

pub mod local;

pub use local::LocalStorage;

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
    // Plans
    async fn save_plan(&self, plan: Plan) -> Result<()>;
    async fn get_plan(&self, id: &str) -> Result<Option<Plan>>;

    // Idempotency
    async fn check_idempotency(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>>;
    async fn record_idempotency(&self, record: IdempotencyRecord) -> Result<()>;
    async fn check_or_record_idempotency(&self, record: IdempotencyRecord) -> Result<Option<IdempotencyRecord>>;

    // Operations & Fencing Leases
    async fn save_operation(&self, operation: Operation) -> Result<()>;
    async fn get_operation(&self, id: &str) -> Result<Option<Operation>>;
    async fn update_operation(&self, operation: Operation) -> Result<()>;

    // Resources
    async fn save_resource(&self, resource: Resource) -> Result<()>;
    async fn get_resource(&self, id: &str) -> Result<Option<Resource>>;
    async fn list_resources(&self, workspace_id: &str) -> Result<Vec<Resource>>;
    async fn delete_resource(&self, id: &str) -> Result<()>;

    // Service Bindings
    async fn save_binding(&self, binding: ServiceBinding) -> Result<()>;
    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>>;

    // Audit Log
    async fn record_audit(&self, event: AuditEvent) -> Result<()>;
    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>>;
}
