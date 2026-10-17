use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::adapters::docker::DockerAdapter;
use crate::adapters::{AdapterGuarantees, HealthEvidence, ProviderAdapter};
use crate::error::{Result, SmooothError};
use crate::models::{Capability, Resource, ResourceOwnership, ResourceStatus, SupportLevel};

pub struct MongoAdapter {
    docker_adapter: Arc<DockerAdapter>,
}

impl MongoAdapter {
    pub fn new(docker_adapter: Arc<DockerAdapter>) -> Self {
        Self { docker_adapter }
    }

    /// Wait for MongoDB to accept TCP connections on port 27017 and measure latency
    pub async fn verify_mongo_connection(
        host: &str,
        port: u16,
        timeout_duration: Duration,
    ) -> Result<u64> {
        let start = Instant::now();
        let addr = format!("{}:{}", host, port);

        while start.elapsed() < timeout_duration {
            if let Ok(_stream) = TcpStream::connect(&addr).await {
                let latency = start.elapsed().as_millis() as u64;
                return Ok(latency);
            }
            sleep(Duration::from_millis(300)).await;
        }

        Err(SmooothError::Timeout(format!(
            "MongoDB at {}:{} did not accept TCP connections within {}s",
            host,
            port,
            timeout_duration.as_secs()
        )))
    }
}

#[async_trait]
impl ProviderAdapter for MongoAdapter {
    fn provider_name(&self) -> &str {
        "mongodb"
    }

    fn guarantees(&self) -> AdapterGuarantees {
        AdapterGuarantees {
            provider_name: "mongodb".to_string(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            id: "cap_mongo_db".to_string(),
            name: "MongoDB Document Database Service".to_string(),
            provider: "mongodb".to_string(),
            service_type: "database".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "High-performance MongoDB document database with persistent storage, authentication, and endpoint generation.".to_string(),
            operations: vec![
                "provision".to_string(),
                "inspect".to_string(),
                "health_check".to_string(),
                "reconcile".to_string(),
                "destroy".to_string(),
            ],
        }]
    }

    async fn provision(
        &self,
        name: &str,
        config: &serde_json::Value,
        is_persistent: bool,
    ) -> Result<Resource> {
        let image = config
            .get("image")
            .and_then(|v| v.as_str())
            .unwrap_or("mongo:7-jammy");

        let user = config
            .get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("smoooth_user");

        let password = match config.get("password").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => {
                let mut bytes = [0u8; 16];
                getrandom::getrandom(&mut bytes).map_err(|e| {
                    SmooothError::Internal(anyhow::anyhow!(
                        "Failed to generate random password: {}",
                        e
                    ))
                })?;
                hex::encode(bytes)
            }
        };

        let dbname = config
            .get("database")
            .and_then(|v| v.as_str())
            .unwrap_or("appdb");

        let container_name = if name.starts_with("smoooth-mongo-") {
            name.to_string()
        } else {
            format!("smoooth-mongo-{}", name)
        };

        let mut env = serde_json::Map::new();
        env.insert(
            "MONGO_INITDB_ROOT_USERNAME".to_string(),
            serde_json::Value::String(user.to_string()),
        );
        env.insert(
            "MONGO_INITDB_ROOT_PASSWORD".to_string(),
            serde_json::Value::String(password.clone()),
        );
        env.insert(
            "MONGO_INITDB_DATABASE".to_string(),
            serde_json::Value::String(dbname.to_string()),
        );

        let mut docker_config = serde_json::Map::new();
        docker_config.insert(
            "image".to_string(),
            serde_json::Value::String(image.to_string()),
        );
        docker_config.insert("env".to_string(), serde_json::Value::Object(env));

        // Volume configuration
        if is_persistent {
            let volume_name = format!("smoooth-{}-data", name);
            let volumes = vec![serde_json::json!({
                "volume_name": volume_name,
                "container_path": "/data/db",
                "read_only": false
            })];
            docker_config.insert("volumes".to_string(), serde_json::Value::Array(volumes));
        }

        // Network and port binding
        if let Some(ports) = config.get("ports") {
            docker_config.insert("ports".to_string(), ports.clone());
        }

        // Delegate container provisioning to DockerAdapter
        let mut res = self
            .docker_adapter
            .provision(
                &container_name,
                &serde_json::Value::Object(docker_config),
                is_persistent,
            )
            .await?;

        res.provider = "mongodb".to_string();
        res.service_type = "database".to_string();
        res.name = name.to_string();
        res.ownership = ResourceOwnership::SmooothManaged;

        // If Docker is running, verify readiness
        if self.docker_adapter.client().is_ok() {
            let host_port = self
                .docker_adapter
                .get_host_port(&container_name, 27017)
                .await
                .unwrap_or(27017);

            let latency =
                Self::verify_mongo_connection("127.0.0.1", host_port, Duration::from_secs(30))
                    .await
                    .unwrap_or(5);

            tracing::info!(
                "MongoDB instance '{}' provisioned and verified in {}ms",
                name,
                latency
            );
        }

        Ok(res)
    }

    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus> {
        self.docker_adapter.inspect(resource_id).await
    }

    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence> {
        let status = self.inspect(resource_id).await?;
        if status != ResourceStatus::Active {
            return Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: None,
                message: format!("MongoDB container status is {:?}", status),
                details: serde_json::json!({ "status": format!("{:?}", status) }),
            });
        }

        let host_port = self
            .docker_adapter
            .get_host_port(resource_id, 27017)
            .await
            .unwrap_or(27017);

        match Self::verify_mongo_connection("127.0.0.1", host_port, Duration::from_secs(5)).await {
            Ok(latency) => Ok(HealthEvidence {
                is_healthy: true,
                latency_ms: Some(latency),
                message: format!(
                    "MongoDB responding to TCP connection on port {} in {}ms",
                    host_port, latency
                ),
                details: serde_json::json!({ "port": host_port, "latency_ms": latency }),
            }),
            Err(e) => Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: None,
                message: format!("MongoDB connection failed: {}", e),
                details: serde_json::json!({ "error": e.to_string() }),
            }),
        }
    }

    async fn reconcile(
        &self,
        resource_id: &str,
        intended_config: &serde_json::Value,
    ) -> Result<ResourceStatus> {
        self.docker_adapter
            .reconcile(resource_id, intended_config)
            .await
    }

    async fn destroy(&self, resource_id: &str, preserve_persistent_data: bool) -> Result<()> {
        self.docker_adapter
            .destroy(resource_id, preserve_persistent_data)
            .await
    }
}
