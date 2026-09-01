# Changelog

All notable changes to Tessera are documented here. Format: Keep a
Changelog; versioning: SemVer (enforced by a public-API diff check as
crates stabilise).

## [Unreleased]

### Added
- The one complete red-green-refactor cycle: `tessera-inv` —
  multi-echelon safety stock under staged service levels, vectors
  written first (12 cases incl. refusals and the pinned explainer),
  both implementations byte-identical on first contract run.
- The operator model (`docs/OPERATOR_MODEL.md`) + ADR-0010 (structure
  configurable, method not) + ADR-0011 (out-of-tree trust boundary) +
  ADR-0012 (ORIGIN delegation, specified with pending vectors).
- Module manifest schema v1 frozen + plugin API contract + module
  template + manifest conformance vectors.
- Collaboration machinery: failing-test bug template, ten staged
  good-first-issues, MAINTAINERS/ADOPTERS/RELEASES/SUPPORT, CoC with
  external escalation, labels as code, "Your first PR" cycle.
- Docs site (mdBook, Diátaxis) with why/why-not pages, og:image, and
  self-hosted brand fonts; brand mark + tokens as the single source
  of truth.
- CI: Dependabot, docs build + link check + Pages deployment; every
  action pinned to a peeled commit SHA.

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
- ROADMAP cut to v0.x = `inv` + the minimum kernel; the other thirteen
  modules moved to "design intent, not scheduled"; the v0.1 success
  criterion is a committed, deliberately failing end-to-end test.
- The federation story now rides existing standards: IDSA DSP for
  negotiation/transfer, ODRL for policy, GS1 EPCIS 2.0/CBV for
  boundary events, EDI/GS1 identifiers for ingest; notarisation
  demoted to an optional appendix (Grid EOL 2023, Sawtooth archived
  2024) with the no-notary guarantee pinned in the e2e criterion.
- Unearned claims swept: the 1,000,000 msg/sec bus number, the
  "fourteen modules ship in-tree" sentence, and every docker/pip
  command that could not run are gone or honestly downgraded to
  design intent.
- README is a landing page: hero with the mark, built-vs-specified
  status, live-only badges, v0.x architecture in light/dark with alt
  text, a repository map, and only CI-executed commands.

### Removed
- The "Tessera naming requirement" appendix from `LICENSE` and
  `ATTRIBUTION.md`: the licence is now plain Apache-2.0, and the name
  is protected by `TRADEMARK.md` instead of a licence condition
  (Apache-2.0 §4 does not permit conditions on the grant, and a
  bespoke appendix defeated SPDX classification and corporate legal
  review). The CI attribution grep went with it.

### Added
- `TRADEMARK.md`: nominative use permitted, forks rename, no implied
  endorsement — the Rust/Linux/Kubernetes/PostgreSQL posture.
- `LICENSING.md`: the patent position in plain language, DCO not CLA,
  third-party licence policy, corporate-CLA handling, export-control
  note.
- `GOVERNANCE.md`, `THIRD_PARTY_LICENSES.md`, `MAINTAINERS.md`,
  `ADOPTERS.md` (see below as they land).

### Changed
- `NOTICE` rewritten for plain Apache-2.0 with the trademark
  distinction stated.

## [0.1.0] — architecture seed

### Added
- Canonical system diagram (development view) at
  `docs/diagrams/tessera-system-architecture.svg`.
- Architecture docs: ARCHITECTURE, MODULES, ADAPTIVE_SPINE,
  AGENT_RUNTIME, FEDERATION_AND_EGRESS, IOT.
- Governance: CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CI, issue & PR
  templates, ROADMAP.
- Apache-2.0 licence.
