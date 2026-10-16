use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::models::{Capability, Resource, ResourceStatus};

pub mod docker;
pub mod mongo;
pub mod postgres;
pub mod redis;

pub use docker::DockerAdapter;
pub use mongo::MongoAdapter;
pub use postgres::PostgresAdapter;
pub use redis::RedisAdapter;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvidence {
    pub is_healthy: bool,
    pub latency_ms: Option<u64>,
    pub message: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterGuarantees {
    pub provider_name: String,
    pub supports_persistent_volumes: bool,
    pub supports_staged_dual_credentials: bool,
    pub supports_safe_rollback: bool,
    pub supports_drift_detection: bool,
    pub is_experimental: bool,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Unique provider identifier (e.g. "docker", "aws", "fly_io", "kubernetes")
    fn provider_name(&self) -> &str;

    /// Explicit provider operational guarantees
    fn guarantees(&self) -> AdapterGuarantees;

    /// Declare supported capabilities and support levels dynamically
    fn supported_capabilities(&self) -> Vec<Capability>;

    /// Idempotent resource provisioning
    async fn provision(&self, name: &str, config: &serde_json::Value, is_persistent: bool) -> Result<Resource>;

    /// Active state inspection (querying actual underlying provider)
    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus>;

    /// Active health check returning verifiable evidence
    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence>;

    /// Reconcile observed state towards intended state
    async fn reconcile(&self, resource_id: &str, intended_config: &serde_json::Value) -> Result<ResourceStatus>;

    /// Safe destruction with persistent volume preservation safeguards
    async fn destroy(&self, resource_id: &str, preserve_persistent_data: bool) -> Result<()>;
}
