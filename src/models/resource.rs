use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    Provisioning,
    Active,
    Degraded,
    Stopping,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub service_type: String,
    pub provider: String,
    pub is_persistent: bool,
    #[serde(default)]
    pub ownership: crate::models::ResourceOwnership,
    pub status: ResourceStatus,
    pub configuration: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
