use crate::error::Result;
use crate::models::{Capability, Resource, ResourceStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    async fn provision(
        &self,
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
    ) -> Result<Resource>;

    /// Active state inspection (querying actual underlying provider)
    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus>;

    /// Active health check returning verifiable evidence
    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence>;

    /// Reconcile observed state towards intended state
    async fn reconcile(
        &self,
        resource_id: &str,
        intended_config: &serde_json::Value,
    ) -> Result<ResourceStatus>;

    /// Safe destruction with persistent volume preservation safeguards
    async fn destroy(&self, resource_id: &str, preserve_persistent_data: bool) -> Result<()>;

    /// Authorized resource provisioning requiring a verified execution permit
    async fn authorized_provision(
        &self,
        permit: &crate::security::VerifiedExecutionPermit<'_>,
        target_urn: &crate::models::ResourceUrn,
        expected_digest: [u8; 32],
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
        current_epoch: u64,
    ) -> Result<Resource> {
        if std::time::Instant::now() > permit.deadline_monotonic {
            return Err(crate::error::SmooothError::Security(
                "Execution permit expired by monotonic deadline".into(),
            ));
        }
        if permit.fencing_epoch != current_epoch {
            return Err(crate::error::SmooothError::Security(format!(
                "Fencing epoch mismatch: permit={}, current={}",
                permit.fencing_epoch, current_epoch
            )));
        }
        if &permit.target_urn != target_urn {
            return Err(crate::error::SmooothError::Security(format!(
                "Target URN mismatch: permit={}, requested={}",
                permit.target_urn, target_urn
            )));
        }
        if permit.operation_digest != expected_digest {
            return Err(crate::error::SmooothError::Security(
                "Operation payload digest mismatch".into(),
            ));
        }
        self.provision(name, config, is_persistent).await
    }

    /// Authorized destruction requiring a verified execution permit
    async fn authorized_destroy(
        &self,
        permit: &crate::security::VerifiedExecutionPermit<'_>,
        target_urn: &crate::models::ResourceUrn,
        expected_digest: [u8; 32],
        scope: crate::models::DestructionScope,
        current_epoch: u64,
    ) -> Result<()> {
        if std::time::Instant::now() > permit.deadline_monotonic {
            return Err(crate::error::SmooothError::Security(
                "Execution permit expired by monotonic deadline".into(),
            ));
        }
        if permit.fencing_epoch != current_epoch {
            return Err(crate::error::SmooothError::Security(format!(
                "Fencing epoch mismatch: permit={}, current={}",
                permit.fencing_epoch, current_epoch
            )));
        }
        if &permit.target_urn != target_urn {
            return Err(crate::error::SmooothError::Security(format!(
                "Target URN mismatch: permit={}, requested={}",
                permit.target_urn, target_urn
            )));
        }
        if permit.operation_digest != expected_digest {
            return Err(crate::error::SmooothError::Security(
                "Operation payload digest mismatch".into(),
            ));
        }
        match (&permit.permitted_action, scope) {
            (
                crate::models::Action::Destroy(
                    crate::models::DestructionScope::ResourceAndPersistentData,
                ),
                crate::models::DestructionScope::ResourceAndPersistentData,
            ) => {}
            (
                crate::models::Action::Destroy(crate::models::DestructionScope::ResourceOnly),
                crate::models::DestructionScope::ResourceOnly,
            ) => {}
            (
                crate::models::Action::Destroy(crate::models::DestructionScope::ResourceOnly),
                crate::models::DestructionScope::ResourceAndPersistentData,
            ) => {
                return Err(crate::error::SmooothError::Security(
                    "Permit authorizes ResourceOnly destruction; cannot destroy persistent data"
                        .into(),
                ));
            }
            _ => {
                return Err(crate::error::SmooothError::Security(
                    "Permit does not authorize Destroy action".into(),
                ));
            }
        }
        let preserve_data = scope == crate::models::DestructionScope::ResourceOnly;
        let res_id = target_urn.segments().last().map(|s| s.id()).unwrap_or("");
        self.destroy(res_id, preserve_data).await
    }
}
