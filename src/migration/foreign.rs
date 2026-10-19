use serde_json::Value;
use std::collections::HashMap;

use super::types::{DetectedWorkload, IngressRoutePreview, PortMapping, VolumeMapping};
use crate::error::SmooothError;
use crate::models::{CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, PolicyConstraints};

pub struct ForeignParser;

impl ForeignParser {
    /// Parse Render Blueprint `render.yaml`
    pub fn parse_render_blueprint(
        yaml_content: &str,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let root: serde_yaml::Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| SmooothError::Validation(format!("Invalid Render Blueprint YAML: {}", e)))?;

        let mut workloads = Vec::new();
        let mut canonical_actions = Vec::new();
        let mut dependencies = Vec::new();
        let ingress_routes = Vec::new();

        // 1. Process databases
        if let Some(dbs) = root.get("databases").and_then(|d| d.as_sequence()) {
            for db in dbs {
                let name = db
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("render-db")
                    .to_lowercase()
                    .replace(' ', "-");

                let mut env_vars = HashMap::new();
                if let Some(user) = db.get("user").and_then(|u| u.as_str()) {
                    env_vars.insert("POSTGRES_USER".to_string(), user.to_string());
                }
                if let Some(dbname) = db.get("databaseName").and_then(|d| d.as_str()) {
                    env_vars.insert("POSTGRES_DB".to_string(), dbname.to_string());
                }

                let mut ports_map = serde_json::Map::new();
                ports_map.insert("5432/tcp".to_string(), serde_json::json!(5432));

                let action_config = serde_json::json!({
                    "image": "postgres:16-alpine",
                    "port": 5432,
                    "ports": ports_map,
                    "env": env_vars,
                    "workspace_id": workspace
                });

                canonical_actions.push(CanonicalActionSpec {
                    resource_name: name.clone(),
                    provider: "postgres".to_string(),
                    service_type: "database".to_string(),
                    action_type: "create".to_string(),
                    config: action_config,
                    is_persistent: true,
                    exposes_network_publicly: false,
                });

                workloads.push(DetectedWorkload {
                    name,
                    service_type: "database".to_string(),
                    provider: "postgres".to_string(),
                    image: "postgres:16-alpine".to_string(),
                    build_source: None,
                    ports: vec![PortMapping {
                        container_port: 5432,
                        host_port: Some(5432),
                        protocol: "tcp".to_string(),
                    }],
                    env_vars,
                    volumes: vec![VolumeMapping {
                        source: "postgres_data".to_string(),
                        target: "/var/lib/postgresql/data".to_string(),
                        is_named_volume: true,
                        read_only: false,
                    }],
                    domains: Vec::new(),
                    is_persistent: true,
                    depends_on: Vec::new(),
                    notes: vec!["Render Managed PostgreSQL translated to Smoooth stateful database adapter.".to_string()],
                });
            }
        }

        // 2. Process services
        if let Some(services) = root.get("services").and_then(|s| s.as_sequence()) {
            for svc in services {
                let name = svc
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("render-svc")
                    .to_lowercase()
                    .replace(' ', "-");

                let svc_type = svc
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("web");

                let is_web = svc_type == "web";

                // Extract environment variables
                let mut env_vars = HashMap::new();
                let mut service_deps = Vec::new();

                if let Some(envs) = svc.get("envVars").and_then(|e| e.as_sequence()) {
                    for e in envs {
                        if let Some(k) = e.get("key").and_then(|k| k.as_str()) {
                            if let Some(v) = e.get("value").and_then(|v| v.as_str()) {
                                env_vars.insert(k.to_string(), v.to_string());
                            } else if let Some(from_db) = e.get("fromDatabase") {
                                if let Some(db_name) = from_db.get("name").and_then(|d| d.as_str()) {
                                    let clean_db = db_name.to_lowercase().replace(' ', "-");
                                    service_deps.push(clean_db.clone());
                                    dependencies.push(DependencyEdge {
                                        from: name.clone(),
                                        to: clean_db,
                                    });
                                }
                            }
                        }
                    }
                }

                let port: u16 = if is_web { 8080 } else { 0 };
                let mut ports_map = serde_json::Map::new();
                if is_web {
                    ports_map.insert("8080/tcp".to_string(), serde_json::json!(8080));
                }

                let effective_image = format!("{}:latest", name);

                let action_config = serde_json::json!({
                    "image": effective_image,
                    "port": port,
                    "ports": ports_map,
                    "env": env_vars,
                    "workspace_id": workspace
                });

                canonical_actions.push(CanonicalActionSpec {
                    resource_name: name.clone(),
                    provider: "docker".to_string(),
                    service_type: if is_web { "compute".to_string() } else { "worker".to_string() },
                    action_type: "create".to_string(),
                    config: action_config,
                    is_persistent: false,
                    exposes_network_publicly: is_web,
                });

                workloads.push(DetectedWorkload {
                    name,
                    service_type: if is_web { "compute".to_string() } else { "worker".to_string() },
                    provider: "docker".to_string(),
                    image: effective_image,
                    build_source: Some(".".to_string()),
                    ports: if is_web {
                        vec![PortMapping {
                            container_port: 8080,
                            host_port: Some(8080),
                            protocol: "tcp".to_string(),
                        }]
                    } else {
                        Vec::new()
                    },
                    env_vars,
                    volumes: Vec::new(),
                    domains: Vec::new(),
                    is_persistent: false,
                    depends_on: service_deps,
                    notes: vec![format!("Render {} service converted to zero-Dockerfile Nixpacks build.", svc_type)],
                });
            }
        }

        if workloads.is_empty() {
            return Err(SmooothError::Validation("No services found in render.yaml".to_string()));
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

    /// Parse Fly.io `fly.toml`
    pub fn parse_fly_toml(
        toml_content: &str,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let root: toml::Value = toml::from_str(toml_content)
            .map_err(|e| SmooothError::Validation(format!("Invalid Fly.io fly.toml: {}", e)))?;

        let app_name = root
            .get("app")
            .and_then(|a| a.as_str())
            .unwrap_or("fly-app")
            .to_lowercase()
            .replace('_', "-");

        let image = root
            .get("build")
            .and_then(|b| b.get("image"))
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();

        let effective_image = if image.is_empty() {
            format!("{}:latest", app_name)
        } else {
            image
        };

        let internal_port = root
            .get("http_service")
            .and_then(|h| h.get("internal_port"))
            .and_then(|p| p.as_integer())
            .map(|p| p as u16)
            .or_else(|| {
                root.get("services")
                    .and_then(|s| s.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|svc| svc.get("internal_port"))
                    .and_then(|p| p.as_integer())
                    .map(|p| p as u16)
            })
            .unwrap_or(8080);

        let mut env_vars = HashMap::new();
        if let Some(envs) = root.get("env").and_then(|e| e.as_table()) {
            for (k, v) in envs {
                let v_str = match v {
                    toml::Value::String(s) => s.clone(),
                    toml::Value::Integer(i) => i.to_string(),
                    toml::Value::Boolean(b) => b.to_string(),
                    _ => String::new(),
                };
                env_vars.insert(k.clone(), v_str);
            }
        }

        let fly_domain = format!("{}.fly.dev", app_name);
        let ingress_routes = vec![IngressRoutePreview {
            domain: fly_domain.clone(),
            target_resource: app_name.clone(),
            target_port: internal_port,
            tls: true,
            path_prefix: None,
        }];

        let mut ports_map = serde_json::Map::new();
        ports_map.insert(format!("{}/tcp", internal_port), serde_json::json!(internal_port));

        let action_config = serde_json::json!({
            "image": effective_image,
            "port": internal_port,
            "ports": ports_map,
            "env": env_vars,
            "workspace_id": workspace
        });

        let action = CanonicalActionSpec {
            resource_name: app_name.clone(),
            provider: "docker".to_string(),
            service_type: "compute".to_string(),
            action_type: "create".to_string(),
            config: action_config,
            is_persistent: false,
            exposes_network_publicly: true,
        };

        let workload = DetectedWorkload {
            name: app_name.clone(),
            service_type: "compute".to_string(),
            provider: "docker".to_string(),
            image: effective_image,
            build_source: None,
            ports: vec![PortMapping {
                container_port: internal_port,
                host_port: Some(internal_port),
                protocol: "tcp".to_string(),
            }],
            env_vars,
            volumes: Vec::new(),
            domains: vec![fly_domain],
            is_persistent: false,
            depends_on: Vec::new(),
            notes: vec!["Migrated from Fly.io fly.toml definition with automatic HTTPS ingress route.".to_string()],
        };

        let canonical_spec = CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: workspace.to_string(),
            environment: "production".to_string(),
            target_id: "local_docker_node".to_string(),
            actions: vec![action],
            dependencies: Vec::new(),
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: true,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(150),
            },
        };

        Ok((vec![workload], canonical_spec, ingress_routes))
    }

    /// Parse Dokku Procfile + env vars
    pub fn parse_dokku_app(
        procfile_content: &str,
        env_content: Option<&str>,
        app_name_hint: Option<&str>,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let app_name = app_name_hint
            .unwrap_or("dokku-app")
            .to_lowercase()
            .replace('_', "-");

        let mut env_vars = HashMap::new();
        if let Some(raw_env) = env_content {
            for line in raw_env.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    env_vars.insert(k.trim().to_string(), v.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }

        let mut workloads = Vec::new();
        let mut canonical_actions = Vec::new();

        for line in procfile_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((role, cmd)) = trimmed.split_once(':') {
                let role_clean = role.trim().to_lowercase();
                let sub_name = if role_clean == "web" {
                    app_name.clone()
                } else {
                    format!("{}-{}", app_name, role_clean)
                };

                let is_web = role_clean == "web";
                let port: u16 = if is_web { 8080 } else { 0 };

                let mut ports_map = serde_json::Map::new();
                if is_web {
                    ports_map.insert("8080/tcp".to_string(), serde_json::json!(8080));
                }

                let effective_image = format!("{}:latest", sub_name);

                let action_config = serde_json::json!({
                    "image": effective_image,
                    "command": cmd.trim(),
                    "port": port,
                    "ports": ports_map,
                    "env": env_vars,
                    "workspace_id": workspace
                });

                canonical_actions.push(CanonicalActionSpec {
                    resource_name: sub_name.clone(),
                    provider: "docker".to_string(),
                    service_type: if is_web { "compute".to_string() } else { "worker".to_string() },
                    action_type: "create".to_string(),
                    config: action_config,
                    is_persistent: false,
                    exposes_network_publicly: is_web,
                });

                workloads.push(DetectedWorkload {
                    name: sub_name,
                    service_type: if is_web { "compute".to_string() } else { "worker".to_string() },
                    provider: "docker".to_string(),
                    image: effective_image,
                    build_source: Some(".".to_string()),
                    ports: if is_web {
                        vec![PortMapping {
                            container_port: 8080,
                            host_port: Some(8080),
                            protocol: "tcp".to_string(),
                        }]
                    } else {
                        Vec::new()
                    },
                    env_vars: env_vars.clone(),
                    volumes: Vec::new(),
                    domains: Vec::new(),
                    is_persistent: false,
                    depends_on: Vec::new(),
                    notes: vec![format!("Dokku Procfile '{}' process mapped to zero-Dockerfile container.", role_clean)],
                });
            }
        }

        if workloads.is_empty() {
            return Err(SmooothError::Validation("No processes defined in Procfile".to_string()));
        }

        let canonical_spec = CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: workspace.to_string(),
            environment: "production".to_string(),
            target_id: "local_docker_node".to_string(),
            actions: canonical_actions,
            dependencies: Vec::new(),
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: true,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(150),
            },
        };

        Ok((workloads, canonical_spec, Vec::new()))
    }

    /// Parse CapRover captain-definition
    pub fn parse_caprover(
        json_content: &str,
        app_name_hint: Option<&str>,
        workspace: &str,
    ) -> Result<(Vec<DetectedWorkload>, CanonicalExecutionSpec, Vec<IngressRoutePreview>), SmooothError> {
        let root: Value = serde_json::from_str(json_content)
            .map_err(|e| SmooothError::Validation(format!("Invalid CapRover captain-definition JSON: {}", e)))?;

        let app_name = app_name_hint
            .unwrap_or("caprover-app")
            .to_lowercase()
            .replace('_', "-");

        let image = root
            .get("imageName")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();

        let effective_image = if image.is_empty() {
            format!("{}:latest", app_name)
        } else {
            image
        };

        let mut ports_map = serde_json::Map::new();
        ports_map.insert("80/tcp".to_string(), serde_json::json!(8080));

        let action_config = serde_json::json!({
            "image": effective_image,
            "port": 8080,
            "ports": ports_map,
            "workspace_id": workspace
        });

        let action = CanonicalActionSpec {
            resource_name: app_name.clone(),
            provider: "docker".to_string(),
            service_type: "compute".to_string(),
            action_type: "create".to_string(),
            config: action_config,
            is_persistent: false,
            exposes_network_publicly: true,
        };

        let workload = DetectedWorkload {
            name: app_name,
            service_type: "compute".to_string(),
            provider: "docker".to_string(),
            image: effective_image,
            build_source: None,
            ports: vec![PortMapping {
                container_port: 80,
                host_port: Some(8080),
                protocol: "tcp".to_string(),
            }],
            env_vars: HashMap::new(),
            volumes: Vec::new(),
            domains: Vec::new(),
            is_persistent: false,
            depends_on: Vec::new(),
            notes: vec!["CapRover captain-definition container imported into Smoooth.".to_string()],
        };

        let canonical_spec = CanonicalExecutionSpec {
            schema_version: 1,
            workspace_id: workspace.to_string(),
            environment: "production".to_string(),
            target_id: "local_docker_node".to_string(),
            actions: vec![action],
            dependencies: Vec::new(),
            immutable_artifacts: HashMap::new(),
            policy_envelope: PolicyConstraints {
                allow_public_network: true,
                allow_data_destruction: false,
                max_monthly_budget_usd: Some(150),
            },
        };

        Ok((vec![workload], canonical_spec, Vec::new()))
    }
}
