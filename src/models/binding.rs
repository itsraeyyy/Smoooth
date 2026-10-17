use crate::models::connection::{ConnectionDetails, Endpoint, ResourceOwnership};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RotationStage {
    Active,
    Overlapping,
    Verifying,
    Retired,
    RolledBack,
}

fn default_binding_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBinding {
    pub id: String,
    pub source_resource_id: String,
    pub target_resource_id: String,
    pub binding_type: String, // e.g. "postgres_connection", "redis_connection", "generic_http", "custom"
    pub rotation_stage: RotationStage,
    #[serde(default)]
    pub ownership: ResourceOwnership,
    #[serde(default = "default_binding_version")]
    pub binding_version: u32,
    #[serde(default)]
    pub resolved_endpoint: Option<Endpoint>,
    /// Safe connection details (password always redacted or omitted from serialization)
    #[serde(default)]
    pub safe_connection_details: Option<ConnectionDetails>,
    /// Encrypted credential payload (stored as ciphertext, never plain secrets)
    #[serde(default)]
    pub encrypted_credentials: Vec<u8>,
    /// Safe metadata exposed to agents and audit logs
    pub exposed_env_keys: Vec<String>,
    #[serde(default)]
    pub env_mappings: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServiceBinding {
    /// Mask sensitive strings like passwords and secrets in URIs
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
            ownership: ResourceOwnership::SmooothManaged,
            binding_version: 1,
            resolved_endpoint: None,
            safe_connection_details: None,
            encrypted_credentials,
            exposed_env_keys,
            env_mappings: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Construct dynamic service binding with typed connection details and environment variable mapping
    #[allow(clippy::too_many_arguments)]
    pub fn new_dynamic(
        id: String,
        source_resource_id: String,
        target_resource_id: String,
        binding_type: String,
        ownership: ResourceOwnership,
        endpoint: Option<Endpoint>,
        details: ConnectionDetails,
        secret_store: &crate::security::SecretStore,
        env_mappings: HashMap<String, String>,
    ) -> crate::error::Result<Self> {
        // Serialize full connection details (including plaintext password) for encrypted storage
        let full_bytes = serde_json::to_vec(&details)
            .map_err(|e| crate::error::SmooothError::Validation(e.to_string()))?;
        let encrypted_credentials = secret_store.encrypt(&full_bytes)?;

        // Strip password for safe struct presentation
        let mut safe_details = details.clone();
        safe_details.password = None;

        let exposed_env_keys = if !env_mappings.is_empty() {
            let mut keys: Vec<String> = env_mappings.keys().cloned().collect();
            keys.sort();
            keys
        } else {
            vec![format!("{}_URL", details.protocol.to_uppercase())]
        };

        Ok(Self {
            id,
            source_resource_id,
            target_resource_id,
            binding_type,
            rotation_stage: RotationStage::Active,
            ownership,
            binding_version: 1,
            resolved_endpoint: endpoint,
            safe_connection_details: Some(safe_details),
            encrypted_credentials,
            exposed_env_keys,
            env_mappings,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Decrypt stored credentials in memory for injection without logging
    pub fn decrypt_credentials(
        &self,
        secret_store: &crate::security::SecretStore,
    ) -> crate::error::Result<Vec<u8>> {
        secret_store.decrypt(&self.encrypted_credentials)
    }

    /// Decrypt full connection details with plaintext password in memory
    pub fn decrypt_connection_details(
        &self,
        secret_store: &crate::security::SecretStore,
    ) -> crate::error::Result<ConnectionDetails> {
        let decrypted_bytes = self.decrypt_credentials(secret_store)?;
        // If it deserializes as ConnectionDetails:
        if let Ok(details) = serde_json::from_slice::<ConnectionDetails>(&decrypted_bytes) {
            return Ok(details);
        }
        // Fallback for legacy raw connection string
        if let Ok(raw_str) = String::from_utf8(decrypted_bytes) {
            if let Ok(url) = url::Url::parse(&raw_str) {
                return Ok(ConnectionDetails {
                    protocol: url.scheme().to_string(),
                    host: url.host_str().unwrap_or("localhost").to_string(),
                    port: url.port().unwrap_or(5432),
                    path_or_database: Some(url.path().trim_start_matches('/').to_string()),
                    username: if url.username().is_empty() {
                        None
                    } else {
                        Some(url.username().to_string())
                    },
                    password: url.password().map(|p| p.to_string()),
                    extra_params: url.query_pairs().into_owned().collect(),
                });
            }
        }
        Err(crate::error::SmooothError::Validation(
            "Failed to deserialize decrypted connection details".to_string(),
        ))
    }
}
