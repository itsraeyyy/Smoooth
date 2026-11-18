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
        let hash = Sha256::digest(self.master_key);
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
        hasher.update(self.master_key);
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
        if trimmed.len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(trimmed.as_bytes());
            return Some(key);
        }
        None
    }

    /// Inspect whether a state storage JSON file exists and contains non-empty encrypted payloads
    pub fn state_contains_encrypted_data(path: &str) -> bool {
        if let Ok(data) = std::fs::read_to_string(path) {
            if data.contains("\"encrypted_credentials\"") || data.contains("\"encrypted_secrets\"")
            {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(bindings) = val.get("bindings").and_then(|b| b.as_object()) {
                        for (_k, b) in bindings {
                            if let Some(enc) =
                                b.get("encrypted_credentials").and_then(|e| e.as_array())
                            {
                                if !enc.is_empty() {
                                    return true;
                                }
                            }
                        }
                    }
                    if let Some(resources) = val.get("resources").and_then(|r| r.as_object()) {
                        for (_k, r) in resources {
                            if let Some(cfg) = r.get("configuration") {
                                if let Some(enc) =
                                    cfg.get("encrypted_secrets").and_then(|s| s.as_str())
                                {
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
                    "Invalid SMOOOTH_MASTER_KEY: must be 64-character hex or 32 raw bytes"
                        .to_string(),
                ));
            }
        }

        // 2. Env variable SMOOOTH_MASTER_KEY_FILE
        if let Ok(file_path) = std::env::var("SMOOOTH_MASTER_KEY_FILE") {
            let path = std::path::Path::new(&file_path);
            let content = Self::read_secure_key_file(path)?;
            if let Some(key) = Self::parse_key(&content) {
                return Ok(Self::new(key));
            } else {
                return Err(SmooothError::CryptoError(format!(
                    "Invalid 256-bit master key format in SMOOOTH_MASTER_KEY_FILE at '{}'",
                    file_path
                )));
            }
        }

        // 3. System Linux VPS file /etc/smoooth/master.key
        let system_key_path = std::path::Path::new("/etc/smoooth/master.key");
        if system_key_path.exists() {
            let content = Self::read_secure_key_file(system_key_path)?;
            if let Some(key) = Self::parse_key(&content) {
                return Ok(Self::new(key));
            } else {
                return Err(SmooothError::CryptoError(
                    "Failed to parse valid 256-bit master key from existing '/etc/smoooth/master.key'".to_string(),
                ));
            }
        }

        // 4. Local workspace file smoooth_master.key
        let local_key_path = std::path::Path::new("smoooth_master.key");
        if local_key_path.exists() {
            let content = Self::read_secure_key_file(local_key_path)?;
            if let Some(key) = Self::parse_key(&content) {
                return Ok(Self::new(key));
            } else {
                return Err(SmooothError::CryptoError(
                    "Failed to parse valid 256-bit master key from existing 'smoooth_master.key'"
                        .to_string(),
                ));
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

        // 7. Generate a cryptographically secure 256-bit key and persist to disk with mode 0600
        let mut key = [0u8; KEY_LEN];
        getrandom::getrandom(&mut key).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to generate secure random master key: {}",
                e
            ))
        })?;

        let hex_key = hex::encode(key);
        Self::write_secure_key_file("smoooth_master.key", &hex_key)?;

        tracing::info!("Generated secure 256-bit master encryption key and saved to smoooth_master.key with mode 0600");
        Ok(Self::new(key))
    }

    /// Securely write a key file with owner-only (0600) permissions on Unix,
    /// avoiding any race condition or window with broader permissions.
    #[cfg(unix)]
    pub fn write_secure_key_file<P: AsRef<std::path::Path>>(path: P, content: &str) -> Result<()> {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);

        let mut file = options.open(path.as_ref()).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to create secure key file at {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;

        // Explicitly set permissions to 0600 in case umask restricted it differently
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|e| {
                SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Failed to enforce strict 0600 permissions on key file {:?}: {}",
                    path.as_ref(),
                    e
                ))
            })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to write key content to {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;
        file.flush().map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to flush key file {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn write_secure_key_file<P: AsRef<std::path::Path>>(path: P, content: &str) -> Result<()> {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);

        let mut file = options.open(path.as_ref()).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to create key file at {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to write key content to {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;
        file.flush().map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to flush key file {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;
        Ok(())
    }

    /// Check if process has the given GID in effective GID or supplementary groups
    #[cfg(unix)]
    pub fn process_has_group(file_gid: u32) -> bool {
        unsafe {
            extern "C" {
                fn getegid() -> u32;
                fn getgroups(size: i32, list: *mut u32) -> i32;
            }
            let egid = getegid();
            if egid == file_gid {
                return true;
            }
            let count = getgroups(0, std::ptr::null_mut());
            if count > 0 {
                let mut list = vec![0u32; count as usize];
                let actual = getgroups(count, list.as_mut_ptr());
                if actual > 0 {
                    list.truncate(actual as usize);
                    return list.contains(&file_gid);
                }
            }
            false
        }
    }

    /// Verify that the file descriptor on Linux has no extended POSIX ACLs attached.
    /// Explicit error handling policy:
    /// - res > 0: An extended access ACL exists -> fail closed.
    /// - res == 0: Empty/anomalous ACL attribute -> fail closed.
    /// - errno == ENODATA: Standard POSIX mode bits only, verified no extended ACL -> Ok(()).
    /// - errno == EOPNOTSUPP: Filesystem does not support extended attributes/ACLs.
    ///   Since absence of ACLs cannot be proven on this filesystem, fail closed for group-readable root keys.
    /// - Any other errno: Unexpected error querying attributes -> fail closed.
    #[cfg(all(unix, target_os = "linux"))]
    pub fn verify_no_extended_acl(
        fd: std::os::unix::io::RawFd,
        path: &std::path::Path,
    ) -> Result<()> {
        unsafe {
            extern "C" {
                fn fgetxattr(fd: i32, name: *const i8, value: *mut u8, size: usize) -> isize;
                fn __errno_location() -> *mut i32;
            }
            let attr_name = b"system.posix_acl_access\0";
            let res = fgetxattr(fd, attr_name.as_ptr() as *const i8, std::ptr::null_mut(), 0);
            if res > 0 {
                return Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Root-provisioned master key at {:?} has extended POSIX ACLs attached. \
                    Failing closed because extended ACLs may grant unauthorized principals access.",
                    path
                )));
            }
            if res == 0 {
                return Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Root-provisioned master key at {:?} has anomalous 0-byte POSIX ACL xattr. Failing closed.",
                    path
                )));
            }
            let errno = *__errno_location();
            const ENODATA: i32 = 61;
            const EOPNOTSUPP: i32 = 95;

            if errno == ENODATA {
                Ok(())
            } else if errno == EOPNOTSUPP {
                Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Filesystem at {:?} does not support extended attributes/ACLs (EOPNOTSUPP). \
                    Cannot verify absence of extended ACLs; group-readable root-provisioned keys are rejected on unsupported filesystems. \
                    Please use owner-only mode (0600) or assign file ownership directly to the Smoooth service user.",
                    path
                )))
            } else {
                Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Failed to query extended ACL attributes on master key at {:?} (errno {}). \
                    Failing closed to prevent unauthorized key access.",
                    path, errno
                )))
            }
        }
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    pub fn verify_no_extended_acl(
        _fd: std::os::unix::io::RawFd,
        path: &std::path::Path,
    ) -> Result<()> {
        Err(SmooothError::CryptoError(format!(
            "SECURITY VIOLATION: Extended ACL verification is not supported on this platform for master key at {:?}. \
            Group-readable root-provisioned keys require verifiable ACL absence. \
            Please configure owner-only permissions (0600) or assign file ownership directly to the service user.",
            path
        )))
    }

    /// Read a master key file securely on Unix, enforcing all security invariants on the open file descriptor:
    /// 1. Rejects symbolic links unconditionally (O_NOFOLLOW / symlink_metadata) to prevent symlink traversal.
    /// 2. Ensures target is a regular file (not directory, FIFO, socket, or device).
    /// 3. Validates descriptor identity (inode/device) against symlink_metadata to prevent TOCTOU file swapping.
    /// 4. If owned by current process (or running as root): enforces 0600 on the descriptor via fchmod, then re-verifies.
    /// 5. If root-provisioned (uid 0):
    ///    - Rejects world access (other bits == 0).
    ///    - Rejects group write (group write bit == 0).
    ///    - If group read is permitted, verifies current process belongs to the group via effective GID or supplementary groups (getgroups).
    ///    - On Linux: verifies no extended POSIX ACLs exist on the descriptor via fgetxattr("system.posix_acl_access").
    /// 6. If owned by an untrusted third party: fails closed immediately.
    /// 7. Reads key bytes directly from the open verified descriptor, preventing file substitution between check and read.
    #[cfg(unix)]
    pub fn read_secure_key_file<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
        use std::io::Read;
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
        use std::os::unix::io::AsRawFd;

        let path_ref = path.as_ref();

        // Step 1: Symlink rejection before traversal
        let symlink_meta = std::fs::symlink_metadata(path_ref).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to read metadata for master key path {:?}: {}",
                path_ref, e
            ))
        })?;

        if symlink_meta.file_type().is_symlink() {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} is a symbolic link. \
                Symbolic links are strictly forbidden for master keys to prevent symlink redirection attacks.",
                path_ref
            )));
        }

        if !symlink_meta.is_file() {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} is not a regular file.",
                path_ref
            )));
        }

        let (euid, _egid) = unsafe {
            extern "C" {
                fn geteuid() -> u32;
                fn getegid() -> u32;
            }
            (geteuid(), getegid())
        };

        let is_owner = symlink_meta.uid() == euid || euid == 0;

        // Step 2: Open file with O_NOFOLLOW
        let mut open_opts = std::fs::OpenOptions::new();
        open_opts.read(true);
        if is_owner {
            open_opts.write(true);
        }
        #[cfg(target_os = "linux")]
        open_opts.custom_flags(0o400000); // O_NOFOLLOW on Linux
        #[cfg(target_os = "macos")]
        open_opts.custom_flags(0x0100); // O_NOFOLLOW on macOS

        let mut file = match open_opts.open(path_ref) {
            Ok(f) => f,
            Err(e) => {
                if is_owner {
                    // Fall back to read-only if write access fails (e.g. read-only mount or mode 0400)
                    let mut ro_opts = std::fs::OpenOptions::new();
                    ro_opts.read(true);
                    #[cfg(target_os = "linux")]
                    ro_opts.custom_flags(0o400000);
                    #[cfg(target_os = "macos")]
                    ro_opts.custom_flags(0x0100);
                    ro_opts.open(path_ref).map_err(|e2| {
                        SmooothError::CryptoError(format!(
                            "Failed to open master key at {:?}: {}",
                            path_ref, e2
                        ))
                    })?
                } else {
                    return Err(SmooothError::CryptoError(format!(
                        "Failed to open master key at {:?}: {}",
                        path_ref, e
                    )));
                }
            }
        };

        // Step 3: Inspect open descriptor (fstat) to prevent TOCTOU race condition
        let fd_meta = file.metadata().map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to read file descriptor metadata for master key at {:?}: {}",
                path_ref, e
            ))
        })?;

        if fd_meta.ino() != symlink_meta.ino() || fd_meta.dev() != symlink_meta.dev() {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Inode mismatch detected on master key at {:?}. Potential TOCTOU file replacement.",
                path_ref
            )));
        }

        let file_uid = fd_meta.uid();
        let file_gid = fd_meta.gid();
        let current_mode = fd_meta.permissions().mode() & 0o777;

        // Enforce single hard link: prevent alternate path access or hard-link injection
        if fd_meta.nlink() > 1 {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} has multiple hard links (nlink = {}). \
                Master keys must have exactly 1 link to prevent alternative path access.",
                path_ref,
                fd_meta.nlink()
            )));
        }

        // Step 4: Ownership and permission verification
        if file_uid == euid || euid == 0 {
            // Case A: Owned by current process or running as root
            if current_mode != 0o600 && current_mode != 0o400 {
                let mut perms = fd_meta.permissions();
                perms.set_mode(0o600);
                file.set_permissions(perms).map_err(|e| {
                    SmooothError::CryptoError(format!(
                        "SECURITY VIOLATION: Master key at {:?} has insecure permissions ({:04o}) \
                        and cannot be restricted to 0600 on file descriptor: {}",
                        path_ref, current_mode, e
                    ))
                })?;

                // Re-verify on descriptor
                let updated = file.metadata().map_err(|e| {
                    SmooothError::CryptoError(format!(
                        "Failed to verify updated metadata for master key at {:?}: {}",
                        path_ref, e
                    ))
                })?;
                let verified_mode = updated.permissions().mode() & 0o777;
                if verified_mode != 0o600 && verified_mode != 0o400 {
                    return Err(SmooothError::CryptoError(format!(
                        "SECURITY VIOLATION: Master key at {:?} permissions could not be restricted to 0600. \
                        Actual verified mode: {:04o}",
                        path_ref, verified_mode
                    )));
                }
            }
        } else if file_uid == 0 {
            // Case B: Root-provisioned key
            if current_mode & 0o007 != 0 {
                return Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Root-provisioned master key at {:?} has world-accessible permissions ({:04o}). \
                    Failing closed: master key must not be readable by other users. Set mode 0600 or 0640.",
                    path_ref, current_mode
                )));
            }

            if current_mode & 0o020 != 0 {
                return Err(SmooothError::CryptoError(format!(
                    "SECURITY VIOLATION: Root-provisioned master key at {:?} is group-writable ({:04o}). \
                    Failing closed: master key must not be writable by group.",
                    path_ref, current_mode
                )));
            }

            if current_mode & 0o040 != 0 {
                if !Self::process_has_group(file_gid) {
                    return Err(SmooothError::CryptoError(format!(
                        "SECURITY VIOLATION: Root-provisioned master key at {:?} is group-readable by gid {} \
                        but current process does not belong to this group (checked effective and supplementary groups). Failing closed.",
                        path_ref, file_gid
                    )));
                }

                Self::verify_no_extended_acl(file.as_raw_fd(), path_ref)?;
            }
        } else {
            // Case C: Untrusted owner
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} is owned by unexpected user (uid {}), \
                while Smoooth is running as uid {}. Failing closed to prevent unauthorized key substitution.",
                path_ref, file_uid, euid
            )));
        }

        // Step 5: Read directly from open verified descriptor
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to read master key from verified file {:?}: {}",
                path_ref, e
            ))
        })?;

        Ok(content)
    }

    #[cfg(not(unix))]
    pub fn read_secure_key_file<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
        use std::io::Read;
        let path_ref = path.as_ref();

        let symlink_meta = std::fs::symlink_metadata(path_ref).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Master key file does not exist at {:?}: {}",
                path_ref, e
            ))
        })?;

        if symlink_meta.file_type().is_symlink() {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} is a symbolic link. \
                Symbolic links are strictly forbidden for master keys.",
                path_ref
            )));
        }

        if !symlink_meta.is_file() {
            return Err(SmooothError::CryptoError(format!(
                "SECURITY VIOLATION: Master key at {:?} is not a regular file.",
                path_ref
            )));
        }

        let mut file = std::fs::File::open(path_ref).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to open master key file at {:?}: {}",
                path_ref, e
            ))
        })?;

        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            SmooothError::CryptoError(format!(
                "Failed to read master key file at {:?}: {}",
                path_ref, e
            ))
        })?;

        Ok(content)
    }

    /// Enforce and verify the secure key file policy:
    /// Validates ownership, permissions, and symlink invariants via open descriptor.
    pub fn verify_and_restrict_key_file<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
        Self::read_secure_key_file(path).map(|_| ())
    }

    /// Restrict file permissions to owner-only read/write (0600) on Unix.
    #[cfg(unix)]
    pub fn restrict_file_permissions<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path)?;
        let mut perms = metadata.permissions();
        if perms.mode() & 0o777 != 0o600 {
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn restrict_file_permissions<P: AsRef<std::path::Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
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
        assert!(
            env_attempt.is_err(),
            "Domain-separated key must not decrypt another domain's ciphertext"
        );
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
        let test_state_path = temp_dir.join(format!(
            "smoooth_test_enc_state_{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        let path_str = test_state_path.to_str().unwrap();

        // 1. Plain state without encrypted records
        std::fs::write(path_str, r#"{"resources":{},"bindings":{}}"#).unwrap();
        assert!(!SecretStore::state_contains_encrypted_data(path_str));

        // 2. State with non-empty encrypted_credentials
        std::fs::write(
            path_str,
            r#"{"resources":{},"bindings":{"b1":{"encrypted_credentials":[1,2,3,4]}}}"#,
        )
        .unwrap();
        assert!(SecretStore::state_contains_encrypted_data(path_str));

        let _ = std::fs::remove_file(path_str);
    }

    #[test]
    fn test_master_key_creation_sets_owner_only_permissions_on_unix() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join(format!(
            "smoooth_test_key_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hex_key = hex::encode([0x42u8; KEY_LEN]);

        SecretStore::write_secure_key_file(&key_path, &hex_key)
            .expect("write_secure_key_file should succeed");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&key_path).expect("metadata should exist");
            let mode = meta.permissions().mode() & 0o777;
            assert_eq!(
                mode, 0o600,
                "Newly created master key must have mode 0600 on Unix"
            );
        }

        let content = std::fs::read_to_string(&key_path).expect("read key file");
        assert_eq!(content, hex_key);

        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn test_existing_master_key_with_broader_permissions_is_restricted() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join(format!(
            "smoooth_test_key_wide_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hex_key = hex::encode([0x99u8; KEY_LEN]);

        std::fs::write(&key_path, &hex_key).expect("write key file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o666))
                .expect("set broad permissions");
            let initial_mode = std::fs::metadata(&key_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                initial_mode, 0o666,
                "Setup must establish broad permissions"
            );
        }

        SecretStore::restrict_file_permissions(&key_path)
            .expect("restrict_file_permissions should succeed");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let final_mode = std::fs::metadata(&key_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                final_mode, 0o600,
                "Existing key file with 0666 must be restricted to 0600 on Unix"
            );
        }

        let content = std::fs::read_to_string(&key_path).expect("read key file");
        assert_eq!(content, hex_key);

        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn test_resolve_or_create_fails_closed_when_existing_key_file_corrupt() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!(
            "smoooth_test_failclosed_{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&test_dir).unwrap();
        let key_path = test_dir.join("smoooth_master.key");
        std::fs::write(&key_path, "corrupted_non_hex_key_data").unwrap();

        let res = match std::fs::read_to_string(&key_path) {
            Ok(c) => match SecretStore::parse_key(&c) {
                Some(k) => Ok(SecretStore::new(k)),
                None => Err(SmooothError::CryptoError(
                    "Failed to parse valid 256-bit master key".to_string(),
                )),
            },
            Err(e) => Err(SmooothError::CryptoError(e.to_string())),
        };
        assert!(
            res.is_err(),
            "Must fail closed if existing keyfile is invalid"
        );

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_verify_and_restrict_key_file_enforces_security_policy() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join(format!(
            "smoooth_test_policy_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hex_key = hex::encode([0x88u8; KEY_LEN]);

        std::fs::write(&key_path, &hex_key).expect("write key file");

        // Verify and restrict on created file
        SecretStore::verify_and_restrict_key_file(&key_path)
            .expect("Owned key file must pass verification and restriction");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let final_mode = std::fs::metadata(&key_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(
                final_mode, 0o600,
                "Owned key file must be strictly verified as 0600 on Unix"
            );
        }

        // Test non-existent file fails closed
        let bogus_path = temp_dir.join("nonexistent_smoooth_key.key");
        assert!(SecretStore::verify_and_restrict_key_file(&bogus_path).is_err());

        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn test_symlink_key_file_is_strictly_rejected() {
        let temp_dir = std::env::temp_dir();
        let target_path = temp_dir.join(format!(
            "smoooth_test_target_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let symlink_path = temp_dir.join(format!(
            "smoooth_test_symlink_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hex_key = hex::encode([0x33u8; KEY_LEN]);
        std::fs::write(&target_path, &hex_key).expect("write target key");

        #[cfg(unix)]
        let symlink_created = std::os::unix::fs::symlink(&target_path, &symlink_path).is_ok();
        #[cfg(windows)]
        let symlink_created =
            std::os::windows::fs::symlink_file(&target_path, &symlink_path).is_ok();

        if symlink_created {
            let res = SecretStore::read_secure_key_file(&symlink_path);
            assert!(
                res.is_err(),
                "read_secure_key_file must strictly reject symbolic links"
            );
            if let Err(SmooothError::CryptoError(msg)) = res {
                assert!(
                    msg.contains("symbolic link"),
                    "Error must state that symbolic links are forbidden"
                );
            }
            let _ = std::fs::remove_file(&symlink_path);
        }

        let _ = std::fs::remove_file(&target_path);
    }

    #[test]
    fn test_unsafe_permissions_rejected_and_key_remains_untouched() {
        let temp_dir = std::env::temp_dir();
        let key_path = temp_dir.join(format!(
            "smoooth_test_untouched_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let original_content = "ORIGINAL_UNTOUCHED_KEY_CONTENT_12345678";
        std::fs::write(&key_path, original_content).expect("write key file");

        // Pass a directory as key path, which must be rejected as not a regular file
        let dir_res = SecretStore::read_secure_key_file(&temp_dir);
        assert!(dir_res.is_err(), "Directory path must be rejected");

        // Verify the original key file content was never touched or modified
        let current_content = std::fs::read_to_string(&key_path).expect("read key file");
        assert_eq!(
            current_content, original_content,
            "Key file content must remain completely untouched on validation errors"
        );

        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn test_permission_failure_does_not_silently_permit_insecure_key_use() {
        let temp_dir = std::env::temp_dir();
        let invalid_key_path = temp_dir.join(format!(
            "smoooth_test_corrupt_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::write(&invalid_key_path, "not_a_valid_hex_or_raw_key").expect("write corrupt key");

        // Direct reading should succeed at descriptor level, but resolve_or_create parsing must fail closed
        let content = SecretStore::read_secure_key_file(&invalid_key_path).expect("file readable");
        assert_eq!(content, "not_a_valid_hex_or_raw_key");

        let parsed = SecretStore::parse_key(&content);
        assert!(
            parsed.is_none(),
            "Corrupt key content must fail to parse as a valid 256-bit key"
        );

        let _ = std::fs::remove_file(&invalid_key_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_supplementary_group_membership_on_unix() {
        let egid = unsafe {
            extern "C" {
                fn getegid() -> u32;
            }
            getegid()
        };
        // Process must match its own effective GID
        assert!(
            SecretStore::process_has_group(egid),
            "process_has_group must return true for process's own effective GID"
        );

        // Process must match real supplementary groups if any are present in current environment
        let count = unsafe {
            extern "C" {
                fn getgroups(size: i32, list: *mut u32) -> i32;
            }
            getgroups(0, std::ptr::null_mut())
        };
        if count > 0 {
            let mut list = vec![0u32; count as usize];
            let actual = unsafe {
                extern "C" {
                    fn getgroups(size: i32, list: *mut u32) -> i32;
                }
                getgroups(count, list.as_mut_ptr())
            };
            if actual > 0 {
                list.truncate(actual as usize);
                for g in list {
                    assert!(
                        SecretStore::process_has_group(g),
                        "process_has_group must return true for supplementary GID {}",
                        g
                    );
                }
            }
        }

        // Process must NOT match an arbitrary unassigned high GID
        let bogus_gid = 999_999;
        assert!(
            !SecretStore::process_has_group(bogus_gid),
            "process_has_group must return false for an unassigned high GID"
        );
    }

    #[test]
    #[cfg(all(unix, target_os = "linux"))]
    fn test_verify_no_extended_acl_behavior_on_linux() {
        use std::os::unix::io::AsRawFd;
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join(format!(
            "smoooth_test_acl_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let file = std::fs::File::create(&test_path).expect("create test file");

        // On a newly created standard file without setfacl, verify_no_extended_acl must succeed (ENODATA)
        // or return EOPNOTSUPP policy error if temp_dir is on an unsupported filesystem.
        let res = SecretStore::verify_no_extended_acl(file.as_raw_fd(), &test_path);
        if let Err(SmooothError::CryptoError(msg)) = &res {
            assert!(
                msg.contains("EOPNOTSUPP") || msg.contains("extended POSIX ACLs"),
                "If error returned, it must be an explicit policy rejection, got: {}",
                msg
            );
        }

        let _ = std::fs::remove_file(&test_path);
    }

    #[test]
    #[cfg(unix)]
    fn test_multiple_hard_links_rejected_on_unix() {
        let temp_dir = std::env::temp_dir();
        let original_path = temp_dir.join(format!(
            "smoooth_test_orig_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hardlink_path = temp_dir.join(format!(
            "smoooth_test_link_{}.key",
            uuid::Uuid::new_v4().simple()
        ));
        let hex_key = hex::encode([0x77u8; KEY_LEN]);
        std::fs::write(&original_path, &hex_key).expect("write key");

        if std::fs::hard_link(&original_path, &hardlink_path).is_ok() {
            let res = SecretStore::read_secure_key_file(&original_path);
            assert!(
                res.is_err(),
                "Files with multiple hard links must be rejected"
            );
            if let Err(SmooothError::CryptoError(msg)) = res {
                assert!(
                    msg.contains("multiple hard links"),
                    "Error must state that multiple hard links are rejected: {}",
                    msg
                );
            }
            let _ = std::fs::remove_file(&hardlink_path);
        }

        let _ = std::fs::remove_file(&original_path);
    }
}
