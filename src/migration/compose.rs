use serde_yaml::Value;
use std::collections::HashMap;

use super::types::{DetectedWorkload, IngressRoutePreview, PortMapping, VolumeMapping};
use crate::error::SmooothError;
use crate::models::{CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, PolicyConstraints};

pub struct ComposeParser;

impl ComposeParser {
    pub fn parse(
        yaml_content: &str,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let root: Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| SmooothError::Validation(format!("Invalid Docker Compose YAML: {}", e)))?;

        let services_map = root
            .get("services")
            .and_then(|s| s.as_mapping())
            .ok_or_else(|| SmooothError::Validation("No 'services' section found in compose file".to_string()))?;

        let mut workloads = Vec::new();
        let mut canonical_actions = Vec::new();
        let mut dependencies = Vec::new();
        let mut ingress_routes = Vec::new();

        for (svc_key, svc_val) in services_map {
            let svc_name = svc_key
                .as_str()
                .unwrap_or("service")
                .trim()
                .to_lowercase()
                .replace('_', "-");

            let image = svc_val
                .get("image")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let build_source = if image.is_empty() {
                if let Some(b) = svc_val.get("build") {
                    if let Some(s) = b.as_str() {
                        Some(s.to_string())
                    } else if let Some(ctx) = b.get("context").and_then(|v| v.as_str()) {
                        Some(ctx.to_string())
                    } else {
                        Some(".".to_string())
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Parse ports
            let mut ports = Vec::new();
            if let Some(ports_seq) = svc_val.get("ports").and_then(|p| p.as_sequence()) {
                for p in ports_seq {
                    if let Some(p_str) = p.as_str() {
                        if let Some(mapping) = Self::parse_port_string(p_str) {
                            ports.push(mapping);
                        }
                    } else if let Some(p_int) = p.as_i64() {
                        ports.push(PortMapping {
                            container_port: p_int as u16,
                            host_port: Some(p_int as u16),
                            protocol: "tcp".to_string(),
                        });
                    } else if p.is_mapping() {
                        let target = p
                            .get("target")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(80) as u16;
                        let pub_port = p
                            .get("published")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as u16);
                        let proto = p
                            .get("protocol")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tcp")
                            .to_string();
                        ports.push(PortMapping {
                            container_port: target,
                            host_port: pub_port,
                            protocol: proto,
                        });
                    }
                }
            }

            // Parse environment
            let mut env_vars = HashMap::new();
            if let Some(env_node) = svc_val.get("environment") {
                if let Some(env_map) = env_node.as_mapping() {
                    for (k, v) in env_map {
                        let k_str = match k {
                            Value::String(s) => s.clone(),
                            _ => continue,
                        };
                        let v_str = match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => String::new(),
                        };
                        env_vars.insert(k_str, v_str);
                    }
                } else if let Some(env_seq) = env_node.as_sequence() {
                    for item in env_seq {
                        if let Some(line) = item.as_str() {
                            if let Some((k, v)) = line.split_once('=') {
                                env_vars.insert(k.trim().to_string(), v.trim().to_string());
                            }
                        }
                    }
                }
            }

            // Parse volumes
            let mut volumes = Vec::new();
            if let Some(vols_seq) = svc_val.get("volumes").and_then(|v| v.as_sequence()) {
                for vol in vols_seq {
                    if let Some(v_str) = vol.as_str() {
                        let parts: Vec<&str> = v_str.split(':').collect();
                        if parts.len() >= 2 {
                            let source = parts[0].to_string();
                            let target = parts[1].to_string();
                            let is_named = !source.starts_with('/') && !source.starts_with('.');
                            let read_only = parts.get(2).map(|&m| m.contains("ro")).unwrap_or(false);
                            volumes.push(VolumeMapping {
                                source,
                                target,
                                is_named_volume: is_named,
                                read_only,
                            });
                        }
                    }
                }
            }

            // Parse labels and discover domains (Traefik, Coolify, Caddy)
            let mut domains = Vec::new();
            if let Some(lbl_node) = svc_val.get("labels") {
                if let Some(lbl_map) = lbl_node.as_mapping() {
                    for (k, v) in lbl_map {
                        if let (Some(k_str), Some(v_str)) = (k.as_str(), v.as_str()) {
                            Self::extract_domain_from_label(k_str, v_str, &mut domains);
                        }
                    }
                } else if let Some(lbl_seq) = lbl_node.as_sequence() {
                    for item in lbl_seq {
                        if let Some(line) = item.as_str() {
                            if let Some((k, v)) = line.split_once('=') {
                                Self::extract_domain_from_label(k.trim(), v.trim(), &mut domains);
                            }
                        }
                    }
                }
            }

            // Also check environment variables for domain indications
            for (k, v) in &env_vars {
                let k_lower = k.to_lowercase();
                if k_lower == "virtual_host" || k_lower == "domain" || k_lower == "fqdn" {
                    for d in v.split(',') {
                        let clean = d.trim().trim_start_matches("https://").trim_start_matches("http://").trim_matches('/');
                        if !clean.is_empty() && !domains.contains(&clean.to_string()) {
                            domains.push(clean.to_string());
                        }
                    }
                }
            }

            // Parse depends_on
            let mut service_deps = Vec::new();
            if let Some(dep_node) = svc_val.get("depends_on") {
                if let Some(dep_seq) = dep_node.as_sequence() {
                    for d in dep_seq {
                        if let Some(dep_str) = d.as_str() {
                            let clean_dep = dep_str.trim().to_lowercase().replace('_', "-");
                            service_deps.push(clean_dep.clone());
                            dependencies.push(DependencyEdge {
                                from: svc_name.clone(),
                                to: clean_dep,
                            });
                        }
                    }
                } else if let Some(dep_map) = dep_node.as_mapping() {
                    for (k, _) in dep_map {
                        if let Some(dep_str) = k.as_str() {
                            let clean_dep = dep_str.trim().to_lowercase().replace('_', "-");
                            service_deps.push(clean_dep.clone());
                            dependencies.push(DependencyEdge {
                                from: svc_name.clone(),
                                to: clean_dep,
                            });
                        }
                    }
                }
            }

            // Determine provider and service_type
            let (provider, service_type, is_db) = Self::classify_service(&image, &svc_name);
            let is_persistent = is_db || !volumes.is_empty();

            let mut notes = Vec::new();
            if is_db {
                notes.push(format!("Identified stateful {} database adapter with persistent volume safeguarding.", provider));
            }
            if !domains.is_empty() {
                notes.push(format!("Configured automatic HTTPS ingress for domain(s): {}", domains.join(", ")));
            }

            // Register ingress routes
            let primary_port = ports
                .iter()
                .find(|p| p.host_port.is_some())
                .map(|p| p.container_port)
                .or_else(|| ports.first().map(|p| p.container_port))
                .unwrap_or(if is_db { 5432 } else { 8080 });

            for dom in &domains {
                ingress_routes.push(IngressRoutePreview {
                    domain: dom.clone(),
                    target_resource: svc_name.clone(),
                    target_port: primary_port,
                    tls: true,
                    path_prefix: None,
                });
            }

            let effective_image = if image.is_empty() {
                format!("{}:latest", svc_name)
            } else {
                image.clone()
            };

            let mut ports_map = serde_json::Map::new();
            for p in &ports {
                let target_port = p.host_port.unwrap_or(p.container_port);
                ports_map.insert(format!("{}/{}", p.container_port, p.protocol), serde_json::json!(target_port));
            }

            let action_config = serde_json::json!({
                "image": effective_image,
                "port": primary_port,
                "ports": ports_map,
                "env": env_vars,
                "volumes": volumes.iter().map(|v| format!("{}:{}", v.source, v.target)).collect::<Vec<_>>(),
                "workspace_id": workspace
            });

            canonical_actions.push(CanonicalActionSpec {
                resource_name: svc_name.clone(),
                provider: provider.clone(),
                service_type: service_type.clone(),
                action_type: "create".to_string(),
                config: action_config,
                is_persistent,
                exposes_network_publicly: !ports.is_empty() || !domains.is_empty(),
            });

            workloads.push(DetectedWorkload {
                name: svc_name,
                service_type,
                provider,
                image: effective_image,
                build_source,
                ports,
                env_vars,
                volumes,
                domains,
                is_persistent,
                depends_on: service_deps,
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
                max_monthly_budget_usd: Some(150),
            },
        };

        Ok((workloads, canonical_spec, ingress_routes))
    }

    fn parse_port_string(port_str: &str) -> Option<PortMapping> {
        let clean = port_str.trim();
        let (proto, port_part) = if let Some((p, pr)) = clean.split_once('/') {
            (pr.to_lowercase(), p)
        } else {
            ("tcp".to_string(), clean)
        };

        let parts: Vec<&str> = port_part.split(':').collect();
        match parts.len() {
            1 => parts[0].parse::<u16>().ok().map(|cp| PortMapping {
                container_port: cp,
                host_port: Some(cp),
                protocol: proto,
            }),
            2 => {
                let hp = parts[0].parse::<u16>().ok();
                let cp = parts[1].parse::<u16>().ok()?;
                Some(PortMapping {
                    container_port: cp,
                    host_port: hp,
                    protocol: proto,
                })
            }
            3 => {
                let hp = parts[1].parse::<u16>().ok();
                let cp = parts[2].parse::<u16>().ok()?;
                Some(PortMapping {
                    container_port: cp,
                    host_port: hp,
                    protocol: proto,
                })
            }
            _ => None,
        }
    }

    fn extract_domain_from_label(key: &str, val: &str, domains: &mut Vec<String>) {
        // Traefik: Host(`domain.com`) or Host("domain.com")
        if key.starts_with("traefik.http.routers") && key.ends_with(".rule") {
            if let Some(start) = val.find("Host(") {
                let rest = &val[start + 5..];
                if let Some(end) = rest.find(')') {
                    let d = rest[..end].trim().trim_matches('`').trim_matches('"').trim_matches('\'');
                    if !d.is_empty() && !domains.contains(&d.to_string()) {
                        domains.push(d.to_string());
                    }
                }
            }
        }
        // Coolify FQDN label
        else if key == "coolify.fqdn" || key == "fqdn" {
            for d in val.split(',') {
                let clean = d.trim().trim_start_matches("https://").trim_start_matches("http://").trim_matches('/');
                if !clean.is_empty() && !domains.contains(&clean.to_string()) {
                    domains.push(clean.to_string());
                }
            }
        }
        // Caddy label
        else if key == "caddy" {
            let clean = val.trim().trim_start_matches("https://").trim_start_matches("http://").trim_matches('/');
            if !clean.is_empty() && !domains.contains(&clean.to_string()) {
                domains.push(clean.to_string());
            }
        }
    }

    fn classify_service(image: &str, name: &str) -> (String, String, bool) {
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
