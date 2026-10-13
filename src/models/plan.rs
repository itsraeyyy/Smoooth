use std::collections::HashMap;
use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Low,
    Moderate,
    High,
    Critical,
}

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
    pub config: serde_json::Value,
    pub is_persistent: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub spec: CanonicalExecutionSpec,
    pub risk_tier: RiskTier,
    pub estimated_monthly_cost_usd: Option<f64>,
    pub plan_hash: String,
    pub approved_plan_hash: Option<String>,
    pub approved_by: Option<String>,
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

    pub fn new(id: String, spec: CanonicalExecutionSpec, risk_tier: RiskTier, cost: Option<f64>) -> Self {
        let plan_hash = Self::compute_canonical_hash(&spec);
        Self {
            id,
            spec,
            risk_tier,
            estimated_monthly_cost_usd: cost,
            plan_hash,
            approved_plan_hash: None,
            approved_by: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_approval_valid(&self) -> bool {
        match &self.approved_plan_hash {
            Some(approved_hash) => approved_hash == &self.plan_hash,
            None => false,
        }
    }

    pub fn requires_approval(&self) -> bool {
        self.risk_tier != RiskTier::Low
    }

    pub fn approve(&mut self, approver: String) {
        self.approved_plan_hash = Some(self.plan_hash.clone());
        self.approved_by = Some(approver);
    }
}
