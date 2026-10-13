use serde::{Deserialize, Serialize};
use crate::models::{Capability, SupportLevel};

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
                    "approved_by": { "type": "string" }
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
    ]
}
