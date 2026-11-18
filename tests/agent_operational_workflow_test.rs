use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use smoooth::api::build_app;
use smoooth::engine::orchestrator::Orchestrator;
use smoooth::mcp::server::{CallerContext, McpServer};
use smoooth::models::agent::AgentScope;
use smoooth::models::plan::{
    CanonicalActionSpec, CanonicalExecutionSpec, Plan, PolicyConstraints, RiskTier,
};
use smoooth::storage::local::LocalStorage;

fn build_dummy_spec(ws: &str) -> CanonicalExecutionSpec {
    CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: ws.to_string(),
        environment: "production".to_string(),
        target_id: "target_local".to_string(),
        actions: vec![CanonicalActionSpec {
            resource_name: "prod-db".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            action_type: "create".to_string(),
            config: serde_json::json!({ "database": "app_db" }),
            is_persistent: true,
            exposes_network_publicly: false,
        }],
        dependencies: vec![],
        immutable_artifacts: std::collections::HashMap::new(),
        policy_envelope: PolicyConstraints {
            allow_public_network: false,
            allow_data_destruction: false,
            max_monthly_budget_usd: Some(100),
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
        .expect("Failed to connect to test server");

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

    stream.write_all(req_str.as_bytes()).await.unwrap();

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await.unwrap();

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
async fn test_human_approval_queue_and_rejection() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let master_token = "operator_secret_approval_test".to_string();
    let (app, _) = build_app(orchestrator.clone(), master_token.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Pair an agent with Admin scope to test that even an Admin-scoped agent cannot self-approve
    let (agent, agent_token) = orchestrator
        .pair_agent(
            "dev-agent",
            "default",
            vec![
                AgentScope::Admin,
                AgentScope::Deploy,
                AgentScope::Plan,
                AgentScope::ReadOnly,
            ],
            None,
        )
        .await
        .unwrap();

    // 1. Save an unapproved plan created by agent
    let spec = build_dummy_spec("default");
    let plan = Plan::new(
        "plan_test_pending_1".to_string(),
        spec,
        RiskTier::High,
        Some(25.0),
    )
    .with_creator(format!("agent:{}", agent.agent_id));
    orchestrator.save_plan(plan.clone()).await.unwrap();

    // 2. Query GET /api/plans/pending
    let (status, body) =
        raw_http_call(addr, "GET", "/api/plans/pending", Some(&master_token), None).await;
    assert_eq!(status, 200);
    let pending_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(pending_json["total"], 1);
    assert_eq!(
        pending_json["pending_plans"][0]["id"],
        "plan_test_pending_1"
    );

    // 3. Agent attempts to self-approve via REST -> MUST FAIL with 403 Forbidden!
    let approve_payload = serde_json::to_string(&serde_json::json!({
        "plan_hash": plan.plan_hash,
    }))
    .unwrap();
    let (self_app_status, self_app_body) = raw_http_call(
        addr,
        "POST",
        "/api/plans/plan_test_pending_1/approve",
        Some(&agent_token),
        Some(&approve_payload),
    )
    .await;
    assert_eq!(self_app_status, 403);
    assert!(self_app_body.contains("cannot approve its own plan"));

    // 4. Operator rejects the pending plan
    let reject_payload = serde_json::to_string(&serde_json::json!({
        "reason": "Exceeds monthly staging budget",
    }))
    .unwrap();
    let (reject_status, _) = raw_http_call(
        addr,
        "POST",
        "/api/plans/plan_test_pending_1/reject",
        Some(&master_token),
        Some(&reject_payload),
    )
    .await;
    assert_eq!(reject_status, 200);

    // 5. Query GET /api/plans/pending again -> now 0 pending!
    let (after_status, after_body) =
        raw_http_call(addr, "GET", "/api/plans/pending", Some(&master_token), None).await;
    assert_eq!(after_status, 200);
    let after_json: serde_json::Value = serde_json::from_str(&after_body).unwrap();
    assert_eq!(after_json["total"], 0);
}

#[tokio::test]
async fn test_agent_native_mcp_aliases_and_diagnostics() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));
    let mcp = McpServer::new(orchestrator.clone());

    let (agent, _) = orchestrator
        .pair_agent(
            "ops-agent",
            "ws-mcp",
            vec![AgentScope::Deploy, AgentScope::Plan, AgentScope::ReadOnly],
            None,
        )
        .await
        .unwrap();

    let caller = CallerContext {
        is_operator: false,
        identity: Some(agent),
    };

    // 1. Test alias infrastructure_inspect -> returns capabilities
    let inspect_res = mcp
        .dispatch_tool_with_caller("infrastructure_inspect", serde_json::json!({}), &caller)
        .await;
    assert!(inspect_res.is_ok());
    let caps = inspect_res.unwrap();
    assert!(caps.as_array().unwrap().len() >= 3);

    // 2. Test alias infrastructure_plan -> creates plan
    let spec = build_dummy_spec("ws-mcp");
    let plan_res = mcp
        .dispatch_tool_with_caller(
            "infrastructure_plan",
            serde_json::json!({ "workspace_id": "ws-mcp", "spec": spec }),
            &caller,
        )
        .await;
    assert!(plan_res.is_ok());
    let plan_data = plan_res.unwrap();
    assert!(plan_data["plan_id"].as_str().is_some());
}
