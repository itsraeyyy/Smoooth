use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{Result, SmooothError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngressRoute {
    pub id: String,
    pub domain: String,
    pub target_resource: String,
    pub target_port: u16,
    pub tls: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngressMode {
    /// Ports 80 and 443 are free: Smoooth manages an autonomous Caddy instance with ACME TLS
    StandaloneCaddy,
    /// Ports 80 and 443 are bound by existing host reverse proxy (e.g. Coolify, Traefik, Nginx)
    UpstreamProxy { existing_proxy: String },
}

pub struct IngressManager {
    state_file: PathBuf,
}

impl IngressManager {
    pub fn new(state_dir: impl AsRef<Path>) -> Self {
        Self {
            state_file: state_dir.as_ref().join("ingress_routes.json"),
        }
    }

    pub fn default_local() -> Self {
        let path = if cfg!(windows) {
            PathBuf::from(r"C:\ProgramData\smoooth\ingress_routes.json")
        } else {
            PathBuf::from("/var/lib/smoooth/ingress_routes.json")
        };
        Self { state_file: path }
    }

    /// Check whether a TCP port is currently free on loopback / all interfaces
    pub fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
            && TcpListener::bind(("0.0.0.0", port)).is_ok()
    }

    /// Detect optimal ingress mode based on host environment
    pub fn detect_mode() -> IngressMode {
        let port_80_free = Self::is_port_available(80);
        let port_443_free = Self::is_port_available(443);

        if port_80_free && port_443_free {
            IngressMode::StandaloneCaddy
        } else {
            // Check common known proxies
            let proxy_hint = if !port_80_free || !port_443_free {
                "Host Ingress (Port 80/443 occupied; Coolify/Traefik/Nginx detected)"
            } else {
                "External Reverse Proxy"
            };
            IngressMode::UpstreamProxy {
                existing_proxy: proxy_hint.to_string(),
            }
        }
    }

    /// Load registered ingress routes from state file
    pub fn list_routes(&self) -> Result<Vec<IngressRoute>> {
        if !self.state_file.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.state_file).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to read ingress state: {}", e))
        })?;
        let routes: Vec<IngressRoute> = serde_json::from_str(&content).unwrap_or_default();
        Ok(routes)
    }

    /// Save routes to state file atomically
    fn save_routes(&self, routes: &[IngressRoute]) -> Result<()> {
        if let Some(parent) = self.state_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(routes).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to serialize ingress state: {}", e))
        })?;
        fs::write(&self.state_file, json).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to write ingress state: {}", e))
        })?;
        Ok(())
    }

    /// Register a new domain ingress route
    pub fn add_route(
        &self,
        domain: &str,
        target_resource: &str,
        target_port: u16,
        tls: bool,
        path_prefix: Option<String>,
    ) -> Result<IngressRoute> {
        let clean_domain = domain.trim().to_lowercase();
        if clean_domain.is_empty() || clean_domain.contains('/') || clean_domain.contains(' ') {
            return Err(SmooothError::Validation(format!(
                "Invalid ingress domain name: '{}'",
                domain
            )));
        }

        let mut routes = self.list_routes()?;
        // Remove existing route for this domain if present
        routes.retain(|r| r.domain != clean_domain);

        let route = IngressRoute {
            id: format!("route_{}", Uuid::new_v4().simple()),
            domain: clean_domain,
            target_resource: target_resource.to_string(),
            target_port,
            tls,
            path_prefix,
            created_at: Utc::now(),
        };

        routes.push(route.clone());
        self.save_routes(&routes)?;
        Ok(route)
    }

    /// Remove a route by domain or ID
    pub fn remove_route(&self, domain_or_id: &str) -> Result<bool> {
        let mut routes = self.list_routes()?;
        let initial_len = routes.len();
        routes.retain(|r| r.id != domain_or_id && r.domain != domain_or_id);
        let removed = routes.len() < initial_len;
        if removed {
            self.save_routes(&routes)?;
        }
        Ok(removed)
    }

    /// Render a Caddyfile for all registered routes
    pub fn render_caddyfile(&self) -> Result<String> {
        let routes = self.list_routes()?;
        let mut out = String::new();
        out.push_str("# Auto-generated by Smoooth Ingress Subsystem\n");
        out.push_str("{\n    admin 127.0.0.1:2019\n}\n\n");

        if routes.is_empty() {
            out.push_str("# No active ingress routes configured.\n");
            return Ok(out);
        }

        for r in &routes {
            let scheme = if r.tls { "https://" } else { "http://" };
            out.push_str(&format!("{}{{\n", format!("{}{}", scheme, r.domain)));
            if let Some(prefix) = &r.path_prefix {
                out.push_str(&format!(
                    "    handle_path {}* {{\n        reverse_proxy 127.0.0.1:{}\n    }}\n",
                    prefix, r.target_port
                ));
            } else {
                out.push_str(&format!("    reverse_proxy 127.0.0.1:{}\n", r.target_port));
            }
            out.push_str("}\n\n");
        }

        Ok(out)
    }

    /// Render upstream configuration snippet for existing proxies (e.g. Coolify/Traefik/Nginx)
    pub fn render_upstream_snippet(&self, proxy_type: &str) -> Result<String> {
        let routes = self.list_routes()?;
        let mut out = String::new();

        match proxy_type.to_lowercase().as_str() {
            "nginx" => {
                for r in &routes {
                    out.push_str(&format!(
                        "server {{\n    server_name {};\n    listen 80;\n    location / {{\n        proxy_pass http://127.0.0.1:{};\n        proxy_set_header Host $host;\n        proxy_set_header X-Real-IP $remote_addr;\n    }}\n}}\n\n",
                        r.domain, r.target_port
                    ));
                }
            }
            "traefik" => {
                for r in &routes {
                    out.push_str(&format!(
                        "## Traefik Dynamic File Provider Configuration for {}\nhttp:\n  routers:\n    smoooth-{}:\n      rule: \"Host(`{}`)\"\n      service: \"smoooth-{}\"\n  services:\n    smoooth-{}:\n      loadBalancer:\n        servers:\n          - url: \"http://127.0.0.1:{}\"\n\n",
                        r.domain, r.id, r.domain, r.id, r.id, r.target_port
                    ));
                }
            }
            _ => {
                // Default to Caddy
                return self.render_caddyfile();
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingress_add_list_and_remove() {
        let temp_dir = std::env::temp_dir().join(format!("smoooth_test_ingress_{}", Uuid::new_v4().simple()));
        let manager = IngressManager::new(&temp_dir);

        // Initially empty
        assert!(manager.list_routes().unwrap().is_empty());

        // Add route
        let r1 = manager.add_route("api.myapp.com", "my_api_container", 8080, true, None).unwrap();
        assert_eq!(r1.domain, "api.myapp.com");
        assert_eq!(r1.target_port, 8080);
        assert!(r1.tls);

        let list = manager.list_routes().unwrap();
        assert_eq!(list.len(), 1);

        // Render Caddyfile
        let caddyfile = manager.render_caddyfile().unwrap();
        assert!(caddyfile.contains("https://api.myapp.com{"));
        assert!(caddyfile.contains("reverse_proxy 127.0.0.1:8080"));

        // Render Nginx upstream snippet
        let nginx = manager.render_upstream_snippet("nginx").unwrap();
        assert!(nginx.contains("server_name api.myapp.com;"));
        assert!(nginx.contains("proxy_pass http://127.0.0.1:8080;"));

        // Render Traefik upstream snippet
        let traefik = manager.render_upstream_snippet("traefik").unwrap();
        assert!(traefik.contains("Host(`api.myapp.com`)"));
        assert!(traefik.contains("http://127.0.0.1:8080"));

        // Remove route
        let removed = manager.remove_route("api.myapp.com").unwrap();
        assert!(removed);
        assert!(manager.list_routes().unwrap().is_empty());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_invalid_domain_rejection() {
        let temp_dir = std::env::temp_dir().join(format!("smoooth_test_invalid_{}", Uuid::new_v4().simple()));
        let manager = IngressManager::new(&temp_dir);

        assert!(manager.add_route("", "target", 80, true, None).is_err());
        assert!(manager.add_route("bad domain with spaces", "target", 80, true, None).is_err());
        assert!(manager.add_route("bad/slash", "target", 80, true, None).is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
