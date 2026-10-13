use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IdempotencyKey {
    pub workspace_id: String,
    pub operation_type: String,
    pub client_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    pub request_hash: String,
    pub operation_id: String,
    pub created_at: DateTime<Utc>,
}
