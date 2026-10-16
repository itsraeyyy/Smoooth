use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};

use smoooth::adapters::docker::DockerAdapter;
use smoooth::api::build_app;
use smoooth::engine::Orchestrator;
use smoooth::models::{
    CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, PolicyConstraints,
};
use smoooth::storage::LocalStorage;

/// Helper function to perform raw HTTP requests over loopback TCP
async fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<&str>,
) -> (u16, HashMap<String, String>, String) {
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
        .expect("Failed to write HTTP request bytes");

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .await
        .expect("Failed to read HTTP response bytes");

    let resp_str = String::from_utf8_lossy(&response_bytes).to_string();
    let (header_part, body_part) = resp_str
        .split_once("\r\n\r\n")
        .unwrap_or((&resp_str, ""));

    let mut lines = header_part.lines();
    let status_line = lines.next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    (status_code, headers, body_part.to_string())
}

#[tokio::test]
async fn test_authenticated_http_gateway_and_remote_mcp_workflow() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::new());
    let postgres_adapter = Arc::new(smoooth::adapters::PostgresAdapter::new(docker_adapter.clone()));
    orchestrator.register_adapter(docker_adapter);
    orchestrator.register_adapter(postgres_adapter);
    let orch_ref = Arc::new(orchestrator);

    let auth_token = "smoooth_test_secret_token_1234567890abcdef".to_string();
    let (app, _state) = build_app(orch_ref.clone(), auth_token.clone());

    // Bind to an ephemeral port on loopback
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral listener");
    let server_addr = listener.local_addr().expect("Failed to get local addr");

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed");
    });

    // Give server a moment to spin up
    sleep(Duration::from_millis(50)).await;

    // -------------------------------------------------------------------------
    // 1. Verify Unauthenticated Health Probe
    // -------------------------------------------------------------------------
    println!("\n[Test 1] Testing unauthenticated public health endpoint...");
    let (status, _, body) = http_request(server_addr, "GET", "/health", None, None).await;
    assert_eq!(status, 200, "GET /health must return 200 OK without token");
    let health_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(health_json["status"], "ok");
    assert_eq!(health_json["service"], "smoooth-control-plane");
    println!(">>> Unauthenticated health probe returned 200 OK.");

    // -------------------------------------------------------------------------
    // 2. Verify Authentication Gate on Protected Endpoints
    // -------------------------------------------------------------------------
    println!("\n[Test 2] Testing authentication rejection on protected endpoints...");
    let protected_paths = vec!["/mcp", "/api/status", "/api/capabilities", "/api/tools"];
    for path in protected_paths {
        // No token
        let (status_no_tok, _, _) = http_request(server_addr, "GET", path, None, None).await;
        assert_eq!(
            status_no_tok, 401,
            "Request to {} without token must return 401 Unauthorized",
            path
        );

        // Wrong token
        let (status_bad_tok, _, _) = http_request(
            server_addr,
            "GET",
            path,
            Some("wrong_bearer_token"),
            None,
        )
        .await;
        assert_eq!(
            status_bad_tok, 401,
            "Request to {} with invalid token must return 401 Unauthorized",
            path
        );
    }
    println!(">>> Auth gate successfully rejected unauthenticated and invalid token requests with 401.");

    // -------------------------------------------------------------------------
    // 3. Verify Remote MCP JSON-RPC over HTTP (`POST /mcp`)
    // -------------------------------------------------------------------------
    println!("\n[Test 3] Testing Remote MCP JSON-RPC over HTTP with Bearer Auth...");

    // 3a. Initialize
    let init_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "claude-code", "version": "1.0.0" }
        }
    })
    .to_string();

    let (status, _, body) = http_request(
        server_addr,
        "POST",
        "/mcp",
        Some(&auth_token),
        Some(&init_payload),
    )
    .await;
    assert_eq!(status, 200, "POST /mcp initialize must return 200 OK");
    let init_resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "smoooth-mcp");
    assert_eq!(init_resp["result"]["protocolVersion"], "2024-11-05");
    println!(">>> MCP initialize response verified: protocolVersion 2024-11-05.");

    // 3b. tools/list
    let list_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    })
    .to_string();

    let (status, _, body) = http_request(
        server_addr,
        "POST",
        "/mcp",
        Some(&auth_token),
        Some(&list_payload),
    )
    .await;
    assert_eq!(status, 200);
    let tools_resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let tools = tools_resp["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 7, "Must contain all golden workflow tools");
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"list_capabilities"));
    assert!(tool_names.contains(&"create_plan"));
    assert!(tool_names.contains(&"approve_plan"));
    assert!(tool_names.contains(&"execute_plan"));
    assert!(tool_names.contains(&"create_service_binding"));
    assert!(tool_names.contains(&"verify_connection"));
    assert!(tool_names.contains(&"reconcile_resource"));
    println!(">>> MCP tools/list verified: {} tools discoverable.", tools.len());

    // 3c. Call list_capabilities
    let cap_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "list_capabilities",
            "arguments": {}
        }
    })
    .to_string();

    let (status, _, body) = http_request(
        server_addr,
        "POST",
        "/mcp",
        Some(&auth_token),
        Some(&cap_call),
    )
    .await;
    assert_eq!(status, 200);
    let cap_resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let text_content = cap_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text_content.contains("cap_docker_container"));
    assert!(text_content.contains("cap_postgres_db"));
    println!(">>> MCP tools/call list_capabilities verified.");

    // 3d. Call create_plan
    let mut artifacts = HashMap::new();
    artifacts.insert("web_app".to_string(), "sha256:fedcba".to_string());
    let spec = CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: "http_ws_test".to_string(),
        environment: "staging".to_string(),
        target_id: "test_node".to_string(),
        actions: vec![
            CanonicalActionSpec {
                resource_name: "test_pg".to_string(),
                provider: "postgres".to_string(),
                service_type: "database".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({ "image": "postgres:16-alpine" }),
                is_persistent: true,
                exposes_network_publicly: false,
            },
            CanonicalActionSpec {
                resource_name: "test_app".to_string(),
                provider: "docker".to_string(),
                service_type: "compute".to_string(),
                action_type: "create".to_string(),
                config: serde_json::json!({ "image": "alpine:latest" }),
                is_persistent: false,
                exposes_network_publicly: true,
            },
        ],
        dependencies: vec![DependencyEdge {
            from: "test_app".to_string(),
            to: "test_pg".to_string(),
        }],
        immutable_artifacts: artifacts,
        policy_envelope: PolicyConstraints {
            allow_public_network: true,
            allow_data_destruction: false,
            max_monthly_budget_usd: Some(50),
        },
    };

    let plan_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "create_plan",
            "arguments": {
                "workspace_id": "http_ws_test",
                "environment": "staging",
                "actions": spec.actions
            }
        }
    })
    .to_string();

    let (status, _, body) = http_request(
        server_addr,
        "POST",
        "/mcp",
        Some(&auth_token),
        Some(&plan_call),
    )
    .await;
    assert_eq!(status, 200);
    let plan_resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let plan_text = plan_resp["result"]["content"][0]["text"].as_str().unwrap();
    let plan_parsed: serde_json::Value = serde_json::from_str(plan_text).unwrap();
    let created_plan_id = plan_parsed["plan_id"].as_str().unwrap();
    let created_plan_hash = plan_parsed["plan_hash"].as_str().unwrap();
    println!(
        ">>> MCP create_plan succeeded. Plan ID: {}, Hash: {}",
        created_plan_id, created_plan_hash
    );

    // 3e. Call approve_plan with exact hash
    let approve_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "approve_plan",
            "arguments": {
                "plan_id": created_plan_id,
                "plan_hash": created_plan_hash,
                "approved_by": "http_claude_agent"
            }
        }
    })
    .to_string();

    let (status, _, body) = http_request(
        server_addr,
        "POST",
        "/mcp",
        Some(&auth_token),
        Some(&approve_call),
    )
    .await;
    assert_eq!(status, 200);
    let approve_resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let approve_text = approve_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(approve_text.contains("\"success\": true"));
    println!(">>> MCP approve_plan succeeded bound to exact canonical hash.");

    // -------------------------------------------------------------------------
    // 4. Verify Streamable HTTP / Server-Sent Events (SSE) MCP Transport
    // -------------------------------------------------------------------------
    println!("\n[Test 4] Testing Streamable HTTP (SSE) MCP Transport...");

    // Connect raw TCP stream for persistent SSE connection
    let mut sse_stream = TcpStream::connect(server_addr)
        .await
        .expect("Failed to connect SSE client");

    let sse_req = format!(
        "GET /sse HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nAccept: text/event-stream\r\n\r\n",
        server_addr, auth_token
    );
    sse_stream
        .write_all(sse_req.as_bytes())
        .await
        .expect("Failed to send SSE request");

    let mut sse_reader = BufReader::new(sse_stream);
    let mut sse_line = String::new();

    // Verify HTTP 200 OK on SSE connection
    sse_reader.read_line(&mut sse_line).await.unwrap();
    assert!(
        sse_line.contains("200 OK"),
        "SSE connection must return 200 OK: {}",
        sse_line
    );

    // Read until initial endpoint announcement
    let mut endpoint_msg = String::new();
    let mut session_id = String::new();
    for _ in 0..15 {
        sse_line.clear();
        sse_reader.read_line(&mut sse_line).await.unwrap();
        if sse_line.starts_with("data: /sse/message?sessionId=") {
            endpoint_msg = sse_line.trim().to_string();
            session_id = endpoint_msg
                .strip_prefix("data: /sse/message?sessionId=")
                .unwrap()
                .to_string();
            break;
        }
    }

    assert!(
        !session_id.is_empty(),
        "Must receive endpoint announcement with sessionId, got: {}",
        endpoint_msg
    );
    println!(">>> SSE connection established. Registered Session ID: {}", session_id);

    // Send JSON-RPC ping to /sse/message?sessionId=...
    let ping_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "ping",
        "params": {}
    })
    .to_string();

    let post_msg_path = format!("/sse/message?sessionId={}", session_id);
    let (msg_status, _, msg_body) = http_request(
        server_addr,
        "POST",
        &post_msg_path,
        Some(&auth_token),
        Some(&ping_payload),
    )
    .await;
    assert_eq!(msg_status, 202, "POST /sse/message must return 202 Accepted");
    assert!(msg_body.contains("\"status\":\"accepted\""));

    // Verify response delivered down the SSE stream
    let mut received_sse_message = false;
    for _ in 0..15 {
        sse_line.clear();
        sse_reader.read_line(&mut sse_line).await.unwrap();
        if sse_line.starts_with("data: {") && sse_line.contains("\"id\":99") {
            received_sse_message = true;
            println!(">>> Received JSON-RPC response over SSE stream: {}", sse_line.trim());
            break;
        }
    }
    assert!(
        received_sse_message,
        "Must receive JSON-RPC response via SSE stream"
    );

    // -------------------------------------------------------------------------
    // 5. Verify REST Control Plane Status Endpoint with Live State
    // -------------------------------------------------------------------------
    println!("\n[Test 5] Testing REST API /api/status endpoint with live workspace data...");
    let (status, _, body) = http_request(
        server_addr,
        "GET",
        "/api/status?workspace=http_ws_test",
        Some(&auth_token),
        None,
    )
    .await;
    assert_eq!(status, 200);
    let status_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(status_json["workspace"], "http_ws_test");
    println!(
        ">>> REST /api/status verified. Live resources: {}",
        status_json["total_resources"]
    );

    println!("\n============================================================");
    println!("✓ All Authenticated HTTP Gateway & Remote MCP Tests Passed!");
    println!("============================================================\n");
}
