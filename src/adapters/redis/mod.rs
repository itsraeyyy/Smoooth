use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::adapters::docker::DockerAdapter;
use crate::adapters::{AdapterGuarantees, HealthEvidence, ProviderAdapter};
use crate::error::{Result, SmooothError};
use crate::models::{Capability, Resource, ResourceOwnership, ResourceStatus, SupportLevel};

pub struct RedisAdapter {
    docker_adapter: Arc<DockerAdapter>,
}

impl RedisAdapter {
    pub fn new(docker_adapter: Arc<DockerAdapter>) -> Self {
        Self { docker_adapter }
    }

    /// Execute raw RESP PING active verification over TCP
    pub async fn verify_redis_connection(
        host: &str,
        port: u16,
        password: Option<&str>,
        timeout_duration: Duration,
    ) -> Result<u64> {
        let start = Instant::now();
        let addr = format!("{}:{}", host, port);

        while start.elapsed() < timeout_duration {
            if let Ok(mut stream) = TcpStream::connect(&addr).await {
                // If password is required, authenticate via RESP AUTH
                if let Some(pass) = password {
                    if !pass.is_empty() {
                        let auth_cmd =
                            format!("*2\r\n$4\r\nAUTH\r\n${}\r\n{}\r\n", pass.len(), pass);
                        if stream.write_all(auth_cmd.as_bytes()).await.is_ok() {
                            let mut buf = [0u8; 128];
                            if let Ok(n) = stream.read(&mut buf).await {
                                let resp = String::from_utf8_lossy(&buf[..n]);
                                if !resp.starts_with("+OK") {
                                    tracing::debug!("Redis AUTH pending/failed: {}", resp);
                                    sleep(Duration::from_millis(300)).await;
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Send RESP PING
                if stream.write_all(b"*1\r\n$4\r\nPING\r\n").await.is_ok() {
                    let mut buf = [0u8; 128];
                    if let Ok(n) = stream.read(&mut buf).await {
                        let resp = String::from_utf8_lossy(&buf[..n]);
                        if resp.starts_with("+PONG") || resp.starts_with("-NOAUTH") {
                            let latency = start.elapsed().as_millis() as u64;
                            return Ok(latency);
                        }
                    }
                }
            }

            sleep(Duration::from_millis(300)).await;
        }

        Err(SmooothError::Timeout(format!(
            "Redis at {}:{} did not respond with +PONG within {}s",
            host,
            port,
            timeout_duration.as_secs()
        )))
    }
}

#[async_trait]
impl ProviderAdapter for RedisAdapter {
    fn provider_name(&self) -> &str {
        "redis"
    }

    fn guarantees(&self) -> AdapterGuarantees {
        AdapterGuarantees {
            provider_name: "redis".to_string(),
            supports_persistent_volumes: true,
            supports_staged_dual_credentials: true,
            supports_safe_rollback: true,
            supports_drift_detection: true,
            is_experimental: false,
        }
    }

    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            id: "redis_cache".to_string(),
            name: "Redis Key-Value Cache & Store".to_string(),
            provider: "redis".to_string(),
            service_type: "cache".to_string(),
            support_level: SupportLevel::SelfHosted,
            description: "High-performance Redis in-memory cache and data store with AOF persistence and authentication.".to_string(),
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
        let password = config
            .get("password")
            .or_else(|| config.get("require_pass"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let host_port = config
            .get("port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16);

        let volume_name = config
            .get("volume_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("redis_vol_{}", name));

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
            ports_map.insert("6379".to_string(), serde_json::json!(p));
        }

        let mut cmd = vec![
            "redis-server".to_string(),
            "--appendonly".to_string(),
            "yes".to_string(),
        ];
        if let Some(ref pass) = password {
            if !pass.is_empty() {
                cmd.push("--requirepass".to_string());
                cmd.push(pass.clone());
            }
        }

        let mut docker_config = serde_json::json!({
            "image": "redis:7-alpine",
            "workspace_id": workspace_id,
            "network": network_name,
            "ports": serde_json::Value::Object(ports_map),
            "cmd": cmd,
            "volumes": [
                {
                    "volume_name": volume_name,
                    "container_path": "/data"
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
            if p == 0 {
                let dynamic_port = self
                    .docker_adapter
                    .get_host_port(&container_res.id, 6379)
                    .await?;
                ("127.0.0.1".to_string(), dynamic_port)
            } else {
                ("127.0.0.1".to_string(), p)
            }
        } else {
            let ip = self
                .docker_adapter
                .get_container_ip(&container_res.id)
                .await?;
            (ip, 6379)
        };

        // 2. Active verification: wait for connection and execute simple RESP PING
        let latency = Self::verify_redis_connection(
            &connect_host,
            connect_port,
            password.as_deref(),
            Duration::from_secs(20),
        )
        .await?;

        tracing::info!(
            "Redis resource '{}' verified healthy (latency: {}ms)",
            name,
            latency
        );

        let mut ports_info = serde_json::Map::new();
        ports_info.insert("6379".to_string(), serde_json::json!(connect_port));

        let mut final_config = serde_json::json!({
            "protocol": "redis",
            "host": connect_host,
            "port": connect_port,
            "ports": serde_json::Value::Object(ports_info),
            "volume_name": volume_name,
            "workspace_id": workspace_id,
        });

        if let Some(ref pass) = password {
            final_config["password"] = serde_json::json!(pass);
        }

        Ok(Resource {
            id: container_res.id,
            name: name.to_string(),
            service_type: "cache".to_string(),
            provider: "redis".to_string(),
            workspace_id: workspace_id.to_string(),
            status: ResourceStatus::Active,
            ownership: ResourceOwnership::SmooothManaged,
            configuration: final_config,
            is_persistent,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn inspect(&self, resource_id: &str) -> Result<ResourceStatus> {
        self.docker_adapter.inspect(resource_id).await
    }

    async fn health_check(&self, resource_id: &str) -> Result<HealthEvidence> {
        let status = self.docker_adapter.inspect(resource_id).await?;
        if status != ResourceStatus::Active {
            return Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: None,
                message: format!(
                    "Redis container '{}' is not running (status: {:?})",
                    resource_id, status
                ),
                details: serde_json::json!({ "container_status": format!("{:?}", status) }),
            });
        }

        let (connect_host, connect_port) =
            match self.docker_adapter.get_host_port(resource_id, 6379).await {
                Ok(p) => ("127.0.0.1".to_string(), p),
                Err(_) => {
                    let ip = self.docker_adapter.get_container_ip(resource_id).await?;
                    (ip, 6379)
                }
            };

        // Attempt to extract password from container inspect if present
        let password = if let Ok(client) = self.docker_adapter.client() {
            if let Ok(inspect) = client.inner().inspect_container(resource_id, None).await {
                inspect.config.and_then(|c| c.cmd).and_then(|cmd| {
                    if let Some(pos) = cmd.iter().position(|arg| arg == "--requirepass") {
                        cmd.get(pos + 1).cloned()
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        } else {
            None
        };

        match Self::verify_redis_connection(
            &connect_host,
            connect_port,
            password.as_deref(),
            Duration::from_secs(5),
        )
        .await
        {
            Ok(latency) => Ok(HealthEvidence {
                is_healthy: true,
                latency_ms: Some(latency),
                message: format!("Redis active probe responsive (latency: {}ms)", latency),
                details: serde_json::json!({
                    "type": "redis_resp_ping",
                    "host": connect_host,
                    "port": connect_port,
                    "latency_ms": latency,
                }),
            }),
            Err(e) => Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: None,
                message: format!("Redis probe failed: {}", e),
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
