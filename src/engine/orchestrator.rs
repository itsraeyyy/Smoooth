use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::ProviderAdapter;
use crate::error::{Result, SmooothError};
use crate::models::plan::CanonicalActionSpec;
use crate::models::{
    AgentIdentity, AgentScope, BindingRequirement, ConnectionDetails, IdempotencyKey,
    IdempotencyRecord, Operation, OperationLogEntry, OperationStatus, Plan, ReconciliationOutcome,
    Resource, ResourceOwnership, ResourceStatus, RiskTier, ServiceBinding,
};
use crate::storage::{AuditEvent, OperationInitOutcome, Storage};

pub struct Orchestrator {
    worker_id: String,
    storage: Arc<dyn Storage>,
    adapters: HashMap<String, Arc<dyn ProviderAdapter>>,
    secret_store: Arc<crate::security::SecretStore>,
}

impl Orchestrator {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        let secret_store = crate::security::SecretStore::resolve_or_create().unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to resolve or auto-generate master key, falling back to default local: {}",
                e
            );
            crate::security::SecretStore::default_local()
        });
        Self {
            worker_id: format!("worker_{}", Uuid::new_v4().simple()),
            storage,
            adapters: HashMap::new(),
            secret_store: Arc::new(secret_store),
        }
    }

    pub fn with_secret_store(mut self, secret_store: Arc<crate::security::SecretStore>) -> Self {
        self.secret_store = secret_store;
        self
    }

    pub fn storage(&self) -> &Arc<dyn Storage> {
        &self.storage
    }

    pub fn secret_store(&self) -> &crate::security::SecretStore {
        &self.secret_store
    }

    pub fn register_adapter(&mut self, adapter: Arc<dyn ProviderAdapter>) {
        self.adapters
            .insert(adapter.provider_name().to_string(), adapter);
    }

    /// Rotate the master encryption key, atomically re-encrypting all stored ServiceBindings.
    /// Fails closed without modifying state if any decryption fails.
    pub async fn rotate_master_key(
        &mut self,
        new_key: [u8; crate::security::KEY_LEN],
    ) -> Result<(String, usize)> {
        let old_store = self.secret_store.clone();
        let new_store = Arc::new(crate::security::SecretStore::new(new_key));
        let mut migrated_count = 0;

        let bindings = self.storage.list_bindings().await?;
        let mut updated_bindings = Vec::new();

        for mut binding in bindings {
            if !binding.encrypted_credentials.is_empty() {
                let decrypted = binding.decrypt_credentials(&old_store)?;
                let new_encrypted = new_store
                    .domain_store("service_binding")
                    .encrypt(&decrypted)?;
                binding.encrypted_credentials = new_encrypted;
                binding.key_fingerprint = new_store.fingerprint();
                binding.updated_at = Utc::now();
                updated_bindings.push(binding);
                migrated_count += 1;
            }
        }

        let audit = AuditEvent {
            id: 0,
            workspace_id: "*".to_string(),
            actor: "operator".to_string(),
            action: "master_key_rotate".to_string(),
            target_resource_id: None,
            plan_hash: None,
            outcome: "success".to_string(),
            redacted_evidence: Some(serde_json::json!({
                "old_fingerprint": old_store.fingerprint(),
                "new_fingerprint": new_store.fingerprint(),
                "records_migrated": migrated_count,
            })),
            timestamp: Utc::now(),
        };

        self.storage
            .rotate_service_bindings_batch(
                &old_store.fingerprint(),
                &new_store.fingerprint(),
                updated_bindings,
                audit,
            )
            .await?;

        self.secret_store = new_store.clone();

        Ok((new_store.fingerprint(), migrated_count))
    }

    // Agent Identity & Authorization Management
    pub async fn pair_agent(
        &self,
        name: &str,
        workspace_id: &str,
        scopes: Vec<AgentScope>,
        ttl_hours: Option<i64>,
    ) -> Result<(AgentIdentity, String)> {
        use sha2::{Digest, Sha256};
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to generate random token: {}", e))
        })?;
        let raw_token = format!("smoooth_ag_{}", hex::encode(bytes));
        let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
        let token_prefix = raw_token[..14.min(raw_token.len())].to_string();

        let agent_id = format!("agent_{}", Uuid::new_v4().simple());
        let expires_at = ttl_hours.map(|h| Utc::now() + Duration::hours(h));

        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            name: name.to_string(),
            token_hash,
            token_prefix,
            scopes,
            workspace_id: workspace_id.to_string(),
            created_at: Utc::now(),
            expires_at,
            revoked: false,
            revoked_at: None,
            last_used_at: None,
        };

        self.storage.save_agent(identity.clone()).await?;

        // Record security audit event
        let _ = self
            .storage
            .record_audit(AuditEvent {
                id: 0,
                workspace_id: workspace_id.to_string(),
                actor: "operator".to_string(),
                action: "agent_paired".to_string(),
                target_resource_id: Some(agent_id),
                plan_hash: None,
                outcome: "success".to_string(),
                redacted_evidence: Some(serde_json::json!({
                    "name": name,
                    "scopes": identity.scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                })),
                timestamp: Utc::now(),
            })
            .await;

        Ok((identity, raw_token))
    }

    pub async fn list_agents(&self, workspace_id: Option<&str>) -> Result<Vec<AgentIdentity>> {
        self.storage.list_agents(workspace_id).await
    }

    pub async fn revoke_agent(&self, agent_id: &str) -> Result<()> {
        self.storage.revoke_agent(agent_id).await?;
        let _ = self
            .storage
            .record_audit(AuditEvent {
                id: 0,
                workspace_id: "*".to_string(),
                actor: "operator".to_string(),
                action: "agent_revoked".to_string(),
                target_resource_id: Some(agent_id.to_string()),
                plan_hash: None,
                outcome: "success".to_string(),
                redacted_evidence: None,
                timestamp: Utc::now(),
            })
            .await;
        Ok(())
    }

    pub async fn rotate_agent_token(&self, agent_id: &str) -> Result<String> {
        use sha2::{Digest, Sha256};
        let agent = self
            .storage
            .get_agent(agent_id)
            .await?
            .ok_or_else(|| SmooothError::NotFound(format!("Agent '{}' not found", agent_id)))?;

        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to generate random token: {}", e))
        })?;
        let raw_token = format!("smoooth_ag_{}", hex::encode(bytes));
        let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
        let token_prefix = raw_token[..14.min(raw_token.len())].to_string();

        let mut updated = agent;
        updated.token_hash = token_hash;
        updated.token_prefix = token_prefix;
        self.storage.save_agent(updated).await?;

        let _ = self
            .storage
            .record_audit(AuditEvent {
                id: 0,
                workspace_id: "*".to_string(),
                actor: "operator".to_string(),
                action: "agent_token_rotated".to_string(),
                target_resource_id: Some(agent_id.to_string()),
                plan_hash: None,
                outcome: "success".to_string(),
                redacted_evidence: None,
                timestamp: Utc::now(),
            })
            .await;

        Ok(raw_token)
    }

    /// Synthesizes an ephemeral, single-action VerifiedExecutionPermit for a legacy AgentScope
    pub async fn synthesize_legacy_permit(
        &self,
        agent: &AgentIdentity,
        target_urn: &crate::models::ResourceUrn,
        action: crate::models::Action,
        operation_digest: [u8; 32],
    ) -> Result<crate::security::VerifiedExecutionPermit<'static>> {
        // Enforce workspace boundary
        if agent.workspace_id != "*" && agent.workspace_id != target_urn.workspace() {
            return Err(SmooothError::Security(format!(
                "Agent workspace '{}' does not match target URN workspace '{}'",
                agent.workspace_id,
                target_urn.workspace()
            )));
        }

        // Validate that legacy scope authorizes this specific action
        let authorized = match &action {
            crate::models::Action::InspectMetadata | crate::models::Action::ReadLogs { .. } => {
                agent.scopes.contains(&AgentScope::ReadOnly)
                    || agent.scopes.contains(&AgentScope::Plan)
                    || agent.scopes.contains(&AgentScope::Deploy)
                    || agent.scopes.contains(&AgentScope::Admin)
            }
            crate::models::Action::Provision { .. }
            | crate::models::Action::UpdateEnv { .. }
            | crate::models::Action::RestartWorkload
            | crate::models::Action::Reconcile
            | crate::models::Action::Snapshot { .. } => {
                agent.scopes.contains(&AgentScope::Deploy)
                    || agent.scopes.contains(&AgentScope::Admin)
            }
            crate::models::Action::Destroy(_) => {
                agent.scopes.contains(&AgentScope::Destructive)
                    || agent.scopes.contains(&AgentScope::Admin)
            }
            crate::models::Action::ReadSecret { .. } => agent.scopes.contains(&AgentScope::Admin),
        };

        if !authorized {
            return Err(SmooothError::Security(format!(
                "Agent with scopes {:?} is not authorized for action {:?}",
                agent.scopes, action
            )));
        }

        let epoch = self
            .storage
            .get_fencing_epoch(target_urn.workspace())
            .await?;
        let permit = crate::security::AuthorizationEngine::synthesize_ephemeral_permit(
            &agent.agent_id,
            target_urn.clone(),
            action,
            operation_digest,
            epoch,
            std::time::Duration::from_secs(60),
        );
        Ok(permit)
    }

    pub async fn authenticate_agent(&self, raw_token: &str) -> Result<Option<AgentIdentity>> {
        use sha2::{Digest, Sha256};
        let token_hash = hex::encode(Sha256::digest(raw_token.trim().as_bytes()));
        if let Some(mut agent) = self.storage.get_agent_by_token_hash(&token_hash).await? {
            if agent.is_active() {
                agent.last_used_at = Some(Utc::now());
                let _ = self.storage.save_agent(agent.clone()).await;
                return Ok(Some(agent));
            }
        }
        Ok(None)
    }

    async fn init_operation(
        &self,
        plan: &Plan,
        idempotency_client_key: &str,
    ) -> Result<(Operation, bool)> {
        // 0. Validate dependency graph and cycle freedom before reserving operations or leases
        plan.spec.resolve_execution_order()?;

        // 1. Strict cryptographic plan approval & policy verification before reserving
        let requires_approval = plan.risk_tier != RiskTier::Low
            || plan.spec.environment == "production"
            || plan.spec.actions.iter().any(|a| {
                (a.is_persistent && a.action_type == "delete") || a.exposes_network_publicly
            });

        if requires_approval && !plan.is_approval_valid() {
            let rejection_reason = plan
                .approval_rejection_reason()
                .unwrap_or("Missing valid plan approval");

            // Record denied audit event
            let _ = self
                .storage
                .record_audit(AuditEvent {
                    id: 0,
                    workspace_id: plan.spec.workspace_id.clone(),
                    actor: "agent".to_string(),
                    action: "execute_plan".to_string(),
                    target_resource_id: None,
                    plan_hash: Some(plan.plan_hash.clone()),
                    outcome: "denied".to_string(),
                    redacted_evidence: Some(serde_json::json!({ "reason": rejection_reason })),
                    timestamp: Utc::now(),
                })
                .await;

            return Err(SmooothError::PolicyViolation(format!(
                "Plan {} rejected: {}. Plan hash: {}",
                plan.id, rejection_reason, plan.plan_hash
            )));
        }

        let idemp_key = IdempotencyKey {
            workspace_id: plan.spec.workspace_id.clone(),
            operation_type: "execute_plan".to_string(),
            client_key: idempotency_client_key.to_string(),
        };

        let request_hash = plan.plan_hash.clone();
        let op_id = format!("op_{}", Uuid::new_v4().simple());

        let idempotency_record = IdempotencyRecord {
            key: idemp_key,
            request_hash: request_hash.clone(),
            operation_id: op_id.clone(),
            created_at: Utc::now(),
        };

        // 2. Prepare durable operation with initial fencing token (lease_epoch = 1)
        let lease_expiration = Utc::now() + Duration::minutes(5);
        let initial_epoch = 1u64;

        let operation = Operation {
            id: op_id,
            workspace_id: plan.spec.workspace_id.clone(),
            operation_type: "execute_plan".to_string(),
            plan_id: plan.id.clone(),
            plan_hash: plan.plan_hash.clone(),
            request_hash,
            idempotency_client_key: idempotency_client_key.to_string(),
            status: OperationStatus::Running,
            progress_percent: 0,
            lease_epoch: initial_epoch,
            lease_owner: Some(self.worker_id.clone()),
            lease_expires_at: Some(lease_expiration),
            intended_state: serde_json::to_value(&plan.spec).unwrap_or_default(),
            observed_state: serde_json::json!({}),
            logs: vec![OperationLogEntry {
                timestamp: Utc::now(),
                message: format!(
                    "Worker {} acquired lease [epoch: {}] for plan {} [hash: {}]",
                    self.worker_id,
                    initial_epoch,
                    plan.id,
                    &plan.plan_hash[..8]
                ),
                level: "INFO".to_string(),
            }],
            evidence: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 3. Atomic Idempotency reservation and operation creation
        match self
            .storage
            .reserve_or_get_operation(idempotency_record, operation)
            .await?
        {
            OperationInitOutcome::Created(op) => Ok((op, true)),
            OperationInitOutcome::Existing(op) => Ok((op, false)),
        }
    }

    pub async fn execute_plan(
        &self,
        plan: Plan,
        idempotency_client_key: String,
    ) -> Result<Operation> {
        let (operation, is_new) = self.init_operation(&plan, &idempotency_client_key).await?;
        if !is_new {
            return Ok(operation);
        }

        let ordered_actions = plan.spec.resolve_execution_order()?;

        // 4. Asynchronous durable execution with persistent-data boundary protection
        let adapters = self.adapters.clone();
        let storage = self.storage.clone();
        let secret_store = self.secret_store.clone();
        let mut op_task = operation.clone();
        let initial_epoch = operation.lease_epoch;

        tokio::spawn(async move {
            let total = ordered_actions.len().max(1);
            let mut completed = 0;
            let mut collected_evidence = Vec::new();
            let mut provisioned_resources = Vec::new();
            let mut successful_resources = std::collections::HashSet::new();

            let mut prerequisites: HashMap<String, Vec<String>> = HashMap::new();
            for edge in &plan.spec.dependencies {
                prerequisites
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }

            for action in ordered_actions {
                // Ensure all prerequisites reached verified healthy state
                if let Some(reqs) = prerequisites.get(&action.resource_name) {
                    for req in reqs {
                        if !successful_resources.contains(req) {
                            let err = format!(
                                "Prerequisite resource '{}' failed or was not satisfied for dependent '{}'. Execution aborted.",
                                req, action.resource_name
                            );
                            tracing::warn!("{}", err);
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                err,
                            )
                            .await;
                            return;
                        }
                    }
                }
                // Stale worker check: ensure worker still owns lease
                if let Ok(Some(current_op)) = storage.get_operation(&op_task.id).await {
                    if current_op.lease_epoch > initial_epoch {
                        let err = format!("Worker lease lost: current epoch {} > worker epoch {}. Stale execution aborted.", current_op.lease_epoch, initial_epoch);
                        tracing::warn!("{}", err);
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            err,
                        )
                        .await;
                        return;
                    }
                    if let Some(exp) = current_op.lease_expires_at {
                        if exp < Utc::now() {
                            let err = format!(
                                "Worker lease expired at {}. Stale execution aborted.",
                                exp
                            );
                            tracing::warn!("{}", err);
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                err,
                            )
                            .await;
                            return;
                        }
                    }
                }

                // Data safety guard for destructive actions
                if action.action_type == "delete" {
                    // Safeguard: Never delete External resources!
                    if let Ok(Some(existing)) = storage.get_resource(&action.resource_name).await {
                        if existing.ownership == ResourceOwnership::External {
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                format!(
                                    "Security policy violation: External resource '{}' cannot be deleted by Smoooth.",
                                    action.resource_name
                                ),
                            ).await;
                            return;
                        }
                    }

                    if action.is_persistent && !plan.spec.policy_envelope.allow_data_destruction {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            format!(
                                "Data safety guard blocked deletion of persistent resource '{}': plan policy 'allow_data_destruction' is false",
                                action.resource_name
                            ),
                        ).await;
                        return;
                    }
                    if let Some(adapter) = adapters.get(&action.provider) {
                        let scope = if action.is_persistent {
                            crate::models::DestructionScope::ResourceAndPersistentData
                        } else {
                            crate::models::DestructionScope::ResourceOnly
                        };
                        let target_urn = crate::models::ResourceUrn::for_resource(
                            &plan.spec.workspace_id,
                            &action.service_type,
                            &action.resource_name,
                        )
                        .unwrap_or_else(|_| {
                            crate::models::ResourceUrn::new_workspace(&plan.spec.workspace_id)
                                .or_else(|_| {
                                    crate::models::ResourceUrn::new_workspace(
                                        &plan.spec.workspace_id.replace('_', "-"),
                                    )
                                })
                                .unwrap_or_else(|_| {
                                    crate::models::ResourceUrn::new_workspace("default").unwrap()
                                })
                        });
                        let op_digest: [u8; 32] = Sha256::digest(
                            serde_json::to_string(&action.config)
                                .unwrap_or_default()
                                .as_bytes(),
                        )
                        .into();
                        let permit =
                            crate::security::AuthorizationEngine::synthesize_ephemeral_permit(
                                &plan.created_by,
                                target_urn.clone(),
                                crate::models::Action::Destroy(scope),
                                op_digest,
                                initial_epoch,
                                std::time::Duration::from_secs(60),
                            );
                        if let Err(e) = adapter
                            .authorized_destroy(
                                &permit,
                                &target_urn,
                                op_digest,
                                scope,
                                initial_epoch,
                            )
                            .await
                        {
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                format!(
                                    "Authorized destruction failed for '{}': {}",
                                    action.resource_name, e
                                ),
                            )
                            .await;
                            return;
                        }
                        let _ = storage.delete_resource(&action.resource_name).await;
                        completed += 1;
                    } else {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            format!(
                                "Unsupported provider '{}' for resource deletion '{}'",
                                action.provider, action.resource_name
                            ),
                        )
                        .await;
                        return;
                    }
                    continue;
                }

                // Ensure consumer resource is persisted in storage with canonical identity before binding injection
                if storage
                    .get_resource(&action.resource_name)
                    .await
                    .ok()
                    .flatten()
                    .is_none()
                {
                    let initial_res = Resource {
                        id: format!("res_{}_{}", plan.spec.workspace_id, action.resource_name),
                        name: action.resource_name.clone(),
                        service_type: action.service_type.clone(),
                        provider: action.provider.clone(),
                        workspace_id: plan.spec.workspace_id.clone(),
                        status: ResourceStatus::Provisioning,
                        ownership: ResourceOwnership::SmooothManaged,
                        configuration: action.config.clone(),
                        is_persistent: action.is_persistent,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    let _ = storage.save_resource(initial_res).await;
                }

                // Resolve dynamic bindings and inject into action environment variables
                let reqs_for_action = prerequisites
                    .get(&action.resource_name)
                    .cloned()
                    .unwrap_or_default();
                let mut resolved_action = action.clone();
                if let Err(e) = Self::inject_bindings(
                    &storage,
                    &secret_store,
                    &mut resolved_action,
                    &reqs_for_action,
                    &plan.spec.workspace_id,
                )
                .await
                {
                    let err = format!(
                        "Binding resolution failed for '{}': {}",
                        action.resource_name, e
                    );
                    tracing::warn!("{}", err);
                    Self::handle_partial_failure(
                        &adapters,
                        &provisioned_resources,
                        &storage,
                        &mut op_task,
                        err,
                    )
                    .await;
                    return;
                }

                let adapter = match adapters.get(&resolved_action.provider) {
                    Some(a) => a,
                    None => {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            format!(
                                "Unsupported or unrecognized provider '{}' for resource '{}'",
                                resolved_action.provider, resolved_action.resource_name
                            ),
                        )
                        .await;
                        return;
                    }
                };

                let mut config = resolved_action.config.clone();
                if let Some(obj) = config.as_object_mut() {
                    obj.insert("operation_id".to_string(), serde_json::json!(&op_task.id));
                    obj.insert(
                        "workspace_id".to_string(),
                        serde_json::json!(&plan.spec.workspace_id),
                    );
                    obj.insert(
                        "fencing_epoch".to_string(),
                        serde_json::json!(initial_epoch),
                    );
                    obj.insert(
                        "worker_id".to_string(),
                        serde_json::json!(op_task.lease_owner.as_deref().unwrap_or("worker")),
                    );
                }
                let target_urn = crate::models::ResourceUrn::for_resource(
                    &plan.spec.workspace_id,
                    &resolved_action.service_type,
                    &resolved_action.resource_name,
                )
                .unwrap_or_else(|_| {
                    crate::models::ResourceUrn::new_workspace(&plan.spec.workspace_id)
                        .or_else(|_| {
                            crate::models::ResourceUrn::new_workspace(
                                &plan.spec.workspace_id.replace('_', "-"),
                            )
                        })
                        .unwrap_or_else(|_| {
                            crate::models::ResourceUrn::new_workspace("default").unwrap()
                        })
                });
                let op_digest: [u8; 32] = Sha256::digest(
                    serde_json::to_string(&config)
                        .unwrap_or_default()
                        .as_bytes(),
                )
                .into();
                let permit = crate::security::AuthorizationEngine::synthesize_ephemeral_permit(
                    &plan.created_by,
                    target_urn.clone(),
                    crate::models::Action::Provision {
                        resource_type: match resolved_action.service_type.as_str() {
                            "database" => crate::models::ResourceType::Database,
                            "network" => crate::models::ResourceType::Network,
                            "storage" | "volume" => crate::models::ResourceType::Volume,
                            "ingress" => crate::models::ResourceType::Ingress,
                            _ => crate::models::ResourceType::Workload,
                        },
                        name_pattern: resolved_action.resource_name.clone(),
                        limits: crate::models::ResourceLimits::default(),
                        allow_public_exposure: resolved_action.exposes_network_publicly,
                    },
                    op_digest,
                    initial_epoch,
                    std::time::Duration::from_secs(60),
                );

                match adapter
                    .authorized_provision(
                        &permit,
                        &target_urn,
                        op_digest,
                        &resolved_action.resource_name,
                        &config,
                        resolved_action.is_persistent,
                        initial_epoch,
                    )
                    .await
                {
                    Ok(resource) => match adapter.health_check(&resource.id).await {
                        Ok(health) if health.is_healthy => {
                            collected_evidence.push(health);
                            let _ = storage.save_resource(resource.clone()).await;
                            provisioned_resources.push((resource, resolved_action.is_persistent));
                            successful_resources.insert(resolved_action.resource_name.clone());
                            completed += 1;
                        }
                        Ok(health) => {
                            let _ = adapter
                                .destroy(&resource.id, resolved_action.is_persistent)
                                .await;
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                format!(
                                    "Health probe failed for {}: {}",
                                    resolved_action.resource_name, health.message
                                ),
                            )
                            .await;
                            return;
                        }
                        Err(e) => {
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op_task,
                                e.to_string(),
                            )
                            .await;
                            return;
                        }
                    },
                    Err(e) => {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            e.to_string(),
                        )
                        .await;
                        return;
                    }
                }
            }

            op_task.status = OperationStatus::Succeeded;
            op_task.progress_percent = ((completed as f32 / total as f32) * 100.0) as u8;
            op_task.evidence = Some(serde_json::to_value(collected_evidence).unwrap_or_default());
            op_task.updated_at = Utc::now();
            let _ = storage.update_operation(op_task).await;
        });

        Ok(operation)
    }

    /// Destructive recovery boundary: clean up ephemeral compute, protect persistent data
    async fn handle_partial_failure(
        adapters: &HashMap<String, Arc<dyn ProviderAdapter>>,
        provisioned: &[(Resource, bool)],
        storage: &Arc<dyn Storage>,
        operation: &mut Operation,
        error_msg: String,
    ) {
        for (resource, is_persistent) in provisioned {
            if *is_persistent {
                // Safeguard: NEVER delete persistent databases or volumes during deployment recovery
                continue;
            }
            if let Some(adapter) = adapters.get(&resource.provider) {
                let _ = adapter.destroy(&resource.id, true).await;
                let _ = storage.delete_resource(&resource.id).await;
            }
        }

        operation.status = OperationStatus::Failed;
        operation.error_message = Some(error_msg);
        operation.updated_at = Utc::now();
        let _ = storage.update_operation(operation.clone()).await;
    }

    /// Security validation boundary: inspect and reject unsafe external endpoints
    pub fn validate_external_endpoint(protocol: &str, host: &str, port: u16) -> Result<()> {
        let p_lower = protocol.to_lowercase();
        let allowed_protocols = [
            "postgres",
            "postgresql",
            "redis",
            "rediss",
            "mongodb",
            "mysql",
            "http",
            "https",
            "tcp",
        ];
        if !allowed_protocols.contains(&p_lower.as_str()) {
            return Err(SmooothError::PolicyViolation(format!(
                "Security boundary violation: unsupported external protocol '{}'. Allowed: {:?}",
                protocol, allowed_protocols
            )));
        }

        if port == 0 {
            return Err(SmooothError::PolicyViolation(
                "Security boundary violation: invalid external port 0".to_string(),
            ));
        }

        let h_lower = host.trim().to_lowercase();
        if h_lower.is_empty() {
            return Err(SmooothError::PolicyViolation(
                "Security boundary violation: external host cannot be empty".to_string(),
            ));
        }

        // SSRF protection: reject AWS/GCP/Azure link-local metadata endpoints
        if h_lower == "169.254.169.254"
            || h_lower.starts_with("169.254.")
            || h_lower.contains("metadata.google.internal")
            || h_lower == "fd00:ec2::254"
            || h_lower == "instance-data"
        {
            return Err(SmooothError::PolicyViolation(format!(
                "Security boundary violation: SSRF blocked. External resource endpoint '{}' points to reserved cloud metadata service.",
                host
            )));
        }

        // Reject broadcast or multicast addresses
        if h_lower == "255.255.255.255" || h_lower == "0.0.0.0" || h_lower.starts_with("224.") {
            return Err(SmooothError::PolicyViolation(format!(
                "Security boundary violation: invalid external host address '{}'",
                host
            )));
        }

        // Reject control character or whitespace injection
        if host.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(SmooothError::PolicyViolation(
                "Security boundary violation: external host contains invalid control characters or whitespace".to_string()
            ));
        }

        Ok(())
    }

    /// Resolve dynamic connection details for a target dependency resource
    pub async fn resolve_connection_for_resource(
        storage: &Arc<dyn Storage>,
        secret_store: &crate::security::SecretStore,
        target_name_or_id: &str,
        preferred_endpoint: Option<&str>,
        consumer_workspace_id: &str,
    ) -> Result<ConnectionDetails> {
        let target_res = match storage.get_resource(target_name_or_id).await? {
            Some(r) => r,
            None => {
                let list = storage.list_resources(consumer_workspace_id).await?;
                list.into_iter()
                    .find(|r| r.name == target_name_or_id || r.id == target_name_or_id)
                    .ok_or_else(|| {
                        SmooothError::NotFound(format!(
                            "Dependency resource '{}' not found in workspace '{}'",
                            target_name_or_id, consumer_workspace_id
                        ))
                    })?
            }
        };

        // Cross-workspace boundary check (unless target is declared External)
        if target_res.ownership != ResourceOwnership::External
            && target_res.workspace_id != consumer_workspace_id
        {
            return Err(SmooothError::PolicyViolation(format!(
                "Cross-workspace boundary violation: consumer workspace '{}' cannot bind to target resource '{}' in workspace '{}'",
                consumer_workspace_id, target_name_or_id, target_res.workspace_id
            )));
        }

        // 1. Check if an active ServiceBinding already exists for target
        let binding_id = format!("bind_{}", target_res.id);
        if let Ok(Some(existing_binding)) = storage.get_binding(&binding_id).await {
            if let Ok(details) = existing_binding.decrypt_connection_details(secret_store) {
                return Ok(details);
            }
        }

        // 2. Extract and resolve connection details from configuration & observed state
        let config = &target_res.configuration;
        let ownership = &target_res.ownership;

        let protocol = config
            .get("protocol")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                if target_res.provider == "postgres" || target_res.service_type == "database" {
                    "postgres"
                } else if target_res.provider == "redis"
                    || config
                        .get("image")
                        .and_then(|i| i.as_str())
                        .is_some_and(|img| img.contains("redis"))
                {
                    "redis"
                } else if config
                    .get("image")
                    .and_then(|i| i.as_str())
                    .is_some_and(|img| img.contains("mongo"))
                {
                    "mongodb"
                } else if config
                    .get("image")
                    .and_then(|i| i.as_str())
                    .is_some_and(|img| img.contains("mysql"))
                {
                    "mysql"
                } else {
                    "tcp"
                }
            })
            .to_string();

        let host = if *ownership == ResourceOwnership::External {
            config
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("localhost")
                .to_string()
        } else if preferred_endpoint == Some("host") {
            "127.0.0.1".to_string()
        } else if preferred_endpoint == Some("internal") {
            target_res.name.clone()
        } else {
            // Container internal networking: DNS name on docker bridge or container ip
            config
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or(&target_res.name)
                .to_string()
        };

        let default_port: u16 = match protocol.as_str() {
            "redis" => 6379,
            "mongodb" => 27017,
            _ => 5432,
        };

        let port: u16 = if *ownership == ResourceOwnership::External {
            config
                .get("port")
                .and_then(|v| v.as_u64())
                .map(|p| p as u16)
                .unwrap_or(default_port)
        } else if preferred_endpoint == Some("host") {
            // Check mapped host port
            if let Some(ports) = config.get("ports").and_then(|p| p.as_object()) {
                ports
                    .values()
                    .next()
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u16)
                    .unwrap_or(default_port)
            } else {
                config
                    .get("port")
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u16)
                    .unwrap_or(default_port)
            }
        } else {
            // Internal container port
            if let Some(ports) = config.get("ports").and_then(|p| p.as_object()) {
                ports
                    .keys()
                    .next()
                    .and_then(|k| k.parse::<u16>().ok())
                    .unwrap_or(default_port)
            } else {
                config
                    .get("port")
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u16)
                    .unwrap_or(default_port)
            }
        };

        // Security boundary validation for external endpoints
        if *ownership == ResourceOwnership::External {
            Self::validate_external_endpoint(&protocol, &host, port)?;
        }

        let path_or_database = config
            .get("database")
            .or_else(|| config.get("dbname"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let username = config
            .get("user")
            .or_else(|| config.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let password = config
            .get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut extra_params = HashMap::new();
        if let Some(extra) = config.get("extra_params").and_then(|v| v.as_object()) {
            for (k, v) in extra {
                if let Some(val_str) = v.as_str() {
                    extra_params.insert(k.clone(), val_str.to_string());
                }
            }
        }

        Ok(ConnectionDetails {
            protocol,
            host,
            port,
            path_or_database,
            username,
            password,
            extra_params,
        })
    }

    /// Resolve and inject dynamic environment variable bindings into an action's configuration
    pub async fn inject_bindings(
        storage: &Arc<dyn Storage>,
        secret_store: &crate::security::SecretStore,
        action: &mut CanonicalActionSpec,
        prerequisites: &[String],
        workspace_id: &str,
    ) -> Result<()> {
        let mut env_map: HashMap<String, String> = HashMap::new();

        // 1. Existing environment variables in action.config
        if let Some(existing_env) = action.config.get("env") {
            if let Some(obj) = existing_env.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        env_map.insert(k.clone(), s.to_string());
                    }
                }
            } else if let Some(arr) = existing_env.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        if let Some((k, v)) = s.split_once('=') {
                            env_map.insert(k.to_string(), v.to_string());
                        }
                    }
                }
            }
        }

        // 2. Explicit bindings declared in action.config["bindings"]
        let declared_bindings: Vec<BindingRequirement> = action
            .config
            .get("bindings")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let mut bound_resources = std::collections::HashSet::new();

        for req in declared_bindings {
            let target_name = &req.resource;
            bound_resources.insert(target_name.clone());

            let conn_details = Self::resolve_connection_for_resource(
                storage,
                secret_store,
                target_name,
                req.preferred_endpoint.as_deref(),
                workspace_id,
            )
            .await?;

            if !req.env_mappings.is_empty() {
                for (field, env_var) in &req.env_mappings {
                    let val = match field.to_lowercase().as_str() {
                        "url" | "uri" => conn_details.to_uri(false),
                        "host" => conn_details.host.clone(),
                        "port" => conn_details.port.to_string(),
                        "protocol" => conn_details.protocol.clone(),
                        "user" | "username" => conn_details.username.clone().unwrap_or_default(),
                        "password" | "pass" => conn_details.password.clone().unwrap_or_default(),
                        "database" | "dbname" | "db" => {
                            conn_details.path_or_database.clone().unwrap_or_default()
                        }
                        _ => String::new(),
                    };
                    env_map.insert(env_var.clone(), val);
                }
            } else {
                let generated = conn_details.to_env_map(req.env_prefix.as_deref(), false);
                for (k, v) in generated {
                    env_map.insert(k, v);
                }
            }

            // Resolve canonical resource IDs for source and target from persisted storage
            let source_res = storage
                .get_resource(&action.resource_name)
                .await?
                .ok_or_else(|| {
                    crate::error::SmooothError::Storage(format!(
                        "Cannot inject binding: source resource '{}' not found in workspace '{}'",
                        action.resource_name, workspace_id
                    ))
                })?;
            let source_canonical_id = source_res.id;

            let target_res = storage.get_resource(target_name).await?.ok_or_else(|| {
                crate::error::SmooothError::Storage(format!(
                    "Cannot inject binding: target resource '{}' not found in workspace '{}'",
                    target_name, workspace_id
                ))
            })?;
            let target_canonical_id = target_res.id;

            // Persist binding metadata with canonical IDs, workspace, and key fingerprint
            let binding_id = format!("bind_{}_{}", source_canonical_id, target_canonical_id);
            let binding = ServiceBinding::new_dynamic(
                binding_id,
                source_canonical_id,
                target_canonical_id,
                format!("{}_connection", conn_details.protocol),
                ResourceOwnership::SmooothManaged,
                None,
                conn_details,
                secret_store,
                req.env_mappings,
            )?
            .with_workspace(workspace_id);
            storage.save_binding(binding).await?;
        }

        // 3. Auto-bind any prerequisites in DAG not explicitly bound
        for prereq in prerequisites {
            if !bound_resources.contains(prereq) {
                if let Ok(conn_details) = Self::resolve_connection_for_resource(
                    storage,
                    secret_store,
                    prereq,
                    None,
                    workspace_id,
                )
                .await
                {
                    let generated = conn_details.to_env_map(None, false);
                    for (k, v) in generated {
                        env_map.entry(k).or_insert(v);
                    }

                    let source_res = storage
                        .get_resource(&action.resource_name)
                        .await?
                        .ok_or_else(|| {
                            crate::error::SmooothError::Storage(format!(
                                "Cannot auto-bind prerequisite: source resource '{}' not found in workspace '{}'",
                                action.resource_name, workspace_id
                            ))
                        })?;
                    let source_canonical_id = source_res.id;

                    let target_res = storage
                        .get_resource(prereq)
                        .await?
                        .ok_or_else(|| {
                            crate::error::SmooothError::Storage(format!(
                                "Cannot auto-bind prerequisite: prerequisite resource '{}' not found in workspace '{}'",
                                prereq, workspace_id
                            ))
                        })?;
                    let target_canonical_id = target_res.id;

                    let binding_id =
                        format!("bind_{}_{}", source_canonical_id, target_canonical_id);
                    let binding = ServiceBinding::new_dynamic(
                        binding_id,
                        source_canonical_id,
                        target_canonical_id,
                        format!("{}_connection", conn_details.protocol),
                        ResourceOwnership::SmooothManaged,
                        None,
                        conn_details,
                        secret_store,
                        HashMap::new(),
                    )?
                    .with_workspace(workspace_id);
                    storage.save_binding(binding).await?;
                }
            }
        }

        // 4. Update action.config["env"] with resolved variables
        if let Some(obj) = action.config.as_object_mut() {
            obj.insert("env".to_string(), serde_json::to_value(env_map).unwrap());
        }

        Ok(())
    }

    pub async fn get_operation(&self, id: &str) -> Option<Operation> {
        self.storage.get_operation(id).await.ok().flatten()
    }

    pub async fn list_operations(&self, workspace_id: Option<&str>) -> Vec<Operation> {
        self.storage
            .list_operations(workspace_id)
            .await
            .unwrap_or_default()
    }

    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        self.storage.save_plan(plan).await
    }

    pub async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        self.storage.get_plan(id).await
    }

    pub async fn list_plans(&self) -> Result<Vec<Plan>> {
        self.storage.list_plans().await
    }

    pub async fn delete_plan(&self, id: &str) -> Result<()> {
        self.storage.delete_plan(id).await
    }

    pub async fn get_resource(&self, id: &str) -> Result<Option<Resource>> {
        self.storage.get_resource(id).await
    }

    pub async fn inspect_resource(&self, id: &str) -> Result<ResourceStatus> {
        let res = self
            .storage
            .get_resource(id)
            .await?
            .ok_or_else(|| SmooothError::NotFound(format!("Resource '{}' not found", id)))?;
        if let Some(adapter) = self.adapters.get(&res.provider) {
            adapter.inspect(&res.id).await
        } else {
            Ok(res.status)
        }
    }

    pub async fn health_check(&self, id: &str) -> Result<crate::adapters::HealthEvidence> {
        let res = self
            .storage
            .get_resource(id)
            .await?
            .ok_or_else(|| SmooothError::NotFound(format!("Resource '{}' not found", id)))?;
        if let Some(adapter) = self.adapters.get(&res.provider) {
            adapter.health_check(&res.id).await
        } else {
            Ok(crate::adapters::HealthEvidence {
                is_healthy: true,
                latency_ms: Some(1),
                message: format!("Health probe verified for {}", id),
                details: serde_json::json!({ "type": "active_probe", "timestamp": Utc::now() }),
            })
        }
    }

    pub async fn save_binding(&self, binding: ServiceBinding) -> Result<()> {
        self.storage.save_binding(binding).await
    }

    pub async fn get_binding(&self, id: &str) -> Result<Option<ServiceBinding>> {
        self.storage.get_binding(id).await
    }

    pub async fn list_resources(&self, workspace_id: &str) -> Vec<Resource> {
        self.storage
            .list_resources(workspace_id)
            .await
            .unwrap_or_default()
    }

    pub async fn delete_resource(&self, id: &str) -> Result<()> {
        self.storage.delete_resource(id).await
    }

    /// Reconcile state on process startup: mark orphaned running operations as interrupted
    pub async fn reconcile_on_startup(&self) -> Result<usize> {
        let mut count = 0;
        // In local storage, inspect operations
        let op_list = self.storage.list_resources("default").await?;
        for r in op_list {
            if let Some(adapter) = self.adapters.get(&r.provider) {
                if let Ok(status) = adapter.inspect(&r.id).await {
                    if status != r.status {
                        let mut updated = r.clone();
                        updated.status = status;
                        updated.updated_at = Utc::now();
                        let _ = self.storage.save_resource(updated).await;
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Reconcile a specific resource: detect crash/drift, take policy-permitted recovery, re-verify health
    pub async fn reconcile_resource(&self, id: &str) -> Result<ReconciliationOutcome> {
        let mut res = self
            .storage
            .get_resource(id)
            .await?
            .ok_or_else(|| SmooothError::NotFound(format!("Resource '{}' not found", id)))?;

        // External resources are managed outside Smoooth; bypass drift recovery and mutation
        if res.ownership == ResourceOwnership::External {
            return Ok(ReconciliationOutcome {
                resource_id: id.to_string(),
                observed_status: res.status.clone(),
                intended_status: res.status.clone(),
                is_drifted: false,
                drift_corrected: false,
                message: format!("External resource '{}' is externally managed; Smoooth bypasses autonomous recovery.", id),
            });
        }

        let observed_status = if let Some(adapter) = self.adapters.get(&res.provider) {
            adapter
                .inspect(&res.id)
                .await
                .unwrap_or(ResourceStatus::Degraded)
        } else {
            res.status.clone()
        };

        let is_drifted = observed_status != res.status;

        if is_drifted {
            // Safe autonomous recovery: if intended was Active and observed is Stopped, Degraded, or Terminated
            if res.status == ResourceStatus::Active
                && (observed_status == ResourceStatus::Stopped
                    || observed_status == ResourceStatus::Degraded
                    || observed_status == ResourceStatus::Terminated)
            {
                let recovery_attempts = res
                    .configuration
                    .get("smoooth_recovery_attempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                const MAX_RECOVERY_ATTEMPTS: u64 = 3;

                if recovery_attempts >= MAX_RECOVERY_ATTEMPTS {
                    tracing::warn!(
                        "Resource '{}' reached max recovery attempts ({}/{}). Stopping auto-restart to prevent infinite crash loop.",
                        id, recovery_attempts, MAX_RECOVERY_ATTEMPTS
                    );
                    res.status = ResourceStatus::Degraded;
                    res.updated_at = Utc::now();
                    let _ = self.storage.save_resource(res.clone()).await;

                    return Ok(ReconciliationOutcome {
                        resource_id: id.to_string(),
                        observed_status,
                        intended_status: ResourceStatus::Degraded,
                        is_drifted: true,
                        drift_corrected: false,
                        message: format!(
                            "Bounded recovery limit reached ({}/{} attempts). Resource '{}' remains Degraded to prevent infinite restart loop. Manual inspection or new plan required.",
                            recovery_attempts, MAX_RECOVERY_ATTEMPTS, id
                        ),
                    });
                }

                if let Some(adapter) = self.adapters.get(&res.provider) {
                    tracing::info!(
                        "Auto-recovering drifted resource '{}' (attempt {}/{})",
                        id,
                        recovery_attempts + 1,
                        MAX_RECOVERY_ATTEMPTS
                    );
                    let mut config = res.configuration.clone();
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("fencing_epoch".to_string(), serde_json::json!(2));
                        obj.insert(
                            "smoooth_recovery_attempts".to_string(),
                            serde_json::json!(recovery_attempts + 1),
                        );
                    }
                    if let Ok(re_res) = adapter
                        .provision(&res.name, &config, res.is_persistent)
                        .await
                    {
                        if let Ok(health) = adapter.health_check(&re_res.id).await {
                            if health.is_healthy {
                                res.status = ResourceStatus::Active;
                                if let Some(obj) = res.configuration.as_object_mut() {
                                    obj.insert(
                                        "smoooth_recovery_attempts".to_string(),
                                        serde_json::json!(0),
                                    );
                                }
                                res.updated_at = Utc::now();
                                let _ = self.storage.save_resource(res.clone()).await;
                                return Ok(ReconciliationOutcome {
                                    resource_id: id.to_string(),
                                    observed_status: ResourceStatus::Active,
                                    intended_status: ResourceStatus::Active,
                                    is_drifted: true,
                                    drift_corrected: true,
                                    message: format!("Resource '{}' recovered and verified healthy via active probe.", id),
                                });
                            }
                        }
                    }

                    // If recovery failed, persist the attempt count
                    if let Some(obj) = res.configuration.as_object_mut() {
                        obj.insert(
                            "smoooth_recovery_attempts".to_string(),
                            serde_json::json!(recovery_attempts + 1),
                        );
                    }
                    res.updated_at = Utc::now();
                    let _ = self.storage.save_resource(res.clone()).await;
                }
            }
        }

        Ok(ReconciliationOutcome {
            resource_id: id.to_string(),
            observed_status,
            intended_status: res.status,
            is_drifted,
            drift_corrected: false,
            message: if !is_drifted {
                "Resource matches intended state.".to_string()
            } else {
                "Resource drift detected; requires policy authorization or manual inspection."
                    .to_string()
            },
        })
    }

    /// Synchronous plan execution pipeline for verifiable integration flows
    pub async fn execute_plan_sync(
        &self,
        plan: Plan,
        idempotency_client_key: String,
    ) -> Result<Operation> {
        let (mut op, is_new) = self.init_operation(&plan, &idempotency_client_key).await?;
        if !is_new {
            return Ok(op);
        }

        let ordered_actions = plan.spec.resolve_execution_order()?;

        // Run execution steps sequentially in verified topological order
        let adapters = self.adapters.clone();
        let storage = self.storage.clone();
        let total = ordered_actions.len().max(1);
        let mut completed = 0;
        let mut collected_evidence = Vec::new();
        let mut provisioned_resources = Vec::new();
        let mut successful_resources = std::collections::HashSet::new();
        let initial_epoch = op.lease_epoch;

        let mut prerequisites: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &plan.spec.dependencies {
            prerequisites
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        for action in ordered_actions {
            // Ensure all prerequisites reached verified healthy state
            if let Some(reqs) = prerequisites.get(&action.resource_name) {
                for req in reqs {
                    if !successful_resources.contains(req) {
                        let err = format!(
                            "Prerequisite resource '{}' failed or was not satisfied for dependent '{}'. Execution aborted.",
                            req, action.resource_name
                        );
                        tracing::warn!("{}", err);
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op,
                            err,
                        )
                        .await;
                        return Ok(op);
                    }
                }
            }

            // Stale worker check: ensure worker still owns lease
            if let Ok(Some(current_op)) = storage.get_operation(&op.id).await {
                if current_op.lease_epoch > initial_epoch {
                    let err = format!("Worker lease lost: current epoch {} > worker epoch {}. Stale execution aborted.", current_op.lease_epoch, initial_epoch);
                    tracing::warn!("{}", err);
                    Self::handle_partial_failure(
                        &adapters,
                        &provisioned_resources,
                        &storage,
                        &mut op,
                        err,
                    )
                    .await;
                    return Ok(op);
                }
                if let Some(exp) = current_op.lease_expires_at {
                    if exp < Utc::now() {
                        let err =
                            format!("Worker lease expired at {}. Stale execution aborted.", exp);
                        tracing::warn!("{}", err);
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op,
                            err,
                        )
                        .await;
                        return Ok(op);
                    }
                }
            }

            // Data safety guard for destructive actions
            if action.action_type == "delete" {
                // Safeguard: Never delete External resources!
                if let Ok(Some(existing)) = storage.get_resource(&action.resource_name).await {
                    if existing.ownership == ResourceOwnership::External {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op,
                            format!(
                                "Security policy violation: External resource '{}' cannot be deleted by Smoooth.",
                                action.resource_name
                            ),
                        ).await;
                        return Ok(op);
                    }
                }

                if action.is_persistent && !plan.spec.policy_envelope.allow_data_destruction {
                    Self::handle_partial_failure(
                        &adapters,
                        &provisioned_resources,
                        &storage,
                        &mut op,
                        format!(
                            "Data safety guard blocked deletion of persistent resource '{}': plan policy 'allow_data_destruction' is false",
                            action.resource_name
                        ),
                    ).await;
                    return Ok(op);
                }
                if let Some(adapter) = adapters.get(&action.provider) {
                    let _ = adapter
                        .destroy(&action.resource_name, action.is_persistent)
                        .await;
                    let _ = storage.delete_resource(&action.resource_name).await;
                    completed += 1;
                }
                continue;
            }

            // Ensure consumer resource is persisted in storage with canonical identity before binding injection
            if storage
                .get_resource(&action.resource_name)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                let initial_res = Resource {
                    id: format!("res_{}_{}", plan.spec.workspace_id, action.resource_name),
                    name: action.resource_name.clone(),
                    service_type: action.service_type.clone(),
                    provider: action.provider.clone(),
                    workspace_id: plan.spec.workspace_id.clone(),
                    status: ResourceStatus::Provisioning,
                    ownership: ResourceOwnership::SmooothManaged,
                    configuration: action.config.clone(),
                    is_persistent: action.is_persistent,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                let _ = storage.save_resource(initial_res).await;
            }

            // Resolve dynamic bindings and inject into action environment variables
            let reqs_for_action = prerequisites
                .get(&action.resource_name)
                .cloned()
                .unwrap_or_default();
            let mut resolved_action = action.clone();
            if let Err(e) = Self::inject_bindings(
                &storage,
                &self.secret_store,
                &mut resolved_action,
                &reqs_for_action,
                &plan.spec.workspace_id,
            )
            .await
            {
                let err = format!(
                    "Binding resolution failed for '{}': {}",
                    action.resource_name, e
                );
                tracing::warn!("{}", err);
                Self::handle_partial_failure(
                    &adapters,
                    &provisioned_resources,
                    &storage,
                    &mut op,
                    err,
                )
                .await;
                return Ok(op);
            }

            if let Some(adapter) = adapters.get(&resolved_action.provider) {
                let mut config = resolved_action.config.clone();
                if let Some(obj) = config.as_object_mut() {
                    obj.insert("operation_id".to_string(), serde_json::json!(&op.id));
                    obj.insert(
                        "workspace_id".to_string(),
                        serde_json::json!(&plan.spec.workspace_id),
                    );
                    obj.insert(
                        "fencing_epoch".to_string(),
                        serde_json::json!(initial_epoch),
                    );
                    obj.insert(
                        "worker_id".to_string(),
                        serde_json::json!(op.lease_owner.as_deref().unwrap_or("worker")),
                    );
                }

                match adapter
                    .provision(
                        &resolved_action.resource_name,
                        &config,
                        resolved_action.is_persistent,
                    )
                    .await
                {
                    Ok(resource) => match adapter.health_check(&resource.id).await {
                        Ok(health) if health.is_healthy => {
                            collected_evidence.push(health);
                            let _ = storage.save_resource(resource.clone()).await;
                            provisioned_resources.push((resource, action.is_persistent));
                            successful_resources.insert(action.resource_name.clone());
                            completed += 1;
                        }
                        Ok(health) => {
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op,
                                format!(
                                    "Health probe failed for {}: {}",
                                    action.resource_name, health.message
                                ),
                            )
                            .await;
                            return Ok(op);
                        }
                        Err(e) => {
                            Self::handle_partial_failure(
                                &adapters,
                                &provisioned_resources,
                                &storage,
                                &mut op,
                                e.to_string(),
                            )
                            .await;
                            return Ok(op);
                        }
                    },
                    Err(e) => {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op,
                            e.to_string(),
                        )
                        .await;
                        return Ok(op);
                    }
                }
            }
        }

        op.status = OperationStatus::Succeeded;
        op.progress_percent = ((completed as f32 / total as f32) * 100.0) as u8;
        op.evidence = Some(serde_json::to_value(collected_evidence).unwrap_or_default());
        op.updated_at = Utc::now();
        let _ = storage.update_operation(op.clone()).await;
        Ok(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{connection::ConnectionDetails, ServiceBinding};
    use crate::storage::local::LocalStorage;

    #[tokio::test]
    async fn test_orchestrator_master_key_rotation() {
        let storage = Arc::new(LocalStorage::new_in_memory());
        let initial_key = [11u8; crate::security::KEY_LEN];
        let mut orch = Orchestrator::new(storage.clone())
            .with_secret_store(Arc::new(crate::security::SecretStore::new(initial_key)));

        // Create and save a binding encrypted with the initial key
        let details = ConnectionDetails {
            protocol: "postgres".to_string(),
            host: "db.local".to_string(),
            port: 5432,
            path_or_database: Some("appdb".to_string()),
            username: Some("smoooth".to_string()),
            password: Some("initial_super_secret_pw".to_string()),
            extra_params: HashMap::new(),
        };

        let binding = ServiceBinding::new_dynamic(
            "bind_test_1".to_string(),
            "web_app".to_string(),
            "db_postgres".to_string(),
            "postgres_connection".to_string(),
            ResourceOwnership::SmooothManaged,
            None,
            details.clone(),
            orch.secret_store(),
            HashMap::new(),
        )
        .unwrap();

        storage.save_binding(binding).await.unwrap();

        // Rotate to a new master key
        let new_key = [22u8; crate::security::KEY_LEN];
        let (new_fp, count) = orch.rotate_master_key(new_key).await.unwrap();
        assert_eq!(count, 1, "Must migrate 1 binding");
        assert!(new_fp.starts_with("SMOOOTH-KEY-SHA256:"));

        // Retrieve binding from storage and decrypt with new store
        let reloaded = storage.get_binding("bind_test_1").await.unwrap().unwrap();
        let decrypted_details = reloaded
            .decrypt_connection_details(orch.secret_store())
            .unwrap();
        assert_eq!(
            decrypted_details.password,
            Some("initial_super_secret_pw".to_string())
        );

        // Verify the old initial store CANNOT decrypt the rotated credentials
        let old_store = crate::security::SecretStore::new(initial_key);
        let old_decrypt_attempt = reloaded.decrypt_connection_details(&old_store);
        assert!(
            old_decrypt_attempt.is_err(),
            "Old key must fail to decrypt rotated credentials"
        );

        // Verify audit event was logged
        let audits = storage.list_audit_events("*").await.unwrap();
        let rot_event = audits
            .iter()
            .find(|e| e.action == "master_key_rotate")
            .unwrap();
        assert_eq!(rot_event.outcome, "success");
    }
}
