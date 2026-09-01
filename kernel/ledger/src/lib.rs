//! Per-tenant append-only hash chains for the Tessera kernel.
//!
//! The ledger is the system of record for *what a value was*. Every record
//! commits to its tenant, its height, its bitemporal timestamps, and its
//! payload through one SHA-256 hash function:
//!
//! ```text
//! entry_hash = SHA-256("tessera-ledger/1" || tenant || 0x00 || prev
//!                      || u64be(height) || u64be(valid_ms) || u64be(system_ms)
//!                      || u32be(payload_len) || payload)
//! ```
//!
//! The domain string and the tenant are inside the hash input, so records
//! from different domains or tenants can never be spliced together, and
//! identical event sequences under different tenants are not linkable.
//! The zero hash anchors genesis.
//!
//! # Cryptography policy (Part 0.4 / 0.8)
//!
//! The digest comes from the `sha2` crate (RustCrypto, pure Rust) — never
//! from hand-written primitives. See `docs/adr/0009-cryptographic-primitives.md`
//! for the dependency decision and its honest audit-status notes. The
//! conformance vectors in `reference/python/vectors/` continuously verify
//! this implementation against Python's `hashlib` (OpenSSL-backed), so a
//! primitive regression cannot pass CI unnoticed.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

use sha2::{Digest, Sha256};
use tessera_ids::TenantId;

/// Domain separation prefix for every ledger hash.
pub const DOMAIN: &[u8] = b"tessera-ledger/1";

/// The hash that anchors genesis: the `prev` of height 0.
pub const GENESIS_PREV: [u8; 32] = [0u8; 32];

/// One ledger entry, as the caller intends to record it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Chain height: 0 for genesis, strictly +1 per record.
    pub height: u64,
    /// Bitemporal: when the fact was true in the real world.
    pub valid_ms: u64,
    /// Bitemporal: when the kernel learned it (caller-supplied; no wall
    /// clock lives in domain logic, Part A3).
    pub system_ms: u64,
    /// Opaque payload bytes. The ledger never interprets them.
    pub payload: Vec<u8>,
}

/// A committed entry plus its chain linkage. The tenant is part of the
/// record: every hash commits to it, and [`verify`] recomputes hashes with
/// the tenant the record names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The tenant this record belongs to (inside the hash input).
    pub tenant: TenantId,
    /// The committed entry.
    pub entry: Entry,
    /// The hash of the previous record (zero hash for genesis).
    pub prev: [u8; 32],
    /// This record's own hash, as committed.
    pub hash: [u8; 32],
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{} prev=", self.tenant, self.entry.height)?;
        for b in self.prev {
            write!(f, "{b:02x}")?;
        }
        write!(f, " hash=")?;
        for b in self.hash {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

/// Computes the domain-separated entry hash. This is the one function the
/// Python reference must reproduce byte-for-byte (vectors are the contract).
pub fn entry_hash(tenant: &TenantId, prev: &[u8; 32], entry: &Entry) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(DOMAIN);
    h.update(tenant.as_str().as_bytes());
    h.update([0x00]); // tenant boundary separator
    h.update(prev);
    h.update(entry.height.to_be_bytes());
    h.update(entry.valid_ms.to_be_bytes());
    h.update(entry.system_ms.to_be_bytes());
    h.update(
        u32::try_from(entry.payload.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    h.update(&entry.payload);
    h.finalize().into()
}

/// Why an append was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LedgerError {
    /// The entry's height does not continue the chain, and no identical
    /// record already occupies that height (which would have been an
    /// idempotent replay, not a conflict).
    #[error("height conflict: chain expects {expected}, entry claims {claimed}")]
    HeightConflict {
        /// The height the chain would accept next.
        expected: u64,
        /// The height the entry claimed.
        claimed: u64,
    },
}

/// Outcome of [`Chain::append`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppendOutcome {
    /// The entry was new; the chain grew by one record.
    Appended(Record),
    /// This exact entry (same height, same bytes, same hash) is already
    /// committed — the retryable write path applied once. Idempotent
    /// replay returns the existing record and changes nothing.
    AlreadyPresent(Record),
}

impl AppendOutcome {
    /// The committed record, whether newly appended or already present.
    pub fn record(&self) -> &Record {
        match self {
            AppendOutcome::Appended(r) | AppendOutcome::AlreadyPresent(r) => r,
        }
    }

    /// Whether this outcome grew the chain (`false` for idempotent replay).
    pub fn appended(&self) -> bool {
        matches!(self, AppendOutcome::Appended(_))
    }
}

/// An in-memory per-tenant chain. Storage engines wrap this; the invariants
/// live here.
#[derive(Debug, Clone)]
pub struct Chain {
    tenant: TenantId,
    records: Vec<Record>,
}

impl Chain {
    /// A fresh, empty chain for `tenant`.
    pub fn new(tenant: TenantId) -> Self {
        Self {
            tenant,
            records: Vec::new(),
        }
    }

    /// The tenant this chain belongs to.
    pub fn tenant(&self) -> &TenantId {
        &self.tenant
    }

    /// The committed records, in height order.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// The next height the chain would accept.
    pub fn next_height(&self) -> u64 {
        self.records.len() as u64
    }

    /// Appends an entry (idempotent on exact replay).
    ///
    /// - If `entry.height` continues the chain, the record is appended.
    /// - If an identical record (same height and bytes, therefore same
    ///   hash) already exists, [`AppendOutcome::AlreadyPresent`] is
    ///   returned and nothing changes — applying the operation twice has
    ///   one effect (Part A7).
    /// - Any other height is a [`LedgerError::HeightConflict`]; the chain
    ///   never rewrites history.
    pub fn append(&mut self, entry: Entry) -> Result<AppendOutcome, LedgerError> {
        let expected = self.next_height();
        if entry.height != expected {
            // Exact replay of an already-committed entry is idempotent: the
            // record at that height must hash identically from the same
            // prev-link and the same bytes.
            if let Some(existing) = self.records.get(entry.height as usize) {
                let replay_hash = entry_hash(&self.tenant, &existing.prev, &entry);
                if replay_hash == existing.hash {
                    return Ok(AppendOutcome::AlreadyPresent(existing.clone()));
                }
            }
            return Err(LedgerError::HeightConflict {
                expected,
                claimed: entry.height,
            });
        }
        let prev = self.records.last().map_or(GENESIS_PREV, |r| r.hash);
        let hash = entry_hash(&self.tenant, &prev, &entry);
        let record = Record {
            tenant: self.tenant.clone(),
            entry,
            prev,
            hash,
        };
        self.records.push(record.clone());
        Ok(AppendOutcome::Appended(record))
    }
}

/// Verifies a sequence of records from genesis: heights must be contiguous
/// from zero, links must chain, every record must name one tenant, and
/// every committed hash must recompute under that tenant. On failure,
/// returns the **first broken height** — pinned, so callers can quarantine
/// exactly where the history was tampered with.
///
/// A record claiming a different tenant than its neighbours fails at the
/// first such record: its hash cannot recompute under two tenants at once.
pub fn verify(records: &[Record]) -> Result<(), u64> {
    let mut prev = GENESIS_PREV;
    for (i, record) in records.iter().enumerate() {
        let height = i as u64;
        if record.entry.height != height
            || record.prev != prev
            || entry_hash(&record.tenant, &prev, &record.entry) != record.hash
        {
            return Err(height);
        }
        prev = record.hash;
    }
    Ok(())
}
