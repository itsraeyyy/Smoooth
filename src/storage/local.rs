use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::models::{
    AgentIdentity, IdempotencyKey, IdempotencyRecord, Operation, Plan, Resource, ServiceBinding,
};
use crate::storage::{AuditEvent, OperationInitOutcome, Storage};

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
    pub version: u64,
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

pub struct FileLock {
    #[cfg(windows)]
    _file: std::fs::File,
    #[cfg(unix)]
    _file: std::fs::File,
}

impl FileLock {
    pub async fn acquire(lock_path: &Path, timeout: std::time::Duration) -> Result<Self> {
        let start = std::time::Instant::now();
        loop {
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                let res = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .share_mode(0) // EXCLUSIVE: denies other processes or handles from opening the file
                    .open(lock_path);

                match res {
                    Ok(file) => return Ok(Self { _file: file }),
                    Err(e)
                        if e.raw_os_error() == Some(32)
                            || e.kind() == std::io::ErrorKind::PermissionDenied =>
                    {
                        // File is currently locked exclusively by another live process/handle
                    }
                    Err(e) => {
                        return Err(crate::error::SmooothError::Storage(format!(
                            "Failed to open lock file at {:?}: {}",
                            lock_path, e
                        )));
                    }
                }
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                use std::os::unix::io::AsRawFd;

                let mut open_opts = std::fs::OpenOptions::new();
                open_opts
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false);
                open_opts.mode(0o600);

                match open_opts.open(lock_path) {
                    Ok(file) => {
                        let _ = crate::security::crypto::SecretStore::restrict_file_permissions(
                            lock_path,
                        );
                        let fd = file.as_raw_fd();
                        let res = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
                        if res == 0 {
                            return Ok(Self { _file: file });
                        }
                        let err = std::io::Error::last_os_error();
                        if err.raw_os_error() != Some(libc::EWOULDBLOCK)
                            && err.raw_os_error() != Some(libc::EAGAIN)
                        {
                            return Err(crate::error::SmooothError::Storage(format!(
                                "flock failed on {:?}: {}",
                                lock_path, err
                            )));
                        }
                    }
                    Err(e) => {
                        return Err(crate::error::SmooothError::Storage(format!(
                            "Failed to open lock file at {:?}: {}",
                            lock_path, e
                        )));
                    }
                }
            }

            if start.elapsed() >= timeout {
                return Err(crate::error::SmooothError::Storage(format!(
                    "Timed out after {:?} waiting for exclusive storage lock at {:?}",
                    timeout, lock_path
                )));
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

#[cfg(unix)]
impl Drop for FileLock {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        unsafe {
            libc::flock(self._file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

#[derive(Clone)]
pub struct LocalStorage {
    snapshot_path: Option<PathBuf>,
    state: Arc<RwLock<StateSnapshot>>,
    last_synced_mtime: Arc<RwLock<Option<std::time::SystemTime>>>,
}

impl LocalStorage {
    pub fn new_in_memory() -> Self {
        Self {
            snapshot_path: None,
            state: Arc::new(RwLock::new(StateSnapshot::default())),
            last_synced_mtime: Arc::new(RwLock::new(None)),
        }
    }

    pub fn canonicalize_storage_path(path: &Path) -> PathBuf {
        if let Ok(canon) = std::fs::canonicalize(path) {
            return canon;
        }
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(path),
                Err(_) => path.to_path_buf(),
            }
        };
        if let Some(parent) = abs_path.parent() {
            if parent.exists() {
                if let Ok(canon_parent) = std::fs::canonicalize(parent) {
                    if let Some(file_name) = abs_path.file_name() {
                        return canon_parent.join(file_name);
                    }
                }
            }
        }
        abs_path
    }

    pub fn new_file_backed<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = Self::canonicalize_storage_path(path.as_ref());
        let initial_mtime = if path_buf.exists() {
            std::fs::metadata(&path_buf)
                .ok()
                .and_then(|m| m.modified().ok())
        } else {
            None
        };
        let initial_state = if path_buf.exists() {
            // Restrict permissions on existing state and backup files on Unix
            #[cfg(unix)]
            {
                let _ = crate::security::crypto::SecretStore::restrict_file_permissions(&path_buf);
                let bak_path = path_buf.with_extension("bak");
                if bak_path.exists() {
                    let _ =
                        crate::security::crypto::SecretStore::restrict_file_permissions(&bak_path);
                }
            }
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
            last_synced_mtime: Arc::new(RwLock::new(initial_mtime)),
        }
    }

    pub async fn persist_snapshot(&self) -> Result<()> {
        self.mutate_and_persist(|_| Ok(((), true))).await
    }

    async fn write_snapshot_atomic(path: &Path, snapshot: &StateSnapshot) -> Result<()> {
        let json = serde_json::to_string_pretty(snapshot).map_err(|e| {
            crate::error::SmooothError::Storage(format!("JSON serialization failed: {}", e))
        })?;

        let rand_suffix = uuid::Uuid::new_v4().simple();
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("smoooth_state");
        let tmp_filename = format!("{}.{}.tmp", file_stem, rand_suffix);
        let tmp_path = path.with_file_name(tmp_filename);
        let bak_path = path.with_extension("bak");

        // 1. Atomic write to temporary file with exclusive creation and 0600 permissions
        use tokio::io::AsyncWriteExt;
        let mut open_opts = tokio::fs::OpenOptions::new();
        open_opts.write(true).create_new(true);
        #[cfg(unix)]
        open_opts.mode(0o600);

        let mut file = open_opts.open(&tmp_path).await.map_err(|e| {
            crate::error::SmooothError::Storage(format!(
                "Failed to create secure temp snapshot at {:?}: {}",
                tmp_path, e
            ))
        })?;

        #[cfg(unix)]
        {
            let _ = crate::security::crypto::SecretStore::restrict_file_permissions(&tmp_path);
        }

        if let Err(e) = file.write_all(json.as_bytes()).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(crate::error::SmooothError::Storage(format!(
                "Failed to write temp snapshot: {}",
                e
            )));
        }

        if let Err(e) = file.sync_all().await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(crate::error::SmooothError::Storage(format!(
                "Failed to sync temp snapshot: {}",
                e
            )));
        }
        drop(file);

        // 2. Backup existing snapshot before atomic overwrite and enforce 0600
        if path.exists() {
            let _ = tokio::fs::copy(path, &bak_path).await;
            #[cfg(unix)]
            {
                let _ = crate::security::crypto::SecretStore::restrict_file_permissions(&bak_path);
            }
        }

        // 3. Atomic rename temp file to primary snapshot path
        if let Err(e) = tokio::fs::rename(&tmp_path, path).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(crate::error::SmooothError::Storage(format!(
                "Atomic rename failed for state snapshot: {}",
                e
            )));
        }

        // 4. Ensure final primary snapshot path has 0600 permissions
        #[cfg(unix)]
        {
            let _ = crate::security::crypto::SecretStore::restrict_file_permissions(path);
        }

        Ok(())
    }

    async fn mutate_and_persist<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut StateSnapshot) -> Result<(R, bool)>,
    {
        let Some(path) = &self.snapshot_path else {
            let mut state = self.state.write().await;
            let (result, _) = f(&mut state)?;
            return Ok(result);
        };

        // 1. Check maintenance sentinel before acquiring lock
        let maintenance_path = path.with_extension("maintenance");
        if maintenance_path.exists() {
            return Err(crate::error::SmooothError::Storage(
                "Storage is frozen for maintenance: maintenance sentinel file present".to_string(),
            ));
        }

        // 2. Acquire cross-process exclusive lock covering the entire RMW cycle
        let lock_path = path.with_extension("lock");
        let _file_lock = FileLock::acquire(&lock_path, std::time::Duration::from_secs(10)).await?;

        // Double-check maintenance sentinel while holding lock
        if maintenance_path.exists() {
            return Err(crate::error::SmooothError::Storage(
                "Storage is frozen for maintenance: maintenance sentinel file present".to_string(),
            ));
        }

        // 3. Reload latest persisted snapshot from disk while holding the lock
        let mut snapshot = if path.exists() {
            match tokio::fs::read_to_string(path).await {
                Ok(data) => serde_json::from_str::<StateSnapshot>(&data).map_err(|e| {
                    crate::error::SmooothError::Storage(format!(
                        "Corrupted state snapshot at {:?}: {}",
                        path, e
                    ))
                })?,
                Err(e) => {
                    return Err(crate::error::SmooothError::Storage(format!(
                        "Failed to read state snapshot at {:?}: {}",
                        path, e
                    )));
                }
            }
        } else {
            let current = self.state.read().await;
            current.clone()
        };

        // 4. Apply the requested mutation to the latest snapshot
        let (result, modified) = f(&mut snapshot)?;

        // 5. If state was modified, persist atomically to disk
        if modified {
            // Triple-check maintenance sentinel immediately before writing to disk
            if maintenance_path.exists() {
                return Err(crate::error::SmooothError::Storage(
                    "Storage is frozen for maintenance: maintenance sentinel file present before commit".to_string(),
                ));
            }

            snapshot.version += 1;
            Self::write_snapshot_atomic(path, &snapshot).await?;

            // 6. Update process's in-memory state ONLY after successful persistence
            let mut mem_state = self.state.write().await;
            *mem_state = snapshot;

            // Track latest mtime in memory
            if let Ok(meta) = tokio::fs::metadata(path).await {
                if let Ok(mtime) = meta.modified() {
                    let mut last = self.last_synced_mtime.write().await;
                    *last = Some(mtime);
                }
            }
        }

        // 7. Lock released on _file_lock drop
        Ok(result)
    }

    /// Enters maintenance mode:
    /// 1. Drops a `.maintenance` sentinel file to immediately fail-closed all subsequent mutations.
    /// 2. Acquires the cross-process exclusive `FileLock`, ensuring any in-flight mutation that passed
    ///    its under-lock check finishes its execution and releases the lock before this function returns.
    ///
    /// While the returned `FileLock` is held, zero mutations can execute, and all mutation attempts fail closed.
    pub async fn acquire_maintenance_lock(
        path: &Path,
        timeout: std::time::Duration,
    ) -> Result<FileLock> {
        let maintenance_path = path.with_extension("maintenance");

        // 1. Atomically drop maintenance sentinel
        tokio::fs::write(&maintenance_path, b"MAINTENANCE_ACTIVE\n")
            .await
            .map_err(|e| {
                crate::error::SmooothError::Storage(format!(
                    "Failed to create maintenance sentinel at {:?}: {}",
                    maintenance_path, e
                ))
            })?;

        // 2. Acquire exclusive lock to drain any active in-flight mutation
        let lock_path = path.with_extension("lock");
        match FileLock::acquire(&lock_path, timeout).await {
            Ok(lock) => Ok(lock),
            Err(e) => {
                let _ = tokio::fs::remove_file(&maintenance_path).await;
                Err(e)
            }
        }
    }

    /// Releases maintenance mode:
    /// 1. Deletes `.maintenance` sentinel files.
    /// 2. Drops the exclusive `FileLock`, reopening storage for normal mutations.
    pub async fn release_maintenance_lock(path: &Path, lock: FileLock) -> Result<()> {
        let maintenance_path = path.with_extension("maintenance");
        let _ = tokio::fs::remove_file(&maintenance_path).await;
        drop(lock);
        Ok(())
    }

    pub async fn sync_from_disk(&self) -> Result<()> {
        let Some(path) = &self.snapshot_path else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }

        // Direct disk read: version ordering is the sole correctness mechanism.
        // Mtime caching is strictly diagnostic and cannot suppress reading a newer version.
        if let Ok(data) = tokio::fs::read_to_string(path).await {
            if let Ok(disk_snapshot) = serde_json::from_str::<StateSnapshot>(&data) {
                let current_mtime = tokio::fs::metadata(path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());

                let mut state = self.state.write().await;
                // Strict monotonic guard against stale snapshot overwriting newer memory state:
                if disk_snapshot.version > state.version {
                    *state = disk_snapshot;
                    if let Some(mtime) = current_mtime {
                        let mut last = self.last_synced_mtime.write().await;
                        *last = Some(mtime);
                    }
                }
            }
        }
        Ok(())
    }

    fn serialize_idemp_key(key: &IdempotencyKey) -> String {
        format!(
            "{}:{}:{}",
            key.workspace_id, key.operation_type, key.client_key
        )
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn save_agent(&self, agent: AgentIdentity) -> Result<()> {
        self.mutate_and_persist(|state| {
            state.agents.insert(agent.agent_id.clone(), agent);
            Ok(((), true))
        })
        .await
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentIdentity>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.agents.get(id).cloned())
    }

    async fn get_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<AgentIdentity>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state
            .agents
            .values()
            .find(|a| a.token_hash == token_hash)
            .cloned())
    }

    async fn list_agents(&self, workspace_id: Option<&str>) -> Result<Vec<AgentIdentity>> {
        let _ = self.sync_from_disk().await;
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
        self.mutate_and_persist(|state| {
            if let Some(agent) = state.agents.get_mut(id) {
                agent.revoked = true;
                agent.revoked_at = Some(chrono::Utc::now());
                Ok(((), true))
            } else {
                Ok(((), false))
            }
        })
        .await
    }

    async fn save_plan(&self, plan: Plan) -> Result<()> {
        self.mutate_and_persist(|state| {
            state.plans.insert(plan.id.clone(), plan);
            Ok(((), true))
        })
        .await
    }

    async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.plans.get(id).cloned())
    }

    async fn list_plans(&self) -> Result<Vec<Plan>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.plans.values().cloned().collect())
    }

    async fn delete_plan(&self, id: &str) -> Result<()> {
        self.mutate_and_persist(|state| {
            let removed = state.plans.remove(id).is_some();
            Ok(((), removed))
        })
        .await
    }

    async fn check_idempotency(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>> {
        let _ = self.sync_from_disk().await;
        let key_str = Self::serialize_idemp_key(key);
        let state = self.state.read().await;
        Ok(state.idempotency.get(&key_str).cloned())
    }

    async fn record_idempotency(&self, record: IdempotencyRecord) -> Result<()> {
        let key_str = Self::serialize_idemp_key(&record.key);
        self.mutate_and_persist(|state| {
            state.idempotency.insert(key_str, record);
            Ok(((), true))
        })
        .await
    }

    async fn check_or_record_idempotency(
        &self,
        record: IdempotencyRecord,
    ) -> Result<Option<IdempotencyRecord>> {
        let key_str = Self::serialize_idemp_key(&record.key);
        self.mutate_and_persist(|state| {
            if let Some(existing) = state.idempotency.get(&key_str) {
                if existing.request_hash != record.request_hash {
                    return Err(crate::error::SmooothError::IdempotencyConflict(format!(
                        "Idempotency key '{}' reused with mismatched request hash. Prior: {}, Current: {}",
                        record.key.client_key, existing.request_hash, record.request_hash
                    )));
                }
                return Ok((Some(existing.clone()), false));
            }
            state.idempotency.insert(key_str, record);
            Ok((None, true))
        })
        .await
    }

    async fn reserve_or_get_operation(
        &self,
        idempotency_record: IdempotencyRecord,
        operation: Operation,
    ) -> Result<OperationInitOutcome> {
        let key_str = Self::serialize_idemp_key(&idempotency_record.key);
        self.mutate_and_persist(|state| {
            if let Some(existing) = state.idempotency.get(&key_str) {
                if existing.request_hash != idempotency_record.request_hash {
                    return Err(crate::error::SmooothError::IdempotencyConflict(format!(
                        "Idempotency key '{}' reused with mismatched request hash. Prior: {}, Current: {}",
                        idempotency_record.key.client_key, existing.request_hash, idempotency_record.request_hash
                    )));
                }
                if let Some(existing_op) = state.operations.get(&existing.operation_id) {
                    return Ok((OperationInitOutcome::Existing(existing_op.clone()), false));
                } else {
                    return Err(crate::error::SmooothError::Storage(format!(
                        "Inconsistent state: idempotency record references nonexistent operation '{}'",
                        existing.operation_id
                    )));
                }
            }

            state.idempotency.insert(key_str, idempotency_record);
            state
                .operations
                .insert(operation.id.clone(), operation.clone());
            Ok((OperationInitOutcome::Created(operation), true))
        })
        .await
    }

    async fn save_operation(&self, operation: Operation) -> Result<()> {
        self.mutate_and_persist(|state| {
            state.operations.insert(operation.id.clone(), operation);
            Ok(((), true))
        })
        .await
    }

    async fn get_operation(&self, id: &str) -> Result<Option<Operation>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.operations.get(id).cloned())
    }

    async fn list_operations(&self, workspace_id: Option<&str>) -> Result<Vec<Operation>> {
        let _ = self.sync_from_disk().await;
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
        self.mutate_and_persist(|state| {
            state.operations.insert(operation.id.clone(), operation);
            Ok(((), true))
        })
        .await
    }

    async fn save_resource(&self, mut resource: Resource) -> Result<Resource> {
        self.mutate_and_persist(|state| {
            // Canonical resource ID retention on (workspace_id, name) collision
            let existing_by_name = state
                .resources
                .values()
                .find(|r| r.workspace_id == resource.workspace_id && r.name == resource.name)
                .map(|r| (r.id.clone(), r.created_at));

            let existing_by_id = if existing_by_name.is_none() {
                state
                    .resources
                    .get(&resource.id)
                    .map(|r| (r.id.clone(), r.created_at))
            } else {
                None
            };

            if let Some((canonical_id, created_at)) = existing_by_name.or(existing_by_id) {
                resource.id = canonical_id.clone();
                resource.created_at = created_at;
                resource.updated_at = chrono::Utc::now();
                state.resources.insert(canonical_id, resource.clone());
            } else {
                if resource.created_at == chrono::DateTime::<chrono::Utc>::default() {
                    resource.created_at = chrono::Utc::now();
                }
                resource.updated_at = chrono::Utc::now();
                state
                    .resources
                    .insert(resource.id.clone(), resource.clone());
            }
            Ok((resource, true))
        })
        .await
    }

    async fn get_resource(&self, id: &str) -> Result<Option<Resource>> {
        let _ = self.sync_from_disk().await;
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
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state
            .resources
            .values()
            .filter(|r| r.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn delete_resource(&self, id: &str) -> Result<()> {
        self.mutate_and_persist(|state| {
            let key_to_remove = if state.resources.contains_key(id) {
                Some(id.to_string())
            } else {
                state
                    .resources
                    .iter()
                    .find(|(_, r)| r.name == id)
                    .map(|(k, _)| k.clone())
            };
            let removed = if let Some(k) = key_to_remove {
                state.resources.remove(&k).is_some()
            } else {
                false
            };
            Ok(((), removed))
        })
        .await
    }

    async fn save_binding(&self, binding: ServiceBinding) -> Result<()> {
        self.mutate_and_persist(|state| {
            if binding.source_resource_id == binding.target_resource_id {
                return Err(crate::error::SmooothError::Validation(
                    "Self-binding prohibited: source and target resources must differ".to_string(),
                ));
            }
            if let Some(src) = state.resources.get(&binding.source_resource_id) {
                if let Some(tgt) = state.resources.get(&binding.target_resource_id) {
                    if src.workspace_id != tgt.workspace_id {
                        return Err(crate::error::SmooothError::Security(format!(
                            "Cross-workspace binding prohibited: source workspace '{}' vs target workspace '{}'",
                            src.workspace_id, tgt.workspace_id
                        )));
                    }
                }
            }
            state.bindings.insert(binding.id.clone(), binding);
            Ok(((), true))
        })
        .await
    }

    async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.bindings.get(id).cloned())
    }

    async fn list_bindings(&self) -> Result<Vec<ServiceBinding>> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.bindings.values().cloned().collect())
    }

    async fn rotate_service_bindings_batch(
        &self,
        expected_old_fingerprint: &str,
        new_fingerprint: &str,
        updated_bindings: Vec<ServiceBinding>,
        audit_event: AuditEvent,
    ) -> Result<usize> {
        self.mutate_and_persist(|state| {
            for binding in &updated_bindings {
                if let Some(existing) = state.bindings.get(&binding.id) {
                    if !existing.key_fingerprint.is_empty()
                        && !expected_old_fingerprint.is_empty()
                        && existing.key_fingerprint != expected_old_fingerprint
                    {
                        return Err(crate::error::SmooothError::Storage(format!(
                            "Binding '{}' fingerprint mismatch: expected '{}', found '{}'",
                            binding.id, expected_old_fingerprint, existing.key_fingerprint
                        )));
                    }
                }
                if binding.key_fingerprint != new_fingerprint {
                    return Err(crate::error::SmooothError::Storage(format!(
                        "Updated binding '{}' has incorrect new fingerprint: expected '{}', found '{}'",
                        binding.id, new_fingerprint, binding.key_fingerprint
                    )));
                }
            }

            let count = updated_bindings.len();
            for binding in updated_bindings {
                state.bindings.insert(binding.id.clone(), binding);
            }
            state.audit_events.push(audit_event);
            Ok((count, true))
        })
        .await
    }

    async fn record_audit(&self, event: AuditEvent) -> Result<()> {
        self.mutate_and_persist(|state| {
            state.audit_events.push(event);
            Ok(((), true))
        })
        .await
    }

    async fn list_audit_events(&self, workspace_id: &str) -> Result<Vec<AuditEvent>> {
        let _ = self.sync_from_disk().await;
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
        self.mutate_and_persist(|state| {
            state.revocations.insert(
                capability_id.to_string(),
                RevocationEntry {
                    capability_id: capability_id.to_string(),
                    reason: reason.to_string(),
                    revoked_at: chrono::Utc::now(),
                    expires_at,
                },
            );
            Ok(((), true))
        })
        .await
    }

    async fn is_revoked(&self, capability_id: &uuid::Uuid) -> Result<bool> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.revocations.contains_key(&capability_id.to_string()))
    }

    async fn record_used_nonce(
        &self,
        nonce: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        self.mutate_and_persist(|state| {
            state.used_nonces.retain(|_, exp| *exp > now);
            state.used_nonces.insert(nonce.to_string(), expires_at);
            Ok(((), true))
        })
        .await
    }

    async fn is_nonce_used(&self, nonce: &str) -> Result<bool> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        if let Some(exp) = state.used_nonces.get(nonce) {
            if *exp > chrono::Utc::now() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn consume_nonce(
        &self,
        nonce: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool> {
        let now = chrono::Utc::now();
        if expires_at <= now {
            return Ok(false);
        }

        self.mutate_and_persist(|state| {
            state.used_nonces.retain(|_, exp| *exp > now);
            if state.used_nonces.contains_key(nonce) {
                return Ok((false, false));
            }
            state.used_nonces.insert(nonce.to_string(), expires_at);
            Ok((true, true))
        })
        .await
    }

    async fn get_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        let _ = self.sync_from_disk().await;
        let state = self.state.read().await;
        Ok(state.fencing_epochs.get(workspace_id).copied().unwrap_or(1))
    }

    async fn increment_fencing_epoch(&self, workspace_id: &str) -> Result<u64> {
        self.mutate_and_persist(|state| {
            let epoch = state
                .fencing_epochs
                .entry(workspace_id.to_string())
                .or_insert(1);
            *epoch += 1;
            Ok((*epoch, true))
        })
        .await
    }

    async fn export_snapshot(&self, workspace_id: &str) -> Result<StateSnapshot> {
        let _ = self.sync_from_disk().await;
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
            .filter(|(_, b)| {
                resource_keys.contains(&b.source_resource_id)
                    || resource_keys.contains(&b.target_resource_id)
            })
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
            version: state.version,
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
        self.mutate_and_persist(|state| {
            let target_version = state.version.max(snapshot.version);
            *state = snapshot;
            state.version = target_version;
            Ok(((), true))
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_newly_persisted_state_snapshot_has_0600_permissions_on_unix() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_state_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);
        let agent = AgentIdentity {
            agent_id: "agent_perm_test".to_string(),
            name: "Permission Test Agent".to_string(),
            token_hash: "hash123".to_string(),
            token_prefix: "smoooth_ag_test".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-default".to_string(),
        };

        storage
            .save_agent(agent)
            .await
            .expect("save_agent must succeed");

        assert!(state_path.exists(), "State file must exist after persist");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&state_path).expect("metadata should exist");
            let mode = meta.permissions().mode() & 0o777;
            assert_eq!(
                mode, 0o600,
                "Persisted state file must have mode 0600 on Unix"
            );
        }

        let _ = std::fs::remove_file(&state_path);
        let bak_path = state_path.with_extension("bak");
        if bak_path.exists() {
            let _ = std::fs::remove_file(&bak_path);
        }
    }

    #[tokio::test]
    async fn test_existing_state_files_safely_restricted_on_load() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_state_wide_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        std::fs::write(&state_path, r#"{"agents":{},"plans":{},"resources":{}}"#)
            .expect("write state file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&state_path, std::fs::Permissions::from_mode(0o666))
                .expect("set broad permissions");
            let initial_mode = std::fs::metadata(&state_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                initial_mode, 0o666,
                "Setup must establish broad permissions"
            );
        }

        let _storage = LocalStorage::new_file_backed(&state_path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let final_mode = std::fs::metadata(&state_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                final_mode, 0o600,
                "Existing state file with 0666 must be restricted to 0600 on load on Unix"
            );
        }

        let _ = std::fs::remove_file(&state_path);
    }

    #[tokio::test]
    async fn test_atomic_persistence_preserves_content_across_reloads() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_atomic_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);

        for i in 0..5 {
            let agent = AgentIdentity {
                agent_id: format!("agent_multi_{}", i),
                name: format!("Multi Test Agent {}", i),
                token_hash: format!("hash_{}", i),
                token_prefix: format!("smoooth_ag_{}", i),
                scopes: vec![crate::models::AgentScope::Plan],
                created_at: chrono::Utc::now(),
                expires_at: None,
                revoked: false,
                revoked_at: None,
                last_used_at: None,
                workspace_id: "ws-atomic-test".to_string(),
            };
            storage
                .save_agent(agent)
                .await
                .expect("save_agent must succeed");
        }

        // Verify that atomic rename completed and left no temporary files in the directory
        let parent = state_path.parent().unwrap();
        let mut tmp_files = 0;
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("smoooth_test_atomic_") && name.ends_with(".tmp") {
                    tmp_files += 1;
                }
            }
        }
        assert_eq!(
            tmp_files, 0,
            "No temporary snapshot files must be left behind after atomic rename"
        );

        // Reload from disk in a fresh LocalStorage instance
        let reload_storage = LocalStorage::new_file_backed(&state_path);
        let list = reload_storage
            .list_agents(Some("ws-atomic-test"))
            .await
            .expect("list_agents");
        assert_eq!(
            list.len(),
            5,
            "All 5 agents must survive atomic persistence and reload"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&state_path).expect("metadata should exist");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }

        let _ = std::fs::remove_file(&state_path);
        let bak_path = state_path.with_extension("bak");
        if bak_path.exists() {
            let _ = std::fs::remove_file(&bak_path);
        }
    }

    #[tokio::test]
    async fn test_backup_file_created_with_0600_permissions_on_unix() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_bak_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);

        // 1. First save creates primary state snapshot
        let agent1 = AgentIdentity {
            agent_id: "agent_first".to_string(),
            name: "First Agent".to_string(),
            token_hash: "hash_first".to_string(),
            token_prefix: "smoooth_ag_first".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-bak-test".to_string(),
        };
        storage.save_agent(agent1).await.expect("first save");

        // 2. Second save triggers copy of existing primary state to .bak
        let agent2 = AgentIdentity {
            agent_id: "agent_second".to_string(),
            name: "Second Agent".to_string(),
            token_hash: "hash_second".to_string(),
            token_prefix: "smoooth_ag_secnd".to_string(),
            scopes: vec![crate::models::AgentScope::Plan],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-bak-test".to_string(),
        };
        storage.save_agent(agent2).await.expect("second save");

        let bak_path = state_path.with_extension("bak");
        assert!(
            bak_path.exists(),
            "Backup file must exist after second save"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&bak_path).expect("bak metadata should exist");
            let mode = meta.permissions().mode() & 0o777;
            assert_eq!(
                mode, 0o600,
                "Backup state file (.bak) must have mode 0600 on Unix"
            );
        }

        let _ = std::fs::remove_file(&state_path);
        if bak_path.exists() {
            let _ = std::fs::remove_file(&bak_path);
        }
    }

    #[tokio::test]
    async fn test_reserve_or_get_operation_atomic_and_idempotent() {
        let storage = LocalStorage::new_in_memory();

        let idemp_key = IdempotencyKey {
            workspace_id: "ws-test".to_string(),
            operation_type: "execute_plan".to_string(),
            client_key: "client-key-123".to_string(),
        };

        let op = Operation {
            id: "op-123".to_string(),
            workspace_id: "ws-test".to_string(),
            operation_type: "execute_plan".to_string(),
            plan_id: "plan-123".to_string(),
            plan_hash: "hash-abc".to_string(),
            request_hash: "req-hash-1".to_string(),
            idempotency_client_key: "client-key-123".to_string(),
            status: crate::models::OperationStatus::Running,
            progress_percent: 0,
            lease_epoch: 1,
            lease_owner: Some("worker-1".to_string()),
            lease_expires_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
            intended_state: serde_json::json!({}),
            observed_state: serde_json::json!({}),
            logs: vec![],
            evidence: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let record = IdempotencyRecord {
            key: idemp_key.clone(),
            request_hash: "req-hash-1".to_string(),
            operation_id: "op-123".to_string(),
            created_at: chrono::Utc::now(),
        };

        // 1. Initial reservation creates the operation
        let outcome = storage
            .reserve_or_get_operation(record.clone(), op.clone())
            .await
            .expect("initial reservation must succeed");
        assert_eq!(outcome, OperationInitOutcome::Created(op.clone()));

        // 2. Exact replay returns Existing(op)
        let replay_outcome = storage
            .reserve_or_get_operation(record.clone(), op.clone())
            .await
            .expect("replay reservation must succeed");
        assert_eq!(replay_outcome, OperationInitOutcome::Existing(op.clone()));

        // 3. Mismatched request hash produces IdempotencyConflict
        let conflicting_record = IdempotencyRecord {
            key: idemp_key.clone(),
            request_hash: "mismatched-req-hash".to_string(),
            operation_id: "op-conflict".to_string(),
            created_at: chrono::Utc::now(),
        };
        let conflict_err = storage
            .reserve_or_get_operation(conflicting_record, op.clone())
            .await
            .unwrap_err();
        match conflict_err {
            crate::error::SmooothError::IdempotencyConflict(_) => {}
            other => panic!("Expected IdempotencyConflict, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_consume_nonce_atomic_and_replay_protected() {
        let storage = LocalStorage::new_in_memory();
        let nonce = "unique-test-nonce-1";
        let future_exp = chrono::Utc::now() + chrono::Duration::seconds(60);

        // 1. First consume succeeds
        let first = storage
            .consume_nonce(nonce, future_exp)
            .await
            .expect("first consume");
        assert!(first, "First nonce consume must return true");

        // 2. Immediate replay is rejected
        let second = storage
            .consume_nonce(nonce, future_exp)
            .await
            .expect("second consume");
        assert!(!second, "Replayed nonce consume must return false");

        // 3. Expired nonce is rejected before touching storage
        let expired_nonce = "expired-test-nonce";
        let past_exp = chrono::Utc::now() - chrono::Duration::seconds(10);
        let expired_result = storage
            .consume_nonce(expired_nonce, past_exp)
            .await
            .expect("expired consume");
        assert!(!expired_result, "Expired nonce must return false");
    }

    #[tokio::test]
    async fn test_save_resource_canonical_identity_preservation() {
        let storage = LocalStorage::new_in_memory();

        let initial_res = Resource {
            id: "res_ws-prod_db".to_string(),
            workspace_id: "ws-prod".to_string(),
            name: "prod-postgres".to_string(),
            service_type: "database".to_string(),
            provider: "postgres".to_string(),
            is_persistent: true,
            ownership: crate::models::ResourceOwnership::SmooothManaged,
            status: crate::models::ResourceStatus::Active,
            configuration: serde_json::json!({"port": 5432}),
            created_at: chrono::Utc::now() - chrono::Duration::hours(2),
            updated_at: chrono::Utc::now() - chrono::Duration::hours(2),
        };

        let saved1 = storage
            .save_resource(initial_res.clone())
            .await
            .expect("save initial");
        assert_eq!(saved1.id, "res_ws-prod_db");

        // Now an adapter provisions an updated resource with the same (workspace_id, name)
        // but with a new surrogate UUID id:
        let updated_res = Resource {
            id: "surrogate-uuid-99999".to_string(),
            workspace_id: "ws-prod".to_string(),
            name: "prod-postgres".to_string(),
            service_type: "database".to_string(),
            provider: "postgres".to_string(),
            is_persistent: true,
            ownership: crate::models::ResourceOwnership::SmooothManaged,
            status: crate::models::ResourceStatus::Active,
            configuration: serde_json::json!({"port": 5433}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let saved2 = storage
            .save_resource(updated_res)
            .await
            .expect("save updated");
        // Must retain canonical id!
        assert_eq!(
            saved2.id, "res_ws-prod_db",
            "save_resource must retain canonical id on (workspace_id, name) match"
        );
        assert_eq!(
            saved2.created_at, initial_res.created_at,
            "save_resource must preserve original creation timestamp"
        );

        let list = storage
            .list_resources("ws-prod")
            .await
            .expect("list resources");
        assert_eq!(list.len(), 1, "Must not create duplicate resources");
        assert_eq!(list[0].id, "res_ws-prod_db");
        assert_eq!(list[0].configuration["port"], 5433);
    }

    #[tokio::test]
    async fn test_rotate_service_bindings_batch_atomic() {
        let storage = LocalStorage::new_in_memory();

        let mut b1 = ServiceBinding {
            id: "bind_1".to_string(),
            workspace_id: "ws-test".to_string(),
            source_resource_id: "res_app".to_string(),
            target_resource_id: "res_db".to_string(),
            binding_type: "postgres_connection".to_string(),
            rotation_stage: crate::models::RotationStage::Active,
            ownership: crate::models::ResourceOwnership::SmooothManaged,
            binding_version: 1,
            key_fingerprint: "fp_old".to_string(),
            resolved_endpoint: None,
            safe_connection_details: None,
            encrypted_credentials: vec![1, 2, 3],
            exposed_env_keys: vec!["DATABASE_URL".to_string()],
            env_mappings: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.save_binding(b1.clone()).await.expect("save b1");

        // Rotate to new fingerprint
        b1.key_fingerprint = "fp_new".to_string();
        b1.encrypted_credentials = vec![4, 5, 6];

        let audit = AuditEvent {
            id: 1,
            workspace_id: "ws-test".to_string(),
            actor: "operator".to_string(),
            action: "master_key_rotate".to_string(),
            target_resource_id: None,
            plan_hash: None,
            outcome: "success".to_string(),
            redacted_evidence: None,
            timestamp: chrono::Utc::now(),
        };

        let count = storage
            .rotate_service_bindings_batch("fp_old", "fp_new", vec![b1], audit)
            .await
            .expect("batch rotate must succeed");
        assert_eq!(count, 1);

        let reloaded = storage
            .get_binding("bind_1")
            .await
            .expect("get binding")
            .unwrap();
        assert_eq!(reloaded.key_fingerprint, "fp_new");
        assert_eq!(reloaded.encrypted_credentials, vec![4, 5, 6]);

        let audits = storage
            .list_audit_events("ws-test")
            .await
            .expect("list audits");
        assert_eq!(audits.len(), 1);
        assert_eq!(audits[0].action, "master_key_rotate");
    }

    #[tokio::test]
    async fn test_persistence_rollback_on_maintenance_sentinel() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_rollback_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);

        let agent1 = AgentIdentity {
            agent_id: "agent_pre_freeze".to_string(),
            name: "Pre-freeze Agent".to_string(),
            token_hash: "token_1".to_string(),
            token_prefix: "smoooth_ag_1".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-freeze".to_string(),
        };
        storage
            .save_agent(agent1)
            .await
            .expect("save initial agent");

        // Place maintenance sentinel to freeze mutations
        let maintenance_path = state_path.with_extension("maintenance");
        std::fs::write(&maintenance_path, "FROZEN_FOR_TEST").expect("write maintenance sentinel");

        let agent2 = AgentIdentity {
            agent_id: "agent_during_freeze".to_string(),
            name: "During Freeze Agent".to_string(),
            token_hash: "token_2".to_string(),
            token_prefix: "smoooth_ag_2".to_string(),
            scopes: vec![crate::models::AgentScope::Plan],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-freeze".to_string(),
        };

        // Mutation must fail because maintenance sentinel is present
        let save_err = storage.save_agent(agent2).await;
        assert!(save_err.is_err(), "Must fail when storage is frozen");

        // CRITICAL: Verify in-memory state was rolled back!
        let in_memory_check = storage
            .get_agent("agent_during_freeze")
            .await
            .expect("query agent");
        assert!(
            in_memory_check.is_none(),
            "Failed persistence must roll back in-memory state completely"
        );

        // Remove maintenance sentinel and retry
        let _ = std::fs::remove_file(&maintenance_path);

        let agent3 = AgentIdentity {
            agent_id: "agent_after_unfreeze".to_string(),
            name: "After Unfreeze Agent".to_string(),
            token_hash: "token_3".to_string(),
            token_prefix: "smoooth_ag_3".to_string(),
            scopes: vec![crate::models::AgentScope::Plan],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-freeze".to_string(),
        };
        storage
            .save_agent(agent3)
            .await
            .expect("save must succeed after sentinel removed");

        let unfreeze_check = storage
            .get_agent("agent_after_unfreeze")
            .await
            .expect("query agent");
        assert!(unfreeze_check.is_some());

        let _ = std::fs::remove_file(&state_path);
        let bak = state_path.with_extension("bak");
        if bak.exists() {
            let _ = std::fs::remove_file(&bak);
        }
    }

    #[tokio::test]
    async fn test_two_independent_local_storage_instances_no_lost_updates() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_no_lost_update_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage1 = LocalStorage::new_file_backed(&state_path);
        let storage2 = LocalStorage::new_file_backed(&state_path);

        let agent1 = AgentIdentity {
            agent_id: "agent_inst_1".to_string(),
            name: "Instance 1 Agent".to_string(),
            token_hash: "hash_1".to_string(),
            token_prefix: "smoooth_ag_1".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-lost-update".to_string(),
        };

        let agent2 = AgentIdentity {
            agent_id: "agent_inst_2".to_string(),
            name: "Instance 2 Agent".to_string(),
            token_hash: "hash_2".to_string(),
            token_prefix: "smoooth_ag_2".to_string(),
            scopes: vec![crate::models::AgentScope::Plan],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-lost-update".to_string(),
        };

        // 1. Instance 1 saves agent1
        storage1
            .save_agent(agent1.clone())
            .await
            .expect("storage1 save");

        // 2. Instance 2 saves agent2 (must reload latest snapshot before writing)
        storage2
            .save_agent(agent2.clone())
            .await
            .expect("storage2 save");

        // 3. Verify neither update was lost: both instances and disk contain BOTH records
        let list1 = storage1
            .list_agents(Some("ws-lost-update"))
            .await
            .expect("list from storage1");
        assert_eq!(
            list1.len(),
            2,
            "Storage1 must see both records (no lost updates)"
        );

        let list2 = storage2
            .list_agents(Some("ws-lost-update"))
            .await
            .expect("list from storage2");
        assert_eq!(
            list2.len(),
            2,
            "Storage2 must see both records (no lost updates)"
        );

        assert!(storage1.get_agent("agent_inst_2").await.unwrap().is_some());
        assert!(storage2.get_agent("agent_inst_1").await.unwrap().is_some());

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_concurrent_mutations_preserve_all_changes() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_test_concurrent_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage1 = LocalStorage::new_file_backed(&state_path);
        let storage2 = LocalStorage::new_file_backed(&state_path);

        let mut tasks = Vec::new();

        // Spawn 5 mutations from storage1 and 5 mutations from storage2 concurrently
        for i in 0..5 {
            let s1 = storage1.clone();
            tasks.push(tokio::spawn(async move {
                let agent = AgentIdentity {
                    agent_id: format!("agent_con_s1_{}", i),
                    name: format!("Concurrent S1 Agent {}", i),
                    token_hash: format!("hash_s1_{}", i),
                    token_prefix: format!("smoooth_s1_{}", i),
                    scopes: vec![crate::models::AgentScope::ReadOnly],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-concurrent".to_string(),
                };
                s1.save_agent(agent).await.expect("save s1 agent");
            }));

            let s2 = storage2.clone();
            tasks.push(tokio::spawn(async move {
                let agent = AgentIdentity {
                    agent_id: format!("agent_con_s2_{}", i),
                    name: format!("Concurrent S2 Agent {}", i),
                    token_hash: format!("hash_s2_{}", i),
                    token_prefix: format!("smoooth_s2_{}", i),
                    scopes: vec![crate::models::AgentScope::Plan],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-concurrent".to_string(),
                };
                s2.save_agent(agent).await.expect("save s2 agent");
            }));
        }

        for t in tasks {
            t.await.expect("task must succeed");
        }

        // Verify all 10 distinct agents were saved and neither was overwritten
        let final_list = storage1
            .list_agents(Some("ws-concurrent"))
            .await
            .expect("list agents");
        assert_eq!(
            final_list.len(),
            10,
            "All 10 concurrent mutations across independent storage instances must be preserved"
        );

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_live_lock_cannot_be_reclaimed_based_on_elapsed_time() {
        let temp_dir = std::env::temp_dir();
        let lock_path = temp_dir.join(format!(
            "smoooth_test_live_lock_{}.lock",
            uuid::Uuid::new_v4().simple()
        ));

        // 1. Process A acquires exclusive lock and holds it
        let lock_a = FileLock::acquire(&lock_path, std::time::Duration::from_secs(5))
            .await
            .expect("lock A must acquire");

        // 2. Process B attempts to acquire with a short timeout.
        // It must NOT steal or reclaim the lock even though elapsed time passed.
        let acquire_b = FileLock::acquire(&lock_path, std::time::Duration::from_millis(150)).await;
        assert!(
            acquire_b.is_err(),
            "Process B must fail with timeout while Process A holds the live lock"
        );

        // 3. Process A still holds the lock safely
        drop(lock_a);

        // 4. Now that Process A released it, Process B can acquire it immediately
        let lock_b = FileLock::acquire(&lock_path, std::time::Duration::from_secs(2))
            .await
            .expect("lock B must acquire after A releases");
        drop(lock_b);

        let _ = std::fs::remove_file(&lock_path);
    }

    #[tokio::test]
    async fn test_process_termination_releases_lock_safely() {
        let temp_dir = std::env::temp_dir();
        let lock_path = temp_dir.join(format!(
            "smoooth_test_term_lock_{}.lock",
            uuid::Uuid::new_v4().simple()
        ));

        // Simulate process 1 running and holding the lock, then terminating (dropping handle)
        {
            let _process1_lock = FileLock::acquire(&lock_path, std::time::Duration::from_secs(5))
                .await
                .expect("process 1 lock");
            // While held, lock cannot be acquired
            let attempt =
                FileLock::acquire(&lock_path, std::time::Duration::from_millis(100)).await;
            assert!(attempt.is_err());
            // _process1_lock is dropped here, closing OS file handle
        }

        // Process 2 starts up and acquires the lock cleanly without needing to delete or reclaim
        let process2_lock = FileLock::acquire(&lock_path, std::time::Duration::from_secs(2))
            .await
            .expect("process 2 must acquire cleanly after process 1 terminates");
        drop(process2_lock);

        let _ = std::fs::remove_file(&lock_path);
    }

    #[tokio::test]
    async fn test_canonical_path_resolution_across_variants() {
        let temp_dir = std::env::temp_dir();
        let file_name = format!("smoooth_canon_test_{}.json", uuid::Uuid::new_v4().simple());
        let abs_path = temp_dir.join(&file_name);

        // Path variant 1: canonicalized path
        let canon = LocalStorage::canonicalize_storage_path(&abs_path);

        // Path variant 2: parent / ./file_name
        let dot_slash_path = temp_dir.join(".").join(&file_name);
        let canon_dot = LocalStorage::canonicalize_storage_path(&dot_slash_path);

        assert_eq!(
            canon, canon_dot,
            "Both path representations must resolve to the identical canonical path"
        );

        let storage1 = LocalStorage::new_file_backed(&abs_path);
        let storage2 = LocalStorage::new_file_backed(&dot_slash_path);

        assert_eq!(
            storage1.snapshot_path, storage2.snapshot_path,
            "Storage instances must share identical canonical paths"
        );
    }

    #[tokio::test]
    async fn test_sync_from_disk_does_not_overwrite_newer_in_memory_state() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_stale_inversion_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);

        // Mutate storage: creates version 1 with agent_1
        let agent1 = AgentIdentity {
            agent_id: "agent_monotonic_1".to_string(),
            name: "Agent 1".to_string(),
            token_hash: "th".to_string(),
            token_prefix: "smoooth_ag_1".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-test".to_string(),
        };
        storage.save_agent(agent1.clone()).await.unwrap();

        // In-memory version is now 1
        let mem_version = storage.state.read().await.version;
        assert_eq!(mem_version, 1);

        // Synthesize an older disk state (version 0) without the agent, and write directly to disk
        let older_disk_snapshot = StateSnapshot {
            version: 0,
            agents: HashMap::new(),
            ..Default::default()
        };
        let older_json = serde_json::to_string_pretty(&older_disk_snapshot).unwrap();
        tokio::fs::write(&state_path, older_json.as_bytes())
            .await
            .unwrap();

        // Ensure mtime has advanced so sync_from_disk inspects disk
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Reset last_synced_mtime to force sync_from_disk to parse disk
        {
            let mut last_mtime = storage.last_synced_mtime.write().await;
            *last_mtime = None;
        }

        // Call sync_from_disk()
        storage.sync_from_disk().await.unwrap();

        // In-memory state MUST NOT be overwritten by the stale disk snapshot
        let in_memory_agent = storage.get_agent("agent_monotonic_1").await.unwrap();
        assert!(
            in_memory_agent.is_some(),
            "Newer in-memory state must NOT be overwritten by stale disk snapshot"
        );
        assert_eq!(storage.state.read().await.version, 1);

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_concurrent_reads_and_writes_across_independent_instances() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_concurrent_rw_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let instance_a = LocalStorage::new_file_backed(&state_path);
        let instance_b = LocalStorage::new_file_backed(&state_path);

        // Instance A writes agent A
        let agent_a = AgentIdentity {
            agent_id: "agent_a".to_string(),
            name: "Agent A".to_string(),
            token_hash: "th_a".to_string(),
            token_prefix: "smoooth_ag_a".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-test".to_string(),
        };
        instance_a.save_agent(agent_a.clone()).await.unwrap();

        // Instance B reads: syncs from disk and observes agent A
        let read_a = instance_b.get_agent("agent_a").await.unwrap();
        assert!(
            read_a.is_some(),
            "Instance B must see Agent A written by Instance A"
        );

        // Instance B writes agent B
        let agent_b = AgentIdentity {
            agent_id: "agent_b".to_string(),
            name: "Agent B".to_string(),
            token_hash: "th_b".to_string(),
            token_prefix: "smoooth_ag_b".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-test".to_string(),
        };
        instance_b.save_agent(agent_b.clone()).await.unwrap();

        // Instance A reads: syncs from disk and observes both Agent A and Agent B
        let read_b = instance_a.get_agent("agent_b").await.unwrap();
        assert!(
            read_b.is_some(),
            "Instance A must see Agent B written by Instance B"
        );
        let read_a_again = instance_a.get_agent("agent_a").await.unwrap();
        assert!(read_a_again.is_some(), "Instance A must still see Agent A");

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_maintenance_freeze_drains_in_flight_and_fails_closed() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_maint_drain_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = LocalStorage::new_file_backed(&state_path);

        // 1. Initial mutation to establish state
        let agent1 = AgentIdentity {
            agent_id: "agent_maint_1".to_string(),
            name: "Agent 1".to_string(),
            token_hash: "th_1".to_string(),
            token_prefix: "smoooth_ag_1".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-maint".to_string(),
        };
        storage.save_agent(agent1.clone()).await.unwrap();

        // 2. Administrator enters maintenance mode via acquire_maintenance_lock
        let maint_lock =
            LocalStorage::acquire_maintenance_lock(&state_path, std::time::Duration::from_secs(5))
                .await
                .expect("Administrator must acquire maintenance lock");

        // 3. Any subsequent mutation must fail closed while in maintenance
        let agent2 = AgentIdentity {
            agent_id: "agent_maint_2".to_string(),
            name: "Agent 2".to_string(),
            token_hash: "th_2".to_string(),
            token_prefix: "smoooth_ag_2".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-maint".to_string(),
        };
        let err = storage.save_agent(agent2.clone()).await.unwrap_err();
        assert!(
            err.to_string()
                .contains("Storage is frozen for maintenance"),
            "Mutations must fail closed while maintenance is active, got: {}",
            err
        );

        // 4. Administrator completes maintenance and releases lock
        LocalStorage::release_maintenance_lock(&state_path, maint_lock)
            .await
            .expect("Administrator must release maintenance lock");

        // 5. Subsequent mutations must now succeed
        storage.save_agent(agent2.clone()).await.unwrap();
        assert!(storage.get_agent("agent_maint_2").await.unwrap().is_some());

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_sync_from_disk_timestamp_collision_never_suppresses_newer_version() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_timestamp_collision_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let instance_a = LocalStorage::new_file_backed(&state_path);
        let instance_b = LocalStorage::new_file_backed(&state_path);

        // Instance A writes agent 1 (version 1)
        let agent1 = AgentIdentity {
            agent_id: "agent_tc_1".to_string(),
            name: "Agent 1".to_string(),
            token_hash: "th_1".to_string(),
            token_prefix: "smoooth_ag_1".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-tc".to_string(),
        };
        instance_a.save_agent(agent1.clone()).await.unwrap();

        // Instance B syncs: version 1 loaded
        instance_b.sync_from_disk().await.unwrap();
        assert_eq!(instance_b.state.read().await.version, 1);

        // Instance A writes agent 2 (version 2)
        let agent2 = AgentIdentity {
            agent_id: "agent_tc_2".to_string(),
            name: "Agent 2".to_string(),
            token_hash: "th_2".to_string(),
            token_prefix: "smoooth_ag_2".to_string(),
            scopes: vec![crate::models::AgentScope::ReadOnly],
            created_at: chrono::Utc::now(),
            expires_at: None,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
            workspace_id: "ws-tc".to_string(),
        };
        instance_a.save_agent(agent2.clone()).await.unwrap();

        // Simulate timestamp collision on Instance B:
        // Set Instance B's last_synced_mtime to match the file's current disk mtime.
        let file_mtime = std::fs::metadata(&state_path)
            .ok()
            .and_then(|m| m.modified().ok());
        *instance_b.last_synced_mtime.write().await = file_mtime;

        // Instance B syncs from disk.
        // Even with identical mtime (timestamp collision), version ordering MUST load version 2!
        instance_b.sync_from_disk().await.unwrap();
        assert_eq!(
            instance_b.state.read().await.version,
            2,
            "Version ordering must apply genuinely newer version despite timestamp collision"
        );
        let found = instance_b.get_agent("agent_tc_2").await.unwrap();
        assert!(found.is_some(), "Agent 2 must be visible in memory");

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test]
    async fn test_concurrent_synchronization_calls_preserve_monotonicity() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_concurrent_sync_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let writer = LocalStorage::new_file_backed(&state_path);
        let reader = LocalStorage::new_file_backed(&state_path);

        let reader_arc = Arc::new(reader);
        let mut reader_handles = Vec::new();

        // Spawn 10 concurrent readers calling sync_from_disk and verifying monotonicity
        for _ in 0..10 {
            let r = reader_arc.clone();
            reader_handles.push(tokio::spawn(async move {
                let mut last_observed_version = 0;
                for _ in 0..20 {
                    r.sync_from_disk().await.unwrap();
                    let current_version = r.state.read().await.version;
                    assert!(
                        current_version >= last_observed_version,
                        "Observed version regressed from {} to {}",
                        last_observed_version,
                        current_version
                    );
                    last_observed_version = current_version;
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
            }));
        }

        // Writer concurrently writes 5 incremental agent updates
        for i in 1..=5 {
            let agent = AgentIdentity {
                agent_id: format!("agent_con_sync_{}", i),
                name: format!("Agent {}", i),
                token_hash: format!("th_{}", i),
                token_prefix: format!("smoooth_ag_{}", i),
                scopes: vec![crate::models::AgentScope::ReadOnly],
                created_at: chrono::Utc::now(),
                expires_at: None,
                revoked: false,
                revoked_at: None,
                last_used_at: None,
                workspace_id: "ws-con".to_string(),
            };
            writer.save_agent(agent).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        for h in reader_handles {
            h.await.unwrap();
        }

        // Final sync ensures reader reaches version 5
        reader_arc.sync_from_disk().await.unwrap();
        assert_eq!(reader_arc.state.read().await.version, 5);

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_maintenance_drain_waits_for_active_mutation_and_handles_timeout() {
        let temp_dir = std::env::temp_dir();
        let state_path = temp_dir.join(format!(
            "smoooth_maint_drain_test_{}.json",
            uuid::Uuid::new_v4().simple()
        ));

        let storage = Arc::new(LocalStorage::new_file_backed(&state_path));
        storage
            .save_agent(AgentIdentity {
                agent_id: "init_agent".to_string(),
                name: "Initial Agent".to_string(),
                token_hash: "th_init".to_string(),
                token_prefix: "smoooth_ag_init".to_string(),
                scopes: vec![crate::models::AgentScope::ReadOnly],
                created_at: chrono::Utc::now(),
                expires_at: None,
                revoked: false,
                revoked_at: None,
                last_used_at: None,
                workspace_id: "ws-maint".to_string(),
            })
            .await
            .expect("initial save");

        let maintenance_path = state_path.with_extension("maintenance");

        // PART 1: Verify maintenance-lock acquisition timeout and error paths
        // do not leave an orphaned sentinel or permanently frozen storage.
        {
            let (holding_lock_tx, holding_lock_rx) = std::sync::mpsc::channel();
            let (release_lock_tx, release_lock_rx) = std::sync::mpsc::channel();
            let holding_lock_tx = Arc::new(std::sync::Mutex::new(Some(holding_lock_tx)));
            let release_lock_rx = Arc::new(std::sync::Mutex::new(release_lock_rx));

            let storage_clone = storage.clone();
            let h_tx = holding_lock_tx.clone();
            let r_rx = release_lock_rx.clone();
            let mutation_handle = tokio::spawn(async move {
                storage_clone
                    .mutate_and_persist(move |state| {
                        // Mutation has acquired lock and passed under-lock sentinel check
                        if let Some(tx) = h_tx.lock().unwrap().take() {
                            tx.send(()).unwrap();
                        }
                        let _ = r_rx.lock().unwrap().recv();
                        state.agents.insert(
                            "in_flight_1".to_string(),
                            AgentIdentity {
                                agent_id: "in_flight_1".to_string(),
                                name: "In Flight 1".to_string(),
                                token_hash: "th_if1".to_string(),
                                token_prefix: "smoooth_ag_if1".to_string(),
                                scopes: vec![crate::models::AgentScope::ReadOnly],
                                created_at: chrono::Utc::now(),
                                expires_at: None,
                                revoked: false,
                                revoked_at: None,
                                last_used_at: None,
                                workspace_id: "ws-maint".to_string(),
                            },
                        );
                        Ok(((), false))
                    })
                    .await
            });

            // Wait until mutation has acquired the storage lock
            holding_lock_rx
                .recv()
                .expect("mutation must acquire storage lock");

            // While mutation holds lock, attempt maintenance lock with a short 50ms timeout.
            // This MUST timeout because mutation holds the lock.
            let timeout_res = LocalStorage::acquire_maintenance_lock(
                &state_path,
                std::time::Duration::from_millis(50),
            )
            .await;
            assert!(
                timeout_res.is_err(),
                "Must timeout waiting for in-flight mutation to release lock"
            );

            // CRITICAL CHECK: Verify maintenance-lock acquisition timeout error path
            // does NOT leave an orphaned sentinel on disk!
            assert!(
                !maintenance_path.exists(),
                "Timeout must clean up .maintenance sentinel and not leave it orphaned"
            );

            // Release the mutation
            let _ = release_lock_tx.send(());
            let mutation_res = mutation_handle.await.unwrap();
            assert!(
                mutation_res.is_ok(),
                "In-flight mutation should complete successfully after timeout aborted"
            );

            // Storage is NOT permanently frozen: subsequent mutations succeed
            let probe_res = storage
                .save_agent(AgentIdentity {
                    agent_id: "probe_agent".to_string(),
                    name: "Probe Agent".to_string(),
                    token_hash: "th_probe".to_string(),
                    token_prefix: "smoooth_ag_probe".to_string(),
                    scopes: vec![crate::models::AgentScope::ReadOnly],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-maint".to_string(),
                })
                .await;
            assert!(
                probe_res.is_ok(),
                "Storage must not be frozen after maintenance lock timeout"
            );
        }

        // PART 2: Deterministic test where a mutation has acquired lock,
        // maintenance begins and waits for in-flight mutation to finish or abort,
        // subsequent mutations fail closed, and maintenance lock acquisition succeeds.
        {
            let (holding_lock_tx, holding_lock_rx) = std::sync::mpsc::channel();
            let (release_lock_tx, release_lock_rx) = std::sync::mpsc::channel();
            let holding_lock_tx = Arc::new(std::sync::Mutex::new(Some(holding_lock_tx)));
            let release_lock_rx = Arc::new(std::sync::Mutex::new(release_lock_rx));

            let storage_clone = storage.clone();
            let h_tx = holding_lock_tx.clone();
            let r_rx = release_lock_rx.clone();
            let mutation_handle = tokio::spawn(async move {
                storage_clone
                    .mutate_and_persist(move |state| {
                        if let Some(tx) = h_tx.lock().unwrap().take() {
                            tx.send(()).unwrap();
                        }
                        let _ = r_rx.lock().unwrap().recv();
                        state.agents.insert(
                            "in_flight_2".to_string(),
                            AgentIdentity {
                                agent_id: "in_flight_2".to_string(),
                                name: "In Flight 2".to_string(),
                                token_hash: "th_if2".to_string(),
                                token_prefix: "smoooth_ag_if2".to_string(),
                                scopes: vec![crate::models::AgentScope::ReadOnly],
                                created_at: chrono::Utc::now(),
                                expires_at: None,
                                revoked: false,
                                revoked_at: None,
                                last_used_at: None,
                                workspace_id: "ws-maint".to_string(),
                            },
                        );
                        // With modified: true, the mutation attempts to persist
                        // but will abort cleanly if it detects the maintenance sentinel before commit
                        Ok(((), true))
                    })
                    .await
            });

            // 1. Mutation has acquired the storage lock and passed initial sentinel check
            holding_lock_rx
                .recv()
                .expect("mutation must acquire storage lock");

            // 2. Start acquiring maintenance lock in background with generous 5s timeout
            let path_clone = state_path.clone();
            let maint_handle = tokio::spawn(async move {
                LocalStorage::acquire_maintenance_lock(
                    &path_clone,
                    std::time::Duration::from_secs(5),
                )
                .await
            });

            // Poll briefly until the maintenance sentinel is created on disk
            let mut sentinel_created = false;
            for _ in 0..50 {
                if maintenance_path.exists() {
                    sentinel_created = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(
                sentinel_created,
                "Maintenance sentinel must be created while waiting for lock"
            );

            // 3. Verify maintenance cannot acquire its lock until the mutation finishes or aborts
            assert!(
                !maint_handle.is_finished(),
                "Maintenance must be waiting for in-flight mutation to release lock"
            );

            // 4. Verify subsequent mutations fail closed immediately because sentinel exists
            let subsequent_err = storage
                .save_agent(AgentIdentity {
                    agent_id: "subsequent_agent".to_string(),
                    name: "Subsequent Agent".to_string(),
                    token_hash: "th_sub".to_string(),
                    token_prefix: "smoooth_ag_sub".to_string(),
                    scopes: vec![crate::models::AgentScope::ReadOnly],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-maint".to_string(),
                })
                .await
                .unwrap_err();
            assert!(
                subsequent_err
                    .to_string()
                    .contains("Storage is frozen for maintenance"),
                "Subsequent mutation must fail closed while maintenance is pending/active, got: {}",
                subsequent_err
            );

            // 5. Release the in-flight mutation
            let _ = release_lock_tx.send(());
            let mutation_res = mutation_handle.await.unwrap();
            // Pre-commit sentinel check safely aborts the mutation if sentinel was dropped
            if let Err(e) = mutation_res {
                assert!(
                    e.to_string().contains("Storage is frozen for maintenance"),
                    "If in-flight mutation aborts, it must be due to maintenance freeze: {}",
                    e
                );
            }

            // 6. Maintenance lock acquisition completes now that mutation has dropped the lock
            let maint_lock = maint_handle
                .await
                .unwrap()
                .expect("Maintenance lock must be acquired after mutation finishes or aborts");

            // 7. While maintenance lock is held, mutations continue to fail closed
            let during_maint_err = storage
                .save_agent(AgentIdentity {
                    agent_id: "during_maint_agent".to_string(),
                    name: "During Maint Agent".to_string(),
                    token_hash: "th_dm".to_string(),
                    token_prefix: "smoooth_ag_dm".to_string(),
                    scopes: vec![crate::models::AgentScope::ReadOnly],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-maint".to_string(),
                })
                .await
                .unwrap_err();
            assert!(
                during_maint_err
                    .to_string()
                    .contains("Storage is frozen for maintenance"),
                "Mutations must fail closed while maintenance lock is held"
            );

            // 8. Release maintenance lock and verify storage is reopened
            LocalStorage::release_maintenance_lock(&state_path, maint_lock)
                .await
                .expect("release maintenance lock");
            assert!(
                !maintenance_path.exists(),
                "Sentinel must be removed on release"
            );

            let post_maint_res = storage
                .save_agent(AgentIdentity {
                    agent_id: "post_maint_agent".to_string(),
                    name: "Post Maint Agent".to_string(),
                    token_hash: "th_post".to_string(),
                    token_prefix: "smoooth_ag_post".to_string(),
                    scopes: vec![crate::models::AgentScope::ReadOnly],
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    revoked: false,
                    revoked_at: None,
                    last_used_at: None,
                    workspace_id: "ws-maint".to_string(),
                })
                .await;
            assert!(
                post_maint_res.is_ok(),
                "Mutations must succeed after maintenance lock is released"
            );
        }

        let _ = std::fs::remove_file(&state_path);
        let _ = std::fs::remove_file(state_path.with_extension("lock"));
        let _ = std::fs::remove_file(state_path.with_extension("bak"));
        let _ = std::fs::remove_file(maintenance_path);
    }
}
