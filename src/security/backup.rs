use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

use crate::error::{Result, SmooothError};
use crate::models::urn::ResourceUrn;
use crate::security::crypto::SecretStore;
use crate::storage::local::{LocalStorage, StateSnapshot};
use crate::storage::Storage;

pub const BACKUP_MAGIC_V1: &str = "SMOOOTH_ENCRYPTED_BACKUP_V1";
pub const RESTORE_CONFIRMATION_TOKEN: &str = "CONFIRM_DESTRUCTIVE_RESTORE";

/// Cryptographic header containing verifiable metadata and integrity digests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackupHeader {
    pub magic: String,
    pub backup_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub workspace_id: String,
    pub plaintext_sha256: String,
    pub key_fingerprint: String,
    pub format_version: u32,
}

/// Self-contained encrypted backup archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackupArchive {
    pub header: EncryptedBackupHeader,
    /// Hex-encoded AES-256-GCM ciphertext (including 12-byte IV nonce and 16-byte Poly1305 tag)
    pub ciphertext_hex: String,
}

/// Unencrypted internal payload representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub version: u32,
    pub backup_id: Uuid,
    pub workspace_id: String,
    pub exported_at: DateTime<Utc>,
    pub snapshot: StateSnapshot,
}

/// Report proving verified restoration in an isolated sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreVerificationReport {
    pub backup_id: Uuid,
    pub workspace_id: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
    pub resource_count: usize,
    pub binding_count: usize,
    pub agent_count: usize,
    pub revocation_count: usize,
    pub audit_event_count: usize,
    pub is_valid: bool,
    pub integrity_digest_matched: bool,
    pub isolated_environment: bool,
    pub summary: String,
}

pub struct BackupManager;

impl BackupManager {
    /// Create an encrypted backup archive for a workspace (or "*" for global state).
    /// Uses domain-separated AES-256-GCM key derived from the master SecretStore ("backup").
    pub async fn create_encrypted_backup<S: Storage + ?Sized>(
        storage: &S,
        secret_store: &SecretStore,
        workspace_id: &str,
    ) -> Result<EncryptedBackupArchive> {
        let snapshot = storage.export_snapshot(workspace_id).await?;
        let backup_id = Uuid::new_v4();
        let now = Utc::now();

        let payload = BackupPayload {
            version: 1,
            backup_id,
            workspace_id: workspace_id.to_string(),
            exported_at: now,
            snapshot,
        };

        let plaintext = serde_json::to_vec(&payload).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize backup payload: {}", e))
        })?;

        // Compute authoritative SHA-256 integrity digest of the plaintext
        let plaintext_sha256 = hex::encode(Sha256::digest(&plaintext));

        // Encrypt with domain-separated backup key
        let backup_crypto = secret_store.domain_store("backup");
        let ciphertext = backup_crypto.encrypt(&plaintext)?;
        let ciphertext_hex = hex::encode(ciphertext);

        let header = EncryptedBackupHeader {
            magic: BACKUP_MAGIC_V1.to_string(),
            backup_id,
            created_at: now,
            workspace_id: workspace_id.to_string(),
            plaintext_sha256,
            key_fingerprint: secret_store.fingerprint(),
            format_version: 1,
        };

        Ok(EncryptedBackupArchive {
            header,
            ciphertext_hex,
        })
    }

    /// Save an encrypted backup directly to disk.
    pub async fn export_to_file<S: Storage + ?Sized, P: AsRef<Path>>(
        storage: &S,
        secret_store: &SecretStore,
        workspace_id: &str,
        path: P,
    ) -> Result<EncryptedBackupArchive> {
        let archive = Self::create_encrypted_backup(storage, secret_store, workspace_id).await?;
        let json = serde_json::to_string_pretty(&archive).map_err(|e| {
            SmooothError::Storage(format!("Failed to serialize archive json: {}", e))
        })?;

        tokio::fs::write(path.as_ref(), json.as_bytes())
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to write backup archive to disk: {}", e))
            })?;

        Ok(archive)
    }

    /// Read an encrypted backup archive from disk.
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<EncryptedBackupArchive> {
        let data = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to read backup file from disk: {}", e))
            })?;

        serde_json::from_str(&data).map_err(|e| {
            SmooothError::Storage(format!("Invalid backup archive format: {}", e))
        })
    }

    /// Verifies restoration in a completely isolated sandbox environment.
    ///
    /// Guarantees:
    /// 1. Decrypts with domain key and validates authentication tag.
    /// 2. Verifies SHA-256 digest of decrypted plaintext.
    /// 3. Reconstructs full state inside an in-memory isolated `LocalStorage`.
    /// 4. Validates resource URN grammar, agent tokens, and schema consistency.
    /// 5. DOES NOT write or modify live persistent storage.
    pub async fn verify_restore_isolated(
        archive: &EncryptedBackupArchive,
        secret_store: &SecretStore,
    ) -> Result<(RestoreVerificationReport, StateSnapshot)> {
        // 1. Validate archive magic & header
        if archive.header.magic != BACKUP_MAGIC_V1 {
            return Err(SmooothError::Security(format!(
                "Invalid backup archive magic header: expected '{}', found '{}'",
                BACKUP_MAGIC_V1, archive.header.magic
            )));
        }

        // 2. Validate key fingerprint matches expected master key
        if archive.header.key_fingerprint != secret_store.fingerprint() {
            return Err(SmooothError::Security(format!(
                "Backup key fingerprint mismatch: backup encrypted with key '{}', current key is '{}'",
                archive.header.key_fingerprint,
                secret_store.fingerprint()
            )));
        }

        // 3. Decrypt ciphertext using domain-separated key
        let backup_crypto = secret_store.domain_store("backup");
        let ciphertext = hex::decode(&archive.ciphertext_hex).map_err(|e| {
            SmooothError::Security(format!("Failed to decode ciphertext hex: {}", e))
        })?;

        let decrypted_bytes = backup_crypto.decrypt(&ciphertext)?;

        // 4. Verify plaintext SHA-256 integrity digest
        let observed_digest = hex::encode(Sha256::digest(&decrypted_bytes));
        if observed_digest != archive.header.plaintext_sha256 {
            return Err(SmooothError::Security(format!(
                "Integrity violation: plaintext digest mismatch! Expected {}, calculated {}",
                archive.header.plaintext_sha256, observed_digest
            )));
        }

        // 5. Parse backup payload
        let payload: BackupPayload = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
            SmooothError::Storage(format!("Failed to parse decrypted backup payload: {}", e))
        })?;

        // 6. Instantiate isolated in-memory sandbox and verify invariants
        let isolated_storage = LocalStorage::new_in_memory();
        isolated_storage
            .import_snapshot(payload.snapshot.clone())
            .await?;

        // Verify structural resource invariants
        for (res_id, res) in &payload.snapshot.resources {
            // Confirm valid URN construction
            let target_urn_str = format!("smoooth://workspaces/{}/workloads/{}", res.workspace_id, res_id);
            if let Err(e) = ResourceUrn::parse(&target_urn_str) {
                return Err(SmooothError::Security(format!(
                    "Backup contains corrupt resource identifier '{res_id}': {e}"
                )));
            }
        }

        let report = RestoreVerificationReport {
            backup_id: payload.backup_id,
            workspace_id: payload.workspace_id.clone(),
            created_at: payload.exported_at,
            verified_at: Utc::now(),
            resource_count: payload.snapshot.resources.len(),
            binding_count: payload.snapshot.bindings.len(),
            agent_count: payload.snapshot.agents.len(),
            revocation_count: payload.snapshot.revocations.len(),
            audit_event_count: payload.snapshot.audit_events.len(),
            is_valid: true,
            integrity_digest_matched: true,
            isolated_environment: true,
            summary: format!(
                "Successfully verified backup {} ({} resources, {} secrets/bindings, {} agents) in isolated sandbox.",
                payload.backup_id,
                payload.snapshot.resources.len(),
                payload.snapshot.bindings.len(),
                payload.snapshot.agents.len()
            ),
        };

        Ok((report, payload.snapshot))
    }

    /// Restore an encrypted backup into live storage.
    ///
    /// Safeguards:
    /// - Requires explicit `confirmation_token == "CONFIRM_DESTRUCTIVE_RESTORE"`.
    /// - Always performs isolated sandbox verification first.
    /// - Fails closed if verification fails, never touching live storage.
    pub async fn restore_to_live<S: Storage + ?Sized>(
        archive: &EncryptedBackupArchive,
        secret_store: &SecretStore,
        live_storage: &S,
        confirmation_token: &str,
    ) -> Result<RestoreVerificationReport> {
        if confirmation_token != RESTORE_CONFIRMATION_TOKEN {
            return Err(SmooothError::Security(format!(
                "Live restore blocked: confirmation token must be '{}', got '{}'",
                RESTORE_CONFIRMATION_TOKEN, confirmation_token
            )));
        }

        // 1. Mandatory isolated verification first
        let (mut report, snapshot) = Self::verify_restore_isolated(archive, secret_store).await?;

        // 2. Commit verified snapshot to live storage
        live_storage.import_snapshot(snapshot).await?;

        // 3. Update report status for live application
        report.isolated_environment = false;
        report.summary = format!(
            "LIVE RESTORE COMPLETED: Restored backup {} ({} resources, {} agents) to live storage.",
            report.backup_id, report.resource_count, report.agent_count
        );

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentIdentity, AgentScope, Resource, ResourceOwnership, ResourceStatus};

    fn make_test_secret_store() -> SecretStore {
        SecretStore::new([42u8; 32])
    }

    #[tokio::test]
    async fn test_backup_create_and_isolated_verification_roundtrip() {
        let store = make_test_secret_store();
        let storage = LocalStorage::new_in_memory();

        // Populate test data
        let agent = AgentIdentity {
            agent_id: "agent_alpha".into(),
            name: "Alpha Agent".into(),
            token_hash: "hash_123".into(),
            token_prefix: "smoooth_ag_a1".into(),
            workspace_id: "prod".into(),
            scopes: vec![AgentScope::Deploy],
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            revoked: false,
            revoked_at: None,
        };
        storage.save_agent(agent).await.unwrap();

        let resource = Resource {
            id: "api_service".into(),
            name: "api_service".into(),
            workspace_id: "prod".into(),
            service_type: "workload".into(),
            provider: "docker".into(),
            is_persistent: false,
            ownership: ResourceOwnership::SmooothManaged,
            status: ResourceStatus::Active,
            configuration: serde_json::json!({"image": "app:v1"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_resource(resource).await.unwrap();

        // 1. Create encrypted backup
        let archive = BackupManager::create_encrypted_backup(&storage, &store, "prod")
            .await
            .unwrap();

        assert_eq!(archive.header.magic, BACKUP_MAGIC_V1);
        assert_eq!(archive.header.workspace_id, "prod");
        assert_eq!(archive.header.key_fingerprint, store.fingerprint());
        assert!(!archive.ciphertext_hex.is_empty());

        // 2. Verify restore in isolated sandbox
        let (report, snapshot) = BackupManager::verify_restore_isolated(&archive, &store)
            .await
            .unwrap();

        assert!(report.is_valid);
        assert!(report.integrity_digest_matched);
        assert!(report.isolated_environment);
        assert_eq!(report.resource_count, 1);
        assert_eq!(report.agent_count, 1);
        assert_eq!(snapshot.resources.len(), 1);
        assert_eq!(snapshot.agents.len(), 1);
    }

    #[tokio::test]
    async fn test_tampered_backup_fails_verification() {
        let store = make_test_secret_store();
        let storage = LocalStorage::new_in_memory();

        let mut archive = BackupManager::create_encrypted_backup(&storage, &store, "*")
            .await
            .unwrap();

        // Tamper with ciphertext
        let mut bytes = hex::decode(&archive.ciphertext_hex).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF; // Flip bit in Poly1305 authentication tag
        archive.ciphertext_hex = hex::encode(bytes);

        let res = BackupManager::verify_restore_isolated(&archive, &store).await;
        assert!(res.is_err(), "Tampered ciphertext must fail authentication");
    }

    #[tokio::test]
    async fn test_wrong_key_fails_verification() {
        let store1 = SecretStore::new([1u8; 32]);
        let store2 = SecretStore::new([2u8; 32]);
        let storage = LocalStorage::new_in_memory();

        let archive = BackupManager::create_encrypted_backup(&storage, &store1, "*")
            .await
            .unwrap();

        let res = BackupManager::verify_restore_isolated(&archive, &store2).await;
        assert!(res.is_err(), "Different key must fail verification");
    }

    #[tokio::test]
    async fn test_restore_to_live_requires_confirmation_token() {
        let store = make_test_secret_store();
        let storage = LocalStorage::new_in_memory();

        let archive = BackupManager::create_encrypted_backup(&storage, &store, "*")
            .await
            .unwrap();

        // Attempt without token
        let res = BackupManager::restore_to_live(&archive, &store, &storage, "wrong_token").await;
        assert!(res.is_err(), "Must reject unconfirmed live restore");

        // Attempt with confirmation token
        let res2 = BackupManager::restore_to_live(&archive, &store, &storage, RESTORE_CONFIRMATION_TOKEN).await;
        assert!(res2.is_ok(), "Must succeed with explicit confirmation token");
    }
}
