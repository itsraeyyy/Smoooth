pub mod compose;
pub mod coolify;
pub mod docker_scanner;
pub mod foreign;
pub mod types;

use std::sync::Arc;
use uuid::Uuid;

use crate::engine::Orchestrator;
use crate::error::SmooothError;
use crate::ingress::IngressManager;
use crate::models::{Plan, RiskTier};

pub use compose::ComposeParser;
pub use coolify::CoolifyParser;
pub use docker_scanner::DockerHostScanner;
pub use foreign::ForeignParser;
pub use types::{
    DetectedWorkload, IngressRoutePreview, MigrationPreview, MigrationResult, MigrationSource,
    PortMapping, VolumeMapping,
};

pub struct MigrationEngine;

impl MigrationEngine {
    /// Auto-detect the format of incoming manifest or configuration text
    pub fn detect_format(content: &str) -> MigrationSource {
        let trimmed = content.trim();

        // 1. JSON-based detection
        if trimmed.starts_with('{') {
            if trimmed.contains("\"docker_compose_raw\"")
                || trimmed.contains("\"applications\"")
                || trimmed.contains("\"coolify\"")
                || (trimmed.contains("\"databases\"") && trimmed.contains("\"postgres_user\""))
            {
                return MigrationSource::CoolifyJson;
            }
            if trimmed.contains("\"imageName\"") || trimmed.contains("\"schemaVersion\"") {
                return MigrationSource::CapRover;
            }
        }

        // 2. TOML-based detection (Fly.io)
        if trimmed.contains("[http_service]")
            || trimmed.contains("[[services]]")
            || (trimmed.contains("app =") && trimmed.contains("[build]"))
        {
            return MigrationSource::FlyToml;
        }

        // 3. Render Blueprint YAML
        if trimmed.contains("fromDatabase:")
            || (trimmed.contains("services:") && trimmed.contains("type: web") && trimmed.contains("buildCommand:"))
        {
            return MigrationSource::RenderBlueprint;
        }

        // 4. Docker Compose YAML
        if trimmed.contains("services:")
            && (trimmed.contains("image:") || trimmed.contains("build:") || trimmed.contains("ports:"))
        {
            return MigrationSource::DockerCompose;
        }

        // 5. Dokku / Heroku Procfile
        let first_lines: Vec<&str> = trimmed.lines().take(5).collect();
        for line in first_lines {
            let l = line.trim();
            if l.starts_with("web:") || l.starts_with("worker:") || l.starts_with("api:") {
                return MigrationSource::DokkuApp;
            }
        }

        MigrationSource::Unknown
    }

    /// Parse manifest content and generate a full migration preview without modifying state
    pub fn preview(
        content: &str,
        forced_source: Option<MigrationSource>,
        workspace: &str,
    ) -> Result<MigrationPreview, SmooothError> {
        let source = forced_source.unwrap_or_else(|| Self::detect_format(content));

        let (workloads, canonical_spec, ingress_routes) = match &source {
            MigrationSource::DockerCompose => ComposeParser::parse(content, workspace)?,
            MigrationSource::CoolifyJson => CoolifyParser::parse_json(content, workspace)?,
            MigrationSource::RenderBlueprint => ForeignParser::parse_render_blueprint(content, workspace)?,
            MigrationSource::FlyToml => ForeignParser::parse_fly_toml(content, workspace)?,
            MigrationSource::DokkuApp => ForeignParser::parse_dokku_app(content, None, None, workspace)?,
            MigrationSource::CapRover => ForeignParser::parse_caprover(content, None, workspace)?,
            MigrationSource::CoolifyLocal => CoolifyParser::scan_local_data_dir("/data/coolify", workspace)?,
            MigrationSource::Unknown | MigrationSource::AutoDetected(_) => {
                // Try Docker Compose first, then Coolify JSON
                if let Ok(res) = ComposeParser::parse(content, workspace) {
                    res
                } else if let Ok(res) = CoolifyParser::parse_json(content, workspace) {
                    res
                } else if let Ok(res) = ForeignParser::parse_render_blueprint(content, workspace) {
                    res
                } else if let Ok(res) = ForeignParser::parse_fly_toml(content, workspace) {
                    res
                } else {
                    return Err(SmooothError::Validation(
                        "Unable to auto-detect configuration format. Supported: Docker Compose, Coolify JSON, Render YAML, Fly TOML, Dokku Procfile.".to_string(),
                    ));
                }
            }
            MigrationSource::DockerHostLive => {
                return Err(SmooothError::Validation(
                    "Use MigrationEngine::scan_host() for live Docker host inspection.".to_string(),
                ));
            }
        };

        Self::build_preview(source, workspace, workloads, canonical_spec, ingress_routes)
    }

    /// Inspect local Docker host VPS for running containers and Coolify stacks
    pub async fn scan_host(workspace: &str) -> Result<MigrationPreview, SmooothError> {
        // Check if /data/coolify exists on VPS
        let local_coolify_exists = std::path::Path::new("/data/coolify").exists();

        let (workloads, canonical_spec, ingress_routes) = if local_coolify_exists {
            if let Ok(res) = CoolifyParser::scan_local_data_dir("/data/coolify", workspace) {
                res
            } else {
                DockerHostScanner::scan(workspace).await?
            }
        } else {
            DockerHostScanner::scan(workspace).await?
        };

        let src = if local_coolify_exists {
            MigrationSource::CoolifyLocal
        } else {
            MigrationSource::DockerHostLive
        };

        Self::build_preview(src, workspace, workloads, canonical_spec, ingress_routes)
    }

    /// Build a structured MigrationPreview from parsed elements
    fn build_preview(
        source_format: MigrationSource,
        workspace: &str,
        workloads: Vec<DetectedWorkload>,
        canonical_spec: crate::models::CanonicalExecutionSpec,
        ingress_routes: Vec<IngressRoutePreview>,
    ) -> Result<MigrationPreview, SmooothError> {
        // Calculate estimated cost
        let mut estimated_monthly_cost = 0.0;
        let mut has_persistent = false;

        for w in &workloads {
            match w.service_type.as_str() {
                "database" => estimated_monthly_cost += 12.0,
                "cache" => estimated_monthly_cost += 5.0,
                "worker" => estimated_monthly_cost += 5.0,
                _ => estimated_monthly_cost += 8.0,
            }
            if w.is_persistent {
                has_persistent = true;
            }
        }

        let risk_tier = if has_persistent {
            RiskTier::Moderate
        } else {
            RiskTier::Low
        };

        let summary = format!(
            "Discovered {} workload(s) from {}. {} persistent storage volume(s), {} ingress domain(s) mapped.",
            workloads.len(),
            source_format,
            workloads.iter().filter(|w| w.is_persistent).count(),
            ingress_routes.len()
        );

        let dependencies = canonical_spec.dependencies.clone();

        Ok(MigrationPreview {
            source_format,
            workspace_id: workspace.to_string(),
            workloads,
            dependencies,
            ingress_routes,
            estimated_monthly_cost_usd: estimated_monthly_cost,
            risk_tier,
            summary,
            canonical_spec,
        })
    }

    /// Apply and provision the migration plan in 1 step
    pub async fn apply(
        preview: &MigrationPreview,
        orchestrator: &Arc<Orchestrator>,
        approver: &str,
    ) -> Result<MigrationResult, SmooothError> {
        let plan_id = format!("migrate_{}_{}", preview.workspace_id, Uuid::new_v4().simple());
        let mut plan = Plan::new(
            plan_id.clone(),
            preview.canonical_spec.clone(),
            preview.risk_tier.clone(),
            Some(preview.estimated_monthly_cost_usd),
        );

        // Pre-approve the migration plan on behalf of the operator
        plan.approve_with_ttl(approver.to_string(), 3600);

        orchestrator.save_plan(plan.clone()).await?;

        let idemp_key = format!("migrate_exec_{}", Uuid::new_v4().simple());
        let op = orchestrator.execute_plan_sync(plan.clone(), idemp_key).await?;

        // Register any discovered ingress routes
        let ingress_mgr = IngressManager::default_local();
        let mut routes_added = Vec::new();
        for r in &preview.ingress_routes {
            if let Ok(route) = ingress_mgr.add_route(
                &r.domain,
                &r.target_resource,
                r.target_port,
                r.tls,
                r.path_prefix.clone(),
            ) {
                routes_added.push(format!("{} -> {} ({})", route.domain, route.target_resource, route.target_port));
            }
        }

        let provisioned: Vec<String> = preview
            .workloads
            .iter()
            .map(|w| w.name.clone())
            .collect();

        Ok(MigrationResult {
            success: true,
            plan_id,
            operation_id: Some(op.id),
            provisioned_resources: provisioned,
            ingress_routes_added: routes_added,
            message: format!(
                "Successfully migrated {} workload(s) into Smoooth workspace '{}'.",
                preview.workloads.len(),
                preview.workspace_id
            ),
        })
    }
}
