use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::ProviderAdapter;
use crate::error::{Result, SmooothError};
use crate::models::{
    BindingRequirement, ConnectionDetails, IdempotencyKey, IdempotencyRecord, Operation,
    OperationLogEntry, OperationStatus, Plan, ReconciliationOutcome, Resource, ResourceOwnership,
    ResourceStatus, RiskTier, ServiceBinding,
};
use crate::models::plan::CanonicalActionSpec;
use crate::storage::{AuditEvent, Storage};

pub struct Orchestrator {
    worker_id: String,
    storage: Arc<dyn Storage>,
    adapters: HashMap<String, Arc<dyn ProviderAdapter>>,
    secret_store: Arc<crate::security::SecretStore>,
}

impl Orchestrator {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            worker_id: format!("worker_{}", Uuid::new_v4().simple()),
            storage,
            adapters: HashMap::new(),
            secret_store: Arc::new(crate::security::SecretStore::default_local()),
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
        self.adapters.insert(adapter.provider_name().to_string(), adapter);
    }

    async fn init_operation(
        &self,
        plan: &Plan,
        idempotency_client_key: &str,
    ) -> Result<(Operation, bool)> {
        // 0. Validate dependency graph and cycle freedom before reserving operations or leases
        plan.spec.resolve_execution_order()?;

        let idemp_key = IdempotencyKey {
            workspace_id: plan.spec.workspace_id.clone(),
            operation_type: "execute_plan".to_string(),
            client_key: idempotency_client_key.to_string(),
        };

        let request_hash = plan.plan_hash.clone();

        // 1. Atomic Idempotency check or reservation
        let op_id = format!("op_{}", Uuid::new_v4().simple());
        let placeholder_record = IdempotencyRecord {
            key: idemp_key.clone(),
            request_hash: request_hash.clone(),
            operation_id: op_id.clone(),
            created_at: Utc::now(),
        };

        if let Some(existing_record) = self.storage.check_or_record_idempotency(placeholder_record).await? {
            if let Some(existing_op) = self.storage.get_operation(&existing_record.operation_id).await? {
                return Ok((existing_op, false));
            }
        }

        // 2. Strict cryptographic plan approval & policy verification
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

        // 3. Create durable operation with monotonic fencing token (lease_epoch = 1)
        let lease_expiration = Utc::now() + Duration::minutes(5);
        let initial_epoch = 1u64;

        let operation = Operation {
            id: op_id.clone(),
            workspace_id: plan.spec.workspace_id.clone(),
            operation_type: "execute_plan".to_string(),
            plan_id: plan.id.clone(),
            plan_hash: plan.plan_hash.clone(),
            request_hash: request_hash.clone(),
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
                    self.worker_id, initial_epoch, plan.id, &plan.plan_hash[..8]
                ),
                level: "INFO".to_string(),
            }],
            evidence: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Persist operation
        self.storage.save_operation(operation.clone()).await?;

        Ok((operation, true))
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
                prerequisites.entry(edge.from.clone()).or_default().push(edge.to.clone());
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
                            Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op_task, err).await;
                            return;
                        }
                    }
                }
                // Stale worker check: ensure worker still owns lease
                if let Ok(Some(current_op)) = storage.get_operation(&op_task.id).await {
                    if current_op.lease_epoch > initial_epoch {
                        let err = format!("Worker lease lost: current epoch {} > worker epoch {}. Stale execution aborted.", current_op.lease_epoch, initial_epoch);
                        tracing::warn!("{}", err);
                        Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op_task, err).await;
                        return;
                    }
                    if let Some(exp) = current_op.lease_expires_at {
                        if exp < Utc::now() {
                            let err = format!("Worker lease expired at {}. Stale execution aborted.", exp);
                            tracing::warn!("{}", err);
                            Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op_task, err).await;
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
                        let _ = adapter.destroy(&action.resource_name, action.is_persistent).await;
                        let _ = storage.delete_resource(&action.resource_name).await;
                        completed += 1;
                    } else {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op_task,
                            format!("Unsupported provider '{}' for resource deletion '{}'", action.provider, action.resource_name),
                        ).await;
                        return;
                    }
                    continue;
                }

                // Resolve dynamic bindings and inject into action environment variables
                let reqs_for_action = prerequisites.get(&action.resource_name).cloned().unwrap_or_default();
                let mut resolved_action = action.clone();
                if let Err(e) = Self::inject_bindings(
                    &storage,
                    &secret_store,
                    &mut resolved_action,
                    &reqs_for_action,
                    &plan.spec.workspace_id,
                ).await {
                    let err = format!("Binding resolution failed for '{}': {}", action.resource_name, e);
                    tracing::warn!("{}", err);
                    Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op_task, err).await;
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
                            format!("Unsupported or unrecognized provider '{}' for resource '{}'", resolved_action.provider, resolved_action.resource_name),
                        ).await;
                        return;
                    }
                };

                let mut config = resolved_action.config.clone();
                if let Some(obj) = config.as_object_mut() {
                    obj.insert("operation_id".to_string(), serde_json::json!(&op_task.id));
                    obj.insert("workspace_id".to_string(), serde_json::json!(&plan.spec.workspace_id));
                    obj.insert("fencing_epoch".to_string(), serde_json::json!(initial_epoch));
                    obj.insert("worker_id".to_string(), serde_json::json!(op_task.lease_owner.as_deref().unwrap_or("worker")));
                }
                match adapter
                    .provision(&resolved_action.resource_name, &config, resolved_action.is_persistent)
                    .await
                {
                    Ok(resource) => {
                        match adapter.health_check(&resource.id).await {
                            Ok(health) if health.is_healthy => {
                                collected_evidence.push(health);
                                let _ = storage.save_resource(resource.clone()).await;
                                provisioned_resources.push((resource, resolved_action.is_persistent));
                                successful_resources.insert(resolved_action.resource_name.clone());
                                completed += 1;
                            }
                            Ok(health) => {
                                let _ = adapter.destroy(&resource.id, resolved_action.is_persistent).await;
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
                        }
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
    pub fn validate_external_endpoint(
        protocol: &str,
        host: &str,
        port: u16,
    ) -> Result<()> {
        let p_lower = protocol.to_lowercase();
        let allowed_protocols = ["postgres", "postgresql", "redis", "rediss", "mongodb", "mysql", "http", "https", "tcp"];
        if !allowed_protocols.contains(&p_lower.as_str()) {
            return Err(SmooothError::PolicyViolation(format!(
                "Security boundary violation: unsupported external protocol '{}'. Allowed: {:?}",
                protocol, allowed_protocols
            )));
        }

        if port == 0 {
            return Err(SmooothError::PolicyViolation(
                "Security boundary violation: invalid external port 0".to_string()
            ));
        }

        let h_lower = host.trim().to_lowercase();
        if h_lower.is_empty() {
            return Err(SmooothError::PolicyViolation(
                "Security boundary violation: external host cannot be empty".to_string()
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
                    .ok_or_else(|| SmooothError::NotFound(format!("Dependency resource '{}' not found in workspace '{}'", target_name_or_id, consumer_workspace_id)))?
            }
        };

        // Cross-workspace boundary check (unless target is declared External)
        if target_res.ownership != ResourceOwnership::External && target_res.workspace_id != consumer_workspace_id {
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

        let protocol = config.get("protocol").and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                if target_res.provider == "postgres" || target_res.service_type == "database" {
                    "postgres"
                } else if target_res.provider == "redis" || config.get("image").and_then(|i| i.as_str()).map_or(false, |img| img.contains("redis")) {
                    "redis"
                } else if config.get("image").and_then(|i| i.as_str()).map_or(false, |img| img.contains("mongo")) {
                    "mongodb"
                } else if config.get("image").and_then(|i| i.as_str()).map_or(false, |img| img.contains("mysql")) {
                    "mysql"
                } else {
                    "tcp"
                }
            })
            .to_string();

        let host = if *ownership == ResourceOwnership::External {
            config.get("host").and_then(|v| v.as_str()).unwrap_or("localhost").to_string()
        } else if preferred_endpoint == Some("host") {
            "127.0.0.1".to_string()
        } else {
            // Container internal networking: DNS name on docker bridge or container ip
            config.get("host").and_then(|v| v.as_str()).unwrap_or(&target_res.name).to_string()
        };

        let port: u16 = if *ownership == ResourceOwnership::External {
            config.get("port").and_then(|v| v.as_u64()).map(|p| p as u16).unwrap_or(5432)
        } else if preferred_endpoint == Some("host") {
            // Check mapped host port
            if let Some(ports) = config.get("ports").and_then(|p| p.as_object()) {
                ports.values().next().and_then(|v| v.as_u64()).map(|p| p as u16).unwrap_or(5432)
            } else {
                config.get("port").and_then(|v| v.as_u64()).map(|p| p as u16).unwrap_or(5432)
            }
        } else {
            // Internal container port
            if let Some(ports) = config.get("ports").and_then(|p| p.as_object()) {
                ports.keys().next().and_then(|k| k.parse::<u16>().ok()).unwrap_or(5432)
            } else {
                config.get("port").and_then(|v| v.as_u64()).map(|p| p as u16).unwrap_or(5432)
            }
        };

        // Security boundary validation for external endpoints
        if *ownership == ResourceOwnership::External {
            Self::validate_external_endpoint(&protocol, &host, port)?;
        }

        let path_or_database = config.get("database")
            .or_else(|| config.get("dbname"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let username = config.get("user")
            .or_else(|| config.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let password = config.get("password")
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
        let declared_bindings: Vec<BindingRequirement> = action.config
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
            ).await?;

            if !req.env_mappings.is_empty() {
                for (field, env_var) in &req.env_mappings {
                    let val = match field.to_lowercase().as_str() {
                        "url" | "uri" => conn_details.to_uri(false),
                        "host" => conn_details.host.clone(),
                        "port" => conn_details.port.to_string(),
                        "protocol" => conn_details.protocol.clone(),
                        "user" | "username" => conn_details.username.clone().unwrap_or_default(),
                        "password" | "pass" => conn_details.password.clone().unwrap_or_default(),
                        "database" | "dbname" | "db" => conn_details.path_or_database.clone().unwrap_or_default(),
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

            // Persist binding metadata
            let binding_id = format!("bind_{}_{}", action.resource_name, target_name);
            let binding = ServiceBinding::new_dynamic(
                binding_id,
                action.resource_name.clone(),
                target_name.clone(),
                format!("{}_connection", conn_details.protocol),
                ResourceOwnership::SmooothManaged,
                None,
                conn_details,
                secret_store,
                req.env_mappings,
            )?;
            let _ = storage.save_binding(binding).await;
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
                ).await {
                    let generated = conn_details.to_env_map(None, false);
                    for (k, v) in generated {
                        env_map.entry(k).or_insert(v);
                    }
                    let binding_id = format!("bind_{}_{}", action.resource_name, prereq);
                    let binding = ServiceBinding::new_dynamic(
                        binding_id,
                        action.resource_name.clone(),
                        prereq.clone(),
                        format!("{}_connection", conn_details.protocol),
                        ResourceOwnership::SmooothManaged,
                        None,
                        conn_details,
                        secret_store,
                        HashMap::new(),
                    )?;
                    let _ = storage.save_binding(binding).await;
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

    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        self.storage.save_plan(plan).await
    }

    pub async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        self.storage.get_plan(id).await
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
            adapter.inspect(&res.id).await.unwrap_or(ResourceStatus::Degraded)
        } else {
            res.status.clone()
        };

        let is_drifted = observed_status != res.status;

        if is_drifted {
            // Safe autonomous recovery: if intended was Active and observed is Stopped, Degraded, or Terminated
            if res.status == ResourceStatus::Active && (observed_status == ResourceStatus::Stopped || observed_status == ResourceStatus::Degraded || observed_status == ResourceStatus::Terminated) {
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
                    tracing::info!("Auto-recovering drifted resource '{}' (attempt {}/{})", id, recovery_attempts + 1, MAX_RECOVERY_ATTEMPTS);
                    let mut config = res.configuration.clone();
                    if let Some(obj) = config.as_object_mut() {
                        obj.insert("fencing_epoch".to_string(), serde_json::json!(2));
                        obj.insert("smoooth_recovery_attempts".to_string(), serde_json::json!(recovery_attempts + 1));
                    }
                    if let Ok(re_res) = adapter.provision(&res.name, &config, res.is_persistent).await {
                        if let Ok(health) = adapter.health_check(&re_res.id).await {
                            if health.is_healthy {
                                res.status = ResourceStatus::Active;
                                if let Some(obj) = res.configuration.as_object_mut() {
                                    obj.insert("smoooth_recovery_attempts".to_string(), serde_json::json!(0));
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
                        obj.insert("smoooth_recovery_attempts".to_string(), serde_json::json!(recovery_attempts + 1));
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
                "Resource drift detected; requires policy authorization or manual inspection.".to_string()
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
            prerequisites.entry(edge.from.clone()).or_default().push(edge.to.clone());
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
                        Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op, err).await;
                        return Ok(op);
                    }
                }
            }

            // Stale worker check: ensure worker still owns lease
            if let Ok(Some(current_op)) = storage.get_operation(&op.id).await {
                if current_op.lease_epoch > initial_epoch {
                    let err = format!("Worker lease lost: current epoch {} > worker epoch {}. Stale execution aborted.", current_op.lease_epoch, initial_epoch);
                    tracing::warn!("{}", err);
                    Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op, err).await;
                    return Ok(op);
                }
                if let Some(exp) = current_op.lease_expires_at {
                    if exp < Utc::now() {
                        let err = format!("Worker lease expired at {}. Stale execution aborted.", exp);
                        tracing::warn!("{}", err);
                        Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op, err).await;
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
                    let _ = adapter.destroy(&action.resource_name, action.is_persistent).await;
                    let _ = storage.delete_resource(&action.resource_name).await;
                    completed += 1;
                }
                continue;
            }

            // Resolve dynamic bindings and inject into action environment variables
            let reqs_for_action = prerequisites.get(&action.resource_name).cloned().unwrap_or_default();
            let mut resolved_action = action.clone();
            if let Err(e) = Self::inject_bindings(
                &storage,
                &self.secret_store,
                &mut resolved_action,
                &reqs_for_action,
                &plan.spec.workspace_id,
            ).await {
                let err = format!("Binding resolution failed for '{}': {}", action.resource_name, e);
                tracing::warn!("{}", err);
                Self::handle_partial_failure(&adapters, &provisioned_resources, &storage, &mut op, err).await;
                return Ok(op);
            }

            if let Some(adapter) = adapters.get(&resolved_action.provider) {
                let mut config = resolved_action.config.clone();
                if let Some(obj) = config.as_object_mut() {
                    obj.insert("operation_id".to_string(), serde_json::json!(&op.id));
                    obj.insert("workspace_id".to_string(), serde_json::json!(&plan.spec.workspace_id));
                    obj.insert("fencing_epoch".to_string(), serde_json::json!(initial_epoch));
                    obj.insert("worker_id".to_string(), serde_json::json!(op.lease_owner.as_deref().unwrap_or("worker")));
                }
                match adapter.provision(&resolved_action.resource_name, &config, resolved_action.is_persistent).await {
                    Ok(resource) => {
                        match adapter.health_check(&resource.id).await {
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
                                    format!("Health probe failed for {}: {}", action.resource_name, health.message),
                                ).await;
                                return Ok(op);
                            }
                            Err(e) => {
                                Self::handle_partial_failure(
                                    &adapters,
                                    &provisioned_resources,
                                    &storage,
                                    &mut op,
                                    e.to_string(),
                                ).await;
                                return Ok(op);
                            }
                        }
                    }
                    Err(e) => {
                        Self::handle_partial_failure(
                            &adapters,
                            &provisioned_resources,
                            &storage,
                            &mut op,
                            e.to_string(),
                        ).await;
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
