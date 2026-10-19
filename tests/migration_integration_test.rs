use smoooth::engine::Orchestrator;
use smoooth::migration::{
    ComposeParser, CoolifyParser, ForeignParser, MigrationEngine, MigrationSource,
};
use smoooth::storage::LocalStorage;
use std::sync::Arc;

#[test]
fn test_docker_compose_migration_parsing() {
    let compose_yaml = r#"
version: "3.8"
services:
  web:
    image: ghcr.io/myorg/web-service:v1.2.3
    ports:
      - "8080:3000"
    environment:
      NODE_ENV: production
      PORT: 3000
      DATABASE_URL: postgresql://postgres:secret@postgres-db:5432/appdb
    labels:
      - "traefik.http.routers.web.rule=Host(`api.example.com`)"
      - "coolify.fqdn=https://api.example.com"
    depends_on:
      - postgres-db
      - redis-cache

  postgres-db:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: appdb
    volumes:
      - pg_data:/var/lib/postgresql/data

  redis-cache:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
"#;

    let (workloads, spec, ingress) = ComposeParser::parse(compose_yaml, "default").unwrap();

    assert_eq!(workloads.len(), 3);
    assert_eq!(spec.actions.len(), 3);

    // Verify dependencies: web depends on postgres-db and redis-cache
    assert_eq!(spec.dependencies.len(), 2);
    let deps_to: Vec<String> = spec.dependencies.iter().map(|d| d.to.clone()).collect();
    assert!(deps_to.contains(&"postgres-db".to_string()));
    assert!(deps_to.contains(&"redis-cache".to_string()));

    // Verify database classification and persistent safeguarding
    let pg = workloads.iter().find(|w| w.name == "postgres-db").unwrap();
    assert_eq!(pg.provider, "postgres");
    assert_eq!(pg.service_type, "database");
    assert!(pg.is_persistent);

    let redis = workloads.iter().find(|w| w.name == "redis-cache").unwrap();
    assert_eq!(redis.provider, "redis");
    assert_eq!(redis.service_type, "cache");
    assert!(redis.is_persistent);

    // Verify web compute and domain ingress extraction
    let web = workloads.iter().find(|w| w.name == "web").unwrap();
    assert_eq!(web.service_type, "compute");
    assert!(!web.domains.is_empty());
    assert_eq!(web.domains[0], "api.example.com");

    assert_eq!(ingress.len(), 1);
    assert_eq!(ingress[0].domain, "api.example.com");
    assert_eq!(ingress[0].target_resource, "web");
}

#[test]
fn test_coolify_json_migration_parsing() {
    let coolify_json = r#"{
      "name": "Production Stack",
      "applications": [
        {
          "name": "shop-frontend",
          "fqdn": "https://shop.coolify.local",
          "docker_image": "node:18-alpine",
          "ports_exposes": "3000",
          "environment_variables": [
            { "key": "PORT", "value": "3000" },
            { "key": "API_URL", "value": "https://api.coolify.local" }
          ]
        }
      ],
      "databases": [
        {
          "name": "shop-database",
          "type": "standalone-postgresql",
          "postgres_user": "pguser",
          "postgres_password": "supersecretpassword",
          "postgres_db": "shopdb"
        }
      ]
    }"#;

    let (workloads, spec, ingress) = CoolifyParser::parse_json(coolify_json, "production").unwrap();

    assert_eq!(workloads.len(), 2);
    assert_eq!(spec.actions.len(), 2);

    let frontend = workloads.iter().find(|w| w.name == "shop-frontend").unwrap();
    assert_eq!(frontend.service_type, "compute");
    assert_eq!(frontend.domains[0], "shop.coolify.local");

    let db = workloads.iter().find(|w| w.name == "shop-database").unwrap();
    assert_eq!(db.service_type, "database");
    assert_eq!(db.provider, "postgres");
    assert!(db.is_persistent);

    assert_eq!(ingress.len(), 1);
    assert_eq!(ingress[0].domain, "shop.coolify.local");
}

#[test]
fn test_render_blueprint_migration() {
    let render_yaml = r#"
services:
  - type: web
    name: render-api-server
    env: node
    buildCommand: npm install
    startCommand: npm start
    envVars:
      - key: NODE_ENV
        value: production
      - key: PORT
        value: 8080
      - key: DATABASE_URL
        fromDatabase:
          name: render-postgres
          property: connectionString

databases:
  - name: render-postgres
    databaseName: my_production_db
    user: db_admin
"#;

    let (workloads, spec, _) = ForeignParser::parse_render_blueprint(render_yaml, "default").unwrap();

    assert_eq!(workloads.len(), 2);
    assert_eq!(spec.actions.len(), 2);
    assert_eq!(spec.dependencies.len(), 1);
    assert_eq!(spec.dependencies[0].from, "render-api-server");
    assert_eq!(spec.dependencies[0].to, "render-postgres");
}

#[test]
fn test_fly_toml_migration() {
    let fly_toml = r#"
app = "my-fly-api"
primary_region = "iad"

[build]
  image = "myorg/api:v3"

[env]
  PORT = "8080"
  LOG_LEVEL = "debug"

[http_service]
  internal_port = 8080
  force_https = true
"#;

    let (workloads, spec, ingress) = ForeignParser::parse_fly_toml(fly_toml, "default").unwrap();

    assert_eq!(workloads.len(), 1);
    assert_eq!(spec.actions.len(), 1);
    assert_eq!(workloads[0].name, "my-fly-api");
    assert_eq!(workloads[0].image, "myorg/api:v3");
    assert_eq!(ingress.len(), 1);
    assert_eq!(ingress[0].domain, "my-fly-api.fly.dev");
}

#[test]
fn test_dokku_procfile_migration() {
    let procfile = r#"
web: node server.js
worker: python worker.py
"#;
    let env_content = "PORT=3000\nENV=production\n";

    let (workloads, spec, _) =
        ForeignParser::parse_dokku_app(procfile, Some(env_content), Some("sample-dokku"), "default")
            .unwrap();

    assert_eq!(workloads.len(), 2);
    assert_eq!(spec.actions.len(), 2);
    assert!(workloads.iter().any(|w| w.name == "sample-dokku" && w.service_type == "compute"));
    assert!(workloads.iter().any(|w| w.name == "sample-dokku-worker" && w.service_type == "worker"));
}

#[test]
fn test_migration_format_autodetection() {
    let compose_str = "version: '3'\nservices:\n  web:\n    image: nginx\n";
    assert_eq!(
        MigrationEngine::detect_format(compose_str),
        MigrationSource::DockerCompose
    );

    let coolify_str = "{\n  \"name\": \"app\",\n  \"applications\": []\n}";
    assert_eq!(
        MigrationEngine::detect_format(coolify_str),
        MigrationSource::CoolifyJson
    );

    let fly_str = "app = 'my-app'\n[http_service]\ninternal_port = 8080\n";
    assert_eq!(
        MigrationEngine::detect_format(fly_str),
        MigrationSource::FlyToml
    );

    let render_str = "services:\n  - type: web\n    buildCommand: npm install\n";
    assert_eq!(
        MigrationEngine::detect_format(render_str),
        MigrationSource::RenderBlueprint
    );
}

struct MockMigrationAdapter {
    provider: String,
}

#[async_trait::async_trait]
impl smoooth::adapters::ProviderAdapter for MockMigrationAdapter {
    fn provider_name(&self) -> &str {
        &self.provider
    }

    fn guarantees(&self) -> smoooth::adapters::AdapterGuarantees {
        smoooth::adapters::AdapterGuarantees {
            provider_name: self.provider.clone(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<smoooth::models::Capability> {
        vec![]
    }

    async fn provision(
        &self,
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
    ) -> smoooth::error::Result<smoooth::models::Resource> {
        Ok(smoooth::models::Resource {
            id: name.to_string(),
            name: name.to_string(),
            workspace_id: config.get("workspace_id").and_then(|w| w.as_str()).unwrap_or("default").to_string(),
            provider: self.provider.clone(),
            service_type: if self.provider == "postgres" { "database".to_string() } else { "compute".to_string() },
            status: smoooth::models::ResourceStatus::Active,
            ownership: smoooth::models::ResourceOwnership::SmooothManaged,
            is_persistent,
            configuration: config.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn inspect(&self, _resource_id: &str) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Active)
    }

    async fn health_check(&self, _resource_id: &str) -> smoooth::error::Result<smoooth::adapters::HealthEvidence> {
        Ok(smoooth::adapters::HealthEvidence {
            is_healthy: true,
            latency_ms: Some(2),
            message: "Mock container healthy".to_string(),
            details: serde_json::json!({}),
        })
    }

    async fn reconcile(
        &self,
        _resource_id: &str,
        _intended_config: &serde_json::Value,
    ) -> smoooth::error::Result<smoooth::models::ResourceStatus> {
        Ok(smoooth::models::ResourceStatus::Active)
    }

    async fn destroy(&self, _resource_id: &str, _preserve_persistent_data: bool) -> smoooth::error::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_migration_engine_end_to_end_preview_and_apply() {
    let compose_yaml = r#"
version: "3"
services:
  migrated-web:
    image: nginx:alpine
    ports:
      - "8080:80"
  migrated-db:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
"#;

    let storage = Arc::new(LocalStorage::new_in_memory());
    let mut orchestrator = Orchestrator::new(storage.clone());
    orchestrator.register_adapter(Arc::new(MockMigrationAdapter { provider: "docker".to_string() }));
    orchestrator.register_adapter(Arc::new(MockMigrationAdapter { provider: "postgres".to_string() }));
    let orchestrator_arc = Arc::new(orchestrator);

    let preview = MigrationEngine::preview(
        compose_yaml,
        Some(MigrationSource::DockerCompose),
        "test_workspace",
    )
    .unwrap();

    assert_eq!(preview.workloads.len(), 2);
    assert_eq!(preview.workspace_id, "test_workspace");
    assert_eq!(preview.risk_tier, smoooth::models::RiskTier::Moderate);

    let result = MigrationEngine::apply(&preview, &orchestrator_arc, "operator_test")
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.provisioned_resources.len(), 2);
    assert!(result.provisioned_resources.contains(&"migrated-web".to_string()));
    assert!(result.provisioned_resources.contains(&"migrated-db".to_string()));

    // Verify storage has recorded both resources
    let resources = orchestrator_arc.list_resources("test_workspace").await;
    assert_eq!(resources.len(), 2);
}
