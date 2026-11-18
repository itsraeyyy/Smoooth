use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::sync::Arc;
use uuid::Uuid;

use smoooth::adapters::ProviderAdapter;
use smoooth::error::SmooothError;
use smoooth::models::{Action, AgentIdentity, AgentScope, DestructionScope, ResourceUrn};
use smoooth::security::{
    AuthorizationEngine, CapabilityChain, CapabilityPayload, ProofOfPossession,
};
use smoooth::storage::{LocalStorage, Storage};

fn make_keypair() -> (SigningKey, VerifyingKey) {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("getrandom failure");
    let signing = SigningKey::from_bytes(&seed);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

#[tokio::test]
async fn test_adversarial_stolen_token_rejected_without_pop_private_key() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (_victim_sk, victim_vk) = make_keypair();
    let (thief_sk, _thief_vk) = make_keypair();
    let storage = LocalStorage::new_in_memory();
    let engine = AuthorizationEngine::new(issuer_vk);

    let target_urn =
        ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();

    // Legitimate capability issued to Victim
    let root_payload = CapabilityPayload {
        id: Uuid::new_v4(),
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(victim_vk.as_bytes()),
        target_urn: target_urn.clone(),
        actions: vec![Action::RestartWorkload],
        caveats: vec![],
        can_delegate: true,
        remaining_depth: 3,
        issued_at: Utc::now(),
        not_before: Utc::now() - chrono::Duration::seconds(5),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    let chain = CapabilityChain::new_root(root_payload.clone(), &issuer_sk).unwrap();

    // Adversary intercepts the capability chain and presents it with their own key
    let method = "POST";
    let uri = "/api/workloads/production/web-api/restart";
    let body = b"{}";
    let thief_nonce = "thief_nonce_123".to_string();

    let thief_pop =
        ProofOfPossession::generate(&thief_sk, root_payload.id, method, uri, body, thief_nonce)
            .unwrap();

    let res = engine
        .authorize_capability_request(
            &storage,
            &chain,
            &thief_pop,
            method,
            uri,
            body,
            &target_urn,
            &Action::RestartWorkload,
        )
        .await;

    assert!(
        res.is_err(),
        "Stolen capability token used with thief key must fail"
    );
    if let Err(SmooothError::Security(msg)) = res {
        assert!(msg.contains("Proof of possession holder key does not match capability holder"));
    } else {
        panic!("Expected SmooothError::Security");
    }
}

#[tokio::test]
async fn test_adversarial_nonce_replay_rejected() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (agent_sk, agent_vk) = make_keypair();
    let storage = LocalStorage::new_in_memory();
    let engine = AuthorizationEngine::new(issuer_vk);

    let target_urn =
        ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
    let root_payload = CapabilityPayload {
        id: Uuid::new_v4(),
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(agent_vk.as_bytes()),
        target_urn: target_urn.clone(),
        actions: vec![Action::RestartWorkload],
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
    let body = b"{}";
    let nonce = "unique_fixed_nonce".to_string();

    let pop =
        ProofOfPossession::generate(&agent_sk, root_payload.id, method, uri, body, nonce).unwrap();

    // 1. Initial invocation succeeds
    let first = engine
        .authorize_capability_request(
            &storage,
            &chain,
            &pop,
            method,
            uri,
            body,
            &target_urn,
            &Action::RestartWorkload,
        )
        .await;
    assert!(first.is_ok(), "First request should succeed");

    // 2. Replayed invocation fails closed
    let second = engine
        .authorize_capability_request(
            &storage,
            &chain,
            &pop,
            method,
            uri,
            body,
            &target_urn,
            &Action::RestartWorkload,
        )
        .await;
    assert!(second.is_err(), "Replayed nonce must be rejected");
    if let Err(SmooothError::Security(msg)) = second {
        assert!(msg.contains("nonce already consumed"));
    } else {
        panic!("Expected SmooothError::Security");
    }
}

#[tokio::test]
async fn test_adversarial_cascading_revocation_invalidates_descendants() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (lead_sk, lead_vk) = make_keypair();
    let (subagent_sk, subagent_vk) = make_keypair();
    let (worker_sk, worker_vk) = make_keypair();
    let storage = LocalStorage::new_in_memory();
    let engine = AuthorizationEngine::new(issuer_vk);

    let root_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
    let root_id = Uuid::new_v4();
    let child_1_id = Uuid::new_v4();
    let child_2_id = Uuid::new_v4();
    let now = Utc::now();

    // 1. Root capability for lead agent
    let root_payload = CapabilityPayload {
        id: root_id,
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(lead_vk.as_bytes()),
        target_urn: root_urn.clone(),
        actions: vec![
            Action::InspectMetadata,
            Action::ReadLogs {
                tail_limit: Some(100),
            },
        ],
        caveats: vec![],
        can_delegate: true,
        remaining_depth: 3,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::hours(1),
    };
    let chain_root = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

    // 2. Lead delegates to Subagent
    let child_1_payload = CapabilityPayload {
        id: child_1_id,
        parent_id: Some(root_id),
        issuer: hex::encode(lead_vk.as_bytes()),
        holder_public_key: hex::encode(subagent_vk.as_bytes()),
        target_urn: root_urn.clone(),
        actions: vec![Action::ReadLogs {
            tail_limit: Some(50),
        }],
        caveats: vec![],
        can_delegate: true,
        remaining_depth: 2,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::minutes(30),
    };
    let chain_sub = chain_root.delegate(child_1_payload, &lead_sk).unwrap();

    // 3. Subagent delegates to Worker
    let child_2_payload = CapabilityPayload {
        id: child_2_id,
        parent_id: Some(child_1_id),
        issuer: hex::encode(subagent_vk.as_bytes()),
        holder_public_key: hex::encode(worker_vk.as_bytes()),
        target_urn: root_urn.clone(),
        actions: vec![Action::ReadLogs {
            tail_limit: Some(25),
        }],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::minutes(15),
    };
    let chain_worker = chain_sub.delegate(child_2_payload, &subagent_sk).unwrap();
    assert_eq!(chain_worker.links.len(), 3);

    // 4. Worker executes successfully prior to revocation
    let method = "GET";
    let uri = "/api/workloads/production/web-api/logs";
    let body = b"";
    let pop_1 = ProofOfPossession::generate(
        &worker_sk,
        child_2_id,
        method,
        uri,
        body,
        "worker_nonce_1".into(),
    )
    .unwrap();

    let pre_revocation = engine
        .authorize_capability_request(
            &storage,
            &chain_worker,
            &pop_1,
            method,
            uri,
            body,
            &root_urn,
            &Action::ReadLogs {
                tail_limit: Some(10),
            },
        )
        .await;
    assert!(pre_revocation.is_ok());

    // 5. Root capability is durably revoked!
    storage
        .record_revocation(
            &root_id,
            "Key compromise detected",
            now + chrono::Duration::hours(1),
        )
        .await
        .unwrap();

    // 6. Worker tries again with fresh nonce: must fail due to cascading ancestor check
    let pop_2 = ProofOfPossession::generate(
        &worker_sk,
        child_2_id,
        method,
        uri,
        body,
        "worker_nonce_2".into(),
    )
    .unwrap();

    let post_revocation = engine
        .authorize_capability_request(
            &storage,
            &chain_worker,
            &pop_2,
            method,
            uri,
            body,
            &root_urn,
            &Action::ReadLogs {
                tail_limit: Some(10),
            },
        )
        .await;

    assert!(
        post_revocation.is_err(),
        "Worker should fail because root ancestor is revoked"
    );
    if let Err(SmooothError::Security(msg)) = post_revocation {
        assert!(msg.contains("has been revoked"));
    } else {
        panic!("Expected SmooothError::Security");
    }
}

#[tokio::test]
async fn test_adversarial_adapter_digest_and_scope_checks() {
    let target_urn =
        ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
    let expected_digest = [42u8; 32];
    let fencing_epoch = 1u64;

    // Permit authorizing ResourceOnly destruction
    let permit = AuthorizationEngine::synthesize_ephemeral_permit(
        "operator",
        target_urn.clone(),
        Action::Destroy(DestructionScope::ResourceOnly),
        expected_digest,
        fencing_epoch,
        std::time::Duration::from_secs(60),
    );

    let adapter = smoooth::adapters::DockerAdapter::new();

    // 1. Calling adapter with mismatched digest must fail
    let tampered_digest = [99u8; 32];
    let res_digest = adapter
        .authorized_destroy(
            &permit,
            &target_urn,
            tampered_digest,
            DestructionScope::ResourceOnly,
            fencing_epoch,
        )
        .await;
    assert!(res_digest.is_err());
    if let Err(SmooothError::Security(msg)) = res_digest {
        assert!(msg.contains("payload digest mismatch"));
    } else {
        panic!("Expected digest mismatch security error");
    }

    // 2. Calling adapter requesting persistent data deletion with ResourceOnly permit must fail
    let res_scope = adapter
        .authorized_destroy(
            &permit,
            &target_urn,
            expected_digest,
            DestructionScope::ResourceAndPersistentData,
            fencing_epoch,
        )
        .await;
    assert!(res_scope.is_err());
    if let Err(SmooothError::Security(msg)) = res_scope {
        assert!(msg.contains(
            "Permit authorizes ResourceOnly destruction; cannot destroy persistent data"
        ));
    } else {
        panic!("Expected destruction scope violation security error");
    }

    // 3. Calling adapter with mismatched target URN must fail
    let different_urn =
        ResourceUrn::parse("smoooth://workspaces/production/databases/postgres").unwrap();
    let res_urn = adapter
        .authorized_destroy(
            &permit,
            &different_urn,
            expected_digest,
            DestructionScope::ResourceOnly,
            fencing_epoch,
        )
        .await;
    assert!(res_urn.is_err());
    if let Err(SmooothError::Security(msg)) = res_urn {
        assert!(msg.contains("Target URN mismatch"));
    } else {
        panic!("Expected URN mismatch security error");
    }
}

#[tokio::test]
async fn test_adversarial_cross_workspace_capability_rejection() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (agent_sk, agent_vk) = make_keypair();
    let storage = LocalStorage::new_in_memory();
    let engine = AuthorizationEngine::new(issuer_vk);

    // Staging capability
    let staging_urn = ResourceUrn::parse("smoooth://workspaces/staging/workloads/web-api").unwrap();
    let root_payload = CapabilityPayload {
        id: Uuid::new_v4(),
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(agent_vk.as_bytes()),
        target_urn: staging_urn,
        actions: vec![Action::RestartWorkload],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: Utc::now(),
        not_before: Utc::now() - chrono::Duration::seconds(5),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    let chain = CapabilityChain::new_root(root_payload.clone(), &issuer_sk).unwrap();

    // Adversary presents staging capability against production resource
    let prod_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
    let method = "POST";
    let uri = "/api/workloads/production/web-api/restart";
    let body = b"{}";
    let pop = ProofOfPossession::generate(
        &agent_sk,
        root_payload.id,
        method,
        uri,
        body,
        "nonce_cross_ws".into(),
    )
    .unwrap();

    let res = engine
        .authorize_capability_request(
            &storage,
            &chain,
            &pop,
            method,
            uri,
            body,
            &prod_urn,
            &Action::RestartWorkload,
        )
        .await;

    assert!(
        res.is_err(),
        "Cross-workspace capability execution must be rejected"
    );
    if let Err(SmooothError::Security(msg)) = res {
        assert!(msg.contains("does not encompass target resource"));
    } else {
        panic!("Expected SmooothError::Security");
    }
}

#[tokio::test]
async fn test_adversarial_dual_mode_legacy_scope_synthesis() {
    let storage = LocalStorage::new_in_memory();
    let orchestrator = smoooth::engine::Orchestrator::new(Arc::new(storage));

    let prod_urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();

    // 1. Agent with Deploy scope
    let deploy_agent = AgentIdentity {
        agent_id: "ag_deploy_1".into(),
        name: "Deployer".into(),
        token_hash: "hash".into(),
        token_prefix: "pref".into(),
        scopes: vec![AgentScope::Deploy],
        workspace_id: "production".into(),
        created_at: Utc::now(),
        expires_at: None,
        revoked: false,
        revoked_at: None,
        last_used_at: None,
    };

    // Synthesizing permit for RestartWorkload succeeds
    let restart_res = orchestrator
        .synthesize_legacy_permit(&deploy_agent, &prod_urn, Action::RestartWorkload, [0u8; 32])
        .await;
    assert!(restart_res.is_ok());

    // Synthesizing permit for Destroy fails closed (Deploy scope cannot destroy!)
    let destroy_res = orchestrator
        .synthesize_legacy_permit(
            &deploy_agent,
            &prod_urn,
            Action::Destroy(DestructionScope::ResourceOnly),
            [0u8; 32],
        )
        .await;
    assert!(
        destroy_res.is_err(),
        "Deploy scope cannot synthesize Destroy permit"
    );

    // Synthesizing permit for ReadSecret fails closed (Deploy scope cannot read secrets!)
    let secret_res = orchestrator
        .synthesize_legacy_permit(
            &deploy_agent,
            &prod_urn,
            Action::ReadSecret {
                key_name: "PASS".into(),
                purpose: "dump".into(),
            },
            [0u8; 32],
        )
        .await;
    assert!(
        secret_res.is_err(),
        "Deploy scope cannot synthesize ReadSecret permit"
    );

    // Attempting cross-workspace synthesis fails closed!
    let staging_urn = ResourceUrn::parse("smoooth://workspaces/staging/workloads/web-api").unwrap();
    let cross_ws_res = orchestrator
        .synthesize_legacy_permit(
            &deploy_agent,
            &staging_urn,
            Action::RestartWorkload,
            [0u8; 32],
        )
        .await;
    assert!(
        cross_ws_res.is_err(),
        "Cross-workspace synthesis must fail closed"
    );
}

#[tokio::test]
async fn test_adversarial_durable_restart_survives_process_termination() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "smoooth_durable_restart_test_{}.json",
        Uuid::new_v4()
    ));

    let cap_id = Uuid::new_v4();
    let nonce = "nonce_must_survive_restart_987".to_string();
    let now = Utc::now();
    let exp_revocation = now + chrono::Duration::days(7);
    let exp_nonce = now + chrono::Duration::hours(2);

    // 1. Initial process: create storage, write records, verify persistence
    {
        let storage = LocalStorage::new_file_backed(&temp_file);
        storage
            .record_revocation(&cap_id, "Host compromise containment", exp_revocation)
            .await
            .expect("record_revocation should succeed");
        storage
            .record_used_nonce(&nonce, exp_nonce)
            .await
            .expect("record_used_nonce should succeed");
        let new_epoch = storage
            .increment_fencing_epoch("production")
            .await
            .expect("increment_fencing_epoch should succeed");
        assert_eq!(new_epoch, 2);
    } // storage dropped here (process termination simulation)

    // Ensure state file exists on disk
    assert!(
        temp_file.exists(),
        "State snapshot file must exist after drop"
    );

    // 2. Restarted process: load fresh LocalStorage from the persisted file
    {
        let restarted_storage = LocalStorage::new_file_backed(&temp_file);

        // Verify revocation survived restart
        let is_revoked = restarted_storage
            .is_revoked(&cap_id)
            .await
            .expect("is_revoked should succeed");
        assert!(
            is_revoked,
            "Revocation record must survive full process restart"
        );

        // Verify nonce survived restart
        let is_nonce_used = restarted_storage
            .is_nonce_used(&nonce)
            .await
            .expect("is_nonce_used should succeed");
        assert!(
            is_nonce_used,
            "Nonce record must survive full process restart"
        );

        // Verify unknown nonce is false
        let unused_nonce = restarted_storage
            .is_nonce_used("fresh_unseen_nonce")
            .await
            .expect("check unused nonce");
        assert!(!unused_nonce, "Unused nonce must return false");

        // Verify fencing epoch survived restart
        let epoch = restarted_storage
            .get_fencing_epoch("production")
            .await
            .expect("get_fencing_epoch should succeed");
        assert_eq!(epoch, 2, "Fencing epoch must survive full process restart");
    }

    // Clean up temporary files
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(temp_file.with_extension("bak"));
    let _ = std::fs::remove_file(temp_file.with_extension("tmp"));
}

#[tokio::test]
async fn test_adversarial_grandchild_revocation_at_intermediate_tier() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (root_sk, root_vk) = make_keypair();
    let (child_sk, child_vk) = make_keypair();
    let (grandchild_sk, grandchild_vk) = make_keypair();
    let storage = LocalStorage::new_in_memory();
    let engine = AuthorizationEngine::new(issuer_vk);

    let urn = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let grandchild_id = Uuid::new_v4();
    let now = Utc::now();

    // 1. Root capability
    let root_payload = CapabilityPayload {
        id: root_id,
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(root_vk.as_bytes()),
        target_urn: urn.clone(),
        actions: vec![
            Action::InspectMetadata,
            Action::ReadLogs {
                tail_limit: Some(100),
            },
        ],
        caveats: vec![],
        can_delegate: true,
        remaining_depth: 3,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::hours(2),
    };
    let root_chain = CapabilityChain::new_root(root_payload, &issuer_sk).unwrap();

    // 2. Child capability
    let child_payload = CapabilityPayload {
        id: child_id,
        parent_id: Some(root_id),
        issuer: hex::encode(root_vk.as_bytes()),
        holder_public_key: hex::encode(child_vk.as_bytes()),
        target_urn: urn.clone(),
        actions: vec![Action::ReadLogs {
            tail_limit: Some(50),
        }],
        caveats: vec![],
        can_delegate: true,
        remaining_depth: 2,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::hours(1),
    };
    let child_chain = root_chain.delegate(child_payload, &root_sk).unwrap();

    // 3. Grandchild capability
    let grandchild_payload = CapabilityPayload {
        id: grandchild_id,
        parent_id: Some(child_id),
        issuer: hex::encode(child_vk.as_bytes()),
        holder_public_key: hex::encode(grandchild_vk.as_bytes()),
        target_urn: urn.clone(),
        actions: vec![Action::ReadLogs {
            tail_limit: Some(25),
        }],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::minutes(30),
    };
    let grandchild_chain = child_chain.delegate(grandchild_payload, &child_sk).unwrap();
    assert_eq!(grandchild_chain.links.len(), 3);

    // 4. Revoking the intermediate Child capability
    storage
        .record_revocation(
            &child_id,
            "Intermediate tier compromised",
            now + chrono::Duration::hours(1),
        )
        .await
        .unwrap();

    // Grandchild attempts execution: must fail because its parent child_id is in the revocation registry
    let method = "GET";
    let uri = "/api/workloads/production/web-api/logs";
    let body = b"";
    let pop_grandchild = ProofOfPossession::generate(
        &grandchild_sk,
        grandchild_id,
        method,
        uri,
        body,
        "nonce_gc_test".into(),
    )
    .unwrap();

    let res_grandchild = engine
        .authorize_capability_request(
            &storage,
            &grandchild_chain,
            &pop_grandchild,
            method,
            uri,
            body,
            &urn,
            &Action::ReadLogs {
                tail_limit: Some(10),
            },
        )
        .await;

    assert!(
        res_grandchild.is_err(),
        "Grandchild must fail when intermediate parent is revoked"
    );
    if let Err(SmooothError::Security(msg)) = res_grandchild {
        assert!(msg.contains("has been revoked"));
    } else {
        panic!("Expected SmooothError::Security");
    }

    // But root capability itself was not revoked; a different child branch remains valid
    let sibling_id = Uuid::new_v4();
    let (sibling_sk, sibling_vk) = make_keypair();
    let sibling_payload = CapabilityPayload {
        id: sibling_id,
        parent_id: Some(root_id),
        issuer: hex::encode(root_vk.as_bytes()),
        holder_public_key: hex::encode(sibling_vk.as_bytes()),
        target_urn: urn.clone(),
        actions: vec![Action::InspectMetadata],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: now,
        not_before: now,
        expires_at: now + chrono::Duration::hours(1),
    };
    let sibling_chain = root_chain.delegate(sibling_payload, &root_sk).unwrap();

    let pop_sibling = ProofOfPossession::generate(
        &sibling_sk,
        sibling_id,
        method,
        uri,
        body,
        "nonce_sibling_test".into(),
    )
    .unwrap();

    let res_sibling = engine
        .authorize_capability_request(
            &storage,
            &sibling_chain,
            &pop_sibling,
            method,
            uri,
            body,
            &urn,
            &Action::InspectMetadata,
        )
        .await;

    assert!(
        res_sibling.is_ok(),
        "Sibling branch under root must remain valid"
    );
}
