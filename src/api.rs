use axum::{
    extract::{Json, Path, Query, Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
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
use crate::mcp::server::JsonRpcRequest;
use crate::mcp::tools::{get_available_tools, get_default_capabilities};
use crate::mcp::McpServer;

pub type SseSender = mpsc::Sender<Result<Event, Infallible>>;

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
        // Strict default: no wildcard Access-Control-Allow-Origin
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    };

    let router = Router::new()
        // Unauthenticated health probes
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
        .route("/api/operations/:id", get(get_operation_handler))
        .route("/api/reconcile/:resource_id", post(reconcile_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(cors)
        .with_state(state.clone());

    (router, state)
}

/// Backwards-compatible router factory
pub fn create_router(orchestrator: Arc<Orchestrator>, auth_token: String) -> Router {
    let (router, _) = build_app(orchestrator, auth_token);
    router
}

/// Authentication middleware enforcing Bearer token validation
async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let path = req.uri().path();
    // Allow public unauthenticated health checks
    if path == "/health" || path == "/api/health" {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token_valid = if let Some(auth) = auth_header {
        if let Some(bearer) = auth.strip_prefix("Bearer ") {
            bearer.trim() == state.auth_token
        } else {
            false
        }
    } else {
        false
    };

    if !token_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Valid Bearer authentication token required. Configure 'Authorization: Bearer <token>' header."
            })),
        ));
    }

    Ok(next.run(req).await)
}

/// Public Health Probe
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "smoooth-control-plane",
        version: "0.1.0",
    })
}

/// Direct JSON-RPC 2.0 over HTTP MCP Handler (used by external AI coding agents)
async fn mcp_post_handler(
    State(state): State<AppState>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let resp = state.mcp_server.handle_request(req).await;
    match resp {
        Some(r) => (StatusCode::OK, Json(serde_json::to_value(r).unwrap_or_default())),
        None => (StatusCode::NO_CONTENT, Json(serde_json::Value::Null)),
    }
}

/// MCP Server-Sent Events (SSE) Stream Endpoint (`GET /sse`)
async fn sse_connect_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    // Initial endpoint announcement event as per MCP specification
    let endpoint_url = format!("/sse/message?sessionId={}", session_id);
    let init_event = Event::default().event("endpoint").data(endpoint_url);
    let _ = tx.send(Ok(init_event)).await;

    // Register active SSE session
    {
        let mut sessions = state.sse_sessions.write().await;
        sessions.insert(session_id.clone(), tx);
    }

    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(ev) => Some((ev, rx)),
            None => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// MCP SSE Message Receiver (`POST /sse/message?sessionId=...`)
async fn sse_message_handler(
    State(state): State<AppState>,
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

    let resp = state.mcp_server.handle_request(req).await;
    if let Some(r) = resp {
        let resp_json = serde_json::to_string(&r).unwrap_or_default();
        let sse_event = Event::default().event("message").data(resp_json);
        if let Err(_) = tx.send(Ok(sse_event)).await {
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

/// Approve Plan Handler
async fn approve_plan_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ApprovePlanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    let approver = body.approver.unwrap_or_else(|| "remote_operator".to_string());
    let ttl = body.ttl_seconds.unwrap_or(3600);
    plan.approve_with_ttl(approver.clone(), ttl);

    state.orchestrator.save_plan(plan.clone()).await.map_err(|e| {
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
    Path(resource_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state.orchestrator.reconcile_resource(&resource_id).await {
        Ok(outcome) => Ok(Json(outcome)),
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

    // Save to local file
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
    println!("  Smoooth Infrastructure Engine & Authenticated MCP Gateway");
    println!("============================================================");
    println!("  • Bind Address      : http://{}", addr);
    println!("  • Public Health     : http://{}/health", addr);
    println!("  • MCP HTTP Post     : http://{}/mcp", addr);
    println!("  • MCP SSE Stream    : http://{}/sse", addr);
    println!("  • Auth Header       : Authorization: Bearer {}...", &auth_token[..6.min(auth_token.len())]);
    println!("============================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
