use std::io::{self, BufRead, Write};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::Orchestrator;
use crate::error::SmooothError;
use crate::mcp::tools::{get_available_tools, get_default_capabilities};
use crate::models::{
    CanonicalActionSpec, CanonicalExecutionSpec, Plan, PolicyConstraints, RiskTier,
    ServiceBinding,
};
use crate::security::SecretStore;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub struct McpServer {
    orchestrator: Arc<Orchestrator>,
    secret_store: SecretStore,
}

impl McpServer {
    pub fn new(orchestrator: Arc<Orchestrator>) -> Self {
        Self {
            orchestrator,
            secret_store: SecretStore::default_local(),
        }
    }

    /// Run the MCP server over standard input and output
    pub async fn run_stdio(&self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
                if let Some(resp) = self.handle_request(req).await {
                    let resp_str = serde_json::to_string(&resp)?;
                    writeln!(stdout, "{}", resp_str)?;
                    stdout.flush()?;
                }
            } else {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error: invalid JSON".to_string(),
                        data: None,
                    }),
                };
                let resp_str = serde_json::to_string(&err_resp)?;
                writeln!(stdout, "{}", resp_str)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    /// Process a single JSON-RPC request and return a response if required
    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id;

        match req.method.as_str() {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "smoooth-mcp",
                        "version": "0.1.0"
                    }
                });
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                })
            }

            "notifications/initialized" => {
                // Notifications do not receive a response
                None
            }

            "ping" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({})),
                error: None,
            }),

            "tools/list" => {
                let raw_tools = get_available_tools();
                let tools_json: Vec<Value> = raw_tools
                    .into_iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema,
                        })
                    })
                    .collect();

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({ "tools": tools_json })),
                    error: None,
                })
            }

            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                match self.dispatch_tool(tool_name, args).await {
                    Ok(tool_result) => {
                        let content = vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&tool_result).unwrap_or_default()
                        })];
                        Some(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: Some(serde_json::json!({
                                "content": content,
                                "isError": false
                            })),
                            error: None,
                        })
                    }
                    Err(e) => {
                        let content = vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Tool execution error: {}", e)
                        })];
                        Some(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id,
                            result: Some(serde_json::json!({
                                "content": content,
                                "isError": true
                            })),
                            error: None,
                        })
                    }
                }
            }

            other => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", other),
                    data: None,
                }),
            }),
        }
    }

    /// Dispatch tool calls to the underlying engine and storage
    pub async fn dispatch_tool(&self, name: &str, args: Value) -> Result<Value, SmooothError> {
        match name {
            "list_capabilities" => {
                let caps = get_default_capabilities();
                Ok(serde_json::to_value(caps).unwrap_or_default())
            }

            "get_capability_schema" => {
                let cap_id = args.get("capability_id").and_then(|v| v.as_str()).unwrap_or("");
                let caps = get_default_capabilities();
                let found = caps.into_iter().find(|c| c.id == cap_id);
                match found {
                    Some(c) => Ok(serde_json::to_value(c).unwrap_or_default()),
                    None => Err(SmooothError::NotFound(format!("Capability '{}' not found", cap_id))),
                }
            }

            "create_plan" => {
                let workspace_id = args
                    .get("workspace_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();

                let env = args
                    .get("environment")
                    .and_then(|v| v.as_str())
                    .unwrap_or("development")
                    .to_string();

                let actions: Vec<CanonicalActionSpec> = args
                    .get("actions")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let spec = CanonicalExecutionSpec {
                    schema_version: 1,
                    workspace_id,
                    environment: env,
                    target_id: "local_docker_node".to_string(),
                    actions,
                    dependencies: vec![],
                    immutable_artifacts: std::collections::HashMap::new(),
                    policy_envelope: PolicyConstraints {
                        allow_public_network: false,
                        allow_data_destruction: false,
                        max_monthly_budget_usd: Some(150),
                    },
                };

                let plan_id = format!("plan_{}", uuid::Uuid::new_v4().simple());
                let plan = Plan::new(plan_id, spec, RiskTier::Moderate, Some(15.0));

                self.orchestrator.save_plan(plan.clone()).await?;

                Ok(serde_json::json!({
                    "plan_id": plan.id,
                    "plan_hash": plan.plan_hash,
                    "risk_tier": plan.risk_tier,
                    "estimated_cost_usd": plan.estimated_monthly_cost_usd,
                    "requires_approval": plan.requires_approval(),
                    "actions_count": plan.spec.actions.len()
                }))
            }

            "approve_plan" => {
                let plan_id = args
                    .get("plan_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'plan_id'".to_string()))?;

                let plan_hash = args
                    .get("plan_hash")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'plan_hash'".to_string()))?;

                let approved_by = args
                    .get("approved_by")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user_agent");

                let mut plan = self
                    .orchestrator
                    .get_plan(plan_id)
                    .await?
                    .ok_or_else(|| SmooothError::NotFound(format!("Plan '{}' not found", plan_id)))?;

                if plan.plan_hash != plan_hash {
                    return Err(SmooothError::PolicyViolation(format!(
                        "Approval hash mismatch! Expected: {}, Provided: {}",
                        plan.plan_hash, plan_hash
                    )));
                }

                plan.approve(approved_by.to_string());
                self.orchestrator.save_plan(plan.clone()).await?;

                Ok(serde_json::json!({
                    "success": true,
                    "plan_id": plan.id,
                    "plan_hash": plan.plan_hash,
                    "approved_by": approved_by
                }))
            }

            "execute_plan" => {
                let plan_id = args
                    .get("plan_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'plan_id'".to_string()))?;

                let idemp_key = args
                    .get("idempotency_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'idempotency_key'".to_string()))?
                    .to_string();

                let plan = self
                    .orchestrator
                    .get_plan(plan_id)
                    .await?
                    .ok_or_else(|| SmooothError::NotFound(format!("Plan '{}' not found", plan_id)))?;

                let op = self.orchestrator.execute_plan(plan, idemp_key).await?;

                Ok(serde_json::json!({
                    "operation_id": op.id,
                    "status": op.status,
                    "lease_epoch": op.lease_epoch,
                    "lease_owner": op.lease_owner,
                    "plan_hash": op.plan_hash
                }))
            }

            "create_service_binding" => {
                let source_id = args
                    .get("source_resource_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'source_resource_id'".to_string()))?;

                let target_id = args
                    .get("target_resource_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'target_resource_id'".to_string()))?;

                let binding_id = format!("bind_{}", uuid::Uuid::new_v4().simple());
                let raw_conn = format!(
                    "postgresql://app_user:p@ssw0rd999!@{}:5432/production_db",
                    target_id
                );

                let binding = ServiceBinding::new_with_secret(
                    binding_id.clone(),
                    source_id.to_string(),
                    target_id.to_string(),
                    "postgres_connection".to_string(),
                    raw_conn.as_bytes(),
                    &self.secret_store,
                    vec!["DATABASE_URL".to_string()],
                )?;

                self.orchestrator.save_binding(binding.clone()).await?;

                // Mask connection string before returning to agent
                let masked_conn = ServiceBinding::mask_connection_string(&raw_conn);

                Ok(serde_json::json!({
                    "binding_id": binding.id,
                    "source_resource_id": source_id,
                    "target_resource_id": target_id,
                    "masked_connection_string": masked_conn,
                    "exposed_env_keys": binding.exposed_env_keys,
                    "rotation_stage": binding.rotation_stage,
                    "encrypted_at_rest": true
                }))
            }

            "verify_connection" => {
                let resource_id = args
                    .get("resource_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'resource_id'".to_string()))?;

                let evidence = self.orchestrator.health_check(resource_id).await?;

                Ok(serde_json::json!({
                    "resource_id": resource_id,
                    "is_healthy": evidence.is_healthy,
                    "latency_ms": evidence.latency_ms,
                    "message": evidence.message,
                    "details": evidence.details
                }))
            }

            "get_operation" => {
                let op_id = args
                    .get("operation_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'operation_id'".to_string()))?;

                let op = self
                    .orchestrator
                    .get_operation(op_id)
                    .await
                    .ok_or_else(|| SmooothError::NotFound(format!("Operation '{}' not found", op_id)))?;

                Ok(serde_json::json!({
                    "operation_id": op.id,
                    "status": op.status,
                    "progress_percent": op.progress_percent,
                    "lease_epoch": op.lease_epoch,
                    "logs": op.logs,
                    "error_message": op.error_message,
                    "created_at": op.created_at,
                    "updated_at": op.updated_at
                }))
            }

            "reconcile_resource" => {
                let resource_id = args
                    .get("resource_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SmooothError::Validation("Missing 'resource_id'".to_string()))?;

                let res_opt = self.orchestrator.get_resource(resource_id).await?;
                match res_opt {
                    Some(resource) => {
                        let status = self.orchestrator.inspect_resource(resource_id).await?;
                        let is_drifted = status != resource.status;

                        Ok(serde_json::json!({
                            "resource_id": resource_id,
                            "observed_status": status,
                            "intended_status": resource.status,
                            "is_drifted": is_drifted,
                            "drift_corrected": false,
                            "message": if is_drifted {
                                "Drift detected between intended and observed state."
                            } else {
                                "Resource state matches intended specification."
                            }
                        }))
                    }
                    None => Err(SmooothError::NotFound(format!("Resource '{}' not found", resource_id))),
                }
            }

            unknown => Err(SmooothError::Validation(format!("Unknown tool '{}'", unknown))),
        }
    }
}
