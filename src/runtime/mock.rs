use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::adapters::HealthEvidence;
use crate::error::{Result, SmooothError};
use crate::models::workload::WorkloadSpec;
use crate::models::ResourceStatus;
use crate::runtime::{ContainerInspection, ContainerSummary, RuntimeProvider};

pub struct MockRuntimeProvider {
    containers: Mutex<HashMap<String, MockContainer>>,
}

struct MockContainer {
    name: String,
    image: String,
    is_running: bool,
    ports: HashMap<u16, u16>,
    logs: String,
}

impl MockRuntimeProvider {
    pub fn new() -> Self {
        Self {
            containers: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockRuntimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuntimeProvider for MockRuntimeProvider {
    fn runtime_name(&self) -> &str {
        "mock"
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(true)
    }

    async fn create_and_start(&self, spec: &WorkloadSpec) -> Result<String> {
        spec.validate()?;

        let mut map = self.containers.lock().unwrap();
        let mut ports = HashMap::new();
        for p in &spec.ports {
            ports.insert(p.container_port, p.host_port);
        }

        let id = format!("mock_{}", spec.name);
        map.insert(
            id.clone(),
            MockContainer {
                name: spec.name.clone(),
                image: spec.image.clone(),
                is_running: true,
                ports,
                logs: format!("Starting mock container {}\nInitialized.\n", spec.name),
            },
        );

        Ok(id)
    }

    async fn stop(&self, id: &str, _timeout_secs: u32) -> Result<()> {
        let mut map = self.containers.lock().unwrap();
        if let Some(c) = map.get_mut(id) {
            c.is_running = false;
            Ok(())
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn restart(&self, id: &str, _timeout_secs: u32) -> Result<()> {
        let mut map = self.containers.lock().unwrap();
        if let Some(c) = map.get_mut(id) {
            c.is_running = true;
            c.logs.push_str("Container restarted.\n");
            Ok(())
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn delete(&self, id: &str, _force: bool) -> Result<()> {
        let mut map = self.containers.lock().unwrap();
        if map.remove(id).is_some() {
            Ok(())
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn inspect(&self, id: &str) -> Result<ContainerInspection> {
        let map = self.containers.lock().unwrap();
        if let Some(c) = map.get(id) {
            Ok(ContainerInspection {
                id: id.to_string(),
                name: c.name.clone(),
                status: if c.is_running {
                    ResourceStatus::Active
                } else {
                    ResourceStatus::Stopped
                },
                is_running: c.is_running,
                ip_address: Some("172.18.0.2".to_string()),
                port_mappings: c.ports.clone(),
                created_at: Some("2026-01-01T00:00:00Z".to_string()),
                image: c.image.clone(),
            })
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn logs(&self, id: &str, _tail: usize) -> Result<String> {
        let map = self.containers.lock().unwrap();
        if let Some(c) = map.get(id) {
            Ok(c.logs.clone())
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn health_check(&self, id: &str) -> Result<HealthEvidence> {
        let map = self.containers.lock().unwrap();
        if let Some(c) = map.get(id) {
            Ok(HealthEvidence {
                is_healthy: c.is_running,
                latency_ms: Some(1),
                message: if c.is_running {
                    "Mock container healthy".to_string()
                } else {
                    "Mock container stopped".to_string()
                },
                details: serde_json::json!({
                    "name": c.name,
                    "running": c.is_running,
                }),
            })
        } else {
            Err(SmooothError::NotFound(format!(
                "Container '{}' not found",
                id
            )))
        }
    }

    async fn list_containers(&self, _label_filter: Option<&str>) -> Result<Vec<ContainerSummary>> {
        let map = self.containers.lock().unwrap();
        let summaries = map
            .iter()
            .map(|(k, v)| ContainerSummary {
                id: k.clone(),
                names: vec![format!("/{}", v.name)],
                image: v.image.clone(),
                state: if v.is_running {
                    "running".to_string()
                } else {
                    "exited".to_string()
                },
                status: if v.is_running {
                    "Up 1 hour".to_string()
                } else {
                    "Exited (0)".to_string()
                },
            })
            .collect();
        Ok(summaries)
    }
}
