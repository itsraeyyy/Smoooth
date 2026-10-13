use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RotationStage {
    Active,
    Overlapping,
    Verifying,
    Retired,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBinding {
    pub id: String,
    pub source_resource_id: String,
    pub target_resource_id: String,
    pub binding_type: String, // e.g. "postgres_connection"
    pub rotation_stage: RotationStage,
    /// Encrypted credential payload (never serialized into plain logs)
    #[serde(skip_serializing)]
    pub encrypted_credentials: Vec<u8>,
    /// Safe metadata exposed to agents and audit logs
    pub exposed_env_keys: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServiceBinding {
    /// Mask sensitive strings like passwords and secrets
    pub fn mask_connection_string(raw: &str) -> String {
        if let Some(at_idx) = raw.find('@') {
            if let Some(proto_end) = raw.find("://") {
                let proto = &raw[..proto_end + 3];
                let host_and_beyond = &raw[at_idx..];
                return format!("{}[REDACTED]{}", proto, host_and_beyond);
            }
        }
        "[REDACTED]".to_string()
    }

    /// Construct a service binding with credentials encrypted using AES-256-GCM
    pub fn new_with_secret(
        id: String,
        source_resource_id: String,
        target_resource_id: String,
        binding_type: String,
        raw_credentials: &[u8],
        secret_store: &crate::security::SecretStore,
        exposed_env_keys: Vec<String>,
    ) -> crate::error::Result<Self> {
        let encrypted_credentials = secret_store.encrypt(raw_credentials)?;
        Ok(Self {
            id,
            source_resource_id,
            target_resource_id,
            binding_type,
            rotation_stage: RotationStage::Active,
            encrypted_credentials,
            exposed_env_keys,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Decrypt stored credentials in memory for injection without logging
    pub fn decrypt_credentials(&self, secret_store: &crate::security::SecretStore) -> crate::error::Result<Vec<u8>> {
        secret_store.decrypt(&self.encrypted_credentials)
    }
}
