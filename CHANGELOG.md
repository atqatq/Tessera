# Changelog

All notable changes to Tessera are documented here. Format: Keep a
Changelog; versioning: SemVer.

## [0.2.0] — executable spec (M1)

### Added
- Conformance vectors: language-neutral expression DSL cases under
  `conformance/`, run by both implementations — agreement by construction.
- Python reference implementation (`reference/python/scor_ref`): lexer,
  parser, evaluator, policy engine, manifest validator, dependency graph;
  168 tests including the vector harness.
- Rust kernel workspace (`rust/`): `scor-expr` (vector-locked DSL),
  `scor-manifest` (independence rule), `scor-policy` (deny-wins column
  ACL) — `#![forbid(unsafe_code)]`, clippy `unwrap/expect/panic/indexing`
  denied.
- Spoke manifest seeds (`spokes/ctr`, `spokes/srm`) with the checker gate
  (`tools/check_manifests.py`).
- `Makefile`: `make check` = fmt, clippy `-D warnings`, both suites,
  vectors, manifest gate, REUSE.
- `GOVERNANCE.md`: Eclipse Foundation as long-term home, DCO, lazy
  consensus, roles.
- REUSE compliance: SPDX headers, `LICENSES/Apache-2.0.txt`, `.reuse/dep5`.
- Platform specifications under `docs/specs/` (permission model,
  expression DSL, AI fabric, logging, KPIs, commercial terms, design
  system).

### Fixed
- CI workflow trigger: corrupted `branches:` glob is now `[main]`.
- Lexer hardening (python + rust): ASCII-only numbers and identifiers —
  unicode digits used to reach `Decimal()` and raise an uncaught
  `InvalidOperation`; both are pinned by new conformance vectors.
- Manifest regexes anchored with `\Z` so a trailing newline cannot sneak
  a manifest through validation.

## [0.1.0] — architecture seed

### Added
- Canonical topology diagram: 14 spokes (PLAN, SOURCE, TRANSFORM, ORDER,
  CRM, FULFILL, RETURN, INVENTORY, SUPPLIERS, CONTRACTS, FINANCE, TASKS,
  PROJECTS, CONNECTORS) around the AI-native hub, with the inter-company
  grid band (`docs/diagrams/`).
- Architecture docs: ARCHITECTURE, SPOKES, ADAPTIVE_SPINE, AGENT_RUNTIME,
  FEDERATION_AND_EGRESS, IOT.
- License: Apache-2.0 with the Tessera naming requirement appendix,
  plus NOTICE and this attribution guide (ATTRIBUTION.md).
- Governance: CONTRIBUTING (five rules + spoke checklist), CODE_OF_CONDUCT,
  SECURITY, CI (attribution check), issue & PR templates, ROADMAP.
