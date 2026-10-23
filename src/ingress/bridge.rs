use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::{Result, SmooothError};
use crate::ingress::IngressRoute;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeStatus {
    pub proxy_container: String,
    pub network: String,
    pub already_connected: bool,
    pub connected: bool,
}

pub struct ProxyBridgeManager;

impl ProxyBridgeManager {
    /// Detect known running reverse proxy containers on the local Docker host
    pub fn detect_active_proxy() -> Result<Option<String>> {
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.Names}}"])
            .output()
            .map_err(|e| SmooothError::Adapter(format!("Failed to execute 'docker ps': {}", e)))?;

        if !output.status.success() {
            return Ok(None);
        }

        let names_str = String::from_utf8_lossy(&output.stdout);
        let names: Vec<&str> = names_str.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

        // Priority list of known reverse proxies
        let known_proxies = [
            "coolify-proxy",
            "traefik",
            "traefik-proxy",
            "nginx-proxy",
            "nginx",
            "caddy",
            "haproxy",
        ];

        for kp in known_proxies {
            for name in &names {
                if name.eq_ignore_ascii_case(kp) || name.contains(kp) {
                    return Ok(Some(name.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Check if a container is attached to a specific Docker network
    pub fn is_container_on_network(container_name: &str, network_name: &str) -> Result<bool> {
        let output = Command::new("docker")
            .args([
                "inspect",
                container_name,
                "--format",
                "{{json .NetworkSettings.Networks}}",
            ])
            .output()
            .map_err(|e| SmooothError::Adapter(format!("Failed to inspect container '{}': {}", container_name, e)))?;

        if !output.status.success() {
            return Err(SmooothError::Adapter(format!(
                "Container inspect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let networks_map: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);

        if let Some(obj) = networks_map.as_object() {
            return Ok(obj.contains_key(network_name));
        }

        Ok(false)
    }

    /// Connect a proxy container to a target Docker network
    pub fn bridge_network(proxy_container: &str, target_network: &str) -> Result<BridgeStatus> {
        let already_connected = Self::is_container_on_network(proxy_container, target_network)?;
        if already_connected {
            return Ok(BridgeStatus {
                proxy_container: proxy_container.to_string(),
                network: target_network.to_string(),
                already_connected: true,
                connected: true,
            });
        }

        let output = Command::new("docker")
            .args(["network", "connect", target_network, proxy_container])
            .output()
            .map_err(|e| SmooothError::Adapter(format!("Failed to execute 'docker network connect': {}", e)))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            // Ignore if already connected race condition
            if err_msg.contains("already exists in network") {
                return Ok(BridgeStatus {
                    proxy_container: proxy_container.to_string(),
                    network: target_network.to_string(),
                    already_connected: true,
                    connected: true,
                });
            }
            return Err(SmooothError::Adapter(format!(
                "Failed to bridge network '{}' to proxy '{}': {}",
                target_network, proxy_container, err_msg
            )));
        }

        Ok(BridgeStatus {
            proxy_container: proxy_container.to_string(),
            network: target_network.to_string(),
            already_connected: false,
            connected: true,
        })
    }

    /// Automatically bridge all given networks to the detected or specified reverse proxy
    pub fn auto_bridge_networks(
        proxy_hint: Option<&str>,
        networks: &[String],
    ) -> Result<Vec<BridgeStatus>> {
        let proxy = match proxy_hint {
            Some(p) => p.to_string(),
            None => match Self::detect_active_proxy()? {
                Some(detected) => detected,
                None => return Ok(Vec::new()),
            },
        };

        let mut results = Vec::new();
        for net in networks {
            if net.is_empty() || net == "none" || net == "host" {
                continue;
            }
            match Self::bridge_network(&proxy, net) {
                Ok(status) => results.push(status),
                Err(e) => {
                    tracing::warn!("Failed bridging network '{}' to proxy '{}': {}", net, proxy, e);
                }
            }
        }

        Ok(results)
    }

    /// Render Traefik dynamic configuration YAML for file providers
    pub fn render_traefik_file_provider_yaml(routes: &[IngressRoute]) -> String {
        let mut yaml = String::new();
        yaml.push_str("## Auto-generated by Smoooth Ingress Bridge\n");
        yaml.push_str("http:\n");
        yaml.push_str("  routers:\n");

        for r in routes {
            let safe_id = r.id.replace('_', "-");
            yaml.push_str(&format!("    smoooth-http-{}:\n", safe_id));
            yaml.push_str(&format!("      entryPoints: [\"http\"]\n"));
            yaml.push_str(&format!("      rule: \"Host(`{}`)\"\n", r.domain));
            yaml.push_str(&format!("      service: \"smoooth-{}\"\n", safe_id));

            if r.tls {
                yaml.push_str(&format!("    smoooth-https-{}:\n", safe_id));
                yaml.push_str(&format!("      entryPoints: [\"https\"]\n"));
                yaml.push_str(&format!("      rule: \"Host(`{}`)\"\n", r.domain));
                yaml.push_str(&format!("      service: \"smoooth-{}\"\n", safe_id));
                yaml.push_str("      tls:\n        certResolver: letsencrypt\n");
            }
        }

        yaml.push_str("  services:\n");
        for r in routes {
            let safe_id = r.id.replace('_', "-");
            yaml.push_str(&format!("    smoooth-{}:\n", safe_id));
            yaml.push_str("      loadBalancer:\n        servers:\n");
            yaml.push_str(&format!("          - url: \"http://{}:{}\"\n", r.target_resource, r.target_port));
        }

        yaml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_render_traefik_file_provider_yaml() {
        let routes = vec![
            IngressRoute {
                id: "route_web_1".to_string(),
                domain: "app.example.com".to_string(),
                target_resource: "my_web_container".to_string(),
                target_port: 3000,
                tls: true,
                path_prefix: None,
                created_at: Utc::now(),
            },
            IngressRoute {
                id: "route_api_2".to_string(),
                domain: "api.example.com".to_string(),
                target_resource: "my_api_container".to_string(),
                target_port: 8080,
                tls: false,
                path_prefix: None,
                created_at: Utc::now(),
            },
        ];

        let yaml = ProxyBridgeManager::render_traefik_file_provider_yaml(&routes);
        assert!(yaml.contains("smoooth-http-route-web-1:"));
        assert!(yaml.contains("smoooth-https-route-web-1:"));
        assert!(yaml.contains("Host(`app.example.com`)"));
        assert!(yaml.contains("http://my_web_container:3000"));
        assert!(yaml.contains("smoooth-http-route-api-2:"));
        assert!(!yaml.contains("smoooth-https-route-api-2:")); // tls: false
    }
}
