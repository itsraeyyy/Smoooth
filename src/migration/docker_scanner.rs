use std::collections::HashMap;

use super::types::{DetectedWorkload, IngressRoutePreview, PortMapping, VolumeMapping};
use crate::error::SmooothError;
use crate::models::{CanonicalActionSpec, CanonicalExecutionSpec, PolicyConstraints};

pub struct DockerHostScanner;

impl DockerHostScanner {
    /// Inspect running containers via Docker CLI or daemon and generate Smoooth migration workloads
    pub async fn scan(
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let output = tokio::process::Command::new("docker")
            .args(["ps", "-a", "--format", "{{.ID}}"])
            .output()
            .await
            .map_err(|e| SmooothError::Adapter(format!("Failed to execute 'docker ps': {}. Ensure Docker daemon is running.", e)))?;

        if !output.status.success() {
            return Err(SmooothError::Adapter(format!(
                "Docker ps returned error: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let id_list = String::from_utf8_lossy(&output.stdout);
        let container_ids: Vec<&str> = id_list.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

        if container_ids.is_empty() {
            return Err(SmooothError::Validation(
                "No containers found on this Docker host to import.".to_string(),
            ));
        }

        let inspect_output = tokio::process::Command::new("docker")
            .arg("inspect")
            .args(&container_ids)
            .output()
            .await
            .map_err(|e| SmooothError::Adapter(format!("Failed to execute 'docker inspect': {}", e)))?;

        if !inspect_output.status.success() {
            return Err(SmooothError::Adapter(format!(
                "Docker inspect returned error: {}",
                String::from_utf8_lossy(&inspect_output.stderr)
            )));
        }

        let inspect_json: serde_json::Value = serde_json::from_slice(&inspect_output.stdout)
            .map_err(|e| SmooothError::Adapter(format!("Failed to parse Docker inspect JSON: {}", e)))?;

        let containers = inspect_json.as_array().ok_or_else(|| {
            SmooothError::Adapter("Unexpected Docker inspect response format (expected array)".to_string())
        })?;

        let mut workloads = Vec::new();
        let mut canonical_actions = Vec::new();
        let dependencies = Vec::new();
        let mut ingress_routes = Vec::new();

        for c in containers {
            let raw_name = c.get("Name").and_then(|n| n.as_str()).unwrap_or("container");
            let clean_name = raw_name.trim_start_matches('/').to_lowercase().replace('_', "-");

            // Filter out Smoooth itself or internal helper containers
            if clean_name.starts_with("smoooth-control") || clean_name == "smoooth" {
                continue;
            }

            let image = c
                .get("Config")
                .and_then(|cfg| cfg.get("Image"))
                .and_then(|i| i.as_str())
                .unwrap_or("unknown:latest")
                .to_string();

            // Extract environment variables
            let mut env_vars = HashMap::new();
            if let Some(envs) = c.get("Config").and_then(|cfg| cfg.get("Env")).and_then(|e| e.as_array()) {
                for item in envs {
                    if let Some(line) = item.as_str() {
                        if let Some((k, v)) = line.split_once('=') {
                            env_vars.insert(k.to_string(), v.to_string());
                        }
                    }
                }
            }

            // Extract ports
            let mut ports = Vec::new();
            if let Some(ports_obj) = c.get("NetworkSettings").and_then(|n| n.get("Ports")).and_then(|p| p.as_object()) {
                for (port_proto, bindings) in ports_obj {
                    let (cp_str, proto) = port_proto.split_once('/').unwrap_or((port_proto.as_str(), "tcp"));
                    let cp = cp_str.parse::<u16>().unwrap_or(8080);
                    let mut host_port = None;
                    if let Some(b_arr) = bindings.as_array() {
                        if let Some(first) = b_arr.first() {
                            if let Some(hp_str) = first.get("HostPort").and_then(|hp| hp.as_str()) {
                                host_port = hp_str.parse::<u16>().ok();
                            }
                        }
                    }
                    ports.push(PortMapping {
                        container_port: cp,
                        host_port,
                        protocol: proto.to_string(),
                    });
                }
            }

            // Extract volume mounts
            let mut volumes = Vec::new();
            if let Some(mounts) = c.get("Mounts").and_then(|m| m.as_array()) {
                for m in mounts {
                    let src = m.get("Source").and_then(|s| s.as_str()).unwrap_or("");
                    let dst = m.get("Destination").and_then(|d| d.as_str()).unwrap_or("");
                    let m_type = m.get("Type").and_then(|t| t.as_str()).unwrap_or("");
                    let ro = m.get("RW").and_then(|rw| rw.as_bool()).map(|rw| !rw).unwrap_or(false);

                    if !src.is_empty() && !dst.is_empty() {
                        volumes.push(VolumeMapping {
                            source: src.to_string(),
                            target: dst.to_string(),
                            is_named_volume: m_type == "volume",
                            read_only: ro,
                        });
                    }
                }
            }

            // Extract labels (Coolify, Traefik, Caddy)
            let mut domains = Vec::new();
            let mut is_coolify_managed = false;
            let mut notes = Vec::new();

            if let Some(labels) = c.get("Config").and_then(|cfg| cfg.get("Labels")).and_then(|l| l.as_object()) {
                for (k, v) in labels {
                    if let Some(v_str) = v.as_str() {
                        if k.starts_with("coolify") {
                            is_coolify_managed = true;
                        }

                        // Traefik router domains
                        if k.starts_with("traefik.http.routers") && k.ends_with(".rule") {
                            if let Some(start) = v_str.find("Host(") {
                                let rest = &v_str[start + 5..];
                                if let Some(end) = rest.find(')') {
                                    let d = rest[..end].trim().trim_matches('`').trim_matches('"').trim_matches('\'');
                                    if !d.is_empty() && !domains.contains(&d.to_string()) {
                                        domains.push(d.to_string());
                                    }
                                }
                            }
                        } else if k == "coolify.fqdn" || k == "caddy" || k == "fqdn" {
                            for d in v_str.split(',') {
                                let clean = d.trim().trim_start_matches("https://").trim_start_matches("http://").trim_matches('/');
                                if !clean.is_empty() && !domains.contains(&clean.to_string()) {
                                    domains.push(clean.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if is_coolify_managed {
                notes.push("Discovered Coolify metadata labels on container.".to_string());
            }

            let (provider, service_type, is_db) = Self::classify_container(&image, &clean_name);
            let is_persistent = is_db || !volumes.is_empty();

            let primary_port = ports
                .iter()
                .find(|p| p.host_port.is_some())
                .map(|p| p.container_port)
                .or_else(|| ports.first().map(|p| p.container_port))
                .unwrap_or(8080);

            for dom in &domains {
                ingress_routes.push(IngressRoutePreview {
                    domain: dom.clone(),
                    target_resource: clean_name.clone(),
                    target_port: primary_port,
                    tls: true,
                    path_prefix: None,
                });
            }

            let mut ports_map = serde_json::Map::new();
            for p in &ports {
                let hp = p.host_port.unwrap_or(p.container_port);
                ports_map.insert(format!("{}/{}", p.container_port, p.protocol), serde_json::json!(hp));
            }

            let action_config = serde_json::json!({
                "image": image,
                "port": primary_port,
                "ports": ports_map,
                "env": env_vars,
                "volumes": volumes.iter().map(|v| format!("{}:{}", v.source, v.target)).collect::<Vec<_>>(),
                "workspace_id": workspace
            });

            canonical_actions.push(CanonicalActionSpec {
                resource_name: clean_name.clone(),
                provider: provider.clone(),
                service_type: service_type.clone(),
                action_type: "create".to_string(),
                config: action_config,
                is_persistent,
                exposes_network_publicly: !ports.is_empty() || !domains.is_empty(),
            });

            workloads.push(DetectedWorkload {
                name: clean_name,
                service_type,
                provider,
                image,
                build_source: None,
                ports,
                env_vars,
                volumes,
                domains,
                is_persistent,
                depends_on: Vec::new(),
                notes,
            });
        }

        let canonical_spec = CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: workspace.to_string(),
            environment: "production".to_string(),
            target_id: "local_docker_node".to_string(),
            actions: canonical_actions,
            dependencies,
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: true,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(200),
            },
        };

        Ok((workloads, canonical_spec, ingress_routes))
    }

    fn classify_container(image: &str, name: &str) -> (String, String, bool) {
        let img_lower = image.to_lowercase();
        let name_lower = name.to_lowercase();

        if img_lower.contains("postgres") || name_lower.contains("postgres") || name_lower.contains("psql") {
            ("postgres".to_string(), "database".to_string(), true)
        } else if img_lower.contains("redis") || name_lower.contains("redis") || img_lower.contains("dragonfly") || img_lower.contains("keydb") {
            ("redis".to_string(), "cache".to_string(), true)
        } else if img_lower.contains("mongo") || name_lower.contains("mongo") {
            ("mongo".to_string(), "database".to_string(), true)
        } else if img_lower.contains("mysql") || img_lower.contains("mariadb") || name_lower.contains("mysql") {
            ("docker".to_string(), "database".to_string(), true)
        } else if name_lower.contains("worker") || img_lower.contains("worker") {
            ("docker".to_string(), "worker".to_string(), false)
        } else {
            ("docker".to_string(), "compute".to_string(), false)
        }
    }
}
