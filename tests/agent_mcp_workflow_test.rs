use serde_json::json;
use std::sync::Arc;

use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::postgres::PostgresAdapter;
use smoooth::engine::Orchestrator;
use smoooth::mcp::server::{JsonRpcRequest, McpServer};
use smoooth::storage::{LocalStorage, Storage};

#[tokio::test]
async fn test_agent_mcp_golden_workflow_end_to_end() {
    // Setup Orchestrator with memory storage and adapters
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::new());
    let postgres_adapter = Arc::new(PostgresAdapter::new(docker_adapter.clone()));
    orchestrator.register_adapter(docker_adapter);
    orchestrator.register_adapter(postgres_adapter);

    let mcp = McpServer::new(Arc::new(orchestrator));

    // =========================================================================
    // STEP 1: Agent initializes MCP protocol connection
    // =========================================================================
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "coding-agent-client", "version": "1.0.0" }
        })),
    };
    let init_resp = mcp
        .handle_request(init_req)
        .await
        .expect("Must respond to initialize");
    assert!(init_resp.error.is_none());
    let server_info = init_resp.result.unwrap()["serverInfo"].clone();
    assert_eq!(server_info["name"], "smoooth-mcp");

    // =========================================================================
    // STEP 2: Agent queries list of available MCP tools
    // =========================================================================
    let list_tools_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };
    let list_tools_resp = mcp
        .handle_request(list_tools_req)
        .await
        .expect("Must list tools");
    let tools = list_tools_resp.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    assert!(tools.len() >= 7, "Must expose all golden workflow tools");
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"list_capabilities"));
    assert!(tool_names.contains(&"create_plan"));
    assert!(tool_names.contains(&"approve_plan"));
    assert!(tool_names.contains(&"execute_plan"));
    assert!(tool_names.contains(&"create_service_binding"));
    assert!(tool_names.contains(&"verify_connection"));

    // =========================================================================
    // STEP 3: Agent discovers infrastructure capabilities
    // =========================================================================
    let discover_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_capabilities",
            "arguments": {}
        })),
    };
    let discover_resp = mcp.handle_request(discover_req).await.unwrap();
    let text = discover_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(text.contains("cap_docker_container"));
    assert!(text.contains("cap_postgres_db"));
    assert!(text.contains("cap_redis_cache"));

    // =========================================================================
    // STEP 3b: Agent inspects capability schemas, required parameters, and boundaries
    // =========================================================================
    let schema_docker_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(31)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_capability_schema",
            "arguments": { "capability_id": "cap_docker_container" }
        })),
    };
    let schema_docker_resp = mcp.handle_request(schema_docker_req).await.unwrap();
    let docker_schema_text = schema_docker_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(docker_schema_text.contains("parameter_schema"));
    assert!(docker_schema_text.contains("supported_health_checks"));
    assert!(docker_schema_text.contains("approval_requirements"));
    assert!(docker_schema_text.contains("limitations"));

    let schema_pg_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(32)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_capability_schema",
            "arguments": { "capability_id": "cap_postgres_db" }
        })),
    };
    let schema_pg_resp = mcp.handle_request(schema_pg_req).await.unwrap();
    let pg_schema_text = schema_pg_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(pg_schema_text.contains("database"));
    assert!(pg_schema_text.contains("DATABASE_URL"));

    // Unknown capability must return NotFound error with available capabilities listed
    let schema_unknown_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(33)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_capability_schema",
            "arguments": { "capability_id": "cap_kubernetes_cluster" }
        })),
    };
    let schema_unknown_resp = mcp.handle_request(schema_unknown_req).await.unwrap();
    let is_err = schema_unknown_resp.result.unwrap()["isError"]
        .as_bool()
        .unwrap();
    assert!(is_err, "Unknown capability must return isError: true");

    // =========================================================================
    // STEP 4: Agent requests Plan generation for Docker app + PostgreSQL
    // =========================================================================
    let plan_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_plan",
            "arguments": {
                "workspace_id": "ws-agent-prod",
                "environment": "production",
                "actions": [
                    {
                        "resource_name": "app_db",
                        "provider": "postgres",
                        "service_type": "database",
                        "action_type": "create",
                        "config": {
                            "database": "production_app_db",
                            "user": "prod_user",
                            "port": 5432
                        },
                        "is_persistent": true,
                        "exposes_network_publicly": false
                    }
                ]
            }
        })),
    };
    let plan_resp = mcp.handle_request(plan_req).await.unwrap();
    let plan_out: serde_json::Value = serde_json::from_str(
        plan_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let plan_id = plan_out["plan_id"].as_str().unwrap().to_string();
    let plan_hash = plan_out["plan_hash"].as_str().unwrap().to_string();
    assert!(!plan_id.is_empty());
    assert_eq!(plan_hash.len(), 64, "Canonical plan hash must be SHA-256");

    // =========================================================================
    // STEP 5: Policy Governance & Approval verification
    // =========================================================================
    // 5a. Attempt approval with wrong hash -> MUST be rejected
    let bad_approve_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(5)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "approve_plan",
            "arguments": {
                "plan_id": plan_id,
                "plan_hash": "deadbeef_corrupted_hash_000000000000000000000000000000000000000000",
                "approved_by": "security_officer"
            }
        })),
    };
    let bad_approve_resp = mcp.handle_request(bad_approve_req).await.unwrap();
    assert!(
        bad_approve_resp.result.unwrap()["isError"]
            .as_bool()
            .unwrap(),
        "Mismatched plan hash approval must fail"
    );

    // 5b. Legitimate approval bound to exact canonical plan hash
    let good_approve_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(6)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "approve_plan",
            "arguments": {
                "plan_id": plan_id,
                "plan_hash": plan_hash,
                "approved_by": "authorized_dev_lead"
            }
        })),
    };
    let good_approve_resp = mcp.handle_request(good_approve_req).await.unwrap();
    let approve_out: serde_json::Value = serde_json::from_str(
        good_approve_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(approve_out["success"], true);

    // =========================================================================
    // STEP 6: Execute Plan with Idempotency Key & Monotonic Fencing Epoch
    // =========================================================================
    let exec_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(7)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "execute_plan",
            "arguments": {
                "plan_id": plan_id,
                "idempotency_key": "agent-tx-run-001"
            }
        })),
    };
    let exec_resp = mcp.handle_request(exec_req).await.unwrap();
    let exec_out: serde_json::Value = serde_json::from_str(
        exec_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let op_id = exec_out["operation_id"].as_str().unwrap().to_string();
    let lease_epoch = exec_out["lease_epoch"].as_u64().unwrap();
    assert!(!op_id.is_empty());
    assert_eq!(
        lease_epoch, 1,
        "Fencing epoch must initialize monotonically to 1"
    );

    // =========================================================================
    // STEP 7: Replay execution with same key -> Idempotent Success
    // =========================================================================
    let replay_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(8)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "execute_plan",
            "arguments": {
                "plan_id": plan_id,
                "idempotency_key": "agent-tx-run-001"
            }
        })),
    };
    let replay_resp = mcp.handle_request(replay_req).await.unwrap();
    let replay_out: serde_json::Value = serde_json::from_str(
        replay_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        replay_out["operation_id"], op_id,
        "Replay must return original operation"
    );

    // =========================================================================
    // STEP 8: Create Service Binding with AES-256-GCM and Zero Secret Leakage
    // =========================================================================
    // Register verified target database in workspace
    let db_pass = "classified_agent_test_pw_999";
    let _ = storage
        .save_resource(smoooth::models::Resource {
            id: "res_app_db_prod".to_string(),
            name: "app_db".to_string(),
            workspace_id: "ws-agent-prod".to_string(),
            service_type: "database".to_string(),
            provider: "postgres".to_string(),
            is_persistent: true,
            ownership: smoooth::models::ResourceOwnership::SmooothManaged,
            status: smoooth::models::ResourceStatus::Active,
            configuration: json!({
                "database": "production_app_db",
                "user": "prod_user",
                "password": db_pass,
                "port": 5432
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await;

    // 8a. Test Cross-Workspace Violation Rejection
    let rogue_bind_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(9)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_service_binding",
            "arguments": {
                "source_resource_id": "app_backend",
                "target_resource_id": "app_db",
                "workspace_id": "ws-other-unauthorized"
            }
        })),
    };
    let rogue_resp = mcp.handle_request(rogue_bind_req).await.unwrap();
    assert!(
        rogue_resp.result.unwrap()["isError"].as_bool().unwrap(),
        "Cross-workspace binding must be rejected"
    );

    // 8b. Authorized binding within workspace
    let bind_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(10)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_service_binding",
            "arguments": {
                "source_resource_id": "app_backend",
                "target_resource_id": "app_db",
                "workspace_id": "ws-agent-prod"
            }
        })),
    };
    let bind_resp = mcp.handle_request(bind_req).await.unwrap();
    let bind_out: serde_json::Value = serde_json::from_str(
        bind_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(bind_out["encrypted_at_rest"], true);
    let masked_conn = bind_out["masked_connection_string"].as_str().unwrap();
    assert!(
        !masked_conn.contains(db_pass),
        "Plaintext password must NEVER appear in MCP output"
    );
    assert!(
        masked_conn.contains("[REDACTED]"),
        "Masked token must be present"
    );

    // =========================================================================
    // STEP 9: Inspect Operation & Verify Health
    // =========================================================================
    let inspect_op_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(11)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_operation",
            "arguments": { "operation_id": op_id }
        })),
    };
    let inspect_op_resp = mcp.handle_request(inspect_op_req).await.unwrap();
    assert!(inspect_op_resp.error.is_none());

    // =========================================================================
    // STEP 10: Drift Reconciliation & Self-Healing via MCP
    // =========================================================================
    // Register a test resource representing an active container
    let test_app_res = smoooth::models::Resource {
        id: "container_web_app".to_string(),
        name: "web_app".to_string(),
        workspace_id: "ws-agent-prod".to_string(),
        service_type: "compute".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: smoooth::models::ResourceOwnership::SmooothManaged,
        status: smoooth::models::ResourceStatus::Active,
        configuration: json!({ "image": "nginx:alpine", "ports": { "80": 8080 } }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let _ = storage.save_resource(test_app_res).await;

    let reconcile_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(12)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "reconcile_resource",
            "arguments": { "resource_id": "container_web_app" }
        })),
    };
    let reconcile_resp = mcp.handle_request(reconcile_req).await.unwrap();
    assert!(reconcile_resp.error.is_none());
    let rec_out: serde_json::Value = serde_json::from_str(
        reconcile_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rec_out["resource_id"], "container_web_app");
}
