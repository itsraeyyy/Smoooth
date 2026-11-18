use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::HealthEvidence;
use crate::error::Result;
use crate::models::workload::WorkloadSpec;
use crate::models::ResourceStatus;

pub mod docker;
pub mod mock;

pub use docker::DockerRuntimeProvider;
pub use mock::MockRuntimeProvider;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInspection {
    pub id: String,
    pub name: String,
    pub status: ResourceStatus,
    pub is_running: bool,
    pub ip_address: Option<String>,
    pub port_mappings: HashMap<u16, u16>,
    pub created_at: Option<String>,
    pub image: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: String,
    pub status: String,
}

/// Unified, container-engine-agnostic execution runtime abstraction.
/// Decouples Smoooth's Orchestrator, Adapters, and API handlers from
/// direct Docker daemon bindings and raw shell invocations.
#[async_trait]
pub trait RuntimeProvider: Send + Sync {
    /// Identifier for the runtime provider backend ("docker", "podman", "mock")
    fn runtime_name(&self) -> &str;

    /// Verifies whether the runtime daemon/socket is accessible and ready
    async fn is_available(&self) -> Result<bool>;

    /// Idempotently create and start a container from a validated WorkloadSpec
    async fn create_and_start(&self, spec: &WorkloadSpec) -> Result<String>;

    /// Gracefully stop a running container
    async fn stop(&self, id: &str, timeout_secs: u32) -> Result<()>;

    /// Restart a running or stopped container
    async fn restart(&self, id: &str, timeout_secs: u32) -> Result<()>;

    /// Remove a container from the runtime
    async fn delete(&self, id: &str, force: bool) -> Result<()>;

    /// Inspect live state, IP address, and port allocations
    async fn inspect(&self, id: &str) -> Result<ContainerInspection>;

    /// Fetch trailing log stream for the container
    async fn logs(&self, id: &str, tail: usize) -> Result<String>;

    /// Perform an active health check probe
    async fn health_check(&self, id: &str) -> Result<HealthEvidence>;

    /// List active containers matching optional label filters
    async fn list_containers(&self, label_filter: Option<&str>) -> Result<Vec<ContainerSummary>>;
}

/// Returns the default runtime provider for the host environment
pub fn default_runtime_provider() -> Arc<dyn RuntimeProvider> {
    Arc::new(DockerRuntimeProvider::new())
}
