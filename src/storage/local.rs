use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::models::{
    AgentIdentity, IdempotencyKey, IdempotencyRecord, Operation, Plan, Resource, ServiceBinding,
};
use crate::storage::{AuditEvent, Storage};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevocationEntry {
    pub capability_id: String,
    pub reason: String,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshot {
    #[serde(default)]
    pub agents: HashMap<String, AgentIdentity>,
    #[serde(default)]
    pub plans: HashMap<String, Plan>,
    #[serde(default)]
    pub idempotency: HashMap<String, IdempotencyRecord>, // key serialized as string for json compatibility
    #[serde(default)]
    pub operations: HashMap<String, Operation>,
    #[serde(default)]
    pub resources: HashMap<String, Resource>,
    #[serde(default)]
    pub bindings: HashMap<String, ServiceBinding>,
    #[serde(default)]
    pub audit_events: Vec<AuditEvent>,
    #[serde(default)]
    pub revocations: HashMap<String, RevocationEntry>,
    #[serde(default)]
    pub used_nonces: HashMap<String, chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub fencing_epochs: HashMap<String, u64>,
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
            match std::fs::read_to_string(&path_buf) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(state) => state,
                    Err(e) => {
                        eprintln!("[WARN] Primary state snapshot corrupted at {:?}: {}. Checking backup...", path_buf, e);
                        let bak_path = path_buf.with_extension("bak");
                        if bak_path.exists() {
                            std::fs::read_to_string(&bak_path)
                                .ok()
                                .and_then(|bak_data| serde_json::from_str(&bak_data).ok())
                                .unwrap_or_default()
                        } else {
                            StateSnapshot::default()
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[WARN] Failed to read state file at {:?}: {}", path_buf, e);
                    StateSnapshot::default()
                }
            }
        } else {
            StateSnapshot::default()
        };

        Self {
            snapshot_path: Some(path_buf),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    async fn persist_snapshot(&self) -> Result<()> {
        if let Some(path) = &self.snapshot_path {
            let state = self.state.read().await;
            let json = serde_json::to_string_pretty(&*state).map_err(|e| {
                crate::error::SmooothError::Storage(format!("JSON serialization failed: {}", e))
            })?;
            drop(state);

            let tmp_path = path.with_extension("tmp");
            let bak_path = path.with_extension("bak");

            // 1. Atomic write to temp file with fsync
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::File::create(&tmp_path).await.map_err(|e| {
                crate::error::SmooothError::Storage(format!(
                    "Failed to create temp snapshot: {}",
                    e
                ))
            })?;
            file.write_all(json.as_bytes()).await.map_err(|e| {
                crate::error::SmooothError::Storage(format!("Failed to write temp snapshot: {}", e))
            })?;
            file.sync_all().await.map_err(|e| {
                crate::error::SmooothError::Storage(format!("Failed to sync temp snapshot: {}", e))
            })?;
            drop(file);

            // 2. Backup existing snapshot before atomic overwrite
            if path.exists() {
                let _ = tokio::fs::copy(path, &bak_path).await;
            }

            // 3. Atomic rename temp file to primary snapshot path
            tokio::fs::rename(&tmp_path, path).await.map_err(|e| {
                crate::error::SmooothError::Storage(format!(
                    "Atomic rename failed for state snapshot: {}",
                    e
                ))
            })?;
        }
        Ok(())
    }

    fn serialize_idemp_key(key: &IdempotencyKey) -> String {
        format!(
            "{}:{}:{}",
            key.workspace_id, key.operation_type, key.client_key
        )
    }

    async fn sync_agents_from_disk(&self) {
        if let Some(path) = &self.snapshot_path {
            if path.exists() {
                if let Ok(data) = tokio::fs::read_to_string(path).await {
                    if let Ok(disk_snapshot) = serde_json::from_str::<StateSnapshot>(&data) {
                        let mut state = self.state.write().await;
                        for (k, v) in disk_snapshot.agents {
                            state.agents.insert(k, v);
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn save_agent(&self, agent: AgentIdentity) -> Result<()> {
        let mut state = self.state.write().await;
        state.agents.insert(agent.agent_id.clone(), agent);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentIdentity>> {
        self.sync_agents_from_disk().await;
        let state = self.state.read().await;
        Ok(state.agents.get(id).cloned())
    }

    async fn get_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<AgentIdentity>> {
        self.sync_agents_from_disk().await;
        let state = self.state.read().await;
        Ok(state
            .agents
            .values()
            .find(|a| a.token_hash == token_hash)
            .cloned())
    }

    async fn list_agents(&self, workspace_id: Option<&str>) -> Result<Vec<AgentIdentity>> {
        self.sync_agents_from_disk().await;
        let state = self.state.read().await;
        let list: Vec<AgentIdentity> = state
            .agents
            .values()
            .filter(|a| {
                if let Some(ws) = workspace_id {
                    a.workspace_id == "*" || a.workspace_id == ws
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        Ok(list)
    }

    async fn revoke_agent(&self, id: &str) -> Result<()> {
        self.sync_agents_from_disk().await;
        let mut state = self.state.write().await;
        if let Some(agent) = state.agents.get_mut(id) {
            agent.revoked = true;
            agent.revoked_at = Some(chrono::Utc::now());
        }
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn save_plan(&self, plan: Plan) -> Result<()> {
        let mut state = self.state.write().await;
        state.plans.insert(plan.id.clone(), plan);
        drop(state);
        self.persist_snapshot().await?;
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
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn check_or_record_idempotency(
        &self,
        record: IdempotencyRecord,
    ) -> Result<Option<IdempotencyRecord>> {
        let key_str = Self::serialize_idemp_key(&record.key);
        let mut state = self.state.write().await;
        if let Some(existing) = state.idempotency.get(&key_str) {
            if existing.request_hash != record.request_hash {
                return Err(crate::error::SmooothError::IdempotencyConflict(format!(
                    "Idempotency key '{}' reused with mismatched request hash. Prior: {}, Current: {}",
                    record.key.client_key, existing.request_hash, record.request_hash
                )));
            }
            return Ok(Some(existing.clone()));
        }
        state.idempotency.insert(key_str, record);
        drop(state);
        self.persist_snapshot().await?;
        Ok(None)
    }

    async fn save_operation(&self, operation: Operation) -> Result<()> {
        let mut state = self.state.write().await;
        state.operations.insert(operation.id.clone(), operation);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn get_operation(&self, id: &str) -> Result<Option<Operation>> {
        let state = self.state.read().await;
        Ok(state.operations.get(id).cloned())
    }

    async fn list_operations(&self, workspace_id: Option<&str>) -> Result<Vec<Operation>> {
        let state = self.state.read().await;
        let mut list: Vec<Operation> = state
            .operations
            .values()
            .filter(|op| {
                if let Some(ws) = workspace_id {
                    op.workspace_id == "*" || op.workspace_id == ws
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        // Sort descending by created_at (most recent operations first)
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        Ok(list)
    }

    async fn update_operation(&self, operation: Operation) -> Result<()> {
        let mut state = self.state.write().await;
        state.operations.insert(operation.id.clone(), operation);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn save_resource(&self, resource: Resource) -> Result<()> {
        let mut state = self.state.write().await;
        // Retain only latest resource with matching name in the same workspace
        state
            .resources
            .retain(|_, r| !(r.name == resource.name && r.workspace_id == resource.workspace_id));
        state.resources.insert(resource.id.clone(), resource);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn get_resource(&self, id: &str) -> Result<Option<Resource>> {
        let state = self.state.read().await;
        if let Some(r) = state.resources.get(id) {
            return Ok(Some(r.clone()));
        }
        // Fallback to lookup by human-readable resource name (most recently updated)
        let by_name = state
            .resources
            .values()
            .filter(|r| r.name == id)
            .max_by_key(|r| r.updated_at)
            .cloned();
        Ok(by_name)
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
        let key_to_remove = if state.resources.contains_key(id) {
            Some(id.to_string())
        } else {
            state
                .resources
                .iter()
                .find(|(_, r)| r.name == id)
                .map(|(k, _)| k.clone())
        };
        if let Some(k) = key_to_remove {
            state.resources.remove(&k);
        }
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn save_binding(&self, binding: ServiceBinding) -> Result<()> {
        let mut state = self.state.write().await;
        state.bindings.insert(binding.id.clone(), binding);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>> {
        let state = self.state.read().await;
        Ok(state.bindings.get(id).cloned())
    }

    async fn list_bindings(&self) -> Result<Vec<ServiceBinding>> {
        let state = self.state.read().await;
        Ok(state.bindings.values().cloned().collect())
    }

    async fn record_audit(&self, event: AuditEvent) -> Result<()> {
        let mut state = self.state.write().await;
        state.audit_events.push(event);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>> {
        let state = self.state.read().await;
        let mut list: Vec<AuditEvent> = state
            .audit_events
            .iter()
            .filter(|e| {
                workspace_id == "*" || workspace_id.is_empty() || e.workspace_id == workspace_id
            })
            .cloned()
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        Ok(list)
    }

    async fn record_revocation(
        &self,
        capability_id: &uuid::Uuid,
        reason: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let mut state = self.state.write().await;
        state.revocations.insert(
            capability_id.to_string(),
            RevocationEntry {
                capability_id: capability_id.to_string(),
                reason: reason.to_string(),
                revoked_at: chrono::Utc::now(),
                expires_at,
            },
        );
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn is_revoked(&self, capability_id: &uuid::Uuid) -> Result<bool> {
        let state = self.state.read().await;
        Ok(state.revocations.contains_key(&capability_id.to_string()))
    }

    async fn record_used_nonce(
        &self,
        nonce: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let mut state = self.state.write().await;
        let now = chrono::Utc::now();
        state.used_nonces.retain(|_, exp| *exp > now);
        state.used_nonces.insert(nonce.to_string(), expires_at);
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }

    async fn is_nonce_used(&self, nonce: &str) -> Result<bool> {
        let state = self.state.read().await;
        if let Some(exp) = state.used_nonces.get(nonce) {
            if *exp > chrono::Utc::now() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn get_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        let state = self.state.read().await;
        Ok(state.fencing_epochs.get(workspace_id).copied().unwrap_or(1))
    }

    async fn increment_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        let mut state = self.state.write().await;
        let epoch = state.fencing_epochs.entry(workspace_id.to_string()).or_insert(1);
        *epoch += 1;
        let new_epoch = *epoch;
        drop(state);
        self.persist_snapshot().await?;
        Ok(new_epoch)
    }

    async fn export_snapshot(&self, workspace_id: &str) -> Result<StateSnapshot> {
        let state = self.state.read().await;
        if workspace_id == "*" || workspace_id.is_empty() {
            return Ok(state.clone());
        }

        // Workspace-scoped export: filter items belonging to this workspace
        let agents: HashMap<String, AgentIdentity> = state
            .agents
            .iter()
            .filter(|(_, a)| a.workspace_id == "*" || a.workspace_id == workspace_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let plans: HashMap<String, Plan> = state
            .plans
            .iter()
            .filter(|(_, p)| p.spec.workspace_id == workspace_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let operations: HashMap<String, Operation> = state
            .operations
            .iter()
            .filter(|(_, o)| o.workspace_id == workspace_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let resources: HashMap<String, Resource> = state
            .resources
            .iter()
            .filter(|(_, r)| r.workspace_id == workspace_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let resource_keys: std::collections::HashSet<String> = resources.keys().cloned().collect();
        let bindings: HashMap<String, ServiceBinding> = state
            .bindings
            .iter()
            .filter(|(_, b)| resource_keys.contains(&b.source_resource_id) || resource_keys.contains(&b.target_resource_id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let audit_events = state
            .audit_events
            .iter()
            .filter(|e| e.workspace_id == workspace_id)
            .cloned()
            .collect();
        let mut fencing_epochs = HashMap::new();
        if let Some(epoch) = state.fencing_epochs.get(workspace_id) {
            fencing_epochs.insert(workspace_id.to_string(), *epoch);
        }

        Ok(StateSnapshot {
            agents,
            plans,
            idempotency: HashMap::new(),
            operations,
            resources,
            bindings,
            audit_events,
            revocations: state.revocations.clone(),
            used_nonces: state.used_nonces.clone(),
            fencing_epochs,
        })
    }

    async fn import_snapshot(&self, snapshot: StateSnapshot) -> Result<()> {
        let mut state = self.state.write().await;
        *state = snapshot;
        drop(state);
        self.persist_snapshot().await?;
        Ok(())
    }
}
