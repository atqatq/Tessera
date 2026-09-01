//! Property tests for the ledger invariants (Part A2 makes these mandatory):
//! append-only, hash chain unbroken. Every single-field tamper in a chain
//! built from arbitrary payloads must be detected at the exact height where
//! it happened — no earlier, no later.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use tessera_ids::TenantId;
use tessera_ledger::{Chain, Entry, Record, verify};

fn tenant(s: &str) -> TenantId {
    TenantId::new(s).expect("valid tenant id")
}

fn payload_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(proptest::num::u8::ANY, 0..64)
}

fn build_chain(tenant_name: &str, payloads: &[Vec<u8>]) -> Vec<Record> {
    let mut chain = Chain::new(tenant(tenant_name));
    for (height, payload) in payloads.iter().enumerate() {
        let e = Entry {
            height: height as u64,
            valid_ms: 1_000 + height as u64,
            system_ms: 2_000 + height as u64,
            payload: payload.clone(),
        };
        chain.append(e).expect("sequential appends always succeed");
    }
    chain.records().to_vec()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Appending arbitrary payloads keeps the chain verifiable.
    #[test]
    fn arbitrary_chains_verify(payloads in prop::collection::vec(payload_strategy(), 1..8)) {
        let records = build_chain("acme", &payloads);
        prop_assert_eq!(verify(&records), Ok(()));
    }

    /// Flipping any byte of any payload is detected at exactly that height.
    #[test]
    fn payload_tamper_is_detected_at_the_exact_height(
        payloads in prop::collection::vec(payload_strategy(), 1..8),
        index in 0usize..8,
        byte_index in 0usize..64,
        flip in proptest::num::u8::ANY,
    ) {
        prop_assume!(index < payloads.len());
        let records = build_chain("acme", &payloads);
        let mut tampered = records.clone();
        let payload = &mut tampered[index].entry.payload;
        if payload.is_empty() {
            payload.push(0u8);
            // appending a byte changes the length encoding too; still caught
        } else {
            let i = byte_index % payload.len();
            payload[i] ^= 0x01 | (flip & 0x01);
        }
        prop_assert_eq!(verify(&tampered), Err(index as u64));
    }

    /// Flipping any byte of any stored hash is detected at exactly that
    /// height (the self-check fails before the link check downstream).
    #[test]
    fn hash_tamper_is_detected_at_the_exact_height(
        payloads in prop::collection::vec(payload_strategy(), 1..8),
        index in 0usize..8,
        byte_index in 0usize..32,
        flip in proptest::num::u8::ANY,
    ) {
        prop_assume!(index < payloads.len());
        let records = build_chain("acme", &payloads);
        let mut tampered = records.clone();
        tampered[index].hash[byte_index % 32] ^= 0x01 | (flip & 0x01);
        prop_assert_eq!(verify(&tampered), Err(index as u64));
    }

    /// Flipping any byte of any stored prev-link is detected at exactly
    /// that height.
    #[test]
    fn prev_tamper_is_detected_at_the_exact_height(
        payloads in prop::collection::vec(payload_strategy(), 1..8),
        index in 0usize..8,
        byte_index in 0usize..32,
        flip in proptest::num::u8::ANY,
    ) {
        prop_assume!(index < payloads.len());
        let records = build_chain("acme", &payloads);
        let mut tampered = records.clone();
        tampered[index].prev[byte_index % 32] ^= 0x01 | (flip & 0x01);
        prop_assert_eq!(verify(&tampered), Err(index as u64));
    }

    /// Replaying every entry of a chain in order appends nothing new.
    #[test]
    fn full_replay_has_one_effect_per_entry(
        payloads in prop::collection::vec(payload_strategy(), 1..8),
    ) {
        let mut chain = Chain::new(tenant("acme"));
        for (height, payload) in payloads.iter().enumerate() {
            let e = Entry {
                height: height as u64,
                valid_ms: 1_000 + height as u64,
                system_ms: 2_000 + height as u64,
                payload: payload.clone(),
            };
            chain.append(e.clone()).expect("first append");
            let outcome = chain.append(e).expect("replay must not error");
            prop_assert!(
                matches!(outcome, tessera_ledger::AppendOutcome::AlreadyPresent(_)),
                "replay at height {height} must be a no-op"
            );
        }
        prop_assert_eq!(chain.records().len(), payloads.len());
    }
}
