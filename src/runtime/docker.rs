use async_trait::async_trait;
use bollard::container::{
    InspectContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
    RestartContainerOptions, StopContainerOptions,
};
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::adapters::docker::client::DockerClient;
use crate::adapters::docker::containers::{ContainerManager, ContainerSpec, PortMappingSpec};
use crate::adapters::HealthEvidence;
use crate::error::{Result, SmooothError};
use crate::models::connection::VolumeMount;
use crate::models::workload::WorkloadSpec;
use crate::models::ResourceStatus;
use crate::runtime::{ContainerInspection, ContainerSummary, RuntimeProvider};

/// Docker implementation of the unified RuntimeProvider.
/// Interacts with the Docker daemon via bollard with resilient fallbacks.
pub struct DockerRuntimeProvider {
    client: Option<DockerClient>,
}

impl DockerRuntimeProvider {
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
                "Docker daemon is not accessible or not running on host.".to_string(),
            )
        })
    }
}

impl Default for DockerRuntimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuntimeProvider for DockerRuntimeProvider {
    fn runtime_name(&self) -> &str {
        "docker"
    }

    async fn is_available(&self) -> Result<bool> {
        if let Some(client) = &self.client {
            Ok(client.ping().await.is_ok())
        } else {
            Ok(false)
        }
    }

    async fn create_and_start(&self, spec: &WorkloadSpec) -> Result<String> {
        // Enforce execution boundary validation
        spec.validate()?;

        let client = self.require_client()?;

        // Convert WorkloadSpec into container engine ContainerSpec
        let mut port_bindings = HashMap::new();
        let mut custom_port_bindings = Vec::new();
        for p in &spec.ports {
            if p.protocol.eq_ignore_ascii_case("tcp") {
                port_bindings.insert(p.container_port, p.host_port);
            }
            custom_port_bindings.push(PortMappingSpec {
                container_port: p.container_port,
                host_port: p.host_port,
                protocol: p.protocol.clone(),
            });
        }

        let volume_mounts = spec
            .volumes
            .iter()
            .map(|v| VolumeMount {
                volume_name: v.volume_name.clone(),
                container_path: v.container_path.clone(),
                read_only: v.read_only,
            })
            .collect();

        let env_vec: Vec<String> = spec
            .env
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let mut labels = HashMap::new();
        labels.insert("smoooth.managed".to_string(), "true".to_string());
        labels.insert("smoooth.workload".to_string(), spec.name.clone());

        let nano_cpus = (spec.limits.cpu_millicores as i64) * 1_000_000;

        let container_spec = ContainerSpec {
            name: spec.name.clone(),
            image: spec.image.clone(),
            cmd: spec.cmd.clone(),
            env: env_vec,
            port_bindings,
            custom_port_bindings,
            volume_mounts,
            network_name: spec.network.clone(),
            labels,
            restart_policy: Some(spec.restart_policy.clone()),
            memory_bytes: Some(spec.limits.memory_bytes),
            cpu_shares: None,
            nano_cpus: Some(nano_cpus),
            health_spec: spec.health_check.clone(),
        };

        ContainerManager::create_and_start(client, &container_spec).await
    }

    async fn stop(&self, id: &str, timeout_secs: u32) -> Result<()> {
        if let Ok(client) = self.require_client() {
            let docker = client.inner();
            let options = StopContainerOptions {
                t: timeout_secs as i64,
            };
            if let Err(e) = docker.stop_container(id, Some(options)).await {
                tracing::warn!(
                    "Bollard stop_container failed for '{}': {}. Trying CLI fallback...",
                    id,
                    e
                );
                let _ = tokio::process::Command::new("docker")
                    .args(["stop", "-t", &timeout_secs.to_string(), id])
                    .output()
                    .await;
            }
            Ok(())
        } else {
            let out = tokio::process::Command::new("docker")
                .args(["stop", "-t", &timeout_secs.to_string(), id])
                .output()
                .await
                .map_err(|e| SmooothError::DockerUnavailable(e.to_string()))?;
            if !out.status.success() {
                return Err(SmooothError::ContainerFailed(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ));
            }
            Ok(())
        }
    }

    async fn restart(&self, id: &str, timeout_secs: u32) -> Result<()> {
        if let Ok(client) = self.require_client() {
            let docker = client.inner();
            let options = RestartContainerOptions {
                t: timeout_secs as isize,
            };
            if let Err(e) = docker.restart_container(id, Some(options)).await {
                tracing::warn!(
                    "Bollard restart_container failed for '{}': {}. Trying CLI fallback...",
                    id,
                    e
                );
                let out = tokio::process::Command::new("docker")
                    .args(["restart", "-t", &timeout_secs.to_string(), id])
                    .output()
                    .await
                    .map_err(|e| SmooothError::DockerUnavailable(e.to_string()))?;
                if !out.status.success() {
                    return Err(SmooothError::ContainerFailed(
                        String::from_utf8_lossy(&out.stderr).to_string(),
                    ));
                }
            }
            Ok(())
        } else {
            let out = tokio::process::Command::new("docker")
                .args(["restart", "-t", &timeout_secs.to_string(), id])
                .output()
                .await
                .map_err(|e| SmooothError::DockerUnavailable(e.to_string()))?;
            if !out.status.success() {
                return Err(SmooothError::ContainerFailed(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ));
            }
            Ok(())
        }
    }

    async fn delete(&self, id: &str, force: bool) -> Result<()> {
        if let Ok(client) = self.require_client() {
            let docker = client.inner();
            let options = RemoveContainerOptions {
                force,
                v: false, // NEVER remove volumes
                ..Default::default()
            };
            if let Err(e) = docker.remove_container(id, Some(options)).await {
                tracing::warn!(
                    "Bollard remove_container failed for '{}': {}. Trying CLI fallback...",
                    id,
                    e
                );
                let mut args = vec!["rm"];
                if force {
                    args.push("-f");
                }
                args.push(id);
                let _ = tokio::process::Command::new("docker")
                    .args(args)
                    .output()
                    .await;
            }
            Ok(())
        } else {
            let mut args = vec!["rm"];
            if force {
                args.push("-f");
            }
            args.push(id);
            let out = tokio::process::Command::new("docker")
                .args(args)
                .output()
                .await
                .map_err(|e| SmooothError::DockerUnavailable(e.to_string()))?;
            if !out.status.success() {
                return Err(SmooothError::ContainerFailed(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ));
            }
            Ok(())
        }
    }

    async fn inspect(&self, id: &str) -> Result<ContainerInspection> {
        let client = self.require_client()?;
        let docker = client.inner();

        let inspect = docker
            .inspect_container(id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| {
                SmooothError::ContainerFailed(format!(
                    "Failed to inspect container '{}': {}",
                    id, e
                ))
            })?;

        let is_running = inspect
            .state
            .as_ref()
            .and_then(|s| s.running)
            .unwrap_or(false);
        let status = if is_running {
            ResourceStatus::Active
        } else {
            ResourceStatus::Stopped
        };

        let mut ip_address = None;
        if let Some(net) = &inspect.network_settings {
            if let Some(ip) = &net.ip_address {
                if !ip.is_empty() {
                    ip_address = Some(ip.clone());
                }
            }
            if ip_address.is_none() {
                if let Some(networks) = &net.networks {
                    for n in networks.values() {
                        if let Some(ip) = &n.ip_address {
                            if !ip.is_empty() {
                                ip_address = Some(ip.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let mut port_mappings = HashMap::new();
        if let Some(ns) = &inspect.network_settings {
            if let Some(ports) = &ns.ports {
                for (k, v) in ports {
                    let parts: Vec<&str> = k.split('/').collect();
                    if let Ok(c_port) = parts[0].parse::<u16>() {
                        if let Some(Some(bindings)) = v.as_ref().map(|b| b.first()) {
                            if let Some(hp) = &bindings.host_port {
                                if let Ok(h_port) = hp.parse::<u16>() {
                                    port_mappings.insert(c_port, h_port);
                                }
                            }
                        }
                    }
                }
            }
        }

        let name = inspect
            .name
            .unwrap_or_else(|| id.to_string())
            .trim_start_matches('/')
            .to_string();
        let image = inspect
            .config
            .as_ref()
            .and_then(|c| c.image.clone())
            .unwrap_or_default();
        let created_at = inspect.created;

        Ok(ContainerInspection {
            id: inspect.id.unwrap_or_else(|| id.to_string()),
            name,
            status,
            is_running,
            ip_address,
            port_mappings,
            created_at,
            image,
        })
    }

    async fn logs(&self, id: &str, tail: usize) -> Result<String> {
        if let Ok(client) = self.require_client() {
            let docker = client.inner();
            let options = LogsOptions {
                stdout: true,
                stderr: true,
                tail: tail.to_string(),
                ..Default::default()
            };

            let mut stream = docker.logs(id, Some(options));
            let mut logs_buf = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(log_output) => logs_buf.push_str(&log_output.to_string()),
                    Err(e) => {
                        tracing::warn!("Stream log chunk error for '{}': {}", id, e);
                        break;
                    }
                }
            }

            if !logs_buf.is_empty() {
                return Ok(logs_buf);
            }
        }

        // CLI fallback for logs
        let out = tokio::process::Command::new("docker")
            .args(["logs", "--tail", &tail.to_string(), id])
            .output()
            .await
            .map_err(|e| SmooothError::DockerUnavailable(e.to_string()))?;

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(combined)
    }

    async fn health_check(&self, id: &str) -> Result<HealthEvidence> {
        let inspect = self.inspect(id).await?;
        Ok(HealthEvidence {
            is_healthy: inspect.is_running,
            latency_ms: Some(1),
            message: if inspect.is_running {
                format!("Container '{}' is running and active", id)
            } else {
                format!(
                    "Container '{}' is not running (status: {:?})",
                    id, inspect.status
                )
            },
            details: serde_json::json!({
                "id": inspect.id,
                "name": inspect.name,
                "ip": inspect.ip_address,
                "ports": inspect.port_mappings,
            }),
        })
    }

    async fn list_containers(&self, label_filter: Option<&str>) -> Result<Vec<ContainerSummary>> {
        if let Ok(client) = self.require_client() {
            let docker = client.inner();
            let mut filters = HashMap::new();
            if let Some(lbl) = label_filter {
                filters.insert("label".to_string(), vec![lbl.to_string()]);
            }

            let options = ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            };

            let list = docker
                .list_containers(Some(options))
                .await
                .map_err(|e| SmooothError::ContainerFailed(e.to_string()))?;

            let summaries = list
                .into_iter()
                .map(|c| ContainerSummary {
                    id: c.id.unwrap_or_default(),
                    names: c.names.unwrap_or_default(),
                    image: c.image.unwrap_or_default(),
                    state: c.state.unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                })
                .collect();

            Ok(summaries)
        } else {
            Ok(vec![])
        }
    }
}
