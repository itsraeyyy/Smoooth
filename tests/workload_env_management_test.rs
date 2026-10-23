use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

use smoooth::api::build_app;
use smoooth::engine::Orchestrator;
use smoooth::models::{Resource, ResourceOwnership, ResourceStatus};
use smoooth::storage::{LocalStorage, Storage};

async fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    auth_token: Option<&str>,
    body: Option<&str>,
) -> (u16, HashMap<String, String>, String) {
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to test server");

    let mut req_str = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method, path, addr);
    req_str.push_str("Connection: close\r\n");

    if let Some(token) = auth_token {
        req_str.push_str(&format!("Authorization: Bearer {}\r\n", token));
    }

    let content_len = body.map(|b| b.len()).unwrap_or(0);
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
        .expect("Failed to write request");

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .await
        .expect("Failed to read response");

    let response_str = String::from_utf8_lossy(&response_bytes);
    let mut lines = response_str.lines();

    let status_line = lines.next().expect("No status line");
    let status_parts: Vec<&str> = status_line.split_whitespace().collect();
    let status_code: u16 = status_parts
        .get(1)
        .expect("No status code")
        .parse()
        .expect("Invalid status code");

    let mut headers = HashMap::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    let body_str: String = lines.collect::<Vec<&str>>().join("\n");
    (status_code, headers, body_str)
}

#[tokio::test]
async fn test_workload_environment_get_and_update_lifecycle() {
    let storage = Arc::new(LocalStorage::new_in_memory());
    let orch = Orchestrator::new(storage.clone());
    let orch_ref = Arc::new(orch);
    let auth_token = "smoooth_secret_operator_token_test_123456789".to_string();

    let (app, _state) = build_app(orch_ref.clone(), auth_token.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral listener");
    let server_addr = listener.local_addr().expect("Failed to get local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    sleep(Duration::from_millis(50)).await;

    // 1. Seed a test resource in storage
    let test_resource_id = "test-node-workload-env";
    let mut initial_env = serde_json::Map::new();
    initial_env.insert("NODE_ENV".to_string(), serde_json::json!("production"));
    initial_env.insert("PORT".to_string(), serde_json::json!("3000"));
    initial_env.insert("DATABASE_URL".to_string(), serde_json::json!("postgres://user:pass@db:5432/app"));

    let res = Resource {
        id: test_resource_id.to_string(),
        name: test_resource_id.to_string(),
        workspace_id: "default".to_string(),
        service_type: "web".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: serde_json::json!({
            "image": "node:20-alpine",
            "port": 3000,
            "env": initial_env
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    storage.save_resource(res).await.expect("Failed to seed resource");

    // 2. Test GET /api/workloads/:id/env
    println!("\n[Test 1] Testing GET /api/workloads/:id/env...");
    let (status, _, body) = http_request(
        server_addr,
        "GET",
        &format!("/api/workloads/{}/env", test_resource_id),
        Some(&auth_token),
        None,
    )
    .await;

    assert_eq!(status, 200, "Expected 200 OK for GET env, got: {}", status);
    let json: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    assert_eq!(json["resource_id"], test_resource_id);
    assert_eq!(json["env"]["NODE_ENV"], "production");
    assert_eq!(json["env"]["PORT"], "3000");
    assert_eq!(json["env"]["DATABASE_URL"], "postgres://user:pass@db:5432/app");
    assert_eq!(json["count"], 3);
    println!(">>> GET /api/workloads/:id/env verified successfully with {} variables.", json["count"]);

    // 3. Test POST /api/workloads/:id/env (Updating environment variables)
    println!("\n[Test 2] Testing POST /api/workloads/:id/env to update environment...");
    let update_payload = serde_json::json!({
        "env": {
            "NODE_ENV": "production",
            "PORT": "8080",
            "NEW_FEATURE_FLAG": "enabled",
            "DATABASE_URL": "postgres://user:rotated_secret@db:5432/app"
        }
    })
    .to_string();

    let (update_status, _, update_body) = http_request(
        server_addr,
        "POST",
        &format!("/api/workloads/{}/env", test_resource_id),
        Some(&auth_token),
        Some(&update_payload),
    )
    .await;

    assert_eq!(update_status, 200, "Expected 200 OK for POST env, got: {}. Body: {}", update_status, update_body);
    let update_json: serde_json::Value = serde_json::from_str(&update_body).expect("Valid JSON");
    assert_eq!(update_json["success"], true);
    assert_eq!(update_json["env_count"], 4);
    println!(">>> POST /api/workloads/:id/env succeeded! Updated {} variables.", update_json["env_count"]);

    // 4. Test GET /api/workloads/:id/env again to confirm persistence
    println!("\n[Test 3] Testing GET /api/workloads/:id/env to verify updated persistence...");
    let (get_status_2, _, get_body_2) = http_request(
        server_addr,
        "GET",
        &format!("/api/workloads/{}/env", test_resource_id),
        Some(&auth_token),
        None,
    )
    .await;

    assert_eq!(get_status_2, 200);
    let json_2: serde_json::Value = serde_json::from_str(&get_body_2).expect("Valid JSON");
    assert_eq!(json_2["env"]["PORT"], "8080");
    assert_eq!(json_2["env"]["NEW_FEATURE_FLAG"], "enabled");
    assert_eq!(json_2["env"]["DATABASE_URL"], "postgres://user:rotated_secret@db:5432/app");
    assert_eq!(json_2["count"], 4);
    println!(">>> Verified updated environment variables are actively persisted!");

    // 5. Test Unauthorized Access (No Token)
    println!("\n[Test 4] Testing unauthorized access rejection...");
    let (unauth_status, _, _) = http_request(
        server_addr,
        "POST",
        &format!("/api/workloads/{}/env", test_resource_id),
        None,
        Some(&update_payload),
    )
    .await;

    assert_eq!(unauth_status, 401, "Expected 401 Unauthorized without token");
    println!(">>> Unauthorized request successfully rejected with 401.");
}
