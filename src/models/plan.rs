use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};

use crate::error::SmooothError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Low,
    Moderate,
    High,
    Critical,
}

/// Explicit dependency relationship between two resources.
///
/// Semantics:
/// - `from` is the dependent resource (e.g. "api_server")
/// - `to` is the prerequisite resource (e.g. "database")
///
/// Prerequisite `to` must reach verified active/healthy state before dependent `from` begins execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyConstraints {
    pub allow_public_network: bool,
    pub allow_data_destruction: bool,
    pub max_monthly_budget_usd: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalActionSpec {
    pub resource_name: String,
    pub provider: String,
    pub service_type: String,
    pub action_type: String, // "create", "update", "delete"
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub is_persistent: bool,
    #[serde(default)]
    pub exposes_network_publicly: bool,
}

/// Versioned, canonical execution specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalExecutionSpec {
    pub schema_version: u32,
    pub workspace_id: String,
    pub environment: String, // "production", "staging", "development"
    pub target_id: String,
    pub actions: Vec<CanonicalActionSpec>,
    pub dependencies: Vec<DependencyEdge>,
    pub immutable_artifacts: HashMap<String, String>, // artifact name -> immutable sha256 digest
    pub policy_envelope: PolicyConstraints,
}

impl CanonicalExecutionSpec {
    /// Resolves the actions into a deterministic topological execution order.
    ///
    /// ## Dependency Semantics:
    /// In a `DependencyEdge { from, to }`:
    /// - `from` is the **dependent** resource (e.g. "web_app").
    /// - `to` is the **prerequisite** resource (e.g. "database").
    ///
    /// Prerequisite `to` MUST be provisioned, verified, and reach a healthy state
    /// before dependent `from` is allowed to execute.
    ///
    /// ## Validation Guarantees:
    /// - Rejects references to undeclared resources.
    /// - Rejects self-dependencies (`from == to`).
    /// - Detects cyclic dependencies (`A -> B -> A` or multi-node cycles) and reports cycle participants.
    /// - Produces a deterministic execution order for equivalent plans using BTreeSet-based queueing.
    /// - If no dependency edges are declared, preserves the original declared action order.
    pub fn resolve_execution_order(&self) -> Result<Vec<CanonicalActionSpec>, SmooothError> {
        let action_map: HashMap<&str, &CanonicalActionSpec> = self
            .actions
            .iter()
            .map(|a| (a.resource_name.as_str(), a))
            .collect();

        // 1. Validate that every dependency references a declared action
        for edge in &self.dependencies {
            if !action_map.contains_key(edge.from.as_str()) {
                return Err(SmooothError::Validation(format!(
                    "Invalid dependency: dependent resource '{}' is not declared in plan actions",
                    edge.from
                )));
            }
            if !action_map.contains_key(edge.to.as_str()) {
                return Err(SmooothError::Validation(format!(
                    "Invalid dependency: prerequisite resource '{}' is not declared in plan actions",
                    edge.to
                )));
            }
            if edge.from == edge.to {
                return Err(SmooothError::Validation(format!(
                    "Self-dependency detected: resource '{}' cannot depend on itself",
                    edge.from
                )));
            }
        }

        // If no dependency edges, preserve original declared order
        if self.dependencies.is_empty() {
            return Ok(self.actions.clone());
        }

        // 2. Build adjacency list and in-degree counts (Kahn's algorithm)
        // Edge in DAG: prerequisite `to` -> dependent `from`
        // Indegree of node `x`: number of prerequisites that must complete before `x` can run.
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();

        for action in &self.actions {
            adj.entry(action.resource_name.as_str()).or_default();
            in_degree.insert(action.resource_name.as_str(), 0);
        }

        for edge in &self.dependencies {
            adj.get_mut(edge.to.as_str())
                .unwrap()
                .push(edge.from.as_str());
            *in_degree.get_mut(edge.from.as_str()).unwrap() += 1;
        }

        // Deterministic queue sorted lexicographically by resource_name
        let mut zero_in_degree: BTreeSet<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut sorted_names: Vec<&str> = Vec::with_capacity(self.actions.len());

        while let Some(node) = zero_in_degree.pop_first() {
            sorted_names.push(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        zero_in_degree.insert(neighbor);
                    }
                }
            }
        }

        // 3. Cycle detection
        if sorted_names.len() != self.actions.len() {
            let mut remaining_unresolved: Vec<&str> = in_degree
                .iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(&name, _)| name)
                .collect();
            remaining_unresolved.sort();
            return Err(SmooothError::Validation(format!(
                "Cyclic dependency detected in execution graph involving resources: {:?}",
                remaining_unresolved
            )));
        }

        let ordered_actions = sorted_names
            .into_iter()
            .map(|name| action_map[name].clone())
            .collect();

        Ok(ordered_actions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanApproval {
    pub plan_hash: String,
    pub workspace_id: String,
    pub environment: String,
    pub policy_version: u32,
    pub approved_by: String,
    pub approved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub spec: CanonicalExecutionSpec,
    pub risk_tier: RiskTier,
    pub estimated_monthly_cost_usd: Option<f64>,
    pub plan_hash: String,
    pub approved_plan_hash: Option<String>,
    pub approved_by: Option<String>,
    pub approval: Option<PlanApproval>,
    pub created_at: DateTime<Utc>,
}

impl Plan {
    /// Compute deterministic canonical plan hash
    pub fn compute_canonical_hash(spec: &CanonicalExecutionSpec) -> String {
        let serialized = serde_json::to_string(spec).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn new(
        id: String,
        spec: CanonicalExecutionSpec,
        risk_tier: RiskTier,
        cost: Option<f64>,
    ) -> Self {
        let plan_hash = Self::compute_canonical_hash(&spec);
        Self {
            id,
            spec,
            risk_tier,
            estimated_monthly_cost_usd: cost,
            plan_hash,
            approved_plan_hash: None,
            approved_by: None,
            approval: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_approval_valid(&self) -> bool {
        if let Some(approval) = &self.approval {
            approval.plan_hash == self.plan_hash
                && approval.workspace_id == self.spec.workspace_id
                && approval.environment == self.spec.environment
                && approval.policy_version == self.spec.schema_version
                && approval.expires_at > Utc::now()
        } else if let Some(approved_hash) = &self.approved_plan_hash {
            approved_hash == &self.plan_hash && self.approved_by.is_some()
        } else {
            false
        }
    }

    pub fn approval_rejection_reason(&self) -> Option<&'static str> {
        if let Some(approval) = &self.approval {
            if approval.plan_hash != self.plan_hash {
                Some("Plan hash mismatch - spec was modified after approval")
            } else if approval.workspace_id != self.spec.workspace_id {
                Some("Workspace mismatch in approval binding")
            } else if approval.environment != self.spec.environment {
                Some("Target environment mismatch in approval binding")
            } else if approval.policy_version != self.spec.schema_version {
                Some("Policy version mismatch in approval binding")
            } else if approval.expires_at <= Utc::now() {
                Some("Approval has expired")
            } else {
                None
            }
        } else if self.approved_plan_hash.is_some() {
            if self.approved_plan_hash.as_ref() != Some(&self.plan_hash) {
                Some("Plan hash mismatch")
            } else {
                None
            }
        } else {
            Some("No approval found for high-risk plan")
        }
    }

    pub fn requires_approval(&self) -> bool {
        self.risk_tier != RiskTier::Low
    }

    pub fn approve(&mut self, approver: String) {
        self.approve_with_ttl(approver, 3600); // Default 1 hour TTL
    }

    pub fn approve_with_ttl(&mut self, approver: String, ttl_seconds: i64) {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds);
        self.approved_plan_hash = Some(self.plan_hash.clone());
        self.approved_by = Some(approver.clone());
        self.approval = Some(PlanApproval {
            plan_hash: self.plan_hash.clone(),
            workspace_id: self.spec.workspace_id.clone(),
            environment: self.spec.environment.clone(),
            policy_version: self.spec.schema_version,
            approved_by: approver,
            approved_at: now,
            expires_at,
        });
    }
}
