//! The ledger's behavioural spec, as sentences (Part A1).
//!
//! The chain is append-only and hash-chained per tenant. One entry hash
//! function, domain-separated (`tessera-ledger/1` + tenant), big-endian
//! fixed-width encoding — the same function the Python reference implements
//! byte-for-byte. Conformance vectors pin the actual digests; these tests
//! pin the behaviour.
//!
//! Written before the implementation: the first run fails to compile
//! because the API does not exist — that is the red state.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tessera_ids::TenantId;
use tessera_ledger::{AppendOutcome, Chain, Entry, LedgerError, Record, entry_hash};
fn tenant(s: &str) -> TenantId {
    TenantId::new(s).expect("valid tenant id")
}

fn append(chain: &mut Chain, e: Entry) -> Record {
    chain.append(e).expect("append succeeds").record().clone()
}

fn entry(height: u64, payload: &[u8]) -> Entry {
    Entry {
        height,
        valid_ms: 1_700_000_000_000,
        system_ms: 1_700_000_000_050,
        payload: payload.to_vec(),
    }
}

#[test]
fn genesis_starts_from_the_zero_hash() {
    let mut chain = Chain::new(tenant("acme"));
    let r = append(&mut chain, entry(0, b"first"));
    assert_eq!(r.entry.height, 0);
    assert_eq!(r.prev, [0u8; 32], "genesis prev must be the zero hash");
}

#[test]
fn appending_advances_height_and_chains_prev() {
    let mut chain = Chain::new(tenant("acme"));
    let r0 = append(&mut chain, entry(0, b"a"));
    let r1 = append(&mut chain, entry(1, b"b"));
    assert_eq!(r1.prev, r0.hash, "record 1 must chain to record 0");
    assert_eq!(r1.entry.height, 1);
}

#[test]
fn entry_hash_changes_when_any_encoded_field_changes() {
    let t = tenant("acme");
    let base = entry_hash(&t, &[7u8; 32], &entry(3, b"payload"));
    // different tenant
    assert_ne!(
        base,
        entry_hash(&tenant("other"), &[7u8; 32], &entry(3, b"payload"))
    );
    // different prev
    assert_ne!(base, entry_hash(&t, &[8u8; 32], &entry(3, b"payload")));
    // different height
    assert_ne!(base, entry_hash(&t, &[7u8; 32], &entry(4, b"payload")));
    // different valid time
    let mut e = entry(3, b"payload");
    e.valid_ms += 1;
    assert_ne!(base, entry_hash(&t, &[7u8; 32], &e));
    // different system time
    let mut e = entry(3, b"payload");
    e.system_ms += 1;
    assert_ne!(base, entry_hash(&t, &[7u8; 32], &e));
    // different payload
    assert_ne!(base, entry_hash(&t, &[7u8; 32], &entry(3, b"payloaD")));
    // deterministic: same inputs, same hash
    assert_eq!(base, entry_hash(&t, &[7u8; 32], &entry(3, b"payload")));
}

#[test]
fn replaying_the_same_entry_has_one_effect() {
    // A7: every retryable write path is idempotent — apply it twice,
    // assert one effect.
    let mut chain = Chain::new(tenant("acme"));
    let first = append(&mut chain, entry(0, b"order#1"));
    let replay = chain
        .append(entry(0, b"order#1"))
        .expect("replay must not error");
    match replay {
        AppendOutcome::AlreadyPresent(r) => {
            assert_eq!(r.hash, first.hash, "replay returns the existing record");
        }
        AppendOutcome::Appended(_) => {
            panic!("replaying an identical entry must not append a second record");
        }
    }
    assert_eq!(chain.records().len(), 1, "one logical entry, one record");
}

#[test]
fn a_conflicting_entry_at_the_same_height_is_rejected() {
    let mut chain = Chain::new(tenant("acme"));
    chain.append(entry(0, b"original")).expect("append");
    let err = chain
        .append(entry(0, b"forged"))
        .expect_err("a different entry at height 0 is a conflict");
    assert!(matches!(err, LedgerError::HeightConflict { .. }));
}

#[test]
fn a_height_gap_is_rejected() {
    let mut chain = Chain::new(tenant("acme"));
    let err = chain
        .append(entry(1, b"skips genesis"))
        .expect_err("gap between genesis and this entry");
    assert!(matches!(err, LedgerError::HeightConflict { .. }));
}

#[test]
fn verify_accepts_an_untampered_chain() {
    let mut chain = Chain::new(tenant("acme"));
    for i in 0..5u64 {
        chain
            .append(entry(i, format!("event-{i}").as_bytes()))
            .expect("append");
    }
    assert_eq!(tessera_ledger::verify(chain.records()), Ok(()));
}

#[test]
fn verify_detects_a_payload_tamper_at_the_exact_height() {
    let mut chain = Chain::new(tenant("acme"));
    for i in 0..3u64 {
        chain
            .append(entry(i, format!("event-{i}").as_bytes()))
            .expect("append");
    }
    let mut records = chain.records().to_vec();
    records[1].entry.payload = b"tampered".to_vec();
    assert_eq!(tessera_ledger::verify(&records), Err(1));
}

#[test]
fn verify_detects_a_broken_link_at_the_exact_height() {
    let mut chain = Chain::new(tenant("acme"));
    for i in 0..3u64 {
        chain
            .append(entry(i, format!("event-{i}").as_bytes()))
            .expect("append");
    }
    let mut records = chain.records().to_vec();
    records[2].prev = [9u8; 32];
    assert_eq!(tessera_ledger::verify(&records), Err(2));
}

#[test]
fn verify_detects_a_forged_hash_at_the_exact_height() {
    let mut chain = Chain::new(tenant("acme"));
    for i in 0..3u64 {
        chain
            .append(entry(i, format!("event-{i}").as_bytes()))
            .expect("append");
    }
    let mut records = chain.records().to_vec();
    records[0].hash[0] ^= 0xFF;
    assert_eq!(tessera_ledger::verify(&records), Err(0));
}

#[test]
fn verify_detects_a_wrong_height_field() {
    let mut chain = Chain::new(tenant("acme"));
    for i in 0..3u64 {
        chain
            .append(entry(i, format!("event-{i}").as_bytes()))
            .expect("append");
    }
    let mut records = chain.records().to_vec();
    records[1].entry.height = 99;
    assert_eq!(tessera_ledger::verify(&records), Err(1));
}

#[test]
fn cross_tenant_chains_do_not_share_hashes() {
    // Identical event sequences under two tenants must not produce
    // identical record hashes — tenant separation is in the hash input,
    // so one tenant's chain cannot be correlated with another's.
    let mut a = Chain::new(tenant("acme"));
    let mut b = Chain::new(tenant("otherco"));
    let ra = a
        .append(entry(0, b"same"))
        .expect("append")
        .record()
        .clone();
    let rb = b
        .append(entry(0, b"same"))
        .expect("append")
        .record()
        .clone();
    assert_ne!(ra.hash, rb.hash);
}
