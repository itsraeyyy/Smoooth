use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

pub mod client;
pub mod containers;
pub mod health;
pub mod images;
pub mod networks;
pub mod volumes;

use client::DockerClient;
use containers::{ContainerManager, ContainerSpec};
use health::HealthProber;
use images::ImageManager;
use networks::NetworkManager;
use volumes::VolumeManager;

use crate::adapters::{AdapterGuarantees, HealthEvidence, ProviderAdapter};
use crate::error::{Result, SmooothError};
use crate::models::{
    Capability, HealthSpec, Resource, ResourceOwnership, ResourceStatus, SupportLevel, VolumeMount,
};

pub struct DockerAdapter {
    client: Option<DockerClient>,
}

impl DockerAdapter {
    pub fn new() -> Self {
        let client = DockerClient::try_new().ok();
        Self { client }
    }

    pub fn with_client(client: DockerClient) -> Self {
        Self {
            client: Some(client),
        }
    }

    fn require_client(&self) -> Result<&DockerClient> {
        self.client.as_ref().ok_or_else(|| {
            SmooothError::DockerUnavailable(
                "Docker daemon is not accessible or not running.".to_string(),
            )
        })
    }

    pub async fn get_container_ip(&self, name_or_id: &str) -> Result<String> {
        let client = self.require_client()?;
        ContainerManager::get_container_ip(client, name_or_id).await
    }

    pub async fn has_host_port_binding(&self, name_or_id: &str) -> Result<bool> {
        let client = self.require_client()?;
        ContainerManager::has_host_port_binding(client, name_or_id).await
    }

    pub async fn get_host_port(&self, name_or_id: &str, container_port: u16) -> Result<u16> {
        let client = self.require_client()?;
        ContainerManager::get_host_port(client, name_or_id, container_port).await
    }

    pub fn client(&self) -> Result<&DockerClient> {
        self.require_client()
    }
}

impl Default for DockerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderAdapter for DockerAdapter {
    fn provider_name(&self) -> &str {
        "docker"
    }

    fn guarantees(&self) -> AdapterGuarantees {
        AdapterGuarantees {
            provider_name: "docker".to_string(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                id: "docker_container".to_string(),
                name: "Docker Container Workload".to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                support_level: SupportLevel::SelfHosted,
                description:
                    "Runs isolated application containers on local or customer Docker engine."
                        .to_string(),
                operations: vec![
                    "provision".to_string(),
                    "inspect".to_string(),
                    "health_check".to_string(),
                    "reconcile".to_string(),
                    "destroy".to_string(),
                ],
            },
            Capability {
                id: "docker_network".to_string(),
                name: "Docker Bridge Network".to_string(),
                provider: "docker".to_string(),
                service_type: "network".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "Isolated inter-service bridge network with DNS resolution."
                    .to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string()],
            },
            Capability {
                id: "docker_volume".to_string(),
                name: "Docker Named Volume".to_string(),
                provider: "docker".to_string(),
                service_type: "storage".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "Persistent named host volumes with data retention guarantees."
                    .to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string()],
            },
        ]
    }

    async fn provision(
        &self,
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
    ) -> Result<Resource> {
        let client = self.require_client()?;

        let image = config
            .get("image")
            .and_then(|v| v.as_str())
            .unwrap_or("alpine:latest");

        let workspace_id = config
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let network_name = config
            .get("network")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pinned_digest = config.get("digest").and_then(|v| v.as_str());

        // 1. Ensure image is pulled
        ImageManager::ensure_image(client, image, pinned_digest).await?;

        // 2. Ensure network exists if requested
        if let Some(net) = &network_name {
            NetworkManager::ensure_network(client, net, workspace_id).await?;
        }

        // 3. Ensure persistent volume exists if requested
        let mut volume_mounts = Vec::new();
        if let Some(vols) = config.get("volumes").and_then(|v| v.as_array()) {
            for v in vols {
                if let (Some(vol_name), Some(mount_path)) = (
                    v.get("volume_name").and_then(|s| s.as_str()),
                    v.get("container_path").and_then(|s| s.as_str()),
                ) {
                    let read_only = v
                        .get("read_only")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false);
                    VolumeManager::ensure_volume(client, vol_name, workspace_id, is_persistent)
                        .await?;
                    volume_mounts.push(VolumeMount::new(
                        vol_name.to_string(),
                        mount_path.to_string(),
                        read_only,
                    ));
                }
            }
        }

        // 4. Configure ports
        let mut port_bindings = HashMap::new();
        if let Some(ports) = config.get("ports").and_then(|v| v.as_object()) {
            for (c_port_str, h_port_val) in ports {
                if let (Ok(c_port), Some(h_port)) = (c_port_str.parse::<u16>(), h_port_val.as_u64())
                {
                    port_bindings.insert(c_port, h_port as u16);
                }
            }
        }
        if port_bindings.is_empty() {
            if let Some(h_port) = config.get("port").and_then(|v| v.as_u64()) {
                let c_port = if image.contains("nginx") || image.contains("http") { 80 } else { h_port as u16 };
                port_bindings.insert(c_port, h_port as u16);
            }
        }

        // 5. Configure environment variables
        let mut env = Vec::new();
        if let Some(env_obj) = config.get("env").and_then(|v| v.as_object()) {
            for (k, val) in env_obj {
                if let Some(val_str) = val.as_str() {
                    env.push(format!("{}={}", k, val_str));
                }
            }
        }

        let mut labels = HashMap::new();
        if let Some(epoch) = config.get("fencing_epoch").and_then(|v| v.as_u64()) {
            labels.insert("smoooth.fencing_epoch".to_string(), epoch.to_string());
        }
        if let Some(worker) = config.get("worker_id").and_then(|v| v.as_str()) {
            labels.insert("smoooth.worker_id".to_string(), worker.to_string());
        }

        let cmd = config.get("cmd").and_then(|v| {
            if let Some(arr) = v.as_array() {
                Some(
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect(),
                )
            } else {
                v.as_str()
                    .map(|s| vec!["sh".to_string(), "-c".to_string(), s.to_string()])
            }
        });

        let restart_policy = config
            .get("restart_policy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let memory_bytes = config.get("memory_bytes").and_then(|v| v.as_i64());

        let cpu_shares = config.get("cpu_shares").and_then(|v| v.as_i64());

        let nano_cpus = config.get("nano_cpus").and_then(|v| v.as_i64());

        let health_spec: Option<HealthSpec> = config
            .get("health_check")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let container_spec = ContainerSpec {
            name: name.to_string(),
            image: image.to_string(),
            cmd,
            env,
            port_bindings,
            volume_mounts,
            network_name,
            labels,
            restart_policy,
            memory_bytes,
            cpu_shares,
            nano_cpus,
            health_spec,
        };

        let container_id = ContainerManager::create_and_start(client, &container_spec).await?;

        let ownership = config
            .get("ownership")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(ResourceOwnership::SmooothManaged);

        Ok(Resource {
            id: container_id,
            name: name.to_string(),
            workspace_id: workspace_id.to_string(),
            service_type: "container".to_string(),
            provider: "docker".to_string(),
            is_persistent,
            ownership,
            status: ResourceStatus::Active,
            configuration: config.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus> {
        let client = self.require_client()?;
        ContainerManager::inspect(client, resource_id).await
    }

    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence> {
        let client = self.require_client()?;
        HealthProber::check_health(client, resource_id, None).await
    }

    async fn reconcile(
        &self,
        resource_id: &str,
        _intended_config: &serde_json::Value,
    ) -> Result<ResourceStatus> {
        self.inspect(resource_id).await
    }

    async fn destroy(&self, resource_id: &str, _preserve_persistent_data: bool) -> Result<()> {
        let client = self.require_client()?;
        ContainerManager::stop_and_remove(client, resource_id).await
    }
}
