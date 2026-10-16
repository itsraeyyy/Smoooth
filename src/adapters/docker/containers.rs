use std::collections::HashMap;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
    RenameContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::models::{HostConfig, PortBinding, PortMap, RestartPolicy, RestartPolicyNameEnum};

use crate::adapters::docker::client::DockerClient;
use crate::error::{Result, SmooothError};
use crate::models::{connection::VolumeMount, HealthSpec, ResourceStatus};

#[derive(Debug, Clone)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub cmd: Option<Vec<String>>,
    pub env: Vec<String>,
    pub port_bindings: HashMap<u16, u16>, // container_port -> host_port (0 = ephemeral)
    pub volume_mounts: Vec<VolumeMount>, // volume_name -> container_path with read_only
    pub network_name: Option<String>,
    pub labels: HashMap<String, String>,
    pub restart_policy: Option<String>,
    pub memory_bytes: Option<i64>,
    pub cpu_shares: Option<i64>,
    pub nano_cpus: Option<i64>,
    pub health_spec: Option<HealthSpec>,
}

pub struct ContainerManager;

impl ContainerManager {
    /// Check if a label key conforms to the explicit allowed metadata namespaces
    pub fn is_allowed_label_key(key: &str) -> bool {
        let k = key.to_lowercase();
        if k.starts_with("smoooth.") || k.starts_with("app.") || k.starts_with("service.") || k.starts_with("org.opencontainers.") || k.starts_with("com.docker.") {
            return true;
        }
        matches!(
            k.as_str(),
            "safe_env_flag" | "environment" | "version" | "role" | "tier"
        )
    }

    /// Check if a label value contains sensitive credential patterns
    pub fn contains_sensitive_value(val: &str) -> bool {
        let v_l = val.to_lowercase();
        v_l.contains("password")
            || v_l.contains("secret")
            || v_l.contains("token")
            || v_l.contains("bearer")
            || v_l.contains("private key")
    }

    /// Zero-credential label security guardrail: strictly enforce explicit key allowlist
    /// and value sanitization.
    pub fn filter_safe_labels(labels: &HashMap<String, String>) -> HashMap<String, String> {
        let mut filtered = HashMap::new();
        for (k, v) in labels {
            if Self::is_allowed_label_key(k) && !Self::contains_sensitive_value(v) {
                let k_l = k.to_lowercase();
                if !k_l.contains("pass") && !k_l.contains("secret") && !k_l.contains("token") && !k_l.contains("cred") && !k_l.contains("key") {
                    filtered.insert(k.clone(), v.clone());
                }
            }
        }
        filtered
    }

    /// Internal helper to create and start a container instance
    async fn create_and_start_instance(
        docker: &bollard::Docker,
        spec: &ContainerSpec,
    ) -> Result<String> {
        let mut port_map: PortMap = HashMap::new();
        for (c_port, h_port) in &spec.port_bindings {
            let binding = PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: if *h_port == 0 { None } else { Some(h_port.to_string()) },
            };
            port_map.insert(format!("{}/tcp", c_port), Some(vec![binding]));
        }

        let binds: Vec<String> = spec
            .volume_mounts
            .iter()
            .map(|vm| vm.to_docker_bind())
            .collect();

        let restart_policy = spec.restart_policy.as_deref().map(|policy| {
            let name = match policy {
                "always" => Some(RestartPolicyNameEnum::ALWAYS),
                "unless-stopped" => Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                "on-failure" => Some(RestartPolicyNameEnum::ON_FAILURE),
                "no" => Some(RestartPolicyNameEnum::NO),
                _ => None,
            };
            RestartPolicy {
                name,
                maximum_retry_count: if policy == "on-failure" { Some(3) } else { None },
            }
        });

        let host_config = HostConfig {
            port_bindings: Some(port_map),
            binds: Some(binds),
            network_mode: spec.network_name.clone(),
            restart_policy,
            memory: spec.memory_bytes,
            cpu_shares: spec.cpu_shares,
            nano_cpus: spec.nano_cpus,
            ..Default::default()
        };

        let mut labels = Self::filter_safe_labels(&spec.labels);
        labels.insert("smoooth.managed".to_string(), "true".to_string());
        if let Some(ref hs) = spec.health_spec {
            if let Ok(serialized_hs) = serde_json::to_string(hs) {
                labels.insert("smoooth.health_spec".to_string(), serialized_hs);
            }
        }

        let config = Config {
            image: Some(spec.image.clone()),
            cmd: spec.cmd.clone(),
            env: Some(spec.env.clone()),
            labels: Some(labels),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: spec.name.clone(),
            platform: None,
        };

        let created = docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to create container '{}': {}", spec.name, e)))?;

        docker
            .start_container(&created.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to start container '{}': {}", spec.name, e)))?;

        Ok(created.id)
    }

    /// Create and start container idempotently with transactional rollback on replacement failure
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

            // Check if environment variables have changed (binding update or credential rotation)
            let mut env_has_drifted = false;
            if let Some(existing_config) = &inspect.config {
                if let Some(existing_env) = &existing_config.env {
                    for expected in &spec.env {
                        if !existing_env.contains(expected) {
                            env_has_drifted = true;
                            break;
                        }
                    }
                } else if !spec.env.is_empty() {
                    env_has_drifted = true;
                }
            }

            if is_running && !env_has_drifted {
                return Ok(container_id);
            } else if env_has_drifted {
                tracing::info!(
                    "Workload '{}' environment or bindings changed. Executing safe container replacement with transactional rollback...",
                    spec.name
                );
                // 1. Rename existing container to backup name so original name is freed up
                let backup_name = format!("{}_smoooth_backup_{}", spec.name, uuid::Uuid::new_v4().simple());
                docker.rename_container(&container_id, RenameContainerOptions { name: &backup_name }).await
                    .map_err(|e| SmooothError::ContainerFailed(format!("Failed to prepare backup container for '{}': {}", spec.name, e)))?;

                // 2. Stop backup container to release host ports and mounts cleanly
                let _ = docker.stop_container(&backup_name, None::<StopContainerOptions>).await;

                // 3. Attempt to create and start the new replacement container
                match Self::create_and_start_instance(docker, spec).await {
                    Ok(new_id) => {
                        // Replacement succeeded! Safely remove the old backup container (named volumes are preserved)
                        let _ = docker.remove_container(&backup_name, None::<RemoveContainerOptions>).await;
                        return Ok(new_id);
                    }
                    Err(create_err) => {
                        tracing::error!(
                            "Replacement container creation/start failed for '{}': {}. Initiating automatic rollback...",
                            spec.name, create_err
                        );
                        // Clean up failed replacement container if partially created
                        let _ = docker.remove_container(&spec.name, Some(RemoveContainerOptions { force: true, ..Default::default() })).await;

                        // ROLLBACK: Rename backup container back to original name and restart it
                        let _ = docker.rename_container(&backup_name, RenameContainerOptions { name: &spec.name }).await;
                        let _ = docker.start_container(&spec.name, None::<StartContainerOptions<String>>).await;

                        return Err(SmooothError::ContainerFailed(format!(
                            "Container replacement failed for '{}': {}; automatically rolled back to previous container instance.",
                            spec.name, create_err
                        )));
                    }
                }
            } else {
                // If container exists but is stopped, start it
                docker
                    .start_container(&container_id, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(|e| SmooothError::ContainerFailed(format!("Failed to start existing container '{}': {}", spec.name, e)))?;
                return Ok(container_id);
            }
        }

        // 2. Fresh container creation if it doesn't already exist
        Self::create_and_start_instance(docker, spec).await
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

    /// Retrieve internal container IP address from Docker network
    pub async fn get_container_ip(
        client: &DockerClient,
        name_or_id: &str,
    ) -> Result<String> {
        let docker = client.inner();
        let inspect = docker
            .inspect_container(name_or_id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to inspect container '{}': {}", name_or_id, e)))?;

        if let Some(network_settings) = inspect.network_settings {
            if let Some(networks) = network_settings.networks {
                for (_, net) in networks {
                    if let Some(ip) = net.ip_address {
                        if !ip.is_empty() {
                            return Ok(ip);
                        }
                    }
                }
            }
            if let Some(ip) = network_settings.ip_address {
                if !ip.is_empty() {
                    return Ok(ip);
                }
            }
        }

        Err(SmooothError::NotFound(format!("No IP address found for container '{}'", name_or_id)))
    }

    /// Check if container publishes any ports to the host interface
    pub async fn has_host_port_binding(
        client: &DockerClient,
        name_or_id: &str,
    ) -> Result<bool> {
        let docker = client.inner();
        let inspect = docker
            .inspect_container(name_or_id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to inspect container '{}': {}", name_or_id, e)))?;

        if let Some(host_config) = inspect.host_config {
            if let Some(port_bindings) = host_config.port_bindings {
                for (_c_port, bindings) in port_bindings {
                    if let Some(b_list) = bindings {
                        if !b_list.is_empty() {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    /// Retrieve mapped host port for a specific container port (dynamically allocated ephemeral or fixed)
    pub async fn get_host_port(
        client: &DockerClient,
        name_or_id: &str,
        container_port: u16,
    ) -> Result<u16> {
        let docker = client.inner();
        let inspect = docker
            .inspect_container(name_or_id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to inspect container '{}': {}", name_or_id, e)))?;

        let port_key = format!("{}/tcp", container_port);

        // Check network_settings.ports first (populated after container start with ephemeral ports)
        if let Some(ns) = &inspect.network_settings {
            if let Some(pm) = &ns.ports {
                if let Some(Some(bindings)) = pm.get(&port_key) {
                    if let Some(first) = bindings.first() {
                        if let Some(hp) = &first.host_port {
                            if let Ok(parsed) = hp.parse::<u16>() {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to host_config.port_bindings
        if let Some(hc) = &inspect.host_config {
            if let Some(pm) = &hc.port_bindings {
                if let Some(Some(bindings)) = pm.get(&port_key) {
                    if let Some(first) = bindings.first() {
                        if let Some(hp) = &first.host_port {
                            if let Ok(parsed) = hp.parse::<u16>() {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
        }

        Err(SmooothError::NotFound(format!(
            "No host port mapping found for container '{}' port {}",
            name_or_id, container_port
        )))
    }
}
