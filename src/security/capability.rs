use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Result, SmooothError};
use crate::models::action::{can_attenuate_action_set, Action};
use crate::models::urn::ResourceUrn;

/// Maximum allowed delegation depth across Smoooth.
pub const MAX_DELEGATION_DEPTH: u8 = 4;

/// Maximum capability chain payload size in bytes (16 KB) to prevent DoS.
pub const MAX_CHAIN_SIZE_BYTES: usize = 16 * 1024;

/// Cryptographic caveats attached to a capability grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "caveat_type", rename_all = "snake_case")]
pub enum Caveat {
    /// Requires cryptographic approval from a human operator bound to the plan digest.
    RequireOperatorApproval { plan_digest: [u8; 32] },

    /// Restricts invocation to requests originating from a specific IP CIDR.
    SourceCidr { cidr: String },

    /// Prohibits persistent volume or database deletion even if Destroy action is held.
    DisallowPersistentDataLoss,
}

impl Caveat {
    /// Evaluates whether `child` caveat monotonically satisfies `self` (parent caveat).
    pub fn is_satisfied_by(&self, child_caveats: &[Caveat]) -> bool {
        match self {
            Caveat::DisallowPersistentDataLoss => {
                child_caveats.contains(&Caveat::DisallowPersistentDataLoss)
            }
            Caveat::RequireOperatorApproval { plan_digest: p_digest } => {
                child_caveats.iter().any(|c| match c {
                    Caveat::RequireOperatorApproval { plan_digest: c_digest } => p_digest == c_digest,
                    _ => false,
                })
            }
            Caveat::SourceCidr { cidr: p_cidr } => {
                child_caveats.iter().any(|c| match c {
                    Caveat::SourceCidr { cidr: c_cidr } => p_cidr == c_cidr,
                    _ => false,
                })
            }
        }
    }
}

/// The immutable payload of an individual capability grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPayload {
    /// Unique identifier of this capability.
    pub id: Uuid,

    /// Identifier of the parent capability, or None if this is the root.
    pub parent_id: Option<Uuid>,

    /// Issuer identification string.
    pub issuer: String,

    /// Hex-encoded Ed25519 public key of the authorized holder.
    pub holder_public_key: String,

    /// Designated resource URN.
    pub target_urn: ResourceUrn,

    /// List of explicitly permitted actions.
    pub actions: Vec<Action>,

    /// Specific caveats and restrictions.
    pub caveats: Vec<Caveat>,

    /// Whether this capability can be further attenuated and delegated.
    pub can_delegate: bool,

    /// Remaining allowed delegation depth.
    pub remaining_depth: u8,

    /// Wall-clock issuance timestamp.
    pub issued_at: DateTime<Utc>,

    /// Wall-clock earliest validity timestamp.
    pub not_before: DateTime<Utc>,

    /// Wall-clock expiration timestamp.
    pub expires_at: DateTime<Utc>,
}

impl CapabilityPayload {
    /// Computes the canonical SHA-256 digest of the payload bytes.
    pub fn canonical_digest(&self) -> Result<[u8; 32]> {
        let canonical_json = serde_json::to_vec(self)
            .map_err(|e| SmooothError::Validation(format!("Failed to serialize payload: {e}")))?;
        let mut hasher = Sha256::new();
        hasher.update(&canonical_json);
        Ok(hasher.finalize().into())
    }

    /// Evaluates monotonic attenuation from `self` (parent) to `child`.
    pub fn can_attenuate_to(&self, child: &CapabilityPayload) -> Result<()> {
        // 1. Delegation must be enabled on parent
        if !self.can_delegate {
            return Err(SmooothError::AttenuationViolation(
                "Parent capability does not permit delegation".into(),
            ));
        }

        // 2. Depth check
        if self.remaining_depth == 0 {
            return Err(SmooothError::AttenuationViolation(
                "Maximum delegation depth reached; cannot delegate further".into(),
            ));
        }
        if child.remaining_depth >= self.remaining_depth {
            return Err(SmooothError::AttenuationViolation(
                "Child remaining_depth must be strictly less than parent remaining_depth".into(),
            ));
        }

        // 3. Parent ID pointer check
        if child.parent_id != Some(self.id) {
            return Err(SmooothError::AttenuationViolation(
                "Child parent_id does not match parent capability id".into(),
            ));
        }

        // 4. Resource URN containment check
        if !self.target_urn.encompasses(&child.target_urn) {
            return Err(SmooothError::AttenuationViolation(format!(
                "Child target URN '{}' is not encompassed by parent target URN '{}'",
                child.target_urn, self.target_urn
            )));
        }

        // 5. Actions monotonic narrowing check
        if !can_attenuate_action_set(&self.actions, &child.actions) {
            return Err(SmooothError::AttenuationViolation(
                "Child actions are not a valid monotonic subset of parent actions".into(),
            ));
        }

        // 6. Time bounds check: child cannot outlive parent
        if child.expires_at > self.expires_at {
            return Err(SmooothError::AttenuationViolation(
                "Child expires_at cannot exceed parent expires_at".into(),
            ));
        }
        if child.not_before < self.not_before {
            return Err(SmooothError::AttenuationViolation(
                "Child not_before cannot precede parent not_before".into(),
            ));
        }

        // 7. Caveat monotonicity: all parent caveats must be preserved or strengthened
        for p_caveat in &self.caveats {
            if !p_caveat.is_satisfied_by(&child.caveats) {
                return Err(SmooothError::AttenuationViolation(format!(
                    "Child does not preserve required parent caveat: {:?}",
                    p_caveat
                )));
            }
        }

        Ok(())
    }
}

/// An individual signed link in a capability delegation chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityLink {
    /// Payload containing permissions and parameters.
    pub payload: CapabilityPayload,

    /// Hex-encoded Ed25519 signature of the payload canonical digest.
    pub signature: String,
}

impl CapabilityLink {
    /// Creates and signs a capability link using the given signing key.
    pub fn sign(payload: CapabilityPayload, signing_key: &SigningKey) -> Result<Self> {
        let digest = payload.canonical_digest()?;
        let signature = signing_key.sign(&digest);
        Ok(Self {
            payload,
            signature: hex::encode(signature.to_bytes()),
        })
    }

    /// Verifies the signature of this link against the expected public key.
    pub fn verify_signature(&self, expected_verifying_key: &VerifyingKey) -> Result<()> {
        let digest = self.payload.canonical_digest()?;
        let sig_bytes = hex::decode(&self.signature)
            .map_err(|e| SmooothError::CryptoError(format!("Invalid signature hex: {e}")))?;
        if sig_bytes.len() != 64 {
            return Err(SmooothError::CryptoError("Signature must be exactly 64 bytes".into()));
        }
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| SmooothError::CryptoError(format!("Failed to parse signature: {e}")))?;

        expected_verifying_key
            .verify(&digest, &signature)
            .map_err(|e| SmooothError::CryptoError(format!("Capability signature verification failed: {e}")))?;

        Ok(())
    }
}

/// An ordered sequence of capability links representing a complete delegation chain.
///
/// Link 0 is the root capability signed by the trusted Smoooth Control Plane Issuer.
/// Each subsequent link $i$ is signed by link $i-1$'s holder public key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityChain {
    pub links: Vec<CapabilityLink>,
}

impl CapabilityChain {
    /// Initializes a new chain with a root capability signed by the issuer.
    pub fn new_root(root_payload: CapabilityPayload, issuer_signing_key: &SigningKey) -> Result<Self> {
        if root_payload.parent_id.is_some() {
            return Err(SmooothError::Validation("Root capability must have no parent_id".into()));
        }
        if root_payload.remaining_depth > MAX_DELEGATION_DEPTH {
            return Err(SmooothError::Validation(format!(
                "Root delegation depth {} exceeds maximum allowed {}",
                root_payload.remaining_depth, MAX_DELEGATION_DEPTH
            )));
        }

        let link = CapabilityLink::sign(root_payload, issuer_signing_key)?;
        Ok(Self { links: vec![link] })
    }

    /// Appends a delegated child capability signed by the current leaf's private key.
    pub fn delegate(
        &self,
        child_payload: CapabilityPayload,
        parent_signing_key: &SigningKey,
    ) -> Result<Self> {
        let leaf = self.leaf();

        // 1. Verify parent holder matches the provided signing key
        let parent_pk_hex = hex::encode(parent_signing_key.verifying_key().as_bytes());
        if leaf.payload.holder_public_key != parent_pk_hex {
            return Err(SmooothError::Security(
                "Signing key does not match parent capability holder public key".into(),
            ));
        }

        // 2. Validate monotonic attenuation
        leaf.payload.can_attenuate_to(&child_payload)?;

        // 3. Sign the child capability
        let child_link = CapabilityLink::sign(child_payload, parent_signing_key)?;

        let mut new_links = self.links.clone();
        new_links.push(child_link);

        let new_chain = Self { links: new_links };

        // 4. Validate chain limits
        if new_chain.links.len() > (MAX_DELEGATION_DEPTH as usize + 1) {
            return Err(SmooothError::Security("Delegation chain exceeds maximum depth limit".into()));
        }
        let chain_bytes = serde_json::to_vec(&new_chain)
            .map_err(|e| SmooothError::Validation(format!("Failed to serialize chain: {e}")))?;
        if chain_bytes.len() > MAX_CHAIN_SIZE_BYTES {
            return Err(SmooothError::Security(format!(
                "Delegation chain size {} bytes exceeds maximum limit {}",
                chain_bytes.len(), MAX_CHAIN_SIZE_BYTES
            )));
        }

        Ok(new_chain)
    }

    /// Returns a reference to the root capability link.
    pub fn root(&self) -> &CapabilityLink {
        &self.links[0]
    }

    /// Returns a reference to the active leaf capability link.
    pub fn leaf(&self) -> &CapabilityLink {
        self.links.last().expect("Chain must have at least one link")
    }

    /// Verifies the entire cryptographic and attenuation validity of the chain back to the issuer.
    pub fn verify(&self, trusted_issuer_key: &VerifyingKey) -> Result<()> {
        if self.links.is_empty() {
            return Err(SmooothError::Security("Capability chain cannot be empty".into()));
        }

        // 1. Verify root signature using trusted issuer key
        let root = &self.links[0];
        root.verify_signature(trusted_issuer_key)?;

        // 2. Verify each child link in sequence
        for i in 1..self.links.len() {
            let parent_link = &self.links[i - 1];
            let child_link = &self.links[i];

            // A. Parse parent holder's verifying key
            let parent_pk_bytes = hex::decode(&parent_link.payload.holder_public_key)
                .map_err(|e| SmooothError::CryptoError(format!("Invalid parent holder key hex: {e}")))?;
            if parent_pk_bytes.len() != 32 {
                return Err(SmooothError::CryptoError("Public key must be 32 bytes".into()));
            }
            let parent_vk = VerifyingKey::from_bytes(parent_pk_bytes[..32].try_into().unwrap())
                .map_err(|e| SmooothError::CryptoError(format!("Failed to parse parent public key: {e}")))?;

            // B. Verify child signature using parent public key
            child_link.verify_signature(&parent_vk)?;

            // C. Programmatically verify monotonic attenuation
            parent_link.payload.can_attenuate_to(&child_link.payload)?;
        }

        Ok(())
    }

    /// Serializes the chain to a URL-safe Base64 string for transmission in headers.
    pub fn to_header_string(&self) -> Result<String> {
        let json_bytes = serde_json::to_vec(self)
            .map_err(|e| SmooothError::Validation(format!("Serialization error: {e}")))?;
        Ok(hex::encode(json_bytes))
    }

    /// Deserializes a chain from a header string.
    pub fn from_header_string(header: &str) -> Result<Self> {
        let bytes = hex::decode(header)
            .map_err(|e| SmooothError::Validation(format!("Header hex decode error: {e}")))?;
        if bytes.len() > MAX_CHAIN_SIZE_BYTES {
            return Err(SmooothError::Security("Chain header exceeds maximum allowed size".into()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| SmooothError::Validation(format!("Header deserialization error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::action::DestructionScope;

    fn make_keypair() -> (SigningKey, VerifyingKey) {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("getrandom failure");
        let signing = SigningKey::from_bytes(&seed);
        let verifying = signing.verifying_key();
        (signing, verifying)
    }

    #[test]
    fn test_root_capability_creation_and_verification() {
        let (issuer_sk, issuer_vk) = make_keypair();
        let (_agent_sk, agent_vk) = make_keypair();

        let urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let payload = CapabilityPayload {
            id: Uuid::new_v4(),
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(agent_vk.as_bytes()),
            target_urn: urn,
            actions: vec![Action::InspectMetadata, Action::ReadLogs { tail_limit: Some(100) }],
            caveats: vec![Caveat::DisallowPersistentDataLoss],
            can_delegate: true,
            remaining_depth: 3,
            issued_at: Utc::now(),
            not_before: Utc::now() - chrono::Duration::seconds(5),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let chain = CapabilityChain::new_root(payload, &issuer_sk).unwrap();
        assert_eq!(chain.links.len(), 1);

        // Verification with correct issuer key succeeds
        assert!(chain.verify(&issuer_vk).is_ok());

        // Verification with forged issuer key fails
        let (_attacker_sk, attacker_vk) = make_keypair();
        assert!(chain.verify(&attacker_vk).is_err());
    }

    #[test]
    fn test_valid_delegation_chain_verification() {
        let (issuer_sk, issuer_vk) = make_keypair();
        let (lead_sk, lead_vk) = make_keypair();
        let (_subagent_sk, subagent_vk) = make_keypair();

        // 1. Root capability for lead agent
        let root_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let root_id = Uuid::new_v4();
        let now = Utc::now();
        let root_payload = CapabilityPayload {
            id: root_id,
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(lead_vk.as_bytes()),
            target_urn: root_urn,
            actions: vec![
                Action::InspectMetadata,
                Action::ReadLogs { tail_limit: Some(100) },
                Action::RestartWorkload,
            ],
            caveats: vec![Caveat::DisallowPersistentDataLoss],
            can_delegate: true,
            remaining_depth: 2,
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::hours(1),
        };

        let root_chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

        // 2. Lead agent delegates narrowed authority to subagent
        let child_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api/logs/stdout").unwrap();
        let child_payload = CapabilityPayload {
            id: Uuid::new_v4(),
            parent_id: Some(root_id),
            issuer: hex::encode(lead_vk.as_bytes()),
            holder_public_key: hex::encode(subagent_vk.as_bytes()),
            target_urn: child_urn,
            actions: vec![Action::ReadLogs { tail_limit: Some(50) }], // Narrowed action & tail
            caveats: vec![Caveat::DisallowPersistentDataLoss],          // Preserved caveat
            can_delegate: false,                                       // No further delegation
            remaining_depth: 1,                                        // Decreased depth
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::minutes(10),           // Shorter expiry
        };

        let delegated_chain = root_chain.delegate(child_payload, &lead_sk).unwrap();
        assert_eq!(delegated_chain.links.len(), 2);

        // Verification back to issuer key succeeds
        assert!(delegated_chain.verify(&issuer_vk).is_ok());
    }

    #[test]
    fn test_delegation_privilege_escalation_rejected() {
        let (issuer_sk, _issuer_vk) = make_keypair();
        let (lead_sk, lead_vk) = make_keypair();
        let (_subagent_sk, subagent_vk) = make_keypair();

        let root_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let root_id = Uuid::new_v4();
        let now = Utc::now();
        let root_payload = CapabilityPayload {
            id: root_id,
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(lead_vk.as_bytes()),
            target_urn: root_urn,
            actions: vec![Action::InspectMetadata],
            caveats: vec![Caveat::DisallowPersistentDataLoss],
            can_delegate: true,
            remaining_depth: 2,
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::hours(1),
        };

        let root_chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

        // Lead tries to delegate Destroy action that it does not possess!
        let child_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let child_payload = CapabilityPayload {
            id: Uuid::new_v4(),
            parent_id: Some(root_id),
            issuer: hex::encode(lead_vk.as_bytes()),
            holder_public_key: hex::encode(subagent_vk.as_bytes()),
            target_urn: child_urn,
            actions: vec![Action::Destroy(DestructionScope::ResourceOnly)],
            caveats: vec![Caveat::DisallowPersistentDataLoss],
            can_delegate: false,
            remaining_depth: 1,
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::minutes(10),
        };

        let res = root_chain.delegate(child_payload, &lead_sk);
        assert!(res.is_err(), "Delegating escalated action must fail");
    }

    #[test]
    fn test_delegation_longer_expiry_rejected() {
        let (issuer_sk, _issuer_vk) = make_keypair();
        let (lead_sk, lead_vk) = make_keypair();
        let (_subagent_sk, subagent_vk) = make_keypair();

        let root_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let root_id = Uuid::new_v4();
        let now = Utc::now();
        let root_payload = CapabilityPayload {
            id: root_id,
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(lead_vk.as_bytes()),
            target_urn: root_urn,
            actions: vec![Action::InspectMetadata],
            caveats: vec![],
            can_delegate: true,
            remaining_depth: 2,
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::minutes(30),
        };

        let root_chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

        // Child attempts to set 2-hour expiry (parent expires in 30 minutes)
        let child_payload = CapabilityPayload {
            id: Uuid::new_v4(),
            parent_id: Some(root_id),
            issuer: hex::encode(lead_vk.as_bytes()),
            holder_public_key: hex::encode(subagent_vk.as_bytes()),
            target_urn: ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap(),
            actions: vec![Action::InspectMetadata],
            caveats: vec![],
            can_delegate: false,
            remaining_depth: 1,
            issued_at: now,
            not_before: now,
            expires_at: now + chrono::Duration::hours(2),
        };

        let res = root_chain.delegate(child_payload, &lead_sk);
        assert!(res.is_err(), "Child cannot outlive parent");
    }
}
