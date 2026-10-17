use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftRiskTier {
    /// Safe to converge automatically (e.g., restart crashed process)
    NonDestructive,
    /// Requires explicit human authorization (e.g., state rollback, volume change)
    Destructive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    pub resource_id: String,
    pub field: String,
    pub intended_value: serde_json::Value,
    pub observed_value: serde_json::Value,
    pub risk_tier: DriftRiskTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub workspace_id: String,
    pub detected_at: DateTime<Utc>,
    pub items: Vec<DriftItem>,
}

impl DriftReport {
    pub fn requires_approval(&self) -> bool {
        self.items
            .iter()
            .any(|i| i.risk_tier == DriftRiskTier::Destructive)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationOutcome {
    pub resource_id: String,
    pub observed_status: crate::models::ResourceStatus,
    pub intended_status: crate::models::ResourceStatus,
    pub is_drifted: bool,
    pub drift_corrected: bool,
    pub message: String,
}
