# Roadmap

Tessera is **docs-first** until the design freeze: specifications, ADRs, the
topology diagram, and governance land now; code directories fill in after.
This file tracks what lands when, and which new modules are on deck.

## Milestones

| Milestone | Scope | Status |
|---|---|---|
| **M0 — architecture seed** | topology diagram (14 modules), ARCHITECTURE/MODULES/ADAPTIVE_SPINE/AGENT_RUNTIME/FEDERATION_AND_EGRESS/IOT docs, Apache-2.0 + naming license, governance files | **shipped** |
| **M1 — executable spec** | `schemas/` (manifest, sharing-contract, signals) + `reference/python` executable spec with conformance vectors | next |
| **M2 — kernel core** | `kernel/` rust workspace: `kernel.origin`, `kernel.access`, `kernel.ledger`, `kernel.master_data`, `kernel.events` | planned |
| **M3 — module runtime** | `kernel.plugin_host`, first three modules (`pln`, `ord`, `inv`), module template + manifest checker | planned |
| **M4 — agents** | `kernel.ai_core` leader, built-in module agents, `kernel.agents` (MCP + REST) sandbox | planned |
| **M5 — grid** | `kernel.grid`, `kernel.notary`, sharing-contract engine, benchmark gates | planned |
| **M6 — remaining modules** | `src`, `trf`, `crm`, `ful`, `ret`, `srm`, `ctr`, `fin`, `tsk`, `prj`, `net` | planned |

## Next module candidates

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
  configuration and extension modules.
- No module-to-module direct calls, ever; the kernel brokers everything.
- No agent, built-in or user-defined, ever holds ORIGIN.
