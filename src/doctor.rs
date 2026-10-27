use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub category: String,
    pub name: String,
    pub status: String, // "pass", "warn", "fail"
    pub message: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub overall_status: String, // "healthy", "warning", "critical"
    pub passed_count: usize,
    pub warning_count: usize,
    pub failure_count: usize,
    pub os: String,
    pub arch: String,
    pub version: &'static str,
    pub checks: Vec<DoctorCheck>,
}

pub struct Doctor;

impl Doctor {
    pub async fn run_diagnostics(storage_path: &str, bind_addr: &str, port: u16) -> DoctorReport {
        let mut checks = Vec::new();
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        // 1. Architecture Check
        if arch == "x86_64" {
            checks.push(DoctorCheck {
                category: "Platform".to_string(),
                name: "CPU Architecture".to_string(),
                status: "pass".to_string(),
                message: format!(
                    "Verified architecture: {} (AMD64 / x86_64 verified baseline)",
                    arch
                ),
                recommendation: None,
            });
        } else if arch == "aarch64" {
            checks.push(DoctorCheck {
                category: "Platform".to_string(),
                name: "CPU Architecture".to_string(),
                status: "warn".to_string(),
                message: format!(
                    "Architecture {} (ARM64) is experimental; official baseline is x86_64",
                    arch
                ),
                recommendation: Some(
                    "Verify Docker images specify multi-arch or linux/arm64 tags.".to_string(),
                ),
            });
        } else {
            checks.push(DoctorCheck {
                category: "Platform".to_string(),
                name: "CPU Architecture".to_string(),
                status: "fail".to_string(),
                message: format!("Unsupported CPU architecture: {}", arch),
                recommendation: Some(
                    "Smoooth requires x86_64 (or aarch64 experimental) host Linux environment."
                        .to_string(),
                ),
            });
        }

        // 2. Operating System Check
        if os == "linux" {
            let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
            if os_release.contains("Ubuntu") && os_release.contains("24.04") {
                checks.push(DoctorCheck {
                    category: "Platform".to_string(),
                    name: "Operating System".to_string(),
                    status: "pass".to_string(),
                    message: "Ubuntu 24.04 LTS confirmed (Target Beta Baseline).".to_string(),
                    recommendation: None,
                });
            } else if os_release.contains("Ubuntu") || os_release.contains("Debian") {
                checks.push(DoctorCheck {
                    category: "Platform".to_string(),
                    name: "Operating System".to_string(),
                    status: "pass".to_string(),
                    message: "Compatible Debian-family Linux detected.".to_string(),
                    recommendation: None,
                });
            } else {
                checks.push(DoctorCheck {
                    category: "Platform".to_string(),
                    name: "Operating System".to_string(),
                    status: "warn".to_string(),
                    message: "Linux distribution detected. Official verified target is Ubuntu 24.04.".to_string(),
                    recommendation: Some(
                        "Ensure standard systemd and Docker packages are installed.".to_string(),
                    ),
                });
            }
        } else {
            checks.push(DoctorCheck {
                category: "Platform".to_string(),
                name: "Operating System".to_string(),
                status: "warn".to_string(),
                message: format!(
                    "Host OS is '{}'. Production target is Linux VPS (Ubuntu 24.04).",
                    os
                ),
                recommendation: Some(
                    "Local development mode active. Production deployments require Linux VPS."
                        .to_string(),
                ),
            });
        }

        // 3. Docker Engine Availability & Privilege Disclosure
        let docker_client = bollard::Docker::connect_with_local_defaults();
        match docker_client {
            Ok(client) => match client.ping().await {
                Ok(_) => {
                    checks.push(DoctorCheck {
                        category: "Container Runtime".to_string(),
                        name: "Docker Engine Daemon".to_string(),
                        status: "pass".to_string(),
                        message: "Docker daemon connection active and responsive via local socket."
                            .to_string(),
                        recommendation: None,
                    });
                    checks.push(DoctorCheck {
                        category: "Security Disclosure".to_string(),
                        name: "Docker Effective Host-Root Access".to_string(),
                        status: "warn".to_string(),
                        message: "Access to Docker daemon grants effective host-root permissions. Service must remain bound to loopback or private network.".to_string(),
                        recommendation: Some("Keep daemon bound to 127.0.0.1. Do not expose 4040 publicly without TLS reverse proxy and authentication tokens.".to_string()),
                    });
                }
                Err(e) => {
                    checks.push(DoctorCheck {
                        category: "Container Runtime".to_string(),
                        name: "Docker Engine Daemon".to_string(),
                        status: "fail".to_string(),
                        message: format!("Docker daemon ping failed: {}", e),
                        recommendation: Some("Ensure Docker is installed and service is running: `sudo systemctl start docker` or add user to docker group: `sudo usermod -aG docker $USER`".to_string()),
                    });
                }
            },
            Err(e) => {
                checks.push(DoctorCheck {
                    category: "Container Runtime".to_string(),
                    name: "Docker Engine Connection".to_string(),
                    status: "fail".to_string(),
                    message: format!("Could not initialize Docker client: {}", e),
                    recommendation: Some(
                        "Verify Docker socket is available at /var/run/docker.sock".to_string(),
                    ),
                });
            }
        }

        // 4. Port Availability Check
        let test_addr = format!("{}:{}", bind_addr, port);
        match TcpListener::bind(&test_addr) {
            Ok(listener) => {
                drop(listener);
                checks.push(DoctorCheck {
                    category: "Networking".to_string(),
                    name: "Daemon Port Availability".to_string(),
                    status: "pass".to_string(),
                    message: format!("Port {} is free and available on {}", port, bind_addr),
                    recommendation: None,
                });
            }
            Err(_) => {
                // If the current process or another instance is bound
                checks.push(DoctorCheck {
                    category: "Networking".to_string(),
                    name: "Daemon Port Availability".to_string(),
                    status: "warn".to_string(),
                    message: format!(
                        "Port {} on {} is currently bound (active daemon or conflicting service).",
                        port, bind_addr
                    ),
                    recommendation: Some(format!(
                        "Check running services with: `ss -tlpn | grep {}` or `smoooth status`",
                        port
                    )),
                });
            }
        }

        // 5. External Reverse Proxy & Coexistence Audit (Coolify, Traefik, Caddy, Nginx)
        let port_80_bound = TcpListener::bind("0.0.0.0:80").is_err();
        let port_443_bound = TcpListener::bind("0.0.0.0:443").is_err();

        if port_80_bound || port_443_bound {
            checks.push(DoctorCheck {
                category: "Coexistence & Ingress".to_string(),
                name: "External Reverse Proxy Coexistence".to_string(),
                status: "pass".to_string(),
                message: format!(
                    "External ingress proxy active on host (Port 80 bound: {}, Port 443 bound: {}). Smoooth runs in isolated coexistence mode on {}:{}.",
                    port_80_bound, port_443_bound, bind_addr, port
                ),
                recommendation: Some(
                    "Zero-Downtime Guarantee: Smoooth will NOT mutate external proxies (Coolify Traefik, Caddy, Nginx) or containers outside its assigned workspace. Use `migrate_stack` or `scan_foreign_workloads` to safely import workloads.".to_string(),
                ),
            });
        } else {
            checks.push(DoctorCheck {
                category: "Coexistence & Ingress".to_string(),
                name: "Standard Ingress Ports".to_string(),
                status: "pass".to_string(),
                message: "Standard HTTP/HTTPS ports (80/443) are available for Smoooth managed ingress.".to_string(),
                recommendation: None,
            });
        }

        // 5. Storage & Persistence Check
        let state_path = Path::new(storage_path);
        let parent_dir = state_path.parent().unwrap_or(Path::new("."));
        let parent_path = if parent_dir.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent_dir
        };

        if let Ok(metadata) = std::fs::metadata(parent_path) {
            if metadata.permissions().readonly() {
                checks.push(DoctorCheck {
                    category: "Storage".to_string(),
                    name: "Storage Directory Write Access".to_string(),
                    status: "fail".to_string(),
                    message: format!("State directory '{:?}' is read-only.", parent_path),
                    recommendation: Some("Ensure directory is writable by current user: `sudo chmod 0750 /var/lib/smoooth`".to_string()),
                });
            } else {
                checks.push(DoctorCheck {
                    category: "Storage".to_string(),
                    name: "Storage Directory Write Access".to_string(),
                    status: "pass".to_string(),
                    message: format!("State directory '{:?}' is writable.", parent_path),
                    recommendation: None,
                });
            }
        } else {
            checks.push(DoctorCheck {
                category: "Storage".to_string(),
                name: "Storage Directory Exists".to_string(),
                status: "warn".to_string(),
                message: format!("Storage directory '{:?}' does not exist yet (will be created automatically on state write).", parent_path),
                recommendation: None,
            });
        }

        // 6. Security Credential File Verification (Linux VPS target)
        if os == "linux" {
            let token_path = Path::new("/etc/smoooth/auth.token");
            if token_path.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = token_path.metadata() {
                        let mode = meta.permissions().mode() & 0o777;
                        if mode == 0o600 {
                            checks.push(DoctorCheck {
                                category: "Security".to_string(),
                                name: "Token File Permissions".to_string(),
                                status: "pass".to_string(),
                                message:
                                    "Auth token file permissions verified as 0600 (owner-only)."
                                        .to_string(),
                                recommendation: None,
                            });
                        } else {
                            checks.push(DoctorCheck {
                                category: "Security".to_string(),
                                name: "Token File Permissions".to_string(),
                                status: "fail".to_string(),
                                message: format!(
                                    "Auth token file mode is 0{:o}; expected 0600.",
                                    mode
                                ),
                                recommendation: Some(
                                    "Secure token file: `sudo chmod 0600 /etc/smoooth/auth.token`"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
            } else {
                checks.push(DoctorCheck {
                    category: "Security".to_string(),
                    name: "System Auth Token".to_string(),
                    status: "warn".to_string(),
                    message: "Standard production token file /etc/smoooth/auth.token not present; using local fallback.".to_string(),
                    recommendation: Some("Run `smoooth agent pair` to provision dedicated agent tokens or run install script.".to_string()),
                });
            }
        }

        let passed = checks.iter().filter(|c| c.status == "pass").count();
        let warnings = checks.iter().filter(|c| c.status == "warn").count();
        let failures = checks.iter().filter(|c| c.status == "fail").count();

        let overall = if failures > 0 {
            "critical".to_string()
        } else if warnings > 0 {
            "warning".to_string()
        } else {
            "healthy".to_string()
        };

        DoctorReport {
            overall_status: overall,
            passed_count: passed,
            warning_count: warnings,
            failure_count: failures,
            os,
            arch,
            version: "0.1.1",
            checks,
        }
    }
}
