# Testing — the invariant → test map

Part F.4 of the definition of done: every invariant stated in a doc has
a test that fails when violated. This file is the map. If you document
a new invariant and do not add a row here, the docs and the tests can
drift — and drifting docs are false claims.

| Invariant (source) | Fails when violated | Where |
|---|---|---|
| A `TenantId` is never assignable to a `ModuleId` (A4) | `compile_fail` doctest | `kernel/ids/src/lib.rs` |
| Identifiers: 1–64 chars, `a-z 0-9 .-_`, first char letter/digit | `accepts_documented_valid_shapes`, `rejects_documented_invalid_shapes` + round-trip & corruption proptests | `kernel/ids/tests/` |
| Deny always wins (ADR 0007) | `explicit_deny_beats_allow_on_the_same_column`; property `explicit_deny_always_wins` | `kernel/access/tests/` |
| Decisions are independent of rule/grant order (A3 determinism) | property `decision_is_order_independent` | `kernel/access/tests/properties.rs` |
| Proposals are judged by write rules; verdicts never diverge | `propose_is_judged_by_the_write_rules`; property `propose_and_write_agree_for_users` | `kernel/access/tests/` |
| Agent tiers are monotonic (observe ⊆ advise ⊆ act) | property `agent_tiers_are_monotonic` | `kernel/access/tests/properties.rs` |
| Grant expiry fails closed at the boundary instant | `an_expired_grant_is_denied_at_the_expiry_instant`; property `expired_grants_never_allow` | `kernel/access/tests/` |
| Unknown columns deny for everyone except ORIGIN | `unknown_column_is_denied_even_when_a_star_rule_allows`; property `unknown_columns_are_denied` | `kernel/access/tests/` |
| L0 module state gates everyone, including ORIGIN | `a_disabled_module_gates_everyone_including_origin`; property `origin_never_passes_a_disabled_module` | `kernel/access/tests/` |
| Cross-module writes do not exist as an operation (ADR 0002) | `peer_writes_do_not_exist_as_an_operation` | `kernel/access/tests/spec.rs` |
| Ledger: append-only, chain unbroken; any single-byte tamper is caught at the exact height (ADR 0003) | five tamper properties + spec tamper tests | `kernel/ledger/tests/` |
| Ledger: idempotent replay — apply twice, one effect (A7) | `replaying_the_same_entry_has_one_effect`; property `full_replay_has_one_effect_per_entry` | `kernel/ledger/tests/` |
| Ledger: cross-tenant chains are not linkable | `cross_tenant_chains_do_not_share_hashes` | `kernel/ledger/tests/spec.rs` |
| Rust and the Python reference agree byte-for-byte on every vector (ADR 0005/0008) | `rust_reproduces_every_access_vector`, `rust_reproduces_every_ledger_vector`, Python drift guards | both crates' `tests/vectors.rs`, `reference/python/tests/` |
| Vectors are current: regenerating produces no diff | `make vectors`, CI python job | `Makefile`, `.github/workflows/ci.yml` |

## Taxonomy (A2)

- **Unit** — pure logic, microseconds: `tests/spec.rs` in each crate.
- **Property** (`proptest`) — every all-inputs invariant above.
- **Conformance vectors** — the Python reference and Rust agree on
  committed data; vectors are the contract (ADR 0008).
- **Integration** (real Postgres, real bus) — lands with the stores
  (ROADMAP v0.x); mocked stores are not permitted for things we own.
- **Fuzzing** (`cargo-fuzz`) — every parser that sees untrusted bytes.
  None exists in the kernel yet (the engine consumes typed values, the
  ledger consumes opaque bytes it does not parse); fuzz targets are a
  tracked good-first-issue for the first parser (ingest, manifest).
- **Concurrency** (`loom`) — the event bus and ledger sequencing do not
  exist as concurrent code yet; loom lands with `kernel.events`.
- **Snapshot** — human-readable output (the recommendation explainer,
  E1) asserts on formatted output reviewed by hand.

## Determinism (A3)

No wall clock, no real randomness, no network, no hash-map ordering in
domain logic. Clocks are injected (`Env::now`, ledger timestamps);
proptest is seeded; collections in decision paths are `BTree`-ordered
or order-agnostic (a property asserts it). A flaky test is quarantined
the day it flakes and fixed that week — never re-run until green.
