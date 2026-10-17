use bollard::container::InspectContainerOptions;
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::ContainerInspectResponse;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::adapters::docker::client::DockerClient;
use crate::adapters::HealthEvidence;
use crate::error::{Result, SmooothError};
use crate::models::HealthSpec;

pub struct HealthProber;

impl HealthProber {
    /// Helper to detect host port from container inspection
    fn detect_port(inspect: &ContainerInspectResponse) -> Option<u16> {
        let from_ns = inspect.network_settings.as_ref().and_then(|ns| {
            ns.ports.as_ref().and_then(|pm| {
                pm.values().find_map(|bindings| {
                    bindings.as_ref().and_then(|b_list| {
                        b_list.first().and_then(|b| {
                            b.host_port.as_ref().and_then(|hp| hp.parse::<u16>().ok())
                        })
                    })
                })
            })
        });
        if from_ns.is_some() {
            return from_ns;
        }
        inspect.host_config.as_ref().and_then(|hc| {
            hc.port_bindings.as_ref().and_then(|pm| {
                pm.values().find_map(|bindings| {
                    bindings.as_ref().and_then(|b_list| {
                        b_list.first().and_then(|b| {
                            b.host_port.as_ref().and_then(|hp| hp.parse::<u16>().ok())
                        })
                    })
                })
            })
        })
    }

    /// Resolve mapped host loopback port from container port if mapped
    fn resolve_host_port(inspect: &ContainerInspectResponse, port: u16) -> u16 {
        let port_key = format!("{}/tcp", port);
        if let Some(ns) = &inspect.network_settings {
            if let Some(pm) = &ns.ports {
                if let Some(Some(b_list)) = pm.get(&port_key) {
                    if let Some(b) = b_list.first() {
                        if let Some(hp) = &b.host_port {
                            if let Ok(parsed) = hp.parse::<u16>() {
                                return parsed;
                            }
                        }
                    }
                }
            }
        }
        if let Some(hc) = &inspect.host_config {
            if let Some(pm) = &hc.port_bindings {
                if let Some(Some(b_list)) = pm.get(&port_key) {
                    if let Some(b) = b_list.first() {
                        if let Some(hp) = &b.host_port {
                            if let Ok(parsed) = hp.parse::<u16>() {
                                return parsed;
                            }
                        }
                    }
                }
            }
        }
        port
    }

    /// Active TCP probe helper
    async fn probe_tcp(
        start: Instant,
        container_name_or_id: &str,
        port: u16,
    ) -> Result<HealthEvidence> {
        let addr = format!("127.0.0.1:{}", port);
        let mut connected = false;
        let mut last_err = String::new();

        for _ in 0..15 {
            match tokio::time::timeout(
                std::time::Duration::from_millis(400),
                TcpStream::connect(&addr),
            )
            .await
            {
                Ok(Ok(mut stream)) => {
                    let mut buf = [0u8; 1];
                    match tokio::time::timeout(
                        std::time::Duration::from_millis(150),
                        stream.read(&mut buf),
                    )
                    .await
                    {
                        Ok(Ok(0)) => {
                            last_err = "Connection closed immediately by peer (no process listening on port)".to_string();
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        }
                        Ok(Err(e)) => {
                            last_err = format!("Connection reset by peer: {}", e);
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        }
                        Ok(Ok(_)) | Err(_) => {
                            connected = true;
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    last_err = e.to_string();
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
                Err(_) => {
                    last_err = "timed out".to_string();
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }
        }

        if connected {
            Ok(HealthEvidence {
                is_healthy: true,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: format!(
                    "Container '{}' responded to active TCP probe on port {}.",
                    container_name_or_id, port
                ),
                details: serde_json::json!({
                    "running": true,
                    "tcp_port": port,
                    "tcp_probe": "passed"
                }),
            })
        } else {
            Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: format!("TCP probe to port {} failed: {}", port, last_err),
                details: serde_json::json!({
                    "running": true,
                    "tcp_port": port,
                    "tcp_probe": "failed",
                    "error": last_err
                }),
            })
        }
    }

    /// Inspect container running state and probe using explicit or inferred HealthSpec
    pub async fn check_health(
        client: &DockerClient,
        container_name_or_id: &str,
        health_spec: Option<&HealthSpec>,
    ) -> Result<HealthEvidence> {
        let start = Instant::now();
        let docker = client.inner();

        // 1. Inspect container state
        let inspect = match docker
            .inspect_container(container_name_or_id, None::<InspectContainerOptions>)
            .await
        {
            Ok(ins) => ins,
            Err(e) => {
                return Ok(HealthEvidence {
                    is_healthy: false,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    message: format!(
                        "Container '{}' unavailable or not found: {}",
                        container_name_or_id, e
                    ),
                    details: serde_json::json!({
                        "running": false,
                        "error": e.to_string()
                    }),
                });
            }
        };

        let state = inspect
            .state
            .as_ref()
            .ok_or_else(|| SmooothError::ContainerFailed("Container has no state".to_string()))?;

        let is_running = state.running.unwrap_or(false);
        if !is_running {
            return Ok(HealthEvidence {
                is_healthy: false,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: format!(
                    "Container '{}' is not running (status: {:?})",
                    container_name_or_id, state.status
                ),
                details: serde_json::json!({
                    "running": false,
                    "exit_code": state.exit_code,
                    "error": state.error
                }),
            });
        }

        // 2. Resolve effective HealthSpec: explicit parameter, or parsed from label, or inferred
        let effective_spec: Option<HealthSpec> = if let Some(hs) = health_spec {
            Some(hs.clone())
        } else if let Some(config) = &inspect.config {
            config
                .labels
                .as_ref()
                .and_then(|l| l.get("smoooth.health_spec"))
                .and_then(|s| serde_json::from_str::<HealthSpec>(s).ok())
        } else {
            None
        };

        // 3. Execute probe according to HealthSpec
        match effective_spec {
            Some(HealthSpec::Http {
                path,
                port,
                expected_status,
            }) => {
                let host_port = Self::resolve_host_port(&inspect, port);
                let addr = format!("127.0.0.1:{}", host_port);
                let mut status_code: Option<u16> = None;
                let mut last_err = String::new();

                for _ in 0..15 {
                    match tokio::time::timeout(
                        std::time::Duration::from_millis(400),
                        TcpStream::connect(&addr),
                    )
                    .await
                    {
                        Ok(Ok(mut stream)) => {
                            let req = format!(
                                "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nUser-Agent: smoooth-probe\r\n\r\n",
                                path, host_port
                            );
                            if let Err(e) = stream.write_all(req.as_bytes()).await {
                                last_err = format!("Failed to send HTTP request: {}", e);
                                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                continue;
                            }

                            let mut buf = vec![0u8; 1024];
                            match tokio::time::timeout(
                                std::time::Duration::from_millis(800),
                                stream.read(&mut buf),
                            )
                            .await
                            {
                                Ok(Ok(n)) if n > 0 => {
                                    let resp = String::from_utf8_lossy(&buf[..n]);
                                    if let Some(first_line) = resp.lines().next() {
                                        let parts: Vec<&str> =
                                            first_line.split_whitespace().collect();
                                        if parts.len() >= 2 {
                                            if let Ok(code) = parts[1].parse::<u16>() {
                                                status_code = Some(code);
                                                break;
                                            }
                                        }
                                    }
                                    last_err = format!(
                                        "Unparseable HTTP response: {}",
                                        resp.lines().next().unwrap_or("")
                                    );
                                }
                                Ok(Ok(_)) => {
                                    last_err = "Empty HTTP response from peer".to_string();
                                }
                                Ok(Err(e)) => {
                                    last_err = format!("Failed to read HTTP response: {}", e);
                                }
                                Err(_) => {
                                    last_err = "Timed out reading HTTP response".to_string();
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            last_err = format!("Connection refused: {}", e);
                        }
                        Err(_) => {
                            last_err = "Connection timed out".to_string();
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }

                if let Some(code) = status_code {
                    if code == expected_status {
                        Ok(HealthEvidence {
                            is_healthy: true,
                            latency_ms: Some(start.elapsed().as_millis() as u64),
                            message: format!(
                                "Container '{}' HTTP check returned expected status {}.",
                                container_name_or_id, code
                            ),
                            details: serde_json::json!({
                                "running": true,
                                "probe": "http",
                                "http_status": code,
                                "expected_status": expected_status,
                                "path": path,
                                "port": port
                            }),
                        })
                    } else {
                        Ok(HealthEvidence {
                            is_healthy: false,
                            latency_ms: Some(start.elapsed().as_millis() as u64),
                            message: format!("HTTP check failed for container '{}': expected status {}, received {}", container_name_or_id, expected_status, code),
                            details: serde_json::json!({
                                "running": true,
                                "probe": "http",
                                "http_status": code,
                                "expected_status": expected_status,
                                "path": path,
                                "port": port
                            }),
                        })
                    }
                } else {
                    Ok(HealthEvidence {
                        is_healthy: false,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        message: format!(
                            "HTTP check to port {} path '{}' failed: {}",
                            port, path, last_err
                        ),
                        details: serde_json::json!({
                            "running": true,
                            "probe": "http",
                            "error": last_err,
                            "expected_status": expected_status,
                            "path": path,
                            "port": port
                        }),
                    })
                }
            }

            Some(HealthSpec::Exec { cmd }) => {
                let exec_options = CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(cmd.clone()),
                    ..Default::default()
                };

                match docker.create_exec(container_name_or_id, exec_options).await {
                    Ok(created) => {
                        let _ = docker
                            .start_exec(&created.id, None::<StartExecOptions>)
                            .await;
                        match docker.inspect_exec(&created.id).await {
                            Ok(inspect_resp) => {
                                let exit_code = inspect_resp.exit_code.unwrap_or(-1);
                                if exit_code == 0 {
                                    Ok(HealthEvidence {
                                        is_healthy: true,
                                        latency_ms: Some(start.elapsed().as_millis() as u64),
                                        message: format!(
                                            "Container '{}' exec probe succeeded (exit code 0).",
                                            container_name_or_id
                                        ),
                                        details: serde_json::json!({
                                            "running": true,
                                            "probe": "exec",
                                            "cmd": cmd,
                                            "exit_code": 0
                                        }),
                                    })
                                } else {
                                    Ok(HealthEvidence {
                                        is_healthy: false,
                                        latency_ms: Some(start.elapsed().as_millis() as u64),
                                        message: format!(
                                            "Container '{}' exec probe failed with exit code {}.",
                                            container_name_or_id, exit_code
                                        ),
                                        details: serde_json::json!({
                                            "running": true,
                                            "probe": "exec",
                                            "cmd": cmd,
                                            "exit_code": exit_code
                                        }),
                                    })
                                }
                            }
                            Err(e) => Ok(HealthEvidence {
                                is_healthy: false,
                                latency_ms: Some(start.elapsed().as_millis() as u64),
                                message: format!(
                                    "Failed to inspect exec probe for container '{}': {}",
                                    container_name_or_id, e
                                ),
                                details: serde_json::json!({ "error": e.to_string() }),
                            }),
                        }
                    }
                    Err(e) => Ok(HealthEvidence {
                        is_healthy: false,
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        message: format!(
                            "Failed to create exec probe for container '{}': {}",
                            container_name_or_id, e
                        ),
                        details: serde_json::json!({ "error": e.to_string() }),
                    }),
                }
            }

            Some(HealthSpec::Liveness) => Ok(HealthEvidence {
                is_healthy: true,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: format!(
                    "Container '{}' liveness verified (running).",
                    container_name_or_id
                ),
                details: serde_json::json!({
                    "running": true,
                    "probe": "liveness",
                    "status": state.status
                }),
            }),

            Some(HealthSpec::Tcp { port }) => {
                let host_port = Self::resolve_host_port(&inspect, port);
                Self::probe_tcp(start, container_name_or_id, host_port).await
            }

            None => {
                // Backwards-compatible fallback: auto-detect port or check running
                let detected_port = Self::detect_port(&inspect);
                if let Some(port) = detected_port {
                    return Self::probe_tcp(start, container_name_or_id, port).await;
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
    }
}
