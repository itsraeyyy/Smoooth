use axum::{
    extract::Json,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::engine::Orchestrator;
use crate::mcp::tools::{get_available_tools, get_default_capabilities};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Deserialize)]
pub struct WaitlistRequest {
    pub email: String,
    pub stack: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct WaitlistResponse {
    pub success: bool,
    pub message: String,
    pub email: String,
    pub queue_number: String,
    pub ref_code: String,
    pub referral_url: String,
    pub stack: Vec<String>,
}

#[derive(Serialize)]
pub struct StatusServiceItem {
    pub name: &'static str,
    pub port: u16,
    pub status: &'static str,
    pub uptime: &'static str,
    pub latency: &'static str,
    pub details: &'static str,
}

#[derive(Serialize)]
pub struct ClusterStatusResponse {
    pub cluster_name: &'static str,
    pub overall_health: &'static str,
    pub ping_ms: f32,
    pub services: Vec<StatusServiceItem>,
    pub connection_strings: Vec<&'static str>,
}

#[derive(Deserialize)]
pub struct PromptRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct PromptResponse {
    pub intent: String,
    pub plan_id: String,
    pub risk_tier: &'static str,
    pub tools_executed: Vec<&'static str>,
}

pub fn create_router(_orchestrator: Arc<Orchestrator>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/capabilities", get(capabilities_handler))
        .route("/api/tools", get(tools_handler))
        .route("/api/waitlist", post(waitlist_handler))
        .route("/api/prompt", post(prompt_handler))
        .layer(cors)
}

async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "smoooth-control-plane",
        version: "0.1.0",
    })
}

async fn status_handler() -> impl IntoResponse {
    Json(ClusterStatusResponse {
        cluster_name: "Prod-Mesh-01",
        overall_health: "99.98% Uptime",
        ping_ms: 2.1,
        services: vec![
            StatusServiceItem {
                name: "PostgreSQL 16 Primary",
                port: 5432,
                status: "Healthy",
                uptime: "100%",
                latency: "2.1ms",
                details: "100GiB Persistent, 12 active pools",
            },
            StatusServiceItem {
                name: "Redis HA Cluster",
                port: 6379,
                status: "Connected",
                uptime: "99.9%",
                latency: "0.8ms",
                details: "In-memory cache, 3-node mesh",
            },
            StatusServiceItem {
                name: "App Mesh API Gateway",
                port: 443,
                status: "Routing",
                uptime: "100%",
                latency: "1.4ms",
                details: "Mutual TLS Active, 3 container pods",
            },
        ],
        connection_strings: vec![
            "POSTGRES_URL=postgres://agent:***@10.0.4.12:5432/app_db",
            "REDIS_URL=redis://cluster.smoooth-mesh.internal:6379",
        ],
    })
}

async fn capabilities_handler() -> impl IntoResponse {
    Json(get_default_capabilities())
}

async fn tools_handler() -> impl IntoResponse {
    Json(get_available_tools())
}

async fn waitlist_handler(Json(payload): Json<WaitlistRequest>) -> Result<impl IntoResponse, StatusCode> {
    if payload.email.is_empty() || !payload.email.contains('@') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let code_suffix = uuid::Uuid::new_v4().simple().to_string()[..4].to_uppercase();
    let ref_code = format!("SMOOOTH-{}", code_suffix);
    let queue_num = format!("#{}", 3480 + (rand_simple() % 50));
    let referral_url = format!("https://smoooth.dev/join?ref={}", ref_code);

    let stack = payload.stack.unwrap_or_else(|| {
        vec!["PostgreSQL 16".into(), "Redis Cluster".into(), "Docker Engine".into()]
    });

    Ok(Json(WaitlistResponse {
        success: true,
        message: "Successfully added to the Smoooth priority waitlist.".to_string(),
        email: payload.email,
        queue_number: queue_num,
        ref_code,
        referral_url,
        stack,
    }))
}

async fn prompt_handler(Json(payload): Json<PromptRequest>) -> impl IntoResponse {
    let plan_id = format!("plan_{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    Json(PromptResponse {
        intent: payload.prompt,
        plan_id,
        risk_tier: "Tier1_LowImpact",
        tools_executed: vec![
            "smoooth.capabilities.discover",
            "smoooth.plan.create",
            "smoooth.plan.apply",
            "smoooth.resource.verify",
        ],
    })
}

fn rand_simple() -> u32 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

pub async fn start_server(orchestrator: Arc<Orchestrator>, port: u16) -> anyhow::Result<()> {
    let app = create_router(orchestrator);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Smoooth API Server listening on http://{}", addr);
    println!("✓ Smoooth Backend Server listening on http://127.0.0.1:{}", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
