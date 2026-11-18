use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Barrier;
use uuid::Uuid;

use smoooth::models::{Action, ResourceUrn};
use smoooth::security::{
    AuthorizationEngine, CapabilityChain, CapabilityPayload, ProofOfPossession,
};
use smoooth::storage::LocalStorage;

fn make_keypair() -> (SigningKey, VerifyingKey) {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("getrandom failure");
    let signing = SigningKey::from_bytes(&seed);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

#[tokio::test]
async fn test_concurrency_authorization_throughput_and_nonce_tracking() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (holder_sk, holder_vk) = make_keypair();
    let storage = Arc::new(LocalStorage::new_in_memory());
    let engine = Arc::new(AuthorizationEngine::new(issuer_vk));

    let target_urn =
        ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();

    let root_payload = CapabilityPayload {
        id: Uuid::new_v4(),
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(holder_vk.as_bytes()),
        target_urn: target_urn.clone(),
        actions: vec![Action::InspectMetadata],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: Utc::now(),
        not_before: Utc::now() - chrono::Duration::seconds(5),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    let chain = Arc::new(CapabilityChain::new_root(root_payload, &issuer_sk).unwrap());
    let holder_sk = Arc::new(holder_sk);

    let num_tasks = 25;
    let requests_per_task = 20;
    let total_requests = num_tasks * requests_per_task;

    let start_barrier = Arc::new(Barrier::new(num_tasks));
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for task_idx in 0..num_tasks {
        let storage_clone = storage.clone();
        let engine_clone = engine.clone();
        let chain_clone = chain.clone();
        let holder_sk_clone = holder_sk.clone();
        let target_urn_clone = target_urn.clone();
        let barrier = start_barrier.clone();

        let handle = tokio::spawn(async move {
            barrier.wait().await;
            let mut successful = 0;

            for req_idx in 0..requests_per_task {
                let nonce = format!("nonce_task_{}_{}", task_idx, req_idx);
                let method = "GET";
                let uri = "/api/v1/workloads/production/web-api";
                let body = b"";

                let pop = ProofOfPossession::generate(
                    &holder_sk_clone,
                    chain_clone.leaf().payload.id,
                    method,
                    uri,
                    body,
                    nonce,
                )
                .expect("PoP generation must succeed");

                let permit = engine_clone
                    .authorize_capability_request(
                        storage_clone.as_ref(),
                        &chain_clone,
                        &pop,
                        method,
                        uri,
                        body,
                        &target_urn_clone,
                        &Action::InspectMetadata,
                    )
                    .await;

                if permit.is_ok() {
                    successful += 1;
                }
            }
            successful
        });
        handles.push(handle);
    }

    let mut total_successful = 0;
    for handle in handles {
        total_successful += handle.await.unwrap();
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = (total_requests as f64) / elapsed.as_secs_f64();

    println!("\n=== High-Concurrency Capability Authorization Benchmark ===");
    println!(
        "• Total Authorized Requests : {} / {}",
        total_successful, total_requests
    );
    println!("• Concurrent Tasks          : {}", num_tasks);
    println!("• Total Time Elapsed        : {:?}", elapsed);
    println!("• Throughput                : {:.2} req/sec", ops_per_sec);
    println!("===========================================================\n");

    assert_eq!(total_successful, total_requests);
}

#[tokio::test]
async fn test_adversarial_concurrent_race_condition_replay_defense() {
    let (issuer_sk, issuer_vk) = make_keypair();
    let (holder_sk, holder_vk) = make_keypair();
    let storage = Arc::new(LocalStorage::new_in_memory());
    let engine = Arc::new(AuthorizationEngine::new(issuer_vk));

    let target_urn =
        ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();

    let root_payload = CapabilityPayload {
        id: Uuid::new_v4(),
        parent_id: None,
        issuer: "smoooth:control-plane".into(),
        holder_public_key: hex::encode(holder_vk.as_bytes()),
        target_urn: target_urn.clone(),
        actions: vec![Action::InspectMetadata],
        caveats: vec![],
        can_delegate: false,
        remaining_depth: 1,
        issued_at: Utc::now(),
        not_before: Utc::now() - chrono::Duration::seconds(5),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    let chain = Arc::new(CapabilityChain::new_root(root_payload, &issuer_sk).unwrap());

    // Generate ONE single Proof of Possession with a fixed nonce
    let shared_nonce = "shared_replayed_nonce_xyz".to_string();
    let method = "GET";
    let uri = "/api/v1/workloads/production/web-api";
    let body = b"";

    let pop = Arc::new(
        ProofOfPossession::generate(
            &holder_sk,
            chain.leaf().payload.id,
            method,
            uri,
            body,
            shared_nonce,
        )
        .unwrap(),
    );

    let num_attackers = 20;
    let barrier = Arc::new(Barrier::new(num_attackers));
    let mut handles = Vec::new();

    // 20 concurrent tasks attack the endpoint simultaneously with the exact same nonce
    for _ in 0..num_attackers {
        let storage_clone = storage.clone();
        let engine_clone = engine.clone();
        let chain_clone = chain.clone();
        let pop_clone = pop.clone();
        let target_urn_clone = target_urn.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await; // Synchronize exact simultaneous release
            engine_clone
                .authorize_capability_request(
                    storage_clone.as_ref(),
                    &chain_clone,
                    &pop_clone,
                    method,
                    uri,
                    body,
                    &target_urn_clone,
                    &Action::InspectMetadata,
                )
                .await
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    let mut rejected_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(smoooth::error::SmooothError::Security(msg)) => {
                assert!(msg.contains("replay attack rejected"));
                rejected_count += 1;
            }
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }

    println!("\n=== Concurrent Race Condition Nonce Replay Defense ===");
    println!("• Simultaneous Attackers : {}", num_attackers);
    println!(
        "• Succeeded Requests     : {} (must be exactly 1)",
        success_count
    );
    println!(
        "• Replay Rejected        : {} (must be exactly 19)",
        rejected_count
    );
    println!("======================================================\n");

    assert_eq!(success_count, 1, "Exactly ONE request must succeed");
    assert_eq!(
        rejected_count, 19,
        "All 19 concurrent duplicates must be rejected"
    );
}
