use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub workspace_id: String,
    pub operation_type: String,
    pub plan_id: String,
    pub plan_hash: String,
    pub request_hash: String,
    pub idempotency_client_key: String,
    pub status: OperationStatus,
    pub progress_percent: u8,
    /// Monotonically increasing fencing generation token to prevent zombie worker writes
    pub lease_epoch: u64,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    /// State tracking for reconciliation
    pub intended_state: serde_json::Value,
    pub observed_state: serde_json::Value,
    pub logs: Vec<OperationLogEntry>,
    pub evidence: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
