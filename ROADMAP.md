# Roadmap

Tessera was docs-first; **M1 shipped the executable spec** — language-neutral
conformance vectors with two implementations (python reference, rust kernel)
locked to the same file. This file tracks what lands when, and which new
spokes are on deck.

## Milestones

| Milestone | Scope | Status |
|---|---|---|
| **M0 — architecture seed** | topology diagram (14 spokes), ARCHITECTURE/SPOKES/ADAPTIVE_SPINE/AGENT_RUNTIME/FEDERATION_AND_EGRESS/IOT docs, Apache-2.0 + naming license, governance files | **shipped** |
| **M1 — executable spec** | `conformance/` vectors, `schemas/`, `reference/python` + `rust/` (scor-expr, scor-manifest, scor-policy), spoke manifests + checker, `make check` gate | **shipped** |
| **M2 — hub core** | `hub/` rust workspace: `hub.origin`, `hub.access`, `hub.ledger`, `hub.master_data`, `hub.events` | planned |
| **M3 — spoke runtime** | `hub.plugin_host`, first three spokes (`pln`, `ord`, `inv`), spoke template + manifest checker | planned |
| **M4 — agents** | `hub.ai_core` leader, built-in spoke agents, `hub.agents` (MCP + REST) sandbox | planned |
| **M5 — grid** | `hub.grid`, `hub.notary`, sharing-contract engine, benchmark gates | planned |
| **M6 — remaining spokes** | `src`, `trf`, `crm`, `ful`, `ret`, `srm`, `ctr`, `fin`, `tsk`, `prj`, `net` | planned |

## Next spoke candidates

Voted per minor release. A candidate becomes a proposal issue, then a spec
PR (manifest, permission matrix, KPI pack, agent tier justification,
conformance vectors — see CONTRIBUTING).

1. **AFTERMARKET** — service parts planning, warranty claims, field service
   dispatch, install-base lifecycle. Distinct from RETURN: returns flow
   backward through the chain; aftermarket serves the product in the field
   for years after the sale.
2. **SUSTAIN** — emissions accounting (scope 1–3 where supply-chain visible),
   circularity loops, supplier ESG scoring and evidence packs. Increasingly
   a procurement gate; belongs beside SUPPLIERS and CONTRACTS, not inside
   them.
3. **WORKFORCE** — labor demand from the plan, skills/certifications, shift
   capacity, shop-floor assignment feeding TASKS and PROJECTS.

Longer backlog (proposed earlier, not yet voted): **COMPLIANCE** (trade &
customs, audit evidence), **ASSETS** (equipment/fleet/tooling lifecycle,
predictive maintenance), **PRODUCT** (NPI, BOM governance, packaging).

## Non-goals

- Tessera never forks the reference-model anchors — niches attach as
  configuration and extension spokes.
- No spoke-to-spoke direct calls, ever; the hub brokers everything.
- No agent, built-in or user-defined, ever holds ORIGIN.
