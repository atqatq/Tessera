# Contributing to Tessera

Thanks for helping build supply chain infrastructure that a global
enterprise and a two-person shop can both run. A few rules keep it trustworthy.

## The five rules

1. **Vector-first.** Behavior changes ship with conformance vectors under
   `reference/python/vectors/` — no vector, no merge. The executable
   reference *is* the spec's arbiter.
2. **Hub services are the only legal dependency.** A spoke's `requires`
   may name `hub.*` services and nothing else. Spoke-to-spoke calls do not
   exist; peer reads go through `hub.access` grants.
3. **Deny wins.** Every permission test fails closed. If your change makes
   any path default-allow, it is wrong.
4. **Append-only.** The ledger and master log never rewrite history.
   Corrections are new facts with provenance.
5. **Monotone + one accent.** UI follows the design tokens: zinc ramp,
   single accent `#6E96E8`. No decorative color.

## Workflow

```bash
git checkout -b feat/my-change
cargo fmt && cargo clippy --workspace -- -D warnings
cargo test  --workspace
pip install -e reference/python && pytest reference/python -q
```

- Conventional Commits (`feat:`, `fix:`, `spoke(pln):`, `docs:`).
- One logical change per PR; sign off commits (`git commit -s`) — DCO 1.1.
- New spokes: open a proposal issue first (see docs/SPOKES.md, "spoke contract"),
  include manifest, permission matrix, KPI set, agent tier justification,
  and conformance vectors.

## Adding a spoke — checklist

- [ ] `schemas/manifest.schema.json` validates
- [ ] requires only hub services
- [ ] permission matrix (L0-L3) documented, deny-wins tests
- [ ] 3+ industry-standard capability lines (MRP-grade depth, not CRUD)
- [ ] agent tier stated (observe/advise/act) + craft description
- [ ] meaningful IoT telemetry declared + ingest mappings
- [ ] egress candidates (which fields may ever leave, at what granularity)
- [ ] dashboard/KPI pack + spoke log events
- [ ] disableability: pause/stop/update tested, log freezes read-only

## Reporting bugs

Use the bug template; include vectors that reproduce. Security issues:
see SECURITY.md — do not open public issues.
