# ADR 0009 — Cryptographic primitives come from audited crates, never from this repository

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

The ledger hashes every record. Part 0 of the hardening brief is
unambiguous: *hashing and signatures are in scope, but write none of it;
audited crates only*, and *never write your own cryptographic
primitive — if you think you need one, stop and raise it*. The only
open question was which crate carries the SHA-256 implementation.

## Options considered

1. Hand-written pure-Rust SHA-256: no dependencies, no audit trail of
   any kind, and a permanent "is our hash right?" question that the
   conformance suite would answer only for the vectors we happened to
   think of. Rejected: this is precisely what Part 0 forbids.
2. `ring`: BoringSSL lineage, extremely widely deployed, professionally
   maintained — but it brings a C toolchain and assembly into the build,
   and its unsafe interior is invisible to `forbid(unsafe_code)`.
3. `sha2` (RustCrypto): pure Rust, no build-time C, no unsafe by
   default, maintained by the RustCrypto organisation, millions of
   downloads, and continuously cross-checked here against CPython's
   OpenSSL-backed `hashlib` through the conformance vectors. No
   standalone formal third-party audit — stated honestly rather than
   implied.

## Decision

Option 3 for hashing, today. Signature verification (planned) will use
`ring` or `aws-lc-rs`, where audit pedigree and side-channel hardening
matter more; key custody will live in a KMS or HSM, never in this
repository. The digest is confined to one function
(`tessera_ledger::entry_hash`), so a future swap is a one-file change
and the vectors are the contract that proves behaviour survived it.

## Consequences

- Every hash the kernel produces is cross-verified against an
  independent, OpenSSL-backed implementation on every CI run (ADR 0008).
- The audit-status caveat above is part of the decision, not a footnote
  to rediscover: if the project later needs a formally audited SHA-256
  lineage, the swap cost is one file plus a vector run.
- Any future primitive need that a crate cannot meet stops the project
  (Part 0.8) — the answer is a raised issue, never a hand-rolled cipher.
