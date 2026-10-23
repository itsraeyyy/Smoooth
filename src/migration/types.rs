use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{CanonicalExecutionSpec, DependencyEdge, RiskTier};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationSource {
    DockerCompose,
    CoolifyJson,
    CoolifyLocal,
    RenderBlueprint,
    FlyToml,
    DokkuApp,
    CapRover,
    DockerHostLive,
    AutoDetected(String),
    Unknown,
}

impl std::fmt::Display for MigrationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DockerCompose => write!(f, "Docker Compose (v2/v3 / compose.yaml)"),
            Self::CoolifyJson => write!(f, "Coolify API / JSON Backup"),
            Self::CoolifyLocal => write!(f, "Coolify Local VPS (/data/coolify)"),
            Self::RenderBlueprint => write!(f, "Render Blueprint (render.yaml)"),
            Self::FlyToml => write!(f, "Fly.io Configuration (fly.toml)"),
            Self::DokkuApp => write!(f, "Dokku Application (Procfile / app.json)"),
            Self::CapRover => write!(f, "CapRover Definition (captain-definition)"),
            Self::DockerHostLive => write!(f, "Live Docker Host Inspection"),
            Self::AutoDetected(s) => write!(f, "Auto-detected ({})", s),
            Self::Unknown => write!(f, "Unknown / Raw Format"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: String, // "tcp" | "udp"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub source: String,
    pub target: String,
    pub is_named_volume: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRoutePreview {
    pub domain: String,
    pub target_resource: String,
    pub target_port: u16,
    pub tls: bool,
    pub path_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedWorkload {
    pub name: String,
    pub service_type: String, // "compute" | "database" | "cache" | "worker"
    pub provider: String,     // "docker" | "postgres" | "redis" | "mongo"
    pub image: String,
    pub build_source: Option<String>,
    pub ports: Vec<PortMapping>,
    pub env_vars: HashMap<String, String>,
    pub volumes: Vec<VolumeMapping>,
    pub domains: Vec<String>,
    pub is_persistent: bool,
    pub depends_on: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPreview {
    pub source_format: MigrationSource,
    pub workspace_id: String,
    pub workloads: Vec<DetectedWorkload>,
    pub dependencies: Vec<DependencyEdge>,
    pub ingress_routes: Vec<IngressRoutePreview>,
    pub estimated_monthly_cost_usd: f64,
    pub risk_tier: RiskTier,
    pub summary: String,
    pub canonical_spec: CanonicalExecutionSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub plan_id: String,
    pub operation_id: Option<String>,
    pub provisioned_resources: Vec<String>,
    pub ingress_routes_added: Vec<String>,
    #[serde(default)]
    pub bridged_proxy_networks: Vec<String>,
    pub message: String,
}
