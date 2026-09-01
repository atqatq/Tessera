# Changelog

All notable changes to Tessera are documented here. Format: Keep a
Changelog; versioning: SemVer (enforced by a public-API diff check as
crates stabilise).

## [Unreleased]

### Added
- Engineering gates as workspace lints: `forbid(unsafe_code)`,
  `deny(missing_docs)`, input-path denial of `unwrap`/`expect`/`panic`/
  `todo`; pinned toolchain; MSRV 1.85; cargo-deny allowlist.
- `tessera-ids` — strongly-typed identifiers (TenantId, ModuleId,
  SubjectId, RoleId, EpochMs) with round-trip and corruption proptests;
  cross-kind assignment proven unrepresentable by a `compile_fail`
  doctest.
- `tessera-access` — pure five-layer permission engine: fourteen
  decision codes, deny-wins, default deny, injected clock. Seven
  property tests pin deny-wins, order independence, propose/write
  agreement, tier monotonicity, expiry fail-closed, unknown-column
  denial, and ORIGIN's L0 boundary.
- `tessera-ledger` — per-tenant append-only SHA-256 chains via the
  audited `sha2` crate (ADR 0009); idempotent replay; five tamper
  properties pinning detection at the exact height.
- Conformance machinery — stdlib-only Python reference, deterministic
  vector generator, and committed vectors (27 access, 7 ledger) that
  both implementations consume byte-identically.
- CI that checks the code that exists: fmt/clippy/test, MSRV job,
  platform matrix, coverage floor, DCO, gitleaks, cargo-deny,
  cargo-audit, conformance-freshness; weekly cargo-mutants on the
  permission engine and the ledger. Every action pinned to a full SHA.
- `make setup/test/lint/check/coverage/mutation/docs`; devcontainer.
- ADRs 0001–0009 and the RFC process (`docs/RFC_PROCESS.md`).
- `docs/TESTING.md` — the invariant → test map.

### Changed
- CI no longer skips rust/python jobs on file-presence gates: with code
  in the tree, the gates check the code.
- CONTRIBUTING documents the actual red-green-refactor cycle from this
  codebase, including the design flaw the properties caught.

### Removed
- Nothing yet in this cycle.

## [0.1.0] — architecture seed

### Added
- Canonical system diagram (development view) at
  `docs/diagrams/tessera-system-architecture.svg`.
- Architecture docs: ARCHITECTURE, MODULES, ADAPTIVE_SPINE,
  AGENT_RUNTIME, FEDERATION_AND_EGRESS, IOT.
- Governance: CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CI, issue & PR
  templates, ROADMAP.
- Apache-2.0 licence.
