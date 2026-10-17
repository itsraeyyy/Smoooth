use crate::models::{Capability, CapabilitySchema, SupportLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub step: String, // "01 Discover", "02 Plan", "03 Govern", "04 Provision", "05 Connect", "06 Verify", "07 Operate"
    pub input_schema: serde_json::Value,
}

pub fn get_available_tools() -> Vec<McpToolDefinition> {
    vec![
        // 01 Discover
        McpToolDefinition {
            name: "list_capabilities".to_string(),
            description: "Discover available capabilities, schemas, provider guarantees, and support levels".to_string(),
            step: "01 Discover".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "provider": { "type": "string" },
                    "service_type": { "type": "string" }
                }
            }),
        },
        McpToolDefinition {
            name: "get_capability_schema".to_string(),
            description: "Inspect parameter schemas, constraints, and support levels for a specific capability".to_string(),
            step: "01 Discover".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["capability_id"],
                "properties": {
                    "capability_id": { "type": "string" }
                }
            }),
        },

        // 02 Plan
        McpToolDefinition {
            name: "create_plan".to_string(),
            description: "Propose a stack and receive a validated, cost-aware execution plan with canonical hash".to_string(),
            step: "02 Plan".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["workspace_id", "spec"],
                "properties": {
                    "workspace_id": { "type": "string" },
                    "spec": { "type": "object" }
                }
            }),
        },

        McpToolDefinition {
            name: "get_plan".to_string(),
            description: "Inspect details, canonical hash, risk tier, cost estimate, and approval state of a plan".to_string(),
            step: "02 Plan".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["plan_id"],
                "properties": {
                    "plan_id": { "type": "string" }
                }
            }),
        },

        // 03 Govern
        McpToolDefinition {
            name: "approve_plan".to_string(),
            description: "Approve actions exceeding autonomous permissions, strictly bound to the exact plan hash".to_string(),
            step: "03 Govern".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["plan_id", "plan_hash"],
                "properties": {
                    "plan_id": { "type": "string" },
                    "plan_hash": { "type": "string" },
                    "approved_by": { "type": "string" },
                    "ttl_seconds": { "type": "integer" }
                }
            }),
        },

        // 04 Provision
        McpToolDefinition {
            name: "execute_plan".to_string(),
            description: "Provision Docker runtime and PostgreSQL through provider adapters with fencing leases".to_string(),
            step: "04 Provision".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["plan_id", "idempotency_key"],
                "properties": {
                    "plan_id": { "type": "string" },
                    "idempotency_key": { "type": "string" }
                }
            }),
        },

        // 05 Connect
        McpToolDefinition {
            name: "create_service_binding".to_string(),
            description: "Create scoped database identity, stage dual-credentials, and bind application securely without secret leakage".to_string(),
            step: "05 Connect".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["source_resource_id", "target_resource_id"],
                "properties": {
                    "source_resource_id": { "type": "string" },
                    "target_resource_id": { "type": "string" }
                }
            }),
        },

        // 06 Verify
        McpToolDefinition {
            name: "verify_connection".to_string(),
            description: "Execute active probes confirming application and database work together with verifiable evidence".to_string(),
            step: "06 Verify".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["resource_id"],
                "properties": {
                    "resource_id": { "type": "string" }
                }
            }),
        },

        // 07 Operate
        McpToolDefinition {
            name: "get_operation".to_string(),
            description: "Inspect operation progress, redacted logs, and diagnostic failure traces".to_string(),
            step: "07 Operate".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["operation_id"],
                "properties": {
                    "operation_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinition {
            name: "reconcile_resource".to_string(),
            description: "Detect drift, diagnose seeded failures, repair configuration, and verify recovery".to_string(),
            step: "07 Operate".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "required": ["resource_id"],
                "properties": {
                    "resource_id": { "type": "string" },
                    "approve_destructive": { "type": "boolean" }
                }
            }),
        },
        McpToolDefinition {
            name: "list_resources".to_string(),
            description: "List all deployed infrastructure resources and status in a workspace".to_string(),
            step: "07 Operate".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workspace_id": { "type": "string" }
                }
            }),
        },
    ]
}

pub fn get_default_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "cap_docker_container".to_string(),
            name: "Docker Container Runtime".to_string(),
            provider: "docker".to_string(),
            service_type: "compute".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "Ephemeral and persistent container workloads with network isolation.".to_string(),
            operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
        },
        Capability {
            id: "cap_postgres_db".to_string(),
            name: "PostgreSQL Database".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "Relational database provisioning with persistent volume protection and staged credential rotation.".to_string(),
            operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
        },
        Capability {
            id: "cap_redis_cache".to_string(),
            name: "Redis Key-Value Cache & Store".to_string(),
            provider: "redis".to_string(),
            service_type: "cache".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "High-performance Redis in-memory cache and data store with AOF persistence and authentication.".to_string(),
            operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
        },
        Capability {
            id: "cap_mongo_db".to_string(),
            name: "MongoDB Document Database".to_string(),
            provider: "mongodb".to_string(),
            service_type: "database".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "High-performance MongoDB document database with persistent storage, authentication, and endpoint generation.".to_string(),
            operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
        },
        Capability {
            id: "cap_sqlite_embedded".to_string(),
            name: "SQLite Embedded Database Volume".to_string(),
            provider: "docker".to_string(),
            service_type: "storage".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "Zero-dependency embedded SQLite database via persistent volume mount with WAL mode concurrency guidance.".to_string(),
            operations: vec!["provision".to_string(), "inspect".to_string(), "reconcile".to_string(), "destroy".to_string()],
        },
    ]
}

pub fn get_capability_schema_detail(capability_id: &str) -> Option<CapabilitySchema> {
    match capability_id {
        "cap_docker_container" => Some(CapabilitySchema {
            capability: Capability {
                id: "cap_docker_container".to_string(),
                name: "Docker Container Runtime".to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "Ephemeral and persistent container workloads with network isolation.".to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
            },
            parameter_schema: serde_json::json!({
                "type": "object",
                "required": ["image"],
                "properties": {
                    "image": { "type": "string", "description": "Docker image repository tag (e.g. node:20-alpine, python:3.11-alpine)" },
                    "cmd": { "type": "array", "items": { "type": "string" }, "description": "Command and arguments passed to container" },
                    "ports": { "type": "object", "description": "Mapping of container port to host port (use 0 for ephemeral host allocation)" },
                    "env": { "type": "object", "description": "Static environment variables key-value map" },
                    "network": { "type": "string", "description": "Docker user-defined bridge network for inter-container DNS service discovery" },
                    "volumes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["volume_name", "container_path"],
                            "properties": {
                                "volume_name": { "type": "string" },
                                "container_path": { "type": "string" },
                                "read_only": { "type": "boolean" }
                            }
                        }
                    },
                    "bindings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["resource"],
                            "properties": {
                                "resource": { "type": "string", "description": "Name of dependency resource (e.g. postgres or redis resource name)" },
                                "env_prefix": { "type": "string", "description": "Environment variable prefix (e.g. 'DATABASE', 'REDIS', 'CACHE')" },
                                "preferred_endpoint": { "type": "string", "enum": ["internal", "host"], "description": "'internal' uses container DNS name; 'host' uses loopback mapped port" }
                            }
                        }
                    },
                    "health_check": {
                        "type": "object",
                        "required": ["type"],
                        "properties": {
                            "type": { "type": "string", "enum": ["http", "tcp", "liveness"] },
                            "path": { "type": "string", "description": "HTTP health request path (e.g. '/healthz')" },
                            "port": { "type": "integer", "description": "Port to probe" },
                            "expected_status": { "type": "integer", "description": "Expected HTTP status code (e.g. 200)" }
                        }
                    },
                    "restart_policy": { "type": "string", "enum": ["no", "on-failure", "always", "unless-stopped"] }
                }
            }),
            supported_health_checks: vec!["http".to_string(), "tcp".to_string(), "liveness".to_string()],
            supported_protocols: vec!["http".to_string(), "https".to_string(), "tcp".to_string()],
            binding_spec: serde_json::json!({
                "supports_injected_bindings": true,
                "preferred_endpoints": ["internal", "host"],
                "injected_variables": [
                    "<PREFIX>_HOST",
                    "<PREFIX>_PORT",
                    "<PREFIX>_PASSWORD",
                    "<PREFIX>_URL",
                    "<PREFIX>_DATABASE"
                ],
                "description": "Orchestrator automatically resolves dependency endpoints and injects them directly into container environment before startup."
            }),
            approval_requirements: vec![
                "Public network exposure requires allow_public_network=true in plan policy envelope".to_string(),
                "Data destruction requires allow_data_destruction=true in plan policy envelope".to_string(),
                "Plan hash must strictly match approved hash before execute_plan can proceed".to_string(),
            ],
            limitations: vec![
                "Single-host Docker engine only; multi-node clustering (Swarm/K8s) is not supported".to_string(),
                "Zero-downtime rolling replacement requires external load balancer; single container replacement performs fast restart handover".to_string(),
                "Containers on default bridge network cannot resolve sibling containers by name; use 'network' for DNS resolution".to_string(),
            ],
        }),

        "cap_postgres_db" => Some(CapabilitySchema {
            capability: Capability {
                id: "cap_postgres_db".to_string(),
                name: "PostgreSQL Database".to_string(),
                provider: "postgres".to_string(),
                service_type: "database".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "Relational database provisioning with persistent volume protection and staged credential rotation.".to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
            },
            parameter_schema: serde_json::json!({
                "type": "object",
                "required": ["database", "user"],
                "properties": {
                    "database": { "type": "string", "description": "PostgreSQL database name" },
                    "user": { "type": "string", "description": "Database owner username" },
                    "password": { "type": "string", "description": "Database user password (auto-generated with secure entropy if omitted)" },
                    "port": { "type": "integer", "description": "Host port binding (omit or 0 for internal container isolation)" },
                    "network": { "type": "string", "description": "Docker bridge network name for application interconnect" },
                    "volume_name": { "type": "string", "description": "Named volume for database storage (preserves data across container destroy)" }
                }
            }),
            supported_health_checks: vec!["tcp".to_string(), "active_sql_probe".to_string()],
            supported_protocols: vec!["postgres".to_string(), "postgresql".to_string()],
            binding_spec: serde_json::json!({
                "service_type": "database",
                "exports": [
                    "DATABASE_URL",
                    "<PREFIX>_HOST",
                    "<PREFIX>_PORT",
                    "<PREFIX>_USER",
                    "<PREFIX>_PASSWORD",
                    "<PREFIX>_DATABASE"
                ],
                "description": "Provides standard PostgreSQL connection credentials, auto-generates DATABASE_URL URI format."
            }),
            approval_requirements: vec![
                "Persistent volume deletion blocked unless allow_data_destruction=true".to_string(),
                "Database destruction operations classified as High Risk Tier".to_string(),
            ],
            limitations: vec![
                "PostgreSQL 16 Alpine containerized engine".to_string(),
                "Self-hosted adapter does not support read-replica pools or automated failover".to_string(),
                "Host port publishing disabled by default for zero-trust boundary security".to_string(),
            ],
        }),

        "cap_redis_cache" => Some(CapabilitySchema {
            capability: Capability {
                id: "cap_redis_cache".to_string(),
                name: "Redis Key-Value Cache & Store".to_string(),
                provider: "redis".to_string(),
                service_type: "cache".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "High-performance Redis in-memory cache and data store with AOF persistence and authentication.".to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
            },
            parameter_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "password": { "type": "string", "description": "Authentication secret passed via --requirepass" },
                    "port": { "type": "integer", "description": "Port mapping (omit or 0 for dynamic ephemeral port)" },
                    "network": { "type": "string", "description": "Docker bridge network name for application interconnect" },
                    "volume_name": { "type": "string", "description": "Named volume for Append-Only File (AOF) persistence at /data" }
                }
            }),
            supported_health_checks: vec!["resp_ping".to_string(), "tcp".to_string()],
            supported_protocols: vec!["redis".to_string(), "rediss".to_string()],
            binding_spec: serde_json::json!({
                "service_type": "cache",
                "exports": [
                    "<PREFIX>_URL",
                    "<PREFIX>_HOST",
                    "<PREFIX>_PORT",
                    "<PREFIX>_PASSWORD"
                ],
                "description": "Exports Redis connection parameters and URI (redis://:password@host:port/0)."
            }),
            approval_requirements: vec![
                "Persistent cache volume deletion blocked unless allow_data_destruction=true".to_string(),
                "Public network exposure requires explicit policy approval".to_string(),
            ],
            limitations: vec![
                "Redis 7 Alpine single instance with AOF persistence".to_string(),
                "Redis Sentinel / Redis Cluster topology not supported in local provider".to_string(),
                "Cache size constrained by host memory limits".to_string(),
            ],
        }),

        "cap_mongo_db" => Some(CapabilitySchema {
            capability: Capability {
                id: "cap_mongo_db".to_string(),
                name: "MongoDB Document Database".to_string(),
                provider: "mongodb".to_string(),
                service_type: "database".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "High-performance MongoDB document database with persistent storage, authentication, and endpoint generation.".to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string(), "health_check".to_string(), "reconcile".to_string(), "destroy".to_string()],
            },
            parameter_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "image": { "type": "string", "default": "mongo:7-jammy", "description": "MongoDB Docker image" },
                    "user": { "type": "string", "default": "smoooth_user", "description": "Root username" },
                    "password": { "type": "string", "description": "Root password (auto-generated if omitted)" },
                    "database": { "type": "string", "default": "appdb", "description": "Default database name" },
                    "port": { "type": "integer", "description": "Port mapping (default 27017)" }
                }
            }),
            supported_health_checks: vec!["tcp".to_string()],
            supported_protocols: vec!["mongodb".to_string()],
            binding_spec: serde_json::json!({
                "service_type": "database",
                "exports": [
                    "<PREFIX>_URL",
                    "<PREFIX>_HOST",
                    "<PREFIX>_PORT",
                    "<PREFIX>_USER",
                    "<PREFIX>_PASSWORD",
                    "<PREFIX>_DATABASE"
                ],
                "description": "Exports MongoDB connection URI (mongodb://user:pass@host:27017/db?authSource=admin)."
            }),
            approval_requirements: vec![
                "Persistent document database volume deletion blocked unless allow_data_destruction=true".to_string(),
                "Database destruction operations classified as High Risk Tier".to_string(),
            ],
            limitations: vec![
                "MongoDB 7 Jammy single replica set or standalone node".to_string(),
                "Sharded cluster topology not supported in local provider".to_string(),
            ],
        }),

        "cap_sqlite_embedded" => Some(CapabilitySchema {
            capability: Capability {
                id: "cap_sqlite_embedded".to_string(),
                name: "SQLite Embedded Database Volume".to_string(),
                provider: "docker".to_string(),
                service_type: "storage".to_string(),
                support_level: SupportLevel::SelfHosted,
                description: "Zero-dependency embedded SQLite database via persistent volume mount with WAL mode concurrency guidance.".to_string(),
                operations: vec!["provision".to_string(), "inspect".to_string(), "reconcile".to_string(), "destroy".to_string()],
            },
            parameter_schema: serde_json::json!({
                "type": "object",
                "required": ["volume_name", "container_path"],
                "properties": {
                    "volume_name": { "type": "string", "description": "Name of host Docker volume preserving SQLite DB files across container restarts" },
                    "container_path": { "type": "string", "description": "Target container directory where SQLite file resides (e.g. /data or /app/data)" },
                    "sqlite_filename": { "type": "string", "default": "app.db", "description": "Database filename inside mount path" }
                }
            }),
            supported_health_checks: vec!["file_exists".to_string(), "liveness".to_string()],
            supported_protocols: vec!["sqlite".to_string(), "file".to_string()],
            binding_spec: serde_json::json!({
                "service_type": "storage",
                "exports": [
                    "<PREFIX>_PATH",
                    "<PREFIX>_URL"
                ],
                "description": "Exports path to SQLite database file within container environment."
            }),
            approval_requirements: vec![
                "Volume deletion requires allow_data_destruction=true".to_string(),
            ],
            limitations: vec![
                "Single-writer locking: multiple writer processes require WAL (Write-Ahead Logging) mode and busy_timeout configuration".to_string(),
                "Network filesystem (NFS/CIFS) mounting discouraged due to POSIX advisory lock reliability issues".to_string(),
                "Application containers must share volume mount if multiple processes read from the database".to_string(),
            ],
        }),

        _ => None,
    }
}
