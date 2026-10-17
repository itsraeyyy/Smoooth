use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use smoooth::api::build_app;
use smoooth::engine::orchestrator::Orchestrator;
use smoooth::mcp::server::{CallerContext, McpServer};
use smoooth::models::agent::AgentScope;
use smoooth::models::plan::{CanonicalActionSpec, CanonicalExecutionSpec, PolicyConstraints};
use smoooth::storage::local::LocalStorage;

fn build_dummy_plan_spec(ws: &str, is_delete: bool) -> CanonicalExecutionSpec {
    CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: ws.to_string(),
        environment: "development".to_string(),
        target_id: "target_local".to_string(),
        actions: vec![CanonicalActionSpec {
            resource_name: "test-workload".to_string(),
            provider: "docker".to_string(),
            service_type: "service".to_string(),
            action_type: if is_delete {
                "delete".to_string()
            } else {
                "create".to_string()
            },
            config: serde_json::json!({
                "image": "alpine:latest"
            }),
            is_persistent: false,
            exposes_network_publicly: false,
        }],
        dependencies: vec![],
        immutable_artifacts: std::collections::HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: true,
            max_monthly_budget_usd: None,
        },
    }
}

async fn raw_http_call(
    addr: SocketAddr,
    method: &str,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to HTTP test server");

    let content_len = body.map(|b| b.len()).unwrap_or(0);
    let mut req_str = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        method, path, addr
    );

    if let Some(tok) = bearer_token {
        req_str.push_str(&format!("Authorization: Bearer {}\r\n", tok));
    }

    if body.is_some() {
        req_str.push_str("Content-Type: application/json\r\n");
    }

    req_str.push_str(&format!("Content-Length: {}\r\n\r\n", content_len));

    if let Some(b) = body {
        req_str.push_str(b);
    }

    stream
        .write_all(req_str.as_bytes())
        .await
        .expect("Failed to write HTTP bytes");

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .await
        .expect("Failed to read HTTP response bytes");

    let resp_str = String::from_utf8_lossy(&response_bytes).to_string();
    let status_line = resp_str.lines().next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let (_, body_part) = resp_str.split_once("\r\n\r\n").unwrap_or((&resp_str, ""));
    (status_code, body_part.to_string())
}

#[tokio::test]
async fn test_read_only_agent_rejected_on_plan_creation_execution_and_deletion() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let mcp = McpServer::new(orchestrator.clone());

    // Pair a read_only agent
    let (identity, _raw_token) = orchestrator
        .pair_agent(
            "reader-agent",
            "ws-test",
            vec![AgentScope::ReadOnly],
            None,
        )
        .await
        .unwrap();

    let caller = CallerContext {
        is_operator: false,
        identity: Some(identity.clone()),
    };

    // 1. Attempt create_plan -> MUST FAIL
    let spec = build_dummy_plan_spec("ws-test", false);
    let create_res = mcp
        .dispatch_tool_with_caller(
            "create_plan",
            serde_json::json!({ "spec": spec }),
            &caller,
        )
        .await;
    assert!(create_res.is_err(), "ReadOnly agent must NOT create plans");
    assert!(create_res.err().unwrap().to_string().contains("Scope policy violation"));

    // 2. Attempt execute_plan -> MUST FAIL
    let exec_res = mcp
        .dispatch_tool_with_caller(
            "execute_plan",
            serde_json::json!({ "plan_id": "dummy_plan", "idempotency_key": "k1" }),
            &caller,
        )
        .await;
    assert!(exec_res.is_err(), "ReadOnly agent must NOT execute plans");

    // 3. Attempt reconcile_resource -> MUST FAIL
    let rec_res = mcp
        .dispatch_tool_with_caller(
            "reconcile_resource",
            serde_json::json!({ "resource_id": "r1" }),
            &caller,
        )
        .await;
    assert!(rec_res.is_err(), "ReadOnly agent must NOT reconcile resources");

    // 4. Allowed read-only operations
    let list_res = mcp
        .dispatch_tool_with_caller(
            "list_capabilities",
            serde_json::json!({}),
            &caller,
        )
        .await;
    assert!(list_res.is_ok(), "ReadOnly agent must inspect capabilities");
}

#[tokio::test]
async fn test_plan_agent_rejected_on_execution() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let mcp = McpServer::new(orchestrator.clone());

    let (identity, _raw_token) = orchestrator
        .pair_agent(
            "planner-agent",
            "ws-test",
            vec![AgentScope::Plan, AgentScope::ReadOnly],
            None,
        )
        .await
        .unwrap();

    let caller = CallerContext {
        is_operator: false,
        identity: Some(identity),
    };

    // 1. create_plan MUST SUCCEED
    let spec = build_dummy_plan_spec("ws-test", false);
    let create_res = mcp
        .dispatch_tool_with_caller(
            "create_plan",
            serde_json::json!({ "spec": spec }),
            &caller,
        )
        .await;
    assert!(create_res.is_ok(), "Plan agent should succeed creating plan");
    let plan_id = create_res.unwrap()["plan_id"].as_str().unwrap().to_string();

    // 2. execute_plan MUST FAIL (Requires Deploy scope)
    let exec_res = mcp
        .dispatch_tool_with_caller(
            "execute_plan",
            serde_json::json!({ "plan_id": plan_id, "idempotency_key": "k1" }),
            &caller,
        )
        .await;
    assert!(exec_res.is_err(), "Plan agent must NOT execute plans");
    assert!(exec_res.err().unwrap().to_string().contains("Scope policy violation"));
}

#[tokio::test]
async fn test_deploy_agent_rejected_on_destructive_deletion_and_cross_workspace() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let mcp = McpServer::new(orchestrator.clone());

    let (identity, _raw_token) = orchestrator
        .pair_agent(
            "deployer-agent",
            "ws-alpha",
            vec![AgentScope::Deploy, AgentScope::Plan, AgentScope::ReadOnly],
            None,
        )
        .await
        .unwrap();

    let caller = CallerContext {
        is_operator: false,
        identity: Some(identity),
    };

    // 1. Cross-workspace access attempt -> MUST FAIL
    let cross_ws_res = mcp
        .dispatch_tool_with_caller(
            "list_resources",
            serde_json::json!({ "workspace_id": "ws-beta" }),
            &caller,
        )
        .await;
    assert!(cross_ws_res.is_err(), "Agent bound to ws-alpha must NOT access ws-beta");
    assert!(cross_ws_res.err().unwrap().to_string().contains("Cross-workspace boundary violation"));

    // 2. Create a plan with a DESTRUCTIVE delete action
    let delete_spec = build_dummy_plan_spec("ws-alpha", true);
    let create_res = mcp
        .dispatch_tool_with_caller(
            "create_plan",
            serde_json::json!({ "spec": delete_spec }),
            &caller,
        )
        .await
        .unwrap();
    let plan_id = create_res["plan_id"].as_str().unwrap().to_string();

    // 3. Deployer attempts to execute destructive plan -> MUST FAIL (Requires Destructive scope!)
    let exec_res = mcp
        .dispatch_tool_with_caller(
            "execute_plan",
            serde_json::json!({ "plan_id": plan_id, "idempotency_key": "k-del" }),
            &caller,
        )
        .await;
    assert!(exec_res.is_err(), "Agent without 'destructive' scope must NOT execute deletion plans");
    assert!(exec_res.err().unwrap().to_string().contains("requires 'destructive' scope"));
}

#[tokio::test]
async fn test_revoked_token_rejected_on_both_mcp_and_rest() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let (app, _) = build_app(orchestrator.clone(), "test_master_token".to_string());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Pair an agent
    let (identity, raw_token) = orchestrator
        .pair_agent(
            "temporary-agent",
            "default",
            vec![AgentScope::ReadOnly],
            None,
        )
        .await
        .unwrap();

    // Verify token works initially
    let (status, _) = raw_http_call(addr, "GET", "/api/status", Some(&raw_token), None).await;
    assert_eq!(status, 200);

    // Revoke the token
    orchestrator.revoke_agent(&identity.agent_id).await.unwrap();

    // Attempt REST access with revoked token -> MUST RETURN 401
    let (status_revoked, _) = raw_http_call(addr, "GET", "/api/status", Some(&raw_token), None).await;
    assert_eq!(status_revoked, 401);

    // Attempt MCP access with revoked token -> MUST RETURN 401
    let mcp_body = serde_json::to_string(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    })).unwrap();
    let (status_mcp, _) = raw_http_call(addr, "POST", "/mcp", Some(&raw_token), Some(&mcp_body)).await;
    assert_eq!(status_mcp, 401);
}

#[tokio::test]
async fn test_unauthorized_requests_rejected_across_all_protected_endpoints() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let (app, _) = build_app(orchestrator, "master_secret".to_string());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let protected_paths = vec![
        ("GET", "/api/status"),
        ("GET", "/api/capabilities"),
        ("GET", "/api/agents"),
        ("GET", "/api/audit"),
        ("GET", "/api/doctor"),
        ("POST", "/mcp"),
    ];

    for (method, path) in protected_paths {
        let (status, _) = raw_http_call(addr, method, path, None, None).await;
        assert_eq!(
            status,
            401,
            "Endpoint {} must reject unauthenticated requests",
            path
        );
    }

    // Public health and dashboard endpoints must succeed without token
    let public_paths = vec!["/", "/dashboard", "/health", "/api/health"];
    for path in public_paths {
        let (status, _) = raw_http_call(addr, "GET", path, None, None).await;
        assert_eq!(
            status,
            200,
            "Public endpoint {} must be reachable without auth",
            path
        );
    }
}

#[tokio::test]
async fn test_zero_secret_leakage_in_api_responses_and_audits() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let (app, _) = build_app(orchestrator.clone(), "master_secret_123".to_string());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Pair an agent
    let pair_body = serde_json::to_string(&serde_json::json!({
        "name": "audit-test-agent",
        "workspace_id": "default",
        "scopes": "read_only"
    })).unwrap();

    let (status, pair_resp) = raw_http_call(
        addr,
        "POST",
        "/api/agents/pair",
        Some("master_secret_123"),
        Some(&pair_body),
    ).await;
    assert_eq!(status, 200);

    let pair_json: serde_json::Value = serde_json::from_str(&pair_resp).unwrap();

    // Raw token is returned ONCE
    let raw_token = pair_json["token"].as_str().unwrap().to_string();
    assert!(raw_token.starts_with("smoooth_ag_"));

    // Verify token_hash is NOT in the agent payload of pair response
    assert!(pair_json["agent"].get("token_hash").is_none());

    // 1. Inspect GET /api/agents -> MUST NOT contain raw_token OR token_hash
    let (list_status, list_body) = raw_http_call(
        addr,
        "GET",
        "/api/agents",
        Some("master_secret_123"),
        None,
    ).await;
    assert_eq!(list_status, 200);

    assert!(!list_body.contains(&raw_token), "Raw token must NEVER appear in agent listing");
    assert!(!list_body.contains("token_hash"), "token_hash must NEVER appear in agent listing API response");

    // 2. Inspect GET /api/audit -> MUST NOT contain raw_token OR token_hash
    let (audit_status, audit_body) = raw_http_call(
        addr,
        "GET",
        "/api/audit",
        Some("master_secret_123"),
        None,
    ).await;
    assert_eq!(audit_status, 200);

    assert!(!audit_body.contains(&raw_token), "Raw token must NEVER appear in audit events");
    assert!(!audit_body.contains("token_hash"), "token_hash must NEVER appear in audit events");
}
