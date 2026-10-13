use std::time::Instant;
use bollard::container::InspectContainerOptions;
use tokio::net::TcpStream;

use crate::adapters::docker::client::DockerClient;
use crate::adapters::HealthEvidence;
use crate::error::{Result, SmooothError};

pub struct HealthProber;

impl HealthProber {
    /// Inspect container running state and probe TCP port if provided
    pub async fn check_health(
        client: &DockerClient,
        container_name_or_id: &str,
        tcp_port: Option<u16>,
    ) -> Result<HealthEvidence> {
        let start = Instant::now();
        let docker = client.inner();

        // 1. Inspect container state
        let inspect = docker
            .inspect_container(container_name_or_id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to inspect container for health: {}", e)))?;

        let state = inspect.state.ok_or_else(|| {
            SmooothError::ContainerFailed("Container has no state".to_string())
        })?;

        let is_running = state.running.unwrap_or(false);
        if !is_running {
            return Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: format!(
                    "Container '{}' is not running (status: {:?})",
                    container_name_or_id,
                    state.status
                ),
                details: serde_json::json!({
                    "running": false,
                    "exit_code": state.exit_code,
                    "error": state.error
                }),
            });
        }

        // 2. Active TCP probe if port is exposed
        if let Some(port) = tcp_port {
            let addr = format!("127.0.0.1:{}", port);
            match tokio::time::timeout(std::time::Duration::from_secs(3), TcpStream::connect(&addr)).await {
                Ok(Ok(_stream)) => {
                    return Ok(HealthEvidence {
                        is_healthy: true,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        message: format!("Container '{}' responded to TCP probe on port {}.", container_name_or_id, port),
                        details: serde_json::json!({
                            "running": true,
                            "tcp_port": port,
                            "tcp_probe": "passed"
                        }),
                    });
                }
                Ok(Err(e)) => {
                    return Ok(HealthEvidence {
                        is_healthy: false,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        message: format!("TCP probe to port {} failed: {}", port, e),
                        details: serde_json::json!({
                            "running": true,
                            "tcp_port": port,
                            "tcp_probe": "connection_refused"
                        }),
                    });
                }
                Err(_) => {
                    return Ok(HealthEvidence {
                        is_healthy: false,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        message: format!("TCP probe to port {} timed out.", port),
                        details: serde_json::json!({
                            "running": true,
                            "tcp_port": port,
                            "tcp_probe": "timed_out"
                        }),
                    });
                }
            }
        }

        Ok(HealthEvidence {
            is_healthy: true,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            message: format!("Container '{}' is running healthy.", container_name_or_id),
            details: serde_json::json!({
                "running": true,
                "status": state.status
            }),
        })
    }
}
