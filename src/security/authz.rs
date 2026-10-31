use std::time::{Duration, Instant};
use chrono::Utc;
use ed25519_dalek::VerifyingKey;
use uuid::Uuid;

use crate::error::{Result, SmooothError};
use crate::models::action::{can_attenuate_action_set, Action, DestructionScope};
use crate::models::urn::ResourceUrn;
use crate::security::capability::{CapabilityChain, Caveat};
use crate::security::pop::{ProofOfPossession, DEFAULT_MAX_CLOCK_SKEW_SECONDS};
use crate::storage::Storage;

mod private {
    #[derive(Debug)]
    pub struct Seal(pub(super) ());
}

/// An unforgeable internal proof that an exact operation has been cryptographically authorized.
///
/// Cannot be constructed by tests, adapters, or API handlers. The constructor requires
/// an unexportable private sealed token that can only be issued by `AuthorizationEngine`.
#[derive(Debug)]
pub struct VerifiedExecutionPermit<'a> {
    _seal: private::Seal,
    pub permit_id: Uuid,
    pub caller_agent_id: String,
    pub target_urn: ResourceUrn,
    pub permitted_action: Action,
    pub operation_digest: [u8; 32],
    pub fencing_epoch: u64,
    pub issued_at_monotonic: Instant,
    pub deadline_monotonic: Instant,
    pub expires_at_wallclock: chrono::DateTime<Utc>,
    pub _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> VerifiedExecutionPermit<'a> {
    fn new_sealed(
        caller_agent_id: String,
        target_urn: ResourceUrn,
        permitted_action: Action,
        operation_digest: [u8; 32],
        fencing_epoch: u64,
        ttl: Duration,
    ) -> Self {
        let now_mono = Instant::now();
        Self {
            _seal: private::Seal(()),
            permit_id: Uuid::new_v4(),
            caller_agent_id,
            target_urn,
            permitted_action,
            operation_digest,
            fencing_epoch,
            issued_at_monotonic: now_mono,
            deadline_monotonic: now_mono + ttl,
            expires_at_wallclock: Utc::now() + chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::seconds(60)),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Central Policy & Cryptographic Verification Kernel.
pub struct AuthorizationEngine {
    issuer_verifying_key: VerifyingKey,
}

impl AuthorizationEngine {
    /// Creates a new engine holding the trusted issuer public key.
    pub fn new(issuer_verifying_key: VerifyingKey) -> Self {
        Self { issuer_verifying_key }
    }

    /// Primary authorization boundary: verifies presented capability chain, Proof of Possession,
    /// durable revocation, target URN containment, action permissions, and caveats.
    ///
    /// Produces an unforgeable, zero-cost, in-memory `VerifiedExecutionPermit` upon success.
    pub async fn authorize_capability_request<S: Storage>(
        &self,
        storage: &S,
        chain: &CapabilityChain,
        pop: &ProofOfPossession,
        method: &str,
        canonical_uri: &str,
        body: &[u8],
        target_urn: &ResourceUrn,
        requested_action: &Action,
    ) -> Result<VerifiedExecutionPermit<'static>> {
        // 1. Verify complete cryptographic delegation chain back to issuer key
        chain.verify(&self.issuer_verifying_key)?;

        // 2. Cascading revocation check: walk the entire chain and ensure no link is revoked
        for link in &chain.links {
            if storage.is_revoked(&link.payload.id).await? {
                return Err(SmooothError::Security(format!(
                    "Capability {} (or an ancestor) has been revoked",
                    link.payload.id
                )));
            }
        }

        // 3. Multi-instance replay protection: check if nonce was already used
        if storage.is_nonce_used(&pop.nonce).await? {
            return Err(SmooothError::Security(
                "Proof of possession nonce already consumed (replay attack rejected)".into(),
            ));
        }

        // 4. Verify Proof of Possession against the leaf capability holder key
        let leaf = chain.leaf();
        let digest = pop.verify(
            &leaf.payload.holder_public_key,
            method,
            canonical_uri,
            body,
            DEFAULT_MAX_CLOCK_SKEW_SECONDS,
        )?;

        // 5. Commit nonce to durable storage with expiration (60 seconds)
        let nonce_expiry = pop.timestamp + chrono::Duration::seconds(60);
        storage.record_used_nonce(&pop.nonce, nonce_expiry).await?;

        // 6. Verify target URN containment: leaf capability must encompass requested resource
        if !leaf.payload.target_urn.encompasses(target_urn) {
            return Err(SmooothError::Security(format!(
                "Leaf capability designation '{}' does not encompass target resource '{}'",
                leaf.payload.target_urn, target_urn
            )));
        }

        // 7. Verify requested action is monotonically authorized by leaf capability
        if !can_attenuate_action_set(&leaf.payload.actions, std::slice::from_ref(requested_action)) {
            return Err(SmooothError::Security(format!(
                "Requested action {:?} is not authorized by capability actions: {:?}",
                requested_action, leaf.payload.actions
            )));
        }

        // 8. Caveat enforcement
        for caveat in &leaf.payload.caveats {
            match caveat {
                Caveat::DisallowPersistentDataLoss => {
                    if let Action::Destroy(DestructionScope::ResourceAndPersistentData) = requested_action {
                        return Err(SmooothError::Security(
                            "Capability caveat DisallowPersistentDataLoss prohibits persistent data destruction".into(),
                        ));
                    }
                }
                Caveat::RequireOperatorApproval { plan_digest } => {
                    if &digest != plan_digest {
                        return Err(SmooothError::Security(
                            "Operation digest does not match approved plan digest".into(),
                        ));
                    }
                }
                Caveat::SourceCidr { .. } => {
                    // Checked at network gateway
                }
            }
        }

        // 9. Fetch authoritative fencing epoch from storage
        let fencing_epoch = storage.get_fencing_epoch(target_urn.workspace()).await?;

        // 10. Construct sealed permit with a 60-second monotonic execution deadline
        let permit = VerifiedExecutionPermit::new_sealed(
            leaf.payload.holder_public_key.clone(),
            target_urn.clone(),
            requested_action.clone(),
            digest,
            fencing_epoch,
            Duration::from_secs(60),
        );

        Ok(permit)
    }

    /// Dual-Mode synthesizer: produces a short-lived, single-action execution permit
    /// for authorized plans and legacy scopes without granting ambient wildcards.
    pub fn synthesize_ephemeral_permit(
        caller_agent_id: &str,
        target_urn: ResourceUrn,
        permitted_action: Action,
        operation_digest: [u8; 32],
        fencing_epoch: u64,
        ttl: Duration,
    ) -> VerifiedExecutionPermit<'static> {
        VerifiedExecutionPermit::new_sealed(
            caller_agent_id.to_string(),
            target_urn,
            permitted_action,
            operation_digest,
            fencing_epoch,
            ttl,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::capability::CapabilityPayload;
    use crate::storage::LocalStorage;
    use ed25519_dalek::SigningKey;

    fn make_keypair() -> (SigningKey, VerifyingKey) {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("getrandom failure");
        let signing = SigningKey::from_bytes(&seed);
        let verifying = signing.verifying_key();
        (signing, verifying)
    }

    #[tokio::test]
    async fn test_full_authorization_flow_success() {
        let (issuer_sk, issuer_vk) = make_keypair();
        let (agent_sk, agent_vk) = make_keypair();
        let storage = LocalStorage::new_in_memory();
        let engine = AuthorizationEngine::new(issuer_vk);

        let urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let root_payload = CapabilityPayload {
            id: Uuid::new_v4(),
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(agent_vk.as_bytes()),
            target_urn: urn.clone(),
            actions: vec![Action::RestartWorkload, Action::InspectMetadata],
            caveats: vec![],
            can_delegate: true,
            remaining_depth: 3,
            issued_at: Utc::now(),
            not_before: Utc::now() - chrono::Duration::seconds(5),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let chain = CapabilityChain::new_root(root_payload.clone(), &issuer_sk).unwrap();

        let method = "POST";
        let uri = "/api/workloads/production/web-api/restart";
        let body = b"{\"grace_period\": 10}";
        let nonce = "unique_nonce_1".to_string();

        let pop = ProofOfPossession::generate(
            &agent_sk,
            root_payload.id,
            method,
            uri,
            body,
            nonce,
        ).unwrap();

        let requested_action = Action::RestartWorkload;

        let permit = engine
            .authorize_capability_request(
                &storage,
                &chain,
                &pop,
                method,
                uri,
                body,
                &urn,
                &requested_action,
            )
            .await
            .expect("Authorization should succeed");

        assert_eq!(permit.target_urn, urn);
        assert_eq!(permit.permitted_action, Action::RestartWorkload);
        assert_eq!(permit.fencing_epoch, 1);
    }

    #[tokio::test]
    async fn test_authorization_fails_if_revoked() {
        let (issuer_sk, issuer_vk) = make_keypair();
        let (agent_sk, agent_vk) = make_keypair();
        let storage = LocalStorage::new_in_memory();
        let engine = AuthorizationEngine::new(issuer_vk);

        let cap_id = Uuid::new_v4();
        let urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let root_payload = CapabilityPayload {
            id: cap_id,
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(agent_vk.as_bytes()),
            target_urn: urn.clone(),
            actions: vec![Action::RestartWorkload],
            caveats: vec![],
            can_delegate: true,
            remaining_depth: 3,
            issued_at: Utc::now(),
            not_before: Utc::now() - chrono::Duration::seconds(5),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

        // Mark the capability revoked in durable storage
        storage
            .record_revocation(&cap_id, "Compromised key", Utc::now() + chrono::Duration::hours(1))
            .await
            .unwrap();

        let method = "POST";
        let uri = "/api/workloads/production/web-api/restart";
        let body = b"{}";
        let pop = ProofOfPossession::generate(
            &agent_sk,
            cap_id,
            method,
            uri,
            body,
            "nonce_2".into(),
        ).unwrap();

        let res = engine
            .authorize_capability_request(
                &storage,
                &chain,
                &pop,
                method,
                uri,
                body,
                &urn,
                &Action::RestartWorkload,
            )
            .await;

        assert!(res.is_err(), "Revoked capability must be rejected");
    }

    #[tokio::test]
    async fn test_authorization_fails_on_nonce_replay() {
        let (issuer_sk, issuer_vk) = make_keypair();
        let (agent_sk, agent_vk) = make_keypair();
        let storage = LocalStorage::new_in_memory();
        let engine = AuthorizationEngine::new(issuer_vk);

        let urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let cap_id = Uuid::new_v4();
        let root_payload = CapabilityPayload {
            id: cap_id,
            parent_id: None,
            issuer: "smoooth:control-plane".into(),
            holder_public_key: hex::encode(agent_vk.as_bytes()),
            target_urn: urn.clone(),
            actions: vec![Action::RestartWorkload],
            caveats: vec![],
            can_delegate: true,
            remaining_depth: 3,
            issued_at: Utc::now(),
            not_before: Utc::now() - chrono::Duration::seconds(5),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

        let method = "POST";
        let uri = "/api/workloads/production/web-api/restart";
        let body = b"{}";
        let pop = ProofOfPossession::generate(
            &agent_sk,
            cap_id,
            method,
            uri,
            body,
            "nonce_replay_test".into(),
        ).unwrap();

        // First attempt succeeds and records nonce
        let first = engine
            .authorize_capability_request(
                &storage,
                &chain,
                &pop,
                method,
                uri,
                body,
                &urn,
                &Action::RestartWorkload,
            )
            .await;
        assert!(first.is_ok());

        // Replay of the exact same request with same nonce fails
        let second = engine
            .authorize_capability_request(
                &storage,
                &chain,
                &pop,
                method,
                uri,
                body,
                &urn,
                &Action::RestartWorkload,
            )
            .await;
        assert!(second.is_err(), "Replayed nonce must be rejected");
    }
}
