use crate::error::{Result, SmooothError};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

/// Key backup payload containing non-reversible fingerprint, raw hex, and metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KeyBackup {
    pub fingerprint: String,
    pub key_hex: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub format_version: u32,
}

/// Secure Secret Store providing AES-256-GCM authenticated encryption
/// for infrastructure credentials, connection tokens, and workload secrets.
#[derive(Clone)]
pub struct SecretStore {
    master_key: [u8; KEY_LEN],
}

impl SecretStore {
    /// Create a new SecretStore with an explicit 256-bit master key
    pub fn new(master_key: [u8; KEY_LEN]) -> Self {
        Self { master_key }
    }

    /// Default store for local development using a deterministic derived key
    pub fn default_local() -> Self {
        let mut key = [0u8; KEY_LEN];
        let seed = b"smoooth_local_platform_key_32b_!";
        key.copy_from_slice(seed);
        Self { master_key: key }
    }

    /// Check if the store is using the hardcoded insecure development seed
    pub fn is_insecure_default(&self) -> bool {
        let seed = b"smoooth_local_platform_key_32b_!";
        self.master_key == *seed
    }

    /// Access raw key bytes for controlled migration and rotation
    pub fn raw_key(&self) -> &[u8; KEY_LEN] {
        &self.master_key
    }

    /// Compute a non-reversible cryptographic SHA-256 fingerprint of the key
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&self.master_key);
        format!("SMOOOTH-KEY-SHA256:{}", &hex::encode(hash)[..16])
    }

    /// Verify whether this store matches an expected key fingerprint
    pub fn matches_fingerprint(&self, expected: &str) -> bool {
        self.fingerprint() == expected
    }

    /// Export key backup envelope with verification fingerprint
    pub fn export_backup(&self) -> KeyBackup {
        KeyBackup {
            fingerprint: self.fingerprint(),
            key_hex: hex::encode(self.master_key),
            created_at: chrono::Utc::now(),
            format_version: 1,
        }
    }

    /// Derive a domain-separated SecretStore using SHA-256 Key Derivation Function (KDF).
    /// This guarantees cryptographic key separation between distinct subsystems:
    /// - "service_binding"
    /// - "workload_secrets"
    /// - "backup"
    pub fn domain_store(&self, domain: &str) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"smoooth_kdf_v1:");
        hasher.update(domain.as_bytes());
        hasher.update(&self.master_key);
        let derived_key: [u8; KEY_LEN] = hasher.finalize().into();
        Self {
            master_key: derived_key,
        }
    }

    /// Parse a 32-byte key from either a 64-char hex string or exact 32 raw bytes
    pub fn parse_key(raw: &str) -> Option<[u8; KEY_LEN]> {
        let trimmed = raw.trim();
        if trimmed.len() == 64 {
            if let Ok(bytes) = hex::decode(trimmed) {
                if bytes.len() == KEY_LEN {
                    let mut key = [0u8; KEY_LEN];
                    key.copy_from_slice(&bytes);
                    return Some(key);
                }
            }
        }
        if trimmed.as_bytes().len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(trimmed.as_bytes());
            return Some(key);
        }
        None
    }

    /// Inspect whether a state storage JSON file exists and contains non-empty encrypted payloads
    pub fn state_contains_encrypted_data(path: &str) -> bool {
        if let Ok(data) = std::fs::read_to_string(path) {
            if data.contains("\"encrypted_credentials\"") || data.contains("\"encrypted_secrets\"") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(bindings) = val.get("bindings").and_then(|b| b.as_object()) {
                        for (_k, b) in bindings {
                            if let Some(enc) = b.get("encrypted_credentials").and_then(|e| e.as_array()) {
                                if !enc.is_empty() {
                                    return true;
                                }
                            }
                        }
                    }
                    if let Some(resources) = val.get("resources").and_then(|r| r.as_object()) {
                        for (_k, r) in resources {
                            if let Some(cfg) = r.get("configuration") {
                                if let Some(enc) = cfg.get("encrypted_secrets").and_then(|s| s.as_str()) {
                                    if !enc.is_empty() {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Resolve or automatically generate a persistent cryptographically secure master key.
    /// Priority:
    /// 1. SMOOOTH_MASTER_KEY environment variable (hex or 32-char string)
    /// 2. SMOOOTH_MASTER_KEY_FILE environment variable (path to key file)
    /// 3. /etc/smoooth/master.key (production Linux VPS target)
    /// 4. smoooth_master.key in working directory
    /// 5. Insecure dev key override (SMOOOTH_DEV_INSECURE_KEY=1)
    /// 6. FAIL-CLOSED CHECK: If existing state has encrypted data, NEVER silently generate a replacement key!
    /// 7. Automatically generates new 256-bit key and persists to smoooth_master.key with 0600 permissions
    pub fn resolve_or_create() -> Result<Self> {
        // 1. Env variable SMOOOTH_MASTER_KEY
        if let Ok(key_str) = std::env::var("SMOOOTH_MASTER_KEY") {
            if let Some(key) = Self::parse_key(&key_str) {
                return Ok(Self::new(key));
            } else {
                return Err(SmooothError::CryptoError(
                    "Invalid SMOOOTH_MASTER_KEY: must be 64-character hex or 32 raw bytes".to_string(),
                ));
            }
        }

        // 2. Env variable SMOOOTH_MASTER_KEY_FILE
        if let Ok(file_path) = std::env::var("SMOOOTH_MASTER_KEY_FILE") {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Some(key) = Self::parse_key(&content) {
                    return Ok(Self::new(key));
                }
            }
            return Err(SmooothError::CryptoError(format!(
                "Failed to read valid 256-bit master key from SMOOOTH_MASTER_KEY_FILE at '{}'",
                file_path
            )));
        }

        // 3. System Linux VPS file /etc/smoooth/master.key
        if let Ok(content) = std::fs::read_to_string("/etc/smoooth/master.key") {
            if let Some(key) = Self::parse_key(&content) {
                return Ok(Self::new(key));
            }
        }

        // 4. Local workspace file smoooth_master.key
        if let Ok(content) = std::fs::read_to_string("smoooth_master.key") {
            if let Some(key) = Self::parse_key(&content) {
                return Ok(Self::new(key));
            }
        }

        // 5. Insecure dev key explicit override for offline unit testing
        if std::env::var("SMOOOTH_DEV_INSECURE_KEY").unwrap_or_default() == "1" {
            return Ok(Self::default_local());
        }

        // 6. Fail-closed protection: if existing encrypted data is detected, NEVER auto-generate a new key!
        let storage_path = std::env::var("SMOOOTH_STORAGE_PATH").unwrap_or_else(|_| {
            if std::path::Path::new("/var/lib/smoooth/smoooth_state.json").exists() {
                "/var/lib/smoooth/smoooth_state.json".to_string()
            } else {
                "smoooth_state.json".to_string()
            }
        });

        if Self::state_contains_encrypted_data(&storage_path) {
            return Err(SmooothError::CryptoError(format!(
                "CRITICAL SECURITY REJECTION: Existing encrypted records were detected in state storage ('{}'), \
                but no valid master key was found via SMOOOTH_MASTER_KEY or keyfiles. \
                Smoooth is failing closed to prevent permanent customer credential loss and state corruption. \
                Please provide SMOOOTH_MASTER_KEY or restore your master key file.",
                storage_path
            )));
        }

        // 7. Generate a cryptographically secure 256-bit key and persist to disk
        let mut key = [0u8; KEY_LEN];
        getrandom::getrandom(&mut key).map_err(|e| {
            SmooothError::CryptoError(format!("Failed to generate secure random master key: {}", e))
        })?;

        let hex_key = hex::encode(key);
        let _ = std::fs::write("smoooth_master.key", &hex_key);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                "smoooth_master.key",
                std::fs::Permissions::from_mode(0o600),
            );
        }

        tracing::info!("Generated secure 256-bit master encryption key and saved to smoooth_master.key");
        Ok(Self::new(key))
    }

    /// Encrypt plaintext bytes using AES-256-GCM with a randomized 96-bit nonce.
    /// Serialized payload format: [12-byte Nonce] + [Ciphertext + 16-byte Poly1305 Tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key).map_err(|e| {
            SmooothError::CryptoError(format!("Failed to initialize cipher: {}", e))
        })?;

        // Generate cryptographically secure random nonce
        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| SmooothError::CryptoError(format!("RNG failure for nonce: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SmooothError::CryptoError(format!("Encryption failed: {}", e)))?;

        let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);

        Ok(payload)
    }

    /// Decrypt an AES-256-GCM payload.
    /// Validates the 128-bit authentication tag before returning any plaintext.
    pub fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < NONCE_LEN + 16 {
            return Err(SmooothError::CryptoError(
                "Payload too short to contain valid nonce and authentication tag".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.master_key).map_err(|e| {
            SmooothError::CryptoError(format!("Failed to initialize cipher: {}", e))
        })?;

        let nonce = Nonce::from_slice(&payload[..NONCE_LEN]);
        let ciphertext = &payload[NONCE_LEN..];

        cipher.decrypt(nonce, ciphertext).map_err(|e| {
            SmooothError::CryptoError(format!("Decryption or authentication tag failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_encryption_and_decryption_roundtrip() {
        let store = SecretStore::default_local();
        let secret = b"super_secret_database_password_999!";

        let encrypted = store.encrypt(secret).unwrap();
        assert_ne!(encrypted, secret, "Ciphertext must not match plaintext");
        assert!(encrypted.len() >= secret.len() + NONCE_LEN + 16);

        let decrypted = store.decrypt(&encrypted).unwrap();
        assert_eq!(
            decrypted, secret,
            "Decrypted bytes must match original plaintext"
        );
    }

    #[test]
    fn test_tampered_ciphertext_fails_authentication() {
        let store = SecretStore::default_local();
        let secret = b"confidential_token";

        let mut encrypted = store.encrypt(secret).unwrap();
        // Tamper with one byte in the ciphertext payload
        let last_idx = encrypted.len() - 1;
        encrypted[last_idx] ^= 0xFF;

        let result = store.decrypt(&encrypted);
        assert!(
            result.is_err(),
            "Tampered ciphertext must fail authentication"
        );
    }

    #[test]
    fn test_parse_key_hex_and_raw() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let parsed = SecretStore::parse_key(hex_key);
        assert!(parsed.is_some());
        assert_eq!(hex::encode(parsed.unwrap()), hex_key);

        let raw_32 = "12345678901234567890123456789012";
        let parsed_raw = SecretStore::parse_key(raw_32);
        assert!(parsed_raw.is_some());
        assert_eq!(parsed_raw.unwrap(), raw_32.as_bytes());

        let invalid = "short";
        assert!(SecretStore::parse_key(invalid).is_none());
    }

    #[test]
    fn test_insecure_default_detection() {
        let default_store = SecretStore::default_local();
        assert!(default_store.is_insecure_default());

        let custom_key = [1u8; KEY_LEN];
        let custom_store = SecretStore::new(custom_key);
        assert!(!custom_store.is_insecure_default());
    }

    #[test]
    fn test_domain_separation_keys_cannot_cross_decrypt() {
        let master_store = SecretStore::new([42u8; KEY_LEN]);
        let db_store = master_store.domain_store("service_binding");
        let env_store = master_store.domain_store("workload_secrets");

        assert_ne!(db_store.raw_key(), master_store.raw_key());
        assert_ne!(env_store.raw_key(), master_store.raw_key());
        assert_ne!(db_store.raw_key(), env_store.raw_key());

        let plaintext = b"db_pass_12345";
        let db_encrypted = db_store.encrypt(plaintext).unwrap();

        // Cross-domain decryption must fail authentication
        let env_attempt = env_store.decrypt(&db_encrypted);
        assert!(env_attempt.is_err(), "Domain-separated key must not decrypt another domain's ciphertext");
    }

    #[test]
    fn test_key_fingerprint_and_backup_roundtrip() {
        let store = SecretStore::new([7u8; KEY_LEN]);
        let fp = store.fingerprint();
        assert!(fp.starts_with("SMOOOTH-KEY-SHA256:"));
        assert!(store.matches_fingerprint(&fp));

        let backup = store.export_backup();
        assert_eq!(backup.fingerprint, fp);
        assert_eq!(backup.format_version, 1);
        let restored_key = SecretStore::parse_key(&backup.key_hex).unwrap();
        let restored_store = SecretStore::new(restored_key);
        assert!(restored_store.matches_fingerprint(&fp));
    }

    #[test]
    fn test_state_contains_encrypted_data_detection() {
        let temp_dir = std::env::temp_dir();
        let test_state_path = temp_dir.join(format!("smoooth_test_enc_state_{}.json", uuid::Uuid::new_v4().simple()));
        let path_str = test_state_path.to_str().unwrap();

        // 1. Plain state without encrypted records
        std::fs::write(path_str, r#"{"resources":{},"bindings":{}}"#).unwrap();
        assert!(!SecretStore::state_contains_encrypted_data(path_str));

        // 2. State with non-empty encrypted_credentials
        std::fs::write(path_str, r#"{"resources":{},"bindings":{"b1":{"encrypted_credentials":[1,2,3,4]}}}"#).unwrap();
        assert!(SecretStore::state_contains_encrypted_data(path_str));

        let _ = std::fs::remove_file(path_str);
    }
}
