use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use super::compose::ComposeParser;
use super::types::{DetectedWorkload, IngressRoutePreview, PortMapping, VolumeMapping};
use crate::error::SmooothError;
use crate::models::{CanonicalActionSpec, CanonicalExecutionSpec, PolicyConstraints};

pub struct CoolifyParser;

impl CoolifyParser {
    /// Parse Coolify JSON export or API response
    pub fn parse_json(
        json_content: &str,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let root: Value = serde_json::from_str(json_content)
            .map_err(|e| SmooothError::Validation(format!("Invalid Coolify JSON format: {}", e)))?;

        let mut workloads = Vec::new();
        let mut canonical_actions = Vec::new();
        let mut dependencies = Vec::new();
        let mut ingress_routes = Vec::new();

        // 1. Check if the root has "services", "applications", or "databases"
        if let Some(apps) = root.get("applications").and_then(|a| a.as_array()) {
            for app in apps {
                Self::process_coolify_app(app, workspace, &mut workloads, &mut canonical_actions, &mut ingress_routes)?;
            }
        }

        if let Some(dbs) = root.get("databases").and_then(|d| d.as_array()) {
            for db in dbs {
                Self::process_coolify_db(db, workspace, &mut workloads, &mut canonical_actions)?;
            }
        }

        if let Some(services) = root.get("services").and_then(|s| s.as_array()) {
            for svc in services {
                // If service contains raw docker-compose, parse with ComposeParser
                if let Some(compose_raw) = svc.get("docker_compose_raw").and_then(|c| c.as_str()) {
                    if let Ok((mut w, mut spec, mut ing)) = ComposeParser::parse(compose_raw, workspace) {
                        workloads.append(&mut w);
                        canonical_actions.append(&mut spec.actions);
                        dependencies.append(&mut spec.dependencies);
                        ingress_routes.append(&mut ing);
                        continue;
                    }
                }
                Self::process_coolify_app(svc, workspace, &mut workloads, &mut canonical_actions, &mut ingress_routes)?;
            }
        }

        // If root is a single application or service object
        if workloads.is_empty() {
            if let Some(compose_raw) = root.get("docker_compose_raw").and_then(|c| c.as_str()) {
                return ComposeParser::parse(compose_raw, workspace);
            }
            if root.get("name").is_some() || root.get("fqdn").is_some() {
                Self::process_coolify_app(&root, workspace, &mut workloads, &mut canonical_actions, &mut ingress_routes)?;
            }
        }

        if workloads.is_empty() {
            return Err(SmooothError::Validation(
                "No deployable applications or databases discovered in Coolify JSON manifest.".to_string(),
            ));
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

    /// Scan local VPS directory `/data/coolify` for services and compose files
    pub fn scan_local_data_dir(
        base_dir: &str,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let base_path = Path::new(base_dir);
        if !base_path.exists() {
            return Err(SmooothError::Validation(format!(
                "Coolify local directory '{}' does not exist on this host.",
                base_dir
            )));
        }

        let mut combined_workloads = Vec::new();
        let mut combined_actions = Vec::new();
        let mut combined_deps = Vec::new();
        let mut combined_ingress = Vec::new();

        // Search for docker-compose.yml files in /data/coolify/services/* and /data/coolify/applications/*
        let search_dirs = [
            base_path.join("services"),
            base_path.join("applications"),
            base_path.join("source"),
            base_path.to_path_buf(),
        ];

        for dir in &search_dirs {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let compose_candidate = if path.is_file() && (path.ends_with("docker-compose.yml") || path.ends_with("compose.yaml")) {
                        Some(path.clone())
                    } else if path.is_dir() {
                        let inner1 = path.join("docker-compose.yml");
                        let inner2 = path.join("compose.yaml");
                        if inner1.exists() {
                            Some(inner1)
                        } else if inner2.exists() {
                            Some(inner2)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(comp_path) = compose_candidate {
                        if let Ok(content) = std::fs::read_to_string(&comp_path) {
                            if let Ok((mut w, mut spec, mut ing)) = ComposeParser::parse(&content, workspace) {
                                combined_workloads.append(&mut w);
                                combined_actions.append(&mut spec.actions);
                                combined_deps.append(&mut spec.dependencies);
                                combined_ingress.append(&mut ing);
                            }
                        }
                    }
                }
            }
        }

        if combined_workloads.is_empty() {
            return Err(SmooothError::Validation(format!(
                "No Docker Compose or Coolify workloads found in '{}'",
                base_dir
            )));
        }

        let canonical_spec = CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: workspace.to_string(),
            environment: "production".to_string(),
            target_id: "local_docker_node".to_string(),
            actions: combined_actions,
            dependencies: combined_deps,
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: true,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(150),
            },
        };

        Ok((combined_workloads, canonical_spec, combined_ingress))
    }

    fn process_coolify_app(
        app: &Value,
        workspace: &str,
        workloads: &mut Vec<DetectedWorkload>,
        actions: &mut Vec<CanonicalActionSpec>,
        ingress_routes: &mut Vec<IngressRoutePreview>,
    ) -> Result<(), SmooothError> {
        let name = app
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("coolify-app")
            .trim()
            .to_lowercase()
            .replace([' ', '_'], "-");

        let image = app
            .get("docker_image")
            .or_else(|| app.get("image"))
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();

        let git_repo = app
            .get("git_repository")
            .and_then(|g| g.as_str())
            .map(|s| s.to_string());

        let effective_image = if !image.is_empty() {
            image
        } else if let Some(ref _repo) = git_repo {
            format!("{}:latest", name)
        } else {
            format!("{}:latest", name)
        };

        // Extract ports
        let ports_raw = app
            .get("ports_exposes")
            .or_else(|| app.get("ports"))
            .and_then(|p| p.as_str())
            .unwrap_or("8080");

        let mut ports = Vec::new();
        for p in ports_raw.split(',') {
            if let Ok(port_num) = p.trim().parse::<u16>() {
                ports.push(PortMapping {
                    container_port: port_num,
                    host_port: Some(port_num),
                    protocol: "tcp".to_string(),
                });
            }
        }
        if ports.is_empty() {
            ports.push(PortMapping {
                container_port: 8080,
                host_port: Some(8080),
                protocol: "tcp".to_string(),
            });
        }

        // Extract environment variables
        let mut env_vars = HashMap::new();
        if let Some(envs) = app.get("environment_variables").and_then(|e| e.as_array()) {
            for e in envs {
                if let (Some(k), Some(v)) = (e.get("key").and_then(|k| k.as_str()), e.get("value").and_then(|v| v.as_str())) {
                    env_vars.insert(k.to_string(), v.to_string());
                }
            }
        } else if let Some(envs) = app.get("environment_variables").and_then(|e| e.as_object()) {
            for (k, v) in envs {
                if let Some(v_str) = v.as_str() {
                    env_vars.insert(k.clone(), v_str.to_string());
                }
            }
        }

        // Extract domains (FQDN)
        let mut domains = Vec::new();
        if let Some(fqdn) = app.get("fqdn").and_then(|f| f.as_str()) {
            for d in fqdn.split(',') {
                let clean = d.trim().trim_start_matches("https://").trim_start_matches("http://").trim_matches('/');
                if !clean.is_empty() {
                    domains.push(clean.to_string());
                }
            }
        }

        let primary_port = ports[0].container_port;
        for dom in &domains {
            ingress_routes.push(IngressRoutePreview {
                domain: dom.clone(),
                target_resource: name.clone(),
                target_port: primary_port,
                tls: true,
                path_prefix: None,
            });
        }

        let mut ports_map = serde_json::Map::new();
        for p in &ports {
            let hp = p.host_port.unwrap_or(p.container_port);
            ports_map.insert(format!("{}/tcp", p.container_port), serde_json::json!(hp));
        }

        let mut notes = vec!["Migrated from Coolify application definition.".to_string()];
        if let Some(repo) = git_repo.as_ref() {
            notes.push(format!("Build source repository: {}", repo));
        }
        if !domains.is_empty() {
            notes.push(format!("Configured automatic HTTPS ingress for domain: {}", domains.join(", ")));
        }

        let action_config = serde_json::json!({
            "image": effective_image,
            "port": primary_port,
            "ports": ports_map,
            "env": env_vars,
            "workspace_id": workspace
        });

        actions.push(CanonicalActionSpec {
            resource_name: name.clone(),
            provider: "docker".to_string(),
            service_type: "compute".to_string(),
            action_type: "create".to_string(),
            config: action_config,
            is_persistent: false,
            exposes_network_publicly: !domains.is_empty() || !ports.is_empty(),
        });

        workloads.push(DetectedWorkload {
            name,
            service_type: "compute".to_string(),
            provider: "docker".to_string(),
            image: effective_image,
            build_source: git_repo,
            ports,
            env_vars,
            volumes: Vec::new(),
            domains,
            is_persistent: false,
            depends_on: Vec::new(),
            notes,
        });

        Ok(())
    }

    fn process_coolify_db(
        db: &Value,
        workspace: &str,
        workloads: &mut Vec<DetectedWorkload>,
        actions: &mut Vec<CanonicalActionSpec>,
    ) -> Result<(), SmooothError> {
        let name = db
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("coolify-db")
            .trim()
            .to_lowercase()
            .replace([' ', '_'], "-");

        let db_type = db
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("postgresql")
            .to_lowercase();

        let (provider, service_type, image, port) = if db_type.contains("postgres") {
            ("postgres", "database", "postgres:16-alpine", 5432)
        } else if db_type.contains("redis") {
            ("redis", "cache", "redis:7-alpine", 6379)
        } else if db_type.contains("mongo") {
            ("mongo", "database", "mongo:6.0", 27017)
        } else {
            ("docker", "database", "postgres:16-alpine", 5432)
        };

        let mut env_vars = HashMap::new();
        if let Some(user) = db.get("postgres_user").and_then(|u| u.as_str()) {
            env_vars.insert("POSTGRES_USER".to_string(), user.to_string());
        }
        if let Some(pass) = db.get("postgres_password").and_then(|p| p.as_str()) {
            env_vars.insert("POSTGRES_PASSWORD".to_string(), pass.to_string());
        }
        if let Some(dbname) = db.get("postgres_db").and_then(|d| d.as_str()) {
            env_vars.insert("POSTGRES_DB".to_string(), dbname.to_string());
        }

        let mut ports_map = serde_json::Map::new();
        ports_map.insert(format!("{}/tcp", port), serde_json::json!(port));

        let action_config = serde_json::json!({
            "image": image,
            "port": port,
            "ports": ports_map,
            "env": env_vars,
            "workspace_id": workspace
        });

        actions.push(CanonicalActionSpec {
            resource_name: name.clone(),
            provider: provider.to_string(),
            service_type: service_type.to_string(),
            action_type: "create".to_string(),
            config: action_config,
            is_persistent: true,
            exposes_network_publicly: false,
        });

        workloads.push(DetectedWorkload {
            name,
            service_type: service_type.to_string(),
            provider: provider.to_string(),
            image: image.to_string(),
            build_source: None,
            ports: vec![PortMapping {
                container_port: port,
                host_port: Some(port),
                protocol: "tcp".to_string(),
            }],
            env_vars,
            volumes: vec![VolumeMapping {
                source: format!("{}_data", provider),
                target: format!("/var/lib/{}/data", provider),
                is_named_volume: true,
                read_only: false,
            }],
            domains: Vec::new(),
            is_persistent: true,
            depends_on: Vec::new(),
            notes: vec![format!("Persistent {} database migrated from Coolify with durable volume protection.", provider)],
        });

        Ok(())
    }
}
