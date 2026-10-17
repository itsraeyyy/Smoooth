use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio_postgres::NoTls;

use crate::adapters::docker::DockerAdapter;
use crate::adapters::{AdapterGuarantees, HealthEvidence, ProviderAdapter};
use crate::error::{Result, SmooothError};
use crate::models::{Capability, Resource, ResourceOwnership, ResourceStatus, SupportLevel};

pub struct PostgresAdapter {
    docker_adapter: Arc<DockerAdapter>,
}

impl PostgresAdapter {
    pub fn new(docker_adapter: Arc<DockerAdapter>) -> Self {
        Self { docker_adapter }
    }

    /// Wait for PostgreSQL to be ready and execute an active verification query
    async fn wait_and_verify_connection(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        dbname: &str,
        timeout_duration: Duration,
    ) -> Result<u64> {
        let start = Instant::now();
        let conn_str = format!(
            "host={} port={} user={} password={} dbname={} connect_timeout=3",
            host, port, user, password, dbname
        );

        while start.elapsed() < timeout_duration {
            match tokio_postgres::connect(&conn_str, NoTls).await {
                Ok((client, connection)) => {
                    // Spawn connection task in background
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            tracing::warn!("Postgres connection error: {}", e);
                        }
                    });

                    // Execute active test query
                    match client.simple_query("SELECT 1;").await {
                        Ok(_) => {
                            let latency = start.elapsed().as_millis() as u64;
                            return Ok(latency);
                        }
                        Err(e) => {
                            tracing::debug!("Postgres query not ready yet: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Postgres not accepting connections yet: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        Err(SmooothError::Timeout(format!(
            "PostgreSQL at {}:{} did not become ready within {}s",
            host,
            port,
            timeout_duration.as_secs()
        )))
    }
}

#[async_trait]
impl ProviderAdapter for PostgresAdapter {
    fn provider_name(&self) -> &str {
        "postgres"
    }

    fn guarantees(&self) -> AdapterGuarantees {
        AdapterGuarantees {
            provider_name: "postgres".to_string(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            id: "postgres_database".to_string(),
            name: "PostgreSQL Database Service".to_string(),
            provider: "postgres".to_string(),
            service_type: "database".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "Dedicated PostgreSQL database with persistent volume protection and staged credential rotation.".to_string(),
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
        let dbname = config
            .get("database")
            .and_then(|v| v.as_str())
            .unwrap_or("smoooth_db");

        let user = config
            .get("user")
            .and_then(|v| v.as_str())
            .unwrap_or("smoooth_user");

        let password = config
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("smoooth_secure_pass");

        let host_port = config
            .get("port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16);

        let volume_name = config
            .get("volume_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("pg_vol_{}", name));

        let network_name = config
            .get("network")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let workspace_id = config
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let mut ports_map = serde_json::Map::new();
        if let Some(p) = host_port {
            ports_map.insert("5432".to_string(), serde_json::json!(p));
        }

        // Prepare container configuration for DockerAdapter (no host port published by default)
        let mut docker_config = serde_json::json!({
            "image": "postgres:16-alpine",
            "workspace_id": workspace_id,
            "network": network_name,
            "ports": serde_json::Value::Object(ports_map),
            "env": {
                "POSTGRES_DB": dbname,
                "POSTGRES_USER": user,
                "POSTGRES_PASSWORD": password
            },
            "volumes": [
                {
                    "volume_name": volume_name,
                    "container_path": "/var/lib/postgresql/data"
                }
            ]
        });

        if let Some(epoch) = config.get("fencing_epoch") {
            docker_config["fencing_epoch"] = epoch.clone();
        }
        if let Some(worker) = config.get("worker_id") {
            docker_config["worker_id"] = worker.clone();
        }

        // 1. Provision via DockerAdapter with persistent volume protection
        let container_res = self
            .docker_adapter
            .provision(name, &docker_config, is_persistent)
            .await?;

        let (connect_host, connect_port) = if let Some(p) = host_port {
            ("127.0.0.1".to_string(), p)
        } else {
            let ip = self
                .docker_adapter
                .get_container_ip(&container_res.id)
                .await?;
            (ip, 5432)
        };

        // 2. Active verification: wait for connection and execute simple query
        let latency = match Self::wait_and_verify_connection(
            &connect_host,
            connect_port,
            user,
            password,
            dbname,
            Duration::from_secs(35),
        )
        .await
        {
            Ok(lat) => lat,
            Err(e) => {
                // If health check times out, clean up container if ephemeral, but preserve persistent volume!
                let _ = self.destroy(&container_res.id, is_persistent).await;
                return Err(SmooothError::PostgresFailed(format!(
                    "PostgreSQL service failed verification: {}",
                    e
                )));
            }
        };

        tracing::info!(
            "PostgreSQL service '{}' active and verified (query latency: {}ms)",
            name,
            latency
        );

        let mut output_config = config.clone();
        if let Some(obj) = output_config.as_object_mut() {
            obj.insert(
                "container_id".to_string(),
                serde_json::json!(container_res.id),
            );
            obj.insert("host".to_string(), serde_json::json!(&connect_host));
            obj.insert("port".to_string(), serde_json::json!(connect_port));
            obj.insert("volume_name".to_string(), serde_json::json!(volume_name));
        }

        Ok(Resource {
            id: container_res.id,
            name: name.to_string(),
            workspace_id: workspace_id.to_string(),
            service_type: "database".to_string(),
            provider: "postgres".to_string(),
            is_persistent: true,
            ownership: ResourceOwnership::SmooothManaged,
            status: ResourceStatus::Active,
            configuration: output_config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus> {
        self.docker_adapter.inspect(resource_id).await
    }

    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence> {
        // First check container status
        let container_health = self.docker_adapter.health_check(resource_id).await?;
        if !container_health.is_healthy {
            return Ok(container_health);
        }

        Ok(HealthEvidence {
            is_healthy: true,
            latency_ms: container_health.latency_ms,
            message: format!("PostgreSQL instance '{}' is running healthy.", resource_id),
            details: serde_json::json!({
                "container_id": resource_id,
                "database_engine": "PostgreSQL 16",
                "status": "healthy"
            }),
        })
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
        // Destroy the container, but NEVER delete the persistent volume if preserve_persistent_data is true!
        self.docker_adapter
            .destroy(resource_id, preserve_persistent_data)
            .await
    }
}
