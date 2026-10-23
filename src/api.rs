use axum::{
    extract::{Extension, Json, Path, Query, Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    routing::{delete, get, post},
    Router,
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::CorsLayer;

use crate::engine::Orchestrator;
use crate::mcp::server::{CallerContext, JsonRpcRequest};
use crate::mcp::tools::{get_available_tools, get_default_capabilities};
use crate::mcp::McpServer;
use crate::models::{AgentIdentity, AgentScope};

pub type SseSender = mpsc::Sender<Result<Event, Infallible>>;

#[derive(Clone, Debug)]
pub struct AuthenticatedCaller {
    pub is_operator: bool,
    pub identity: Option<AgentIdentity>,
}

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub mcp_server: Arc<McpServer>,
    pub auth_token: String,
    pub sse_sessions: Arc<RwLock<HashMap<String, SseSender>>>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Deserialize)]
pub struct StatusQuery {
    pub workspace: Option<String>,
}

#[derive(Serialize)]
pub struct ResourceSummary {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub service_type: String,
    pub status: String,
    pub is_persistent: bool,
    pub configuration: serde_json::Value,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub workspace: String,
    pub total_resources: usize,
    pub resources: Vec<ResourceSummary>,
}

#[derive(Deserialize)]
pub struct ApprovePlanRequest {
    pub plan_hash: String,
    pub approver: Option<String>,
    pub ttl_seconds: Option<i64>,
}

#[derive(Deserialize)]
pub struct SseMessageQuery {
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PairAgentRequest {
    pub name: String,
    pub workspace_id: Option<String>,
    pub scopes: Option<String>,
    pub ttl_hours: Option<i64>,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct BuildPlanRequest {
    pub source: String,
}

#[derive(Deserialize)]
pub struct BuildNixpacksRequest {
    pub source: String,
    pub image_tag: String,
    pub env_vars: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct AddIngressRequest {
    pub domain: String,
    pub target_resource: String,
    pub target_port: u16,
    pub tls: Option<bool>,
    pub path_prefix: Option<String>,
}

/// Build the API application router and associated AppState
pub fn build_app(orchestrator: Arc<Orchestrator>, auth_token: String) -> (Router, AppState) {
    let mcp_server = Arc::new(McpServer::new(orchestrator.clone()));
    let sse_sessions = Arc::new(RwLock::new(HashMap::new()));

    let state = AppState {
        orchestrator,
        mcp_server,
        auth_token,
        sse_sessions,
    };

    // Hardened CORS: Restrictive by default; no permissive wildcard access
    let cors = if let Ok(allowed_origin) = std::env::var("SMOOOTH_ALLOWED_ORIGIN") {
        if let Ok(header_val) = allowed_origin.parse::<axum::http::HeaderValue>() {
            CorsLayer::new()
                .allow_origin(header_val)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        } else {
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        }
    } else {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    };

    let router = Router::new()
        // Unauthenticated Web Dashboard & Health Probes
        .route("/", get(dashboard_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        // Remote MCP Endpoints
        .route("/mcp", post(mcp_post_handler))
        .route("/sse", get(sse_connect_handler))
        .route("/sse/message", post(sse_message_handler))
        // REST Control Plane Endpoints
        .route("/api/status", get(status_handler))
        .route("/api/capabilities", get(capabilities_handler))
        .route("/api/tools", get(tools_handler))
        .route("/api/plans/:id", get(get_plan_handler))
        .route("/api/plans/:id/approve", post(approve_plan_handler))
        .route("/api/operations", get(list_operations_handler))
        .route("/api/operations/:id", get(get_operation_handler))
        .route("/api/reconcile/:resource_id", post(reconcile_handler))
        // Agent Access & Security Endpoints
        .route("/api/agents", get(list_agents_handler))
        .route("/api/agents/pair", post(pair_agent_handler))
        .route("/api/agents/:id/revoke", post(revoke_agent_handler))
        .route("/api/agents/:id/rotate", post(rotate_agent_handler))
        .route("/api/audit", get(audit_handler))
        .route("/api/doctor", get(doctor_handler))
        // Nixpacks Automated Build Engine
        .route("/api/build/plan", post(build_plan_handler))
        .route("/api/build/nixpacks", post(build_nixpacks_handler))
        // Ingress & Custom Domains
        .route("/api/ingress", get(list_ingress_handler).post(add_ingress_handler))
        .route("/api/ingress/:domain", delete(remove_ingress_handler))
        .route("/api/ingress/export/:format", get(export_ingress_handler))
        // Real-Time Container Logs (Overseer Drawer)
        .route("/api/resources/:id/logs", get(resource_logs_handler))
        // 1-Click Workload Deployment & Lifecycle (UI & Operators)
        .route("/api/workloads/deploy", post(deploy_workload_handler))
        .route("/api/workloads/:id", delete(delete_workload_handler))
        .route("/api/workloads/:id/restart", post(restart_workload_handler))
        .route("/api/workloads/:id/env", get(get_workload_env_handler).post(update_workload_env_handler))
        // Migration Engine (Coolify, Compose, Render, Fly, Dokku, Host Scanner)
        .route("/api/migrate/detect", post(migrate_detect_handler))
        .route("/api/migrate/preview", post(migrate_preview_handler))
        .route("/api/migrate/apply", post(migrate_apply_handler))
        .route("/api/migrate/scan-host", get(migrate_scan_host_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(cors)
        .with_state(state.clone());

    (router, state)
}

/// Backwards-compatible router factory
pub fn create_router(orchestrator: Arc<Orchestrator>, auth_token: String) -> Router {
    let (router, _) = build_app(orchestrator, auth_token);
    router
}

/// Authentication middleware enforcing Bearer token validation against operator secret or hashed agent tokens
async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let path = req.uri().path();
    // Allow public unauthenticated health probes and web dashboard UI
    if path == "/" || path == "/dashboard" || path == "/health" || path == "/api/health" {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let raw_token = if let Some(auth) = auth_header {
        if let Some(bearer) = auth.strip_prefix("Bearer ") {
            bearer.trim()
        } else {
            ""
        }
    } else {
        ""
    };

    if raw_token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Valid Bearer authentication token required. Configure 'Authorization: Bearer <token>' header."
            })),
        ));
    }

    let caller = if raw_token == state.auth_token {
        AuthenticatedCaller {
            is_operator: true,
            identity: None,
        }
    } else {
        match state.orchestrator.authenticate_agent(raw_token).await {
            Ok(Some(agent)) => AuthenticatedCaller {
                is_operator: false,
                identity: Some(agent),
            },
            _ => {
                // Record failed unauthorized attempt in security audit without logging token characters
                let _ = state
                    .orchestrator
                    .storage()
                    .record_audit(crate::storage::AuditEvent {
                        id: 0,
                        workspace_id: "*".to_string(),
                        actor: "unauthenticated".to_string(),
                        action: "auth_attempt".to_string(),
                        target_resource_id: None,
                        plan_hash: None,
                        outcome: "denied".to_string(),
                        redacted_evidence: Some(serde_json::json!({
                            "path": path,
                            "token_provided": !raw_token.is_empty()
                        })),
                        timestamp: chrono::Utc::now(),
                    })
                    .await;

                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Invalid or revoked credentials. Use 'smoooth agent pair' to issue valid agent tokens."
                    })),
                ));
            }
        }
    };

    req.extensions_mut().insert(caller);
    Ok(next.run(req).await)
}

/// Control Dashboard UI with strict HTTP security headers
async fn dashboard_handler() -> impl IntoResponse {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        "DENY".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        "nosniff".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );

    (headers, Html(crate::dashboard::get_dashboard_html()))
}

/// Public Health Probe
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "smoooth-control-plane",
        version: "0.1.1",
    })
}

/// Direct JSON-RPC 2.0 over HTTP MCP Handler (authenticated with caller context)
async fn mcp_post_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let caller_ctx = CallerContext {
        is_operator: caller.is_operator,
        identity: caller.identity,
    };
    let resp = state
        .mcp_server
        .handle_request_with_caller(req, &caller_ctx)
        .await;
    match resp {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::to_value(r).unwrap_or_default()),
        ),
        None => (StatusCode::NO_CONTENT, Json(serde_json::Value::Null)),
    }
}

/// MCP Server-Sent Events (SSE) Stream Endpoint (`GET /sse`)
async fn sse_connect_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    let endpoint_url = format!("/sse/message?sessionId={}", session_id);
    let init_event = Event::default().event("endpoint").data(endpoint_url);
    let _ = tx.send(Ok(init_event)).await;

    {
        let mut sessions = state.sse_sessions.write().await;
        sessions.insert(session_id.clone(), tx);
    }

    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|ev| (ev, rx))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// MCP SSE Message Receiver (`POST /sse/message?sessionId=...`)
async fn sse_message_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Query(query): Query<SseMessageQuery>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let session_id = match query.session_id {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing sessionId query parameter"
                })),
            );
        }
    };

    let tx = {
        let sessions = state.sse_sessions.read().await;
        sessions.get(&session_id).cloned()
    };

    let tx = match tx {
        Some(t) => t,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "SSE Session not found or closed"
                })),
            );
        }
    };

    let caller_ctx = CallerContext {
        is_operator: caller.is_operator,
        identity: caller.identity,
    };

    let resp = state
        .mcp_server
        .handle_request_with_caller(req, &caller_ctx)
        .await;
    if let Some(r) = resp {
        let resp_json = serde_json::to_string(&r).unwrap_or_default();
        let sse_event = Event::default().event("message").data(resp_json);
        if tx.send(Ok(sse_event)).await.is_err() {
            let mut sessions = state.sse_sessions.write().await;
            sessions.remove(&session_id);
        }
    }

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "sessionId": session_id
        })),
    )
}

/// Workspace Resources Status Handler (Real Orchestrator Data)
async fn status_handler(
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let workspace = query.workspace.unwrap_or_else(|| "default".to_string());
    let resources = state.orchestrator.list_resources(&workspace).await;

    let items: Vec<ResourceSummary> = resources
        .into_iter()
        .map(|r| ResourceSummary {
            id: r.id,
            name: r.name,
            provider: r.provider,
            service_type: r.service_type,
            status: format!("{:?}", r.status),
            is_persistent: r.is_persistent,
            configuration: r.configuration,
        })
        .collect();

    let total = items.len();
    Json(StatusResponse {
        workspace,
        total_resources: total,
        resources: items,
    })
}

/// Discover Capabilities Handler
async fn capabilities_handler() -> impl IntoResponse {
    Json(get_default_capabilities())
}

/// List Tools Handler
async fn tools_handler() -> impl IntoResponse {
    Json(get_available_tools())
}

/// Get Plan Handler
async fn get_plan_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    match state.orchestrator.get_plan(&id).await {
        Ok(Some(plan)) => Ok(Json(plan)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Approve Plan Handler (Enforces Admin privilege)
async fn approve_plan_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
    Json(body): Json<ApprovePlanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Admin))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Admin privileges required to approve plans"
            })),
        ));
    }

    let plan = state.orchestrator.get_plan(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let mut plan = match plan {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Plan '{}' not found", id) })),
            ));
        }
    };

    if plan.plan_hash != body.plan_hash {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Plan hash mismatch",
                "expected": plan.plan_hash,
                "provided": body.plan_hash
            })),
        ));
    }

    let approver = body
        .approver
        .unwrap_or_else(|| "remote_operator".to_string());
    let ttl = body.ttl_seconds.unwrap_or(3600);
    plan.approve_with_ttl(approver.clone(), ttl);

    state
        .orchestrator
        .save_plan(plan.clone())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "plan_id": plan.id,
        "plan_hash": plan.plan_hash,
        "approved_by": approver,
        "ttl_seconds": ttl
    })))
}

/// List Operations Handler
async fn list_operations_handler(
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let ops = state
        .orchestrator
        .list_operations(query.workspace.as_deref())
        .await;
    Json(serde_json::json!({
        "operations": ops
    }))
}

/// Get Operation Status and Evidence Handler
async fn get_operation_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    match state.orchestrator.get_operation(&id).await {
        Some(op) => Ok(Json(op)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Reconcile Resource Handler
async fn reconcile_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(resource_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Deploy scope required to reconcile resources"
            })),
        ));
    }

    match state.orchestrator.reconcile_resource(&resource_id).await {
        Ok(outcome) => Ok(Json(outcome)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// List Agents Handler
async fn list_agents_handler(
    State(state): State<AppState>,
    Extension(_caller): Extension<AuthenticatedCaller>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.orchestrator.list_agents(None).await {
        Ok(agents) => {
            let sanitized: Vec<serde_json::Value> = agents
                .into_iter()
                .map(|a| {
                    let mut val = serde_json::to_value(&a).unwrap_or_default();
                    if let Some(obj) = val.as_object_mut() {
                        obj.remove("token_hash");
                    }
                    val
                })
                .collect();
            Ok(Json(serde_json::json!({ "agents": sanitized })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Pair Agent Handler (Enforces Admin privilege)
async fn pair_agent_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Json(body): Json<PairAgentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Admin))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Admin privileges required to pair new agents"
            })),
        ));
    }

    let ws = body.workspace_id.unwrap_or_else(|| "default".to_string());
    let scopes_str = body
        .scopes
        .unwrap_or_else(|| "read_only,plan,deploy".to_string());
    let scopes = AgentScope::parse_csv(&scopes_str);

    match state
        .orchestrator
        .pair_agent(&body.name, &ws, scopes, body.ttl_hours)
        .await
    {
        Ok((agent, raw_token)) => {
            let mut agent_val = serde_json::to_value(&agent).unwrap_or_default();
            if let Some(obj) = agent_val.as_object_mut() {
                obj.remove("token_hash");
            }
            Ok(Json(serde_json::json!({
                "success": true,
                "agent": agent_val,
                "token": raw_token
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Revoke Agent Handler
async fn revoke_agent_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Admin))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Admin privileges required to revoke agents"
            })),
        ));
    }

    match state.orchestrator.revoke_agent(&id).await {
        Ok(()) => Ok(Json(serde_json::json!({ "success": true, "agent_id": id }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Rotate Agent Token Handler
async fn rotate_agent_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Admin))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Admin privileges required to rotate agent tokens"
            })),
        ));
    }

    match state.orchestrator.rotate_agent_token(&id).await {
        Ok(new_token) => Ok(Json(serde_json::json!({
            "success": true,
            "agent_id": id,
            "token": new_token
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Security Audit Trail Handler
async fn audit_handler(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ws = query.workspace.unwrap_or_else(|| "*".to_string());
    match state.orchestrator.storage().list_audit_events(&ws).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Live Preflight Doctor Diagnostics Handler
async fn doctor_handler(State(_state): State<AppState>) -> impl IntoResponse {
    let storage_path =
        std::env::var("SMOOOTH_STORAGE_PATH").unwrap_or_else(|_| "smoooth_state.json".to_string());
    let report = crate::doctor::Doctor::run_diagnostics(&storage_path, "127.0.0.1", 4040).await;
    Json(report)
}

/// Nixpacks Plan Generation Handler
async fn build_plan_handler(
    State(_state): State<AppState>,
    Json(body): Json<BuildPlanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match crate::build::NixpacksBuilder::plan(&body.source).await {
        Ok(plan) => Ok(Json(plan)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Nixpacks Automated Image Build Handler
async fn build_nixpacks_handler(
    State(_state): State<AppState>,
    Json(body): Json<BuildNixpacksRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match crate::build::NixpacksBuilder::build(&body.source, &body.image_tag, body.env_vars).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Ingress Routes List Handler
async fn list_ingress_handler(State(_state): State<AppState>) -> impl IntoResponse {
    let ingress = crate::ingress::IngressManager::default_local();
    let routes = ingress.list_routes().unwrap_or_default();
    let mode = crate::ingress::IngressManager::detect_mode();
    Json(serde_json::json!({
        "ingress_mode": mode,
        "routes": routes,
    }))
}

/// Ingress Route Add Handler
async fn add_ingress_handler(
    State(_state): State<AppState>,
    Json(body): Json<AddIngressRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ingress = crate::ingress::IngressManager::default_local();
    match ingress.add_route(
        &body.domain,
        &body.target_resource,
        body.target_port,
        body.tls.unwrap_or(true),
        body.path_prefix,
    ) {
        Ok(route) => Ok(Json(route)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Ingress Route Remove Handler
async fn remove_ingress_handler(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ingress = crate::ingress::IngressManager::default_local();
    match ingress.remove_route(&domain) {
        Ok(true) => Ok(Json(serde_json::json!({ "success": true, "domain": domain }))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Route for domain '{}' not found", domain) })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Ingress Route Config Export Handler (Caddyfile, Nginx, Traefik)
async fn export_ingress_handler(
    State(_state): State<AppState>,
    Path(format): Path<String>,
) -> impl IntoResponse {
    let ingress = crate::ingress::IngressManager::default_local();
    let config = ingress.render_upstream_snippet(&format).unwrap_or_else(|e| format!("# Error generating config: {}", e));
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        config,
    )
}

/// Real-Time Container Logs Handler for Overseer Drawer
async fn resource_logs_handler(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let logs_cmd = tokio::process::Command::new("docker")
        .args(["logs", "--tail", "150", &id])
        .output()
        .await;

    match logs_cmd {
        Ok(output) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            Json(serde_json::json!({
                "resource_id": id,
                "logs": combined,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "resource_id": id,
            "logs": format!("Unable to fetch container logs: {}", e),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeployWorkloadRequest {
    pub name: String,
    pub image: String,
    pub port: Option<u16>,
    pub env_vars: Option<HashMap<String, String>>,
    pub service_type: Option<String>,
    pub workspace: Option<String>,
    pub domain: Option<String>,
}

/// 1-Click Workload Deployment Handler for UI & Operators
async fn deploy_workload_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Json(body): Json<DeployWorkloadRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Forbidden",
                "message": "Deploy scope required to deploy workloads"
            })),
        ));
    }

    let clean_name = body.name.trim().to_lowercase().replace(' ', "-");
    let clean_image = body.image.trim().to_string();

    if clean_name.is_empty() || clean_image.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Bad Request",
                "message": "Both 'name' and 'image' are required."
            })),
        ));
    }

    let ws = body.workspace.unwrap_or_else(|| "default".to_string());
    let service_type = body.service_type.unwrap_or_else(|| "web".to_string());
    let port = body.port.unwrap_or(8080);
    let mut env_vars = body.env_vars.unwrap_or_default();
    env_vars.entry("PORT".to_string()).or_insert_with(|| port.to_string());

    let container_port = if clean_image.contains("nginx") {
        80
    } else if clean_image.contains("redis") {
        6379
    } else if clean_image.contains("postgres") {
        5432
    } else if clean_image.contains("mongo") {
        27017
    } else {
        port
    };

    let mut ports_map = serde_json::Map::new();
    ports_map.insert(format!("{}/tcp", container_port), serde_json::json!(port));

    let action = crate::models::CanonicalActionSpec {
        action_type: "create".to_string(),
        provider: "docker".to_string(),
        resource_name: clean_name.clone(),
        service_type: service_type.clone(),
        is_persistent: false,
        exposes_network_publicly: true,
        config: serde_json::json!({
            "image": clean_image,
            "port": port,
            "ports": ports_map,
            "env": env_vars,
            "workspace_id": ws
        }),
    };

    let spec = crate::models::CanonicalExecutionSpec {
        schema_version: 1,
        workspace_id: ws.clone(),
        environment: "production".to_string(),
        target_id: "local_docker_node".to_string(),
        actions: vec![action],
        dependencies: Vec::new(),
        immutable_artifacts: HashMap::new(),
        policy_envelope: crate::models::PolicyConstraints {
            allow_public_network: true,
            allow_data_destruction: false,
            max_monthly_budget_usd: Some(150),
        },
    };

    let plan_id = format!("plan_{}", uuid::Uuid::new_v4().simple());
    let mut plan = crate::models::Plan::new(plan_id, spec, crate::models::RiskTier::Low, Some(10.0));
    plan.approve_with_ttl("operator_dashboard".to_string(), 3600);

    state.orchestrator.save_plan(plan.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let idemp_key = format!("ui_deploy_{}", uuid::Uuid::new_v4().simple());
    let op = state
        .orchestrator
        .execute_plan_sync(plan.clone(), idemp_key)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    // If domain specified, register ingress route automatically
    if let Some(domain) = body.domain {
        let trimmed_dom = domain.trim();
        if !trimmed_dom.is_empty() {
            let ingress = crate::ingress::IngressManager::default_local();
            let _ = ingress.add_route(trimmed_dom, &clean_name, port, true, None);
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "resource_id": clean_name,
        "operation_id": op.id,
        "status": format!("{:?}", op.status),
        "plan_id": plan.id,
        "port": port
    })))
}

/// Restart Workload Container Handler
async fn restart_workload_handler(
    State(_state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Deploy scope required" })),
        ));
    }

    let output = tokio::process::Command::new("docker")
        .args(["restart", &id])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => Ok(Json(serde_json::json!({
            "success": true,
            "resource_id": id,
            "message": "Container restarted successfully."
        }))),
        Ok(o) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Restart failed",
                "message": String::from_utf8_lossy(&o.stderr).to_string()
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Delete Workload Container Handler
async fn delete_workload_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Deploy scope required" })),
        ));
    }

    // Stop and remove container via docker CLI
    let _ = tokio::process::Command::new("docker")
        .args(["rm", "-f", &id])
        .output()
        .await;

    // Delete from state storage
    let _ = state.orchestrator.delete_resource(&id).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "resource_id": id
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkloadEnvRequest {
    pub env: HashMap<String, String>,
}

/// Inspect Workload Environment Variables Handler
async fn get_workload_env_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut env_map = serde_json::Map::new();
    let mut image_name = String::new();
    let mut container_name = id.clone();

    // 1. Try docker inspect first
    let inspect_output = tokio::process::Command::new("docker")
        .args(["inspect", &id])
        .output()
        .await;

    if let Ok(output) = inspect_output {
        if output.status.success() {
            if let Ok(json_arr) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(c) = json_arr.as_array().and_then(|a| a.first()) {
                    if let Some(name) = c.get("Name").and_then(|n| n.as_str()) {
                        container_name = name.trim_start_matches('/').to_string();
                    }
                    if let Some(img) = c.get("Config").and_then(|cfg| cfg.get("Image")).and_then(|i| i.as_str()) {
                        image_name = img.to_string();
                    }
                    if let Some(envs) = c.get("Config").and_then(|cfg| cfg.get("Env")).and_then(|e| e.as_array()) {
                        for item in envs {
                            if let Some(line) = item.as_str() {
                                if let Some((k, v)) = line.split_once('=') {
                                    env_map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Storage fallback if Docker inspect was empty or failed
    if env_map.is_empty() {
        if let Ok(Some(resource)) = state.orchestrator.storage().get_resource(&id).await {
            if let Some(env_obj) = resource.configuration.get("env").and_then(|e| e.as_object()) {
                for (k, v) in env_obj {
                    env_map.insert(k.clone(), v.clone());
                }
            }
            if let Some(img) = resource.configuration.get("image").and_then(|i| i.as_str()) {
                image_name = img.to_string();
            }
            container_name = resource.name;
        }
    }

    let count = env_map.len();
    Ok(Json(serde_json::json!({
        "resource_id": id,
        "name": container_name,
        "image": image_name,
        "env": env_map,
        "count": count
    })))
}

/// Update Workload Environment Variables & Safely Recreate Container
async fn update_workload_env_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Path(id): Path<String>,
    Json(body): Json<UpdateWorkloadEnvRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Deploy scope required to update workload environment" })),
        ));
    }

    // 1. Inspect existing container from Docker
    let inspect_output = tokio::process::Command::new("docker")
        .args(["inspect", &id])
        .output()
        .await;

    match inspect_output {
        Ok(output) if output.status.success() => {
            let json_arr: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to parse container inspect: {}", e) })),
                )
            })?;

            let c = json_arr.as_array().and_then(|a| a.first()).ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Container not found in inspect output" })),
                )
            })?;

            let raw_name = c.get("Name").and_then(|n| n.as_str()).unwrap_or(&id);
            let clean_name = raw_name.trim_start_matches('/').to_string();

            let image = c
                .get("Config")
                .and_then(|cfg| cfg.get("Image"))
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string();

            if image.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Unable to determine container image tag" })),
                ));
            }

            let working_dir = c
                .get("Config")
                .and_then(|cfg| cfg.get("WorkingDir"))
                .and_then(|w| w.as_str())
                .unwrap_or("")
                .to_string();

            let restart_policy = c
                .get("HostConfig")
                .and_then(|h| h.get("RestartPolicy"))
                .and_then(|r| r.get("Name"))
                .and_then(|n| n.as_str())
                .unwrap_or("unless-stopped")
                .to_string();

            let mut binds = Vec::new();
            if let Some(arr) = c.get("HostConfig").and_then(|h| h.get("Binds")).and_then(|b| b.as_array()) {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        binds.push(s.to_string());
                    }
                }
            }

            let mut port_args = Vec::new();
            if let Some(ports_obj) = c.get("HostConfig").and_then(|h| h.get("PortBindings")).and_then(|p| p.as_object()) {
                for (container_port_proto, host_bindings) in ports_obj {
                    if let Some(bindings_arr) = host_bindings.as_array() {
                        for b in bindings_arr {
                            let host_port = b.get("HostPort").and_then(|hp| hp.as_str()).unwrap_or("");
                            let host_ip = b.get("HostIp").and_then(|ip| ip.as_str()).unwrap_or("");
                            if !host_port.is_empty() {
                                if !host_ip.is_empty() && host_ip != "0.0.0.0" {
                                    port_args.push(format!("{}:{}:{}", host_ip, host_port, container_port_proto));
                                } else {
                                    port_args.push(format!("{}:{}", host_port, container_port_proto));
                                }
                            }
                        }
                    }
                }
            }

            let mut networks = Vec::new();
            if let Some(nets_obj) = c.get("NetworkSettings").and_then(|n| n.get("Networks")).and_then(|net| net.as_object()) {
                for (net_name, _) in nets_obj {
                    if net_name != "none" && net_name != "host" {
                        networks.push(net_name.clone());
                    }
                }
            }

            let mut labels = Vec::new();
            if let Some(labels_obj) = c.get("Config").and_then(|cfg| cfg.get("Labels")).and_then(|l| l.as_object()) {
                for (k, v) in labels_obj {
                    if let Some(val_str) = v.as_str() {
                        if !k.starts_with("com.docker.compose.config-hash") {
                            labels.push((k.clone(), val_str.to_string()));
                        }
                    }
                }
            }

            let mut cmd_args = Vec::new();
            if let Some(arr) = c.get("Config").and_then(|cfg| cfg.get("Cmd")).and_then(|a| a.as_array()) {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        cmd_args.push(s.to_string());
                    }
                }
            }

            let mut entrypoint_str = None;
            if let Some(arr) = c.get("Config").and_then(|cfg| cfg.get("Entrypoint")).and_then(|a| a.as_array()) {
                if let Some(first) = arr.first().and_then(|s| s.as_str()) {
                    entrypoint_str = Some(first.to_string());
                }
            }

            // 2. Transactional replacement with rollback
            let backup_name = format!("{}_smoooth_backup_{}", clean_name, uuid::Uuid::new_v4().simple());

            // Stop container
            let _ = tokio::process::Command::new("docker")
                .args(["stop", "-t", "10", &clean_name])
                .output()
                .await;

            // Rename to backup
            let rename_res = tokio::process::Command::new("docker")
                .args(["rename", &clean_name, &backup_name])
                .output()
                .await;

            if let Err(e) = rename_res {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to rename container for backup",
                        "message": e.to_string()
                    })),
                ));
            }

            // Build new container docker run command
            let mut run_cmd = tokio::process::Command::new("docker");
            run_cmd.arg("run").arg("-d");
            run_cmd.args(["--name", &clean_name]);

            if !restart_policy.is_empty() && restart_policy != "no" {
                run_cmd.args(["--restart", &restart_policy]);
            }
            if !working_dir.is_empty() {
                run_cmd.args(["-w", &working_dir]);
            }
            if let Some(primary_net) = networks.first() {
                run_cmd.args(["--network", primary_net]);
            }
            for b in &binds {
                run_cmd.args(["-v", b]);
            }
            for p in &port_args {
                run_cmd.args(["-p", p]);
            }
            for (k, v) in &labels {
                run_cmd.args(["-l", &format!("{}={}", k, v)]);
            }
            if let Some(ep) = &entrypoint_str {
                run_cmd.args(["--entrypoint", ep]);
            }

            // Inject updated environment variables
            for (k, v) in &body.env {
                run_cmd.args(["-e", &format!("{}={}", k, v)]);
            }

            run_cmd.arg(&image);
            for arg in &cmd_args {
                run_cmd.arg(arg);
            }

            let run_output = run_cmd.output().await;

            match run_output {
                Ok(out) if out.status.success() => {
                    // Connect any secondary networks
                    for sec_net in networks.iter().skip(1) {
                        let _ = tokio::process::Command::new("docker")
                            .args(["network", "connect", sec_net, &clean_name])
                            .output()
                            .await;
                    }

                    // Safely remove the backup container
                    let _ = tokio::process::Command::new("docker")
                        .args(["rm", "-f", &backup_name])
                        .output()
                        .await;

                    // Update orchestrator state if registered
                    if let Ok(Some(mut resource)) = state.orchestrator.storage().get_resource(&id).await {
                        resource.configuration["env"] = serde_json::to_value(&body.env).unwrap_or_default();
                        resource.updated_at = chrono::Utc::now();
                        let _ = state.orchestrator.storage().save_resource(resource).await;
                    }

                    Ok(Json(serde_json::json!({
                        "success": true,
                        "resource_id": id,
                        "container_name": clean_name,
                        "env_count": body.env.len(),
                        "status": "recreated_and_started"
                    })))
                }
                Ok(out) => {
                    let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                    tracing::error!("Failed to start replacement container for '{}': {}. Rolling back...", clean_name, err_msg);

                    // Rollback
                    let _ = tokio::process::Command::new("docker").args(["rm", "-f", &clean_name]).output().await;
                    let _ = tokio::process::Command::new("docker").args(["rename", &backup_name, &clean_name]).output().await;
                    let _ = tokio::process::Command::new("docker").args(["start", &clean_name]).output().await;

                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to start container with new environment variables; rolled back to previous state",
                            "details": err_msg
                        })),
                    ))
                }
                Err(e) => {
                    tracing::error!("Execution error running replacement container for '{}': {}. Rolling back...", clean_name, e);

                    // Rollback
                    let _ = tokio::process::Command::new("docker").args(["rm", "-f", &clean_name]).output().await;
                    let _ = tokio::process::Command::new("docker").args(["rename", &backup_name, &clean_name]).output().await;
                    let _ = tokio::process::Command::new("docker").args(["start", &clean_name]).output().await;

                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to execute docker run; rolled back to previous state",
                            "details": e.to_string()
                        })),
                    ))
                }
            }
        }
        _ => {
            // Fallback for storage-only / non-Docker test environment
            if let Ok(Some(mut resource)) = state.orchestrator.storage().get_resource(&id).await {
                let resource_name = resource.name.clone();
                resource.configuration["env"] = serde_json::to_value(&body.env).unwrap_or_default();
                resource.updated_at = chrono::Utc::now();
                let _ = state.orchestrator.storage().save_resource(resource).await;

                Ok(Json(serde_json::json!({
                    "success": true,
                    "resource_id": id,
                    "container_name": resource_name,
                    "env_count": body.env.len(),
                    "status": "updated_in_storage"
                })))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Workload container not found in Docker or storage state"
                    })),
                ))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct MigrateDetectRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct MigratePreviewRequest {
    pub content: String,
    pub format: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Deserialize)]
pub struct MigrateApplyRequest {
    pub preview: crate::migration::MigrationPreview,
    pub approver: Option<String>,
}

/// Detect format of foreign manifest
async fn migrate_detect_handler(
    Json(body): Json<MigrateDetectRequest>,
) -> impl IntoResponse {
    let source = crate::migration::MigrationEngine::detect_format(&body.content);
    Json(serde_json::json!({
        "format": source,
        "format_display": format!("{}", source)
    }))
}

/// Generate migration preview and canonical plan
async fn migrate_preview_handler(
    Json(body): Json<MigratePreviewRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ws = body.workspace.unwrap_or_else(|| "default".to_string());
    let forced = body.format.as_deref().and_then(|f| match f.to_lowercase().as_str() {
        "compose" | "docker-compose" => Some(crate::migration::MigrationSource::DockerCompose),
        "coolify" | "coolify-json" => Some(crate::migration::MigrationSource::CoolifyJson),
        "render" => Some(crate::migration::MigrationSource::RenderBlueprint),
        "fly" | "fly.toml" => Some(crate::migration::MigrationSource::FlyToml),
        "dokku" | "procfile" => Some(crate::migration::MigrationSource::DokkuApp),
        "caprover" => Some(crate::migration::MigrationSource::CapRover),
        _ => None,
    });

    match crate::migration::MigrationEngine::preview(&body.content, forced, &ws) {
        Ok(preview) => Ok(Json(preview)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Apply migration plan in 1-click
async fn migrate_apply_handler(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    Json(body): Json<MigrateApplyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !caller.is_operator
        && !caller
            .identity
            .as_ref()
            .is_some_and(|i| i.has_scope(&AgentScope::Deploy))
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Deploy scope required to execute migration" })),
        ));
    }

    let approver = body.approver.unwrap_or_else(|| {
        if let Some(id) = &caller.identity {
            format!("agent:{}", id.name)
        } else {
            "operator_migration_hub".to_string()
        }
    });

    match crate::migration::MigrationEngine::apply(&body.preview, &state.orchestrator, &approver).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Live scan local host Docker daemon and /data/coolify
async fn migrate_scan_host_handler(
    Query(query): Query<StatusQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ws = query.workspace.unwrap_or_else(|| "default".to_string());
    match crate::migration::MigrationEngine::scan_host(&ws).await {
        Ok(preview) => Ok(Json(preview)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// Resolve the authentication token using environment, CLI arguments, or configuration files
pub fn resolve_auth_token(
    cli_token: Option<String>,
    cli_token_file: Option<String>,
) -> anyhow::Result<String> {
    // 1. Explicit CLI token parameter
    if let Some(tok) = cli_token {
        if !tok.trim().is_empty() {
            return Ok(tok.trim().to_string());
        }
    }

    // 2. Environment variable SMOOOTH_AUTH_TOKEN
    if let Ok(tok) = std::env::var("SMOOOTH_AUTH_TOKEN") {
        if !tok.trim().is_empty() {
            return Ok(tok.trim().to_string());
        }
    }

    // 3. Explicit CLI token file parameter
    if let Some(path) = cli_token_file {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let tok = content.trim();
            if !tok.is_empty() {
                return Ok(tok.to_string());
            }
        }
    }

    // 4. Environment variable SMOOOTH_TOKEN_FILE
    if let Ok(path) = std::env::var("SMOOOTH_TOKEN_FILE") {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let tok = content.trim();
            if !tok.is_empty() {
                return Ok(tok.to_string());
            }
        }
    }

    // 5. System configuration file /etc/smoooth/auth.token (Linux VPS target)
    if let Ok(content) = std::fs::read_to_string("/etc/smoooth/auth.token") {
        let tok = content.trim();
        if !tok.is_empty() {
            return Ok(tok.to_string());
        }
    }

    // 6. Local development fallback file smoooth.token
    if let Ok(content) = std::fs::read_to_string("smoooth.token") {
        let tok = content.trim();
        if !tok.is_empty() {
            return Ok(tok.to_string());
        }
    }

    // 7. Auto-generate secure 256-bit token
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| anyhow::anyhow!("Failed to generate secure random token: {}", e))?;
    let token = hex::encode(bytes);

    let _ = std::fs::write("smoooth.token", &token);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions("smoooth.token", std::fs::Permissions::from_mode(0o600));
    }

    eprintln!("[INFO] Generated new authentication token and saved to smoooth.token");
    Ok(token)
}

/// Start the Smoooth HTTP API Server & MCP Gateway
pub async fn start_server(
    orchestrator: Arc<Orchestrator>,
    bind: &str,
    port: u16,
    cli_token: Option<String>,
    cli_token_file: Option<String>,
) -> anyhow::Result<()> {
    let auth_token = resolve_auth_token(cli_token, cli_token_file)?;
    let (app, _) = build_app(orchestrator, auth_token.clone());

    let ip: std::net::IpAddr = bind.parse()?;
    let addr = SocketAddr::from((ip, port));

    println!("============================================================");
    println!("  Smoooth Infrastructure Engine & Control Plane Dashboard");
    println!("============================================================");
    println!("  • Control Dashboard : http://{}/", addr);
    println!("  • Public Health     : http://{}/health", addr);
    println!("  • MCP HTTP Post     : http://{}/mcp", addr);
    println!("  • MCP SSE Stream    : http://{}/sse", addr);
    println!(
        "  • Master Token      : {}...",
        &auth_token[..6.min(auth_token.len())]
    );
    println!("============================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
