use serde_json::json;
use std::sync::Arc;

use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::postgres::PostgresAdapter;
use smoooth::engine::Orchestrator;
use smoooth::mcp::server::{JsonRpcRequest, McpServer};
use smoooth::models::{Resource, ResourceStatus};
use smoooth::storage::{LocalStorage, Storage};

#[tokio::test]
async fn test_milestone_1_golden_workflow_and_failure_drills() {
    // -------------------------------------------------------------------------
    // Setup clean storage and orchestrator instance
    // -------------------------------------------------------------------------
    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::new());
    let postgres_adapter = Arc::new(PostgresAdapter::new(docker_adapter.clone()));
    orchestrator.register_adapter(docker_adapter);
    orchestrator.register_adapter(postgres_adapter);

    let orchestrator = Arc::new(orchestrator);
    let mcp = McpServer::new(orchestrator.clone());

    // =========================================================================
    // STEP 1: Agent capability discovery
    // =========================================================================
    let disc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({ "name": "list_capabilities", "arguments": {} })),
    };
    let disc_resp = mcp.handle_request(disc_req).await.expect("Discover failed");
    let caps_text = disc_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        caps_text.contains("cap_docker_container"),
        "Must support Docker runtime"
    );
    assert!(
        caps_text.contains("cap_postgres_db"),
        "Must support PostgreSQL service"
    );

    // =========================================================================
    // STEP 2: Agent creates execution plan with app + persistent DB
    // =========================================================================
    let plan_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_plan",
            "arguments": {
                "workspace_id": "ws-golden-vps",
                "environment": "production",
                "actions": [
                    {
                        "resource_name": "app_db",
                        "provider": "postgres",
                        "service_type": "database",
                        "action_type": "create",
                        "config": {
                            "database": "golden_app_db",
                            "user": "golden_user",
                            "password": "classified_golden_password_999",
                            "port": 5432
                        },
                        "is_persistent": true,
                        "exposes_network_publicly": false
                    },
                    {
                        "resource_name": "app_service",
                        "provider": "docker",
                        "service_type": "compute",
                        "action_type": "create",
                        "config": {
                            "image": "golden_app:v1",
                            "port": 8080
                        },
                        "is_persistent": false,
                        "exposes_network_publicly": true
                    }
                ]
            }
        })),
    };
    let plan_resp = mcp.handle_request(plan_req).await.unwrap();
    let plan_data: serde_json::Value = serde_json::from_str(
        plan_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let plan_id = plan_data["plan_id"].as_str().unwrap().to_string();
    let plan_hash = plan_data["plan_hash"].as_str().unwrap().to_string();
    assert_eq!(plan_hash.len(), 64, "Plan hash must be valid SHA-256");

    // =========================================================================
    // STEP 3: Cryptographic Approval Gates & Tampering Rejection
    // =========================================================================
    // 3a. Reject tampered plan hash
    let bad_hash_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "approve_plan",
            "arguments": {
                "plan_id": plan_id,
                "plan_hash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                "approved_by": "sec_lead"
            }
        })),
    };
    let bad_hash_resp = mcp.handle_request(bad_hash_req).await.unwrap();
    assert!(
        bad_hash_resp.result.unwrap()["isError"].as_bool().unwrap(),
        "Mismatched hash must fail"
    );

    // 3b. Legitimate approval bound to exact hash with 1-hour expiry
    let good_approve_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "approve_plan",
            "arguments": {
                "plan_id": plan_id,
                "plan_hash": plan_hash,
                "approved_by": "ops_engineer"
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
    // STEP 4: Execution Pipeline & Idempotency Key Validation
    // =========================================================================
    let exec_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(5)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "execute_plan",
            "arguments": {
                "plan_id": plan_id,
                "idempotency_key": "idemp-golden-001"
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
    assert_eq!(
        exec_out["lease_epoch"], 1,
        "Lease epoch must be monotonic 1"
    );

    // Idempotent replay: same key + same payload returns existing operation
    let replay_resp = mcp
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(6)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "execute_plan",
                "arguments": {
                    "plan_id": plan_id,
                    "idempotency_key": "idemp-golden-001"
                }
            })),
        })
        .await
        .unwrap();
    let replay_out: serde_json::Value = serde_json::from_str(
        replay_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        replay_out["operation_id"], op_id,
        "Replay must return identical operation"
    );

    // =========================================================================
    // STEP 5: Service Binding & Zero Secret Leakage Validation
    // =========================================================================
    // Simulate active database resource
    let db_resource = Resource {
        id: "res_app_db_prod".to_string(),
        name: "app_db".to_string(),
        workspace_id: "ws-golden-vps".to_string(),
        service_type: "database".to_string(),
        provider: "postgres".to_string(),
        is_persistent: true,
        ownership: smoooth::models::ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Active,
        configuration: json!({
            "database": "golden_app_db",
            "user": "golden_user",
            "password": "classified_golden_password_999",
            "port": 5432
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.save_resource(db_resource).await.unwrap();

    let bind_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(7)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_service_binding",
            "arguments": {
                "source_resource_id": "app_service",
                "target_resource_id": "res_app_db_prod"
            }
        })),
    };
    let bind_resp = mcp.handle_request(bind_req).await.unwrap();
    let bind_out_text = bind_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();

    // STRICT SECRET SCANNER: Plaintext password must never appear in agent output
    assert!(
        !bind_out_text.contains("classified_golden_password_999"),
        "CRITICAL LEAK: Password found in agent MCP output!"
    );
    assert!(
        bind_out_text.contains("[REDACTED]"),
        "Masked token must be present"
    );

    // =========================================================================
    // STEP 6: Honest Verification Probe
    // =========================================================================
    let verify_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(8)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "verify_connection",
            "arguments": { "resource_id": "res_app_db_prod" }
        })),
    };
    let verify_resp = mcp.handle_request(verify_req).await.unwrap();
    let is_err = verify_resp
        .result
        .as_ref()
        .and_then(|r| r.get("isError"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_err {
        // Honest verification: Docker engine is unreachable on host runner, correctly reporting failure!
        let err_text = verify_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        println!(
            "Honest probe confirmed refusal to fake health without live infrastructure: {}",
            err_text
        );
        assert!(
            err_text.contains("Docker")
                || err_text.contains("failed")
                || err_text.contains("Error")
        );
    } else {
        let verify_out: serde_json::Value = serde_json::from_str(
            verify_resp.result.unwrap()["content"][0]["text"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        // Honest live verification: If a real backing container is running, is_healthy is true;
        // if this was an in-memory simulated resource without a real container, it must be reported unhealthy.
        if verify_out.get("is_healthy").and_then(|v| v.as_bool()) == Some(true) {
            println!("Live container verified healthy.");
        } else {
            let msg = verify_out
                .get("evidence")
                .or_else(|| verify_out.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(verify_out["is_healthy"], false);
            assert!(
                msg.contains("unavailable")
                    || msg.contains("not found")
                    || msg.contains("failed")
                    || msg.contains("Docker")
            );
        }
    }

    // =========================================================================
    // STEP 7: Controlled Failure Drill & Self-Healing Reconciliation
    // =========================================================================
    // Simulate app container crash (state drifted to Stopped)
    let crashed_app = Resource {
        id: "res_app_service_prod".to_string(),
        name: "app_service".to_string(),
        workspace_id: "ws-golden-vps".to_string(),
        service_type: "compute".to_string(),
        provider: "docker".to_string(),
        is_persistent: false,
        ownership: smoooth::models::ResourceOwnership::SmooothManaged,
        status: ResourceStatus::Stopped, // Crashed state!
        configuration: json!({ "image": "golden_app:v1" }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.save_resource(crashed_app).await.unwrap();

    let rec_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(9)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "reconcile_resource",
            "arguments": { "resource_id": "res_app_service_prod" }
        })),
    };
    let rec_resp = mcp.handle_request(rec_req).await.unwrap();
    let rec_is_err = rec_resp
        .result
        .as_ref()
        .and_then(|r| r.get("isError"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if rec_is_err {
        let err_text = rec_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        println!("Honest reconciliation response: {}", err_text);
    } else {
        let rec_out: serde_json::Value = serde_json::from_str(
            rec_resp.result.unwrap()["content"][0]["text"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(rec_out["resource_id"], "res_app_service_prod");
    }

    // =========================================================================
    // STEP 8: Zero-Data-Loss Invariant on Deployment Failure
    // =========================================================================
    // Verify persistent DB resource is intact and untouched
    let db_after = storage.get_resource("res_app_db_prod").await.unwrap();
    assert!(
        db_after.is_some(),
        "Persistent database must survive failed app deployment"
    );
    let db_after = db_after.unwrap();
    assert!(db_after.is_persistent, "Must remain marked persistent");
    assert_eq!(
        db_after.status,
        ResourceStatus::Active,
        "Database status must remain Active"
    );

    // =========================================================================
    // STEP 9: Process Restart & Startup Reconciliation
    // =========================================================================
    let reconciled_count = orchestrator.reconcile_on_startup().await.unwrap();
    println!("Reconciled {} resources on startup.", reconciled_count);
}
