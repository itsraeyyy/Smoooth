use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use crate::error::{Result, SmooothError};

pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

/// Secure Secret Store providing AES-256-GCM authenticated encryption
/// for infrastructure credentials and connection tokens.
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

    /// Encrypt plaintext bytes using AES-256-GCM with a randomized 96-bit nonce.
    /// Serialized payload format: [12-byte Nonce] + [Ciphertext + 16-byte Poly1305 Tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SmooothError::CryptoError(format!("Failed to initialize cipher: {}", e)))?;

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

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SmooothError::CryptoError(format!("Failed to initialize cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&payload[..NONCE_LEN]);
        let ciphertext = &payload[NONCE_LEN..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SmooothError::CryptoError(format!("Decryption or authentication tag failed: {}", e)))
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
        assert_eq!(decrypted, secret, "Decrypted bytes must match original plaintext");
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
        assert!(result.is_err(), "Tampered ciphertext must fail authentication");
    }
}
