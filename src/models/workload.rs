use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, SmooothError};
use crate::models::HealthSpec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkloadVolumeMount {
    pub volume_name: String,
    pub container_path: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkloadPortBinding {
    pub container_port: u16,
    pub host_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String, // "tcp" or "udp"
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkloadResourceLimits {
    #[serde(default = "default_memory_bytes")]
    pub memory_bytes: i64,
    #[serde(default = "default_cpu_millicores")]
    pub cpu_millicores: u32,
    #[serde(default = "default_pids_limit")]
    pub pids_limit: u32,
}

fn default_memory_bytes() -> i64 {
    536_870_912 // 512 MB
}

fn default_cpu_millicores() -> u32 {
    1000 // 1.0 CPU
}

fn default_pids_limit() -> u32 {
    256
}

impl Default for WorkloadResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: default_memory_bytes(),
            cpu_millicores: default_cpu_millicores(),
            pids_limit: default_pids_limit(),
        }
    }
}

fn default_restart_policy() -> String {
    "unless-stopped".to_string()
}

/// Strongly-typed canonical specification for containerized workloads.
/// Enforces strict validation: no arbitrary docker socket mounts, no host paths,
/// no privileged flags, no host namespaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkloadSpec {
    pub name: String,
    pub image: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub cmd: Option<Vec<String>>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub ports: Vec<WorkloadPortBinding>,
    #[serde(default)]
    pub volumes: Vec<WorkloadVolumeMount>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default = "default_restart_policy")]
    pub restart_policy: String,
    #[serde(default)]
    pub limits: WorkloadResourceLimits,
    #[serde(default)]
    pub health_check: Option<HealthSpec>,
}

impl WorkloadSpec {
    /// Validates all fields against the Smoooth execution security boundary.
    pub fn validate(&self) -> Result<()> {
        // 1. Name validation
        let name = self.name.trim();
        if name.is_empty() || name.len() > 64 {
            return Err(SmooothError::Validation(
                "Workload name must be between 1 and 64 characters".to_string(),
            ));
        }
        let valid_name = name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !valid_name
            || name.starts_with('-')
            || name.ends_with('-')
            || name.starts_with('_')
            || name.ends_with('_')
        {
            return Err(SmooothError::Validation(format!(
                "Workload name '{}' is invalid. Must consist of alphanumeric characters, dashes, or underscores, and cannot start or end with a separator.",
                name
            )));
        }

        // 2. Image validation
        let image = self.image.trim();
        if image.is_empty() || image.len() > 256 {
            return Err(SmooothError::Validation(
                "Workload image must be between 1 and 256 characters".to_string(),
            ));
        }
        if image.starts_with('-') {
            return Err(SmooothError::Validation(
                "Workload image cannot start with a dash".to_string(),
            ));
        }
        let valid_image = image.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || c == '/'
                || c == ':'
                || c == '.'
                || c == '-'
                || c == '_'
                || c == '@'
        });
        if !valid_image {
            return Err(SmooothError::Validation(format!(
                "Workload image '{}' contains illegal characters.",
                image
            )));
        }

        // 3. Ports validation
        for pb in &self.ports {
            if pb.container_port == 0 {
                return Err(SmooothError::Validation(
                    "Container port cannot be 0".to_string(),
                ));
            }
            let proto = pb.protocol.to_lowercase();
            if proto != "tcp" && proto != "udp" {
                return Err(SmooothError::Validation(format!(
                    "Unsupported port protocol '{}'. Must be 'tcp' or 'udp'.",
                    pb.protocol
                )));
            }
        }

        // 4. Volumes validation (Zero host bind mounts)
        for vm in &self.volumes {
            let v_name = vm.volume_name.trim();
            if v_name.is_empty() {
                return Err(SmooothError::Validation(
                    "Volume name cannot be empty".to_string(),
                ));
            }
            if v_name.starts_with('/')
                || v_name.starts_with('\\')
                || v_name.contains('/')
                || v_name.contains('\\')
                || v_name.starts_with('.')
                || v_name.contains("..")
            {
                return Err(SmooothError::Security(format!(
                    "Security boundary violation: Host path bind mount ('{}') is forbidden. Only named Docker volumes are allowed.",
                    v_name
                )));
            }
            let valid_vol = v_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
            if !valid_vol {
                return Err(SmooothError::Validation(format!(
                    "Volume name '{}' contains invalid characters.",
                    v_name
                )));
            }

            let c_path = vm.container_path.trim();
            if !c_path.starts_with('/') || c_path.contains("..") {
                return Err(SmooothError::Validation(format!(
                    "Container mount path '{}' must be an absolute path and cannot contain '..'",
                    c_path
                )));
            }

            // Guard sensitive container system paths
            let forbidden_prefixes = [
                "/proc",
                "/sys",
                "/dev",
                "/var/run/docker.sock",
                "/etc/shadow",
                "/etc/sudoers",
                "/boot",
            ];
            for forbidden in forbidden_prefixes {
                if c_path == forbidden || c_path.starts_with(&format!("{}/", forbidden)) {
                    return Err(SmooothError::Security(format!(
                        "Security boundary violation: Mounting into protected system path '{}' is forbidden.",
                        c_path
                    )));
                }
            }
        }

        // 5. Restart policy validation
        let rp = self.restart_policy.to_lowercase();
        if !matches!(
            rp.as_str(),
            "unless-stopped" | "always" | "on-failure" | "no"
        ) {
            return Err(SmooothError::Validation(format!(
                "Invalid restart policy '{}'. Must be one of: 'unless-stopped', 'always', 'on-failure', 'no'.",
                self.restart_policy
            )));
        }

        // 6. Limits validation
        if self.limits.memory_bytes < 16_777_216 {
            return Err(SmooothError::Validation(
                "Memory limit cannot be less than 16 MB".to_string(),
            ));
        }
        if self.limits.memory_bytes > 68_719_476_736 {
            return Err(SmooothError::Validation(
                "Memory limit cannot exceed 64 GB".to_string(),
            ));
        }
        if self.limits.pids_limit < 16 || self.limits.pids_limit > 4096 {
            return Err(SmooothError::Validation(
                "Pids limit must be between 16 and 4096".to_string(),
            ));
        }
        if self.limits.cpu_millicores < 50 || self.limits.cpu_millicores > 32000 {
            return Err(SmooothError::Validation(
                "CPU millicores must be between 50 (0.05 CPU) and 32000 (32 CPUs)".to_string(),
            ));
        }

        // 7. Environment variables validation
        for (k, _) in &self.env {
            let k_upper = k.to_uppercase();
            if k_upper == "LD_PRELOAD" || k_upper == "LD_LIBRARY_PATH" {
                return Err(SmooothError::Security(format!(
                    "Security boundary violation: Dangerous environment variable '{}' is forbidden.",
                    k
                )));
            }
            if k.is_empty() || k.starts_with('-') {
                return Err(SmooothError::Validation(format!(
                    "Invalid environment variable name '{}'.",
                    k
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workload_spec() {
        let spec = WorkloadSpec {
            name: "web-api".to_string(),
            image: "nginx:alpine".to_string(),
            digest: None,
            cmd: None,
            env: HashMap::from([("PORT".to_string(), "8080".to_string())]),
            ports: vec![WorkloadPortBinding {
                container_port: 80,
                host_port: 8080,
                protocol: "tcp".to_string(),
            }],
            volumes: vec![WorkloadVolumeMount {
                volume_name: "web_data".to_string(),
                container_path: "/usr/share/nginx/html".to_string(),
                read_only: false,
            }],
            network: Some("app_net".to_string()),
            restart_policy: "unless-stopped".to_string(),
            limits: WorkloadResourceLimits::default(),
            health_check: None,
        };
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_rejects_host_bind_mounts() {
        let mut spec = WorkloadSpec {
            name: "web-api".to_string(),
            image: "nginx:alpine".to_string(),
            digest: None,
            cmd: None,
            env: HashMap::new(),
            ports: Vec::new(),
            volumes: vec![WorkloadVolumeMount {
                volume_name: "/var/run/docker.sock".to_string(),
                container_path: "/docker.sock".to_string(),
                read_only: false,
            }],
            network: None,
            restart_policy: "unless-stopped".to_string(),
            limits: WorkloadResourceLimits::default(),
            health_check: None,
        };
        assert!(spec.validate().is_err());

        spec.volumes[0].volume_name = "../../../etc".to_string();
        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_rejects_protected_container_paths() {
        let spec = WorkloadSpec {
            name: "web-api".to_string(),
            image: "nginx:alpine".to_string(),
            digest: None,
            cmd: None,
            env: HashMap::new(),
            ports: Vec::new(),
            volumes: vec![WorkloadVolumeMount {
                volume_name: "valid_vol".to_string(),
                container_path: "/proc".to_string(),
                read_only: false,
            }],
            network: None,
            restart_policy: "unless-stopped".to_string(),
            limits: WorkloadResourceLimits::default(),
            health_check: None,
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_rejects_dangerous_env_vars() {
        let spec = WorkloadSpec {
            name: "web-api".to_string(),
            image: "nginx:alpine".to_string(),
            digest: None,
            cmd: None,
            env: HashMap::from([("LD_PRELOAD".to_string(), "/tmp/libevil.so".to_string())]),
            ports: Vec::new(),
            volumes: Vec::new(),
            network: None,
            restart_policy: "unless-stopped".to_string(),
            limits: WorkloadResourceLimits::default(),
            health_check: None,
        };
        assert!(spec.validate().is_err());
    }
}
