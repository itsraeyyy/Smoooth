use std::collections::HashMap;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::{HostConfig, PortBinding, PortMap};

use crate::adapters::docker::client::DockerClient;
use crate::error::{Result, SmooothError};
use crate::models::ResourceStatus;

#[derive(Debug, Clone)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub env: Vec<String>,
    pub port_bindings: HashMap<u16, u16>, // container_port -> host_port
    pub volume_mounts: Vec<(String, String)>, // volume_name -> container_path
    pub network_name: Option<String>,
    pub labels: HashMap<String, String>,
}

pub struct ContainerManager;

impl ContainerManager {
    /// Create and start container idempotently
    pub async fn create_and_start(
        client: &DockerClient,
        spec: &ContainerSpec,
    ) -> Result<String> {
        let docker = client.inner();

        // 1. Check if container with identical name already exists (Idempotent recovery & Fencing check)
        if let Ok(inspect) = docker.inspect_container(&spec.name, None::<InspectContainerOptions>).await {
            let container_id = inspect.id.clone().unwrap_or_else(|| spec.name.clone());

            // Fencing validation: check if existing container has a higher epoch
            if let Some(config) = &inspect.config {
                if let Some(existing_labels) = &config.labels {
                    let existing_epoch: u64 = existing_labels
                        .get("smoooth.fencing_epoch")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let caller_epoch: u64 = spec
                        .labels
                        .get("smoooth.fencing_epoch")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);

                    if existing_epoch > caller_epoch {
                        return Err(SmooothError::FencingViolation(format!(
                            "Container '{}' is controlled by higher fencing epoch ({}) than worker epoch ({})",
                            spec.name, existing_epoch, caller_epoch
                        )));
                    }
                }
            }

            let is_running = inspect.state.and_then(|s| s.running).unwrap_or(false);

            if is_running {
                return Ok(container_id);
            } else {
                // If container exists but is stopped, start it
                docker
                    .start_container(&container_id, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(|e| SmooothError::ContainerFailed(format!("Failed to start existing container '{}': {}", spec.name, e)))?;
                return Ok(container_id);
            }
        }

        // 2. Configure HostConfig (port bindings, volume mounts, network)
        let mut port_map: PortMap = HashMap::new();
        for (c_port, h_port) in &spec.port_bindings {
            let binding = PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(h_port.to_string()),
            };
            port_map.insert(format!("{}/tcp", c_port), Some(vec![binding]));
        }

        let binds: Vec<String> = spec
            .volume_mounts
            .iter()
            .map(|(vol, path)| format!("{}:{}", vol, path))
            .collect();

        let host_config = HostConfig {
            port_bindings: Some(port_map),
            binds: Some(binds),
            network_mode: spec.network_name.clone(),
            ..Default::default()
        };

        let mut labels = spec.labels.clone();
        labels.insert("smoooth.managed".to_string(), "true".to_string());

        let config = Config {
            image: Some(spec.image.clone()),
            env: Some(spec.env.clone()),
            labels: Some(labels),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: spec.name.clone(),
            platform: None,
        };

        // 3. Create Container
        let created = docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to create container '{}': {}", spec.name, e)))?;

        // 4. Start Container
        docker
            .start_container(&created.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to start container '{}': {}", spec.name, e)))?;

        Ok(created.id)
    }

    /// Inspect container live state
    pub async fn inspect(
        client: &DockerClient,
        name_or_id: &str,
    ) -> Result<ResourceStatus> {
        let docker = client.inner();

        match docker.inspect_container(name_or_id, None::<InspectContainerOptions>).await {
            Ok(inspect) => {
                if let Some(state) = inspect.state {
                    if state.running.unwrap_or(false) {
                        Ok(ResourceStatus::Active)
                    } else if state.restarting.unwrap_or(false) {
                        Ok(ResourceStatus::Provisioning)
                    } else if state.paused.unwrap_or(false) {
                        Ok(ResourceStatus::Degraded)
                    } else {
                        Ok(ResourceStatus::Stopped)
                    }
                } else {
                    Ok(ResourceStatus::Degraded)
                }
            }
            Err(_) => Ok(ResourceStatus::Terminated),
        }
    }

    /// Stop and remove container, checking fencing lease epoch if specified
    pub async fn stop_and_remove_with_fencing(
        client: &DockerClient,
        name_or_id: &str,
        caller_epoch: Option<u64>,
    ) -> Result<()> {
        let docker = client.inner();

        if let Ok(inspect) = docker.inspect_container(name_or_id, None::<InspectContainerOptions>).await {
            if let Some(caller) = caller_epoch {
                if let Some(config) = &inspect.config {
                    if let Some(labels) = &config.labels {
                        let existing_epoch: u64 = labels
                            .get("smoooth.fencing_epoch")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0);
                        if existing_epoch > caller {
                            return Err(SmooothError::FencingViolation(format!(
                                "Cannot remove container '{}': owned by higher fencing epoch ({}) vs worker epoch ({})",
                                name_or_id, existing_epoch, caller
                            )));
                        }
                    }
                }
            }
        }

        let stop_options = StopContainerOptions { t: 5 };
        let _ = docker.stop_container(name_or_id, Some(stop_options)).await;

        let remove_options = RemoveContainerOptions {
            force: true,
            v: false, // NEVER remove volumes with container removal!
            ..Default::default()
        };

        docker
            .remove_container(name_or_id, Some(remove_options))
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to remove container '{}': {}", name_or_id, e)))?;

        Ok(())
    }

    /// Stop and remove container
    pub async fn stop_and_remove(
        client: &DockerClient,
        name_or_id: &str,
    ) -> Result<()> {
        Self::stop_and_remove_with_fencing(client, name_or_id, None).await
    }
}
