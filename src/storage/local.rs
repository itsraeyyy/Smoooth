use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::models::{
    IdempotencyKey, IdempotencyRecord, Operation, Plan, Resource, ServiceBinding,
};
use crate::storage::{AuditEvent, Storage};

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
struct StateSnapshot {
    plans: HashMap<String, Plan>,
    idempotency: HashMap<String, IdempotencyRecord>, // key serialized as string for json compatibility
    operations: HashMap<String, Operation>,
    resources: HashMap<String, Resource>,
    bindings: HashMap<String, ServiceBinding>,
    audit_events: Vec<AuditEvent>,
}

#[derive(Clone)]
pub struct LocalStorage {
    snapshot_path: Option<PathBuf>,
    state: Arc<RwLock<StateSnapshot>>,
}

impl LocalStorage {
    pub fn new_in_memory() -> Self {
        Self {
            snapshot_path: None,
            state: Arc::new(RwLock::new(StateSnapshot::default())),
        }
    }

    pub fn new_file_backed<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let initial_state = if path_buf.exists() {
            std::fs::read_to_string(&path_buf)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        } else {
            StateSnapshot::default()
        };

        Self {
            snapshot_path: Some(path_buf),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    async fn persist_snapshot(&self) {
        if let Some(path) = &self.snapshot_path {
            let state = self.state.read().await;
            if let Ok(json) = serde_json::to_string_pretty(&*state) {
                let _ = tokio::fs::write(path, json).await;
            }
        }
    }

    fn serialize_idemp_key(key: &IdempotencyKey) -> String {
        format!("{}:{}:{}", key.workspace_id, key.operation_type, key.client_key)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn save_plan(&self, plan: Plan) -> Result<()> {
        let mut state = self.state.write().await;
        state.plans.insert(plan.id.clone(), plan);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        let state = self.state.read().await;
        Ok(state.plans.get(id).cloned())
    }

    async fn check_idempotency(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>> {
        let key_str = Self::serialize_idemp_key(key);
        let state = self.state.read().await;
        Ok(state.idempotency.get(&key_str).cloned())
    }

    async fn record_idempotency(&self, record: IdempotencyRecord) -> Result<()> {
        let key_str = Self::serialize_idemp_key(&record.key);
        let mut state = self.state.write().await;
        state.idempotency.insert(key_str, record);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn save_operation(&self, operation: Operation) -> Result<()> {
        let mut state = self.state.write().await;
        state.operations.insert(operation.id.clone(), operation);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn get_operation(&self, id: &str) -> Result<Option<Operation>> {
        let state = self.state.read().await;
        Ok(state.operations.get(id).cloned())
    }

    async fn update_operation(&self, operation: Operation) -> Result<()> {
        let mut state = self.state.write().await;
        state.operations.insert(operation.id.clone(), operation);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn save_resource(&self, resource: Resource) -> Result<()> {
        let mut state = self.state.write().await;
        state.resources.insert(resource.id.clone(), resource);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn get_resource(&self, id: &str) -> Result<Option<Resource>> {
        let state = self.state.read().await;
        Ok(state.resources.get(id).cloned())
    }

    async fn list_resources(&self, workspace_id: &str) -> Result<Vec<Resource>> {
        let state = self.state.read().await;
        Ok(state
            .resources
            .values()
            .filter(|r| r.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn delete_resource(&self, id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.resources.remove(id);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn save_binding(&self, binding: ServiceBinding) -> Result<()> {
        let mut state = self.state.write().await;
        state.bindings.insert(binding.id.clone(), binding);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>> {
        let state = self.state.read().await;
        Ok(state.bindings.get(id).cloned())
    }

    async fn record_audit(&self, event: AuditEvent) -> Result<()> {
        let mut state = self.state.write().await;
        state.audit_events.push(event);
        drop(state);
        self.persist_snapshot().await;
        Ok(())
    }

    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>> {
        let state = self.state.read().await;
        Ok(state
            .audit_events
            .iter()
            .filter(|e| e.workspace_id == workspace_id)
            .cloned()
            .collect())
    }
}
