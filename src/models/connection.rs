use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Indicates who owns and manages the lifecycle of the infrastructure resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceOwnership {
    /// Fully provisioned and managed by Smoooth (can be reconciled, stopped, destroyed when authorized)
    #[default]
    SmooothManaged,
    /// Pre-existing or third-party service (e.g. RDS, existing VPS DB, cloud SaaS).
    /// CRITICAL SAFETY GUARANTEE: Smoooth will NEVER stop, delete, or destroy external resources.
    External,
    /// User-defined container workload running on local engine
    CustomWorkload,
}

/// Representation of a reachable network endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Endpoint {
    /// Container-to-container private network address (DNS name or private IP)
    Internal { host: String, port: u16 },
    /// Loopback-bound host port for access from host or external prober
    Host { host: String, port: u16 },
    /// Remote external endpoint
    External { host: String, port: u16 },
}

/// Dynamic connection details resolved from live infrastructure or external config
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionDetails {
    pub protocol: String, // e.g. "postgres", "redis", "mongodb", "http", "tcp", "mysql"
    pub host: String,
    pub port: u16,
    pub path_or_database: Option<String>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub extra_params: HashMap<String, String>,
}

impl ConnectionDetails {
    /// Format into standard connection URI/URL
    pub fn to_uri(&self, mask_password: bool) -> String {
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) => {
                if mask_password {
                    format!("{}:[REDACTED]@", u)
                } else {
                    format!("{}:{}@", u, p)
                }
            }
            (Some(u), None) => format!("{}@", u),
            _ => String::new(),
        };

        let path = match &self.path_or_database {
            Some(db) if !db.is_empty() => {
                if db.starts_with('/') {
                    db.clone()
                } else {
                    format!("/{}", db)
                }
            }
            _ => String::new(),
        };

        let query = if self.extra_params.is_empty() {
            String::new()
        } else {
            let mut pairs: Vec<String> = self
                .extra_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            pairs.sort();
            format!("?{}", pairs.join("&"))
        };

        format!(
            "{}://{}{}:{}{}{}",
            self.protocol, auth, self.host, self.port, path, query
        )
    }

    /// Produce individual environment variables (HOST, PORT, PROTOCOL, USER, PASSWORD, DATABASE, URL)
    pub fn to_env_map(&self, prefix: Option<&str>, mask_password: bool) -> HashMap<String, String> {
        let pfx = match prefix {
            Some(p) if !p.is_empty() => format!("{}_", p.trim_end_matches('_').to_uppercase()),
            _ => String::new(),
        };

        let mut env = HashMap::new();
        env.insert(format!("{}HOST", pfx), self.host.clone());
        env.insert(format!("{}PORT", pfx), self.port.to_string());
        env.insert(format!("{}PROTOCOL", pfx), self.protocol.clone());

        if let Some(user) = &self.username {
            env.insert(format!("{}USER", pfx), user.clone());
            env.insert(format!("{}USERNAME", pfx), user.clone());
        }

        if let Some(pass) = &self.password {
            let val = if mask_password {
                "[REDACTED]".to_string()
            } else {
                pass.clone()
            };
            env.insert(format!("{}PASSWORD", pfx), val);
        }

        if let Some(db) = &self.path_or_database {
            env.insert(format!("{}DATABASE", pfx), db.clone());
            env.insert(format!("{}DB", pfx), db.clone());
        }

        let uri_var = format!("{}URL", pfx);
        env.insert(uri_var, self.to_uri(mask_password));

        env
    }
}

/// Mapping rule defining how a consumer application consumes connection details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BindingRequirement {
    /// Name or ID of the dependency resource (e.g. "my_database" or "cache")
    pub resource: String,
    /// Custom environment variable mappings:
    /// e.g. {"url": "DATABASE_URL"} or {"host": "DB_HOST", "port": "DB_PORT"}
    #[serde(default)]
    pub env_mappings: HashMap<String, String>,
    /// Optional prefix for automatic variable mapping (e.g. "PG" generates PG_HOST, PG_PORT, PG_URL)
    pub env_prefix: Option<String>,
    /// Preferred endpoint type: internal (container network) or host (loopback)
    #[serde(default)]
    pub preferred_endpoint: Option<String>,
}

/// Volume Mount Specification with read-only permission flag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeMount {
    pub volume_name: String,
    pub container_path: String,
    #[serde(default)]
    pub read_only: bool,
}

impl VolumeMount {
    pub fn new(volume_name: String, container_path: String, read_only: bool) -> Self {
        Self {
            volume_name,
            container_path,
            read_only,
        }
    }

    /// Format for Docker HostConfig binds
    pub fn to_docker_bind(&self) -> String {
        if self.read_only {
            format!("{}:{}:ro", self.volume_name, self.container_path)
        } else {
            format!("{}:{}", self.volume_name, self.container_path)
        }
    }
}
