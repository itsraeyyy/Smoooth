use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Result, SmooothError};

/// Default maximum allowable wall-clock skew in seconds (±30s).
pub const DEFAULT_MAX_CLOCK_SKEW_SECONDS: i64 = 30;

/// Computes the deterministic SHA-256 digest of a canonical HTTP or MCP request.
///
/// Digest Format:
/// SHA256( Method || "\n" || CanonicalURI || "\n" || BodyHash || "\n" || CapabilityId || "\n" || Timestamp || "\n" || Nonce )
pub fn compute_request_digest(
    method: &str,
    canonical_uri: &str,
    body: &[u8],
    capability_id: &Uuid,
    timestamp: &DateTime<Utc>,
    nonce: &str,
) -> [u8; 32] {
    let mut body_hasher = Sha256::new();
    body_hasher.update(body);
    let body_hash: [u8; 32] = body_hasher.finalize().into();

    let mut hasher = Sha256::new();
    hasher.update(method.to_uppercase().as_bytes());
    hasher.update(b"\n");
    hasher.update(canonical_uri.as_bytes());
    hasher.update(b"\n");
    hasher.update(&body_hash);
    hasher.update(b"\n");
    hasher.update(capability_id.to_string().as_bytes());
    hasher.update(b"\n");
    hasher.update(timestamp.to_rfc3339().as_bytes());
    hasher.update(b"\n");
    hasher.update(nonce.as_bytes());

    hasher.finalize().into()
}

/// A cryptographic Proof of Possession presented with an authorized operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofOfPossession {
    /// Identifier of the leaf capability being presented.
    pub capability_id: Uuid,

    /// Hex-encoded Ed25519 public key of the presenter.
    pub holder_public_key: String,

    /// Wall-clock timestamp when the request proof was generated.
    pub timestamp: DateTime<Utc>,

    /// Unique cryptographically secure single-use nonce.
    pub nonce: String,

    /// Hex-encoded Ed25519 signature of the computed canonical request digest.
    pub signature: String,
}

impl ProofOfPossession {
    /// Generates a signed Proof of Possession using the holder's private key.
    pub fn generate(
        signing_key: &SigningKey,
        capability_id: Uuid,
        method: &str,
        canonical_uri: &str,
        body: &[u8],
        nonce: String,
    ) -> Result<Self> {
        let timestamp = Utc::now();
        let digest = compute_request_digest(
            method,
            canonical_uri,
            body,
            &capability_id,
            &timestamp,
            &nonce,
        );

        let signature = signing_key.sign(&digest);
        let holder_public_key = hex::encode(signing_key.verifying_key().as_bytes());

        Ok(Self {
            capability_id,
            holder_public_key,
            timestamp,
            nonce,
            signature: hex::encode(signature.to_bytes()),
        })
    }

    /// Verifies the Proof of Possession against the incoming request and expected public key.
    ///
    /// Validates:
    /// 1. Wall-clock timestamp is within the permitted clock skew window.
    /// 2. Holder public key matches the expected capability holder.
    /// 3. Ed25519 signature accurately authenticates the canonical request digest.
    ///
    /// Returns the authenticated request digest on success.
    pub fn verify(
        &self,
        expected_holder_pk_hex: &str,
        method: &str,
        canonical_uri: &str,
        body: &[u8],
        max_clock_skew_seconds: i64,
    ) -> Result<[u8; 32]> {
        // 1. Holder public key match
        if self.holder_public_key != expected_holder_pk_hex {
            return Err(SmooothError::Security(
                "Proof of possession holder key does not match capability holder".into(),
            ));
        }

        // 2. Clock skew verification
        let now = Utc::now();
        let diff = (now - self.timestamp).num_seconds().abs();
        if diff > max_clock_skew_seconds {
            return Err(SmooothError::Security(format!(
                "Proof of possession timestamp out of bounds: diff {}s exceeds max allowed {}s",
                diff, max_clock_skew_seconds
            )));
        }

        // 3. Compute expected request digest
        let expected_digest = compute_request_digest(
            method,
            canonical_uri,
            body,
            &self.capability_id,
            &self.timestamp,
            &self.nonce,
        );

        // 4. Parse public key
        let pk_bytes = hex::decode(&self.holder_public_key)
            .map_err(|e| SmooothError::CryptoError(format!("Invalid public key hex: {e}")))?;
        if pk_bytes.len() != 32 {
            return Err(SmooothError::CryptoError("Public key must be 32 bytes".into()));
        }
        let verifying_key = VerifyingKey::from_bytes(pk_bytes[..32].try_into().unwrap())
            .map_err(|e| SmooothError::CryptoError(format!("Failed to parse public key: {e}")))?;

        // 5. Parse signature
        let sig_bytes = hex::decode(&self.signature)
            .map_err(|e| SmooothError::CryptoError(format!("Invalid signature hex: {e}")))?;
        if sig_bytes.len() != 64 {
            return Err(SmooothError::CryptoError("Signature must be 64 bytes".into()));
        }
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| SmooothError::CryptoError(format!("Failed to parse signature: {e}")))?;

        // 6. Verify signature
        verifying_key
            .verify(&expected_digest, &signature)
            .map_err(|e| SmooothError::Security(format!("Proof of possession signature verification failed: {e}")))?;

        Ok(expected_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keypair() -> (SigningKey, VerifyingKey) {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("getrandom failure");
        let signing = SigningKey::from_bytes(&seed);
        let verifying = signing.verifying_key();
        (signing, verifying)
    }

    #[test]
    fn test_pop_generation_and_verification_roundtrip() {
        let (sk, vk) = make_keypair();
        let cap_id = Uuid::new_v4();
        let method = "POST";
        let uri = "/api/workloads/production/web-api/restart";
        let body = b"{\"grace_period_seconds\": 30}";
        let nonce = "nonce_123456789abcdef".to_string();

        let pop = ProofOfPossession::generate(
            &sk,
            cap_id,
            method,
            uri,
            body,
            nonce,
        ).unwrap();

        let expected_pk_hex = hex::encode(vk.as_bytes());

        // Verifying with matching inputs succeeds
        let digest_res = pop.verify(&expected_pk_hex, method, uri, body, 30);
        assert!(digest_res.is_ok());

        // Tampering with body fails
        let tampered_body = b"{\"grace_period_seconds\": 0}";
        assert!(pop.verify(&expected_pk_hex, method, uri, tampered_body, 30).is_err());

        // Tampering with URI fails
        let tampered_uri = "/api/workloads/production/database/restart";
        assert!(pop.verify(&expected_pk_hex, method, tampered_uri, body, 30).is_err());

        // Wrong expected holder key fails
        let (_other_sk, other_vk) = make_keypair();
        let other_pk_hex = hex::encode(other_vk.as_bytes());
        assert!(pop.verify(&other_pk_hex, method, uri, body, 30).is_err());
    }

    #[test]
    fn test_pop_timestamp_skew_rejection() {
        let (sk, vk) = make_keypair();
        let cap_id = Uuid::new_v4();
        let method = "GET";
        let uri = "/api/workloads/production/web-api/env";
        let body = b"";
        let nonce = "nonce_skew_test".to_string();

        let mut pop = ProofOfPossession::generate(
            &sk,
            cap_id,
            method,
            uri,
            body,
            nonce,
        ).unwrap();

        // Artificially age the timestamp by 2 minutes
        pop.timestamp = Utc::now() - chrono::Duration::seconds(120);

        let expected_pk_hex = hex::encode(vk.as_bytes());
        let res = pop.verify(&expected_pk_hex, method, uri, body, 30);
        assert!(res.is_err(), "Timestamp skew beyond 30s must fail");
    }
}
