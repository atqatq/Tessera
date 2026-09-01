# Tessera

**A hub-and-spoke supply chain operating system, aligned with the SCOR
reference model.** One AI-native hub orchestrating fourteen pluggable
spokes — from retail to pharma to chemicals — with a permission engine
that assumes nobody is trusted, an append-only ledger for everything that
happens, and a grid that lets companies share select data without ever
sharing trust. One spine at any scale: the same system runs a global
enterprise and a two-person shop, with every capability scoped to the
tenant it serves.

![license](https://img.shields.io/badge/license-Apache--2.0%20%2B%20naming-blue)
![attribution](https://img.shields.io/badge/attribution-required-orange)
![rust](https://img.shields.io/badge/core-rust-dea584)
![python](https://img.shields.io/badge/reference-python-3670A0)
![PRs](https://img.shields.io/badge/PRs-welcome-6E96E8)

> **Attribution — use requires naming.** Tessera is open source under
> Apache-2.0 **plus one condition**: any product, deployment, or
> derivative that uses Tessera must visibly name it — a "Powered by
> **Tessera**" credit in the UI footer, about dialog, or product docs,
> with a link to
> [github.com/tessera-scm/tessera](https://github.com/tessera-scm/tessera).
> Commercial or non-commercial: same rule. See
> [LICENSE (Appendix)](LICENSE#appendix--tessera-naming-requirement),
> [NOTICE](NOTICE), and [ATTRIBUTION.md](ATTRIBUTION.md).

---

## The picture

The canonical topology diagram lives at
[`docs/diagrams/tessera-hub-spoke-architecture.svg`](docs/diagrams/tessera-hub-spoke-architecture.svg)
(a PNG sits alongside it).

- **The Hub** is the AI-native control plane and single source of truth.
  Spokes never talk to each other directly — every peer access is brokered,
  permission-checked, and ledger-stamped by the hub.
- **Spokes** are pluggable domain engines (PLAN, SOURCE, TRANSFORM, ORDER,
  CRM, FULFILL, RETURN, INVENTORY, SUPPLIERS, CONTRACTS, FINANCE, TASKS,
  PROJECTS, CONNECTORS). Each ships industry-standard depth, its own agent,
  its own dashboard/KPIs, multi-tenancy with local currency + USD reserve,
  native IoT telemetry, and can be paused, stopped, or updated independently.
- **The Grid** is the inter-company layer above it all: external hubs and
  spokes exchange *select* data through signed sharing contracts — consent-
  scoped, policy-gated, revocable, notarized. Writes never cross a boundary;
  changes travel as requests a human accepts.

## Why it exists

Supply chain software today forces a terrible trade: buy an enterprise suite
that assumes you have 10,000 SKUs and a systems-integration budget, or bolt
together spreadsheets and hope. Tessera refuses the trade. The same spine
runs a global manufacturer and a two-person workshop — the **adaptive spine**
morphs entities, flows, KPIs, and terminology to the tenant during setup, and
every capability **scopes to tenant size**. One codebase, one set of
guarantees, a proportionate surface for every tenant.

## Core concepts

### Hub services (the only names a hard `requires` may contain)

| Service | Responsibility |
|---|---|
| `hub.origin` | superuser identity above root — hardware key + threshold signature, no passwords, intent logged before effect |
| `hub.access` | five-layer permission engine (below) |
| `hub.master_data` | bitemporal master data, custom columns/types, expression DSL |
| `hub.ingest` | CSV / XLSX / API framework / IoT-MQTT edge, dead-letter quarantine |
| `hub.ledger` | what a value was — bitemporal, hash-chained per tenant |
| `hub.events` | message bus, 1,000,000 msg/sec duplex to every spoke |
| `hub.plugin_host` | spoke lifecycle: install, pause, stop, update, archive |
| `hub.tenancy` | tenants, accounts, roles registry (AUD/TNA/RSK + per-spoke roles) |
| `hub.ai_core` | leader agent (AIL), signal ranking, task/proposal routing |
| `hub.agents` | user-defined agent runtime — MCP + REST API, sandboxed, scoped |
| `hub.grid` | inter-company sharing: parties, relationships, contracts |
| `hub.notary` | anchor-never-store Merkle notarization (swappable providers) |

### The fourteen spokes

| Code | Spoke | Echelon | Agent tier | Standard depth highlights | Native IoT telemetry |
|---|---|---|---|---|---|
| `pln` | PLAN | core | advise | MRP · MPS · ML demand forecasting · supply design & planning · S&OP/S&OE | POS & shelf sensors, weather feeds |
| `src` | SOURCE | core | advise | strategic sourcing · RFx · PO lifecycle · 3-way match handoff | inbound GPS, gate scans, cold-chain probes |
| `trf` | TRANSFORM | core | act | MES · finite-capacity scheduling · BOM/where-used · SPC · OEE | line PLCs, OEE counters, vision QA |
| `ord` | ORDER | core | advise | OMS · ATP/CTP promising · order-to-cash · allocation | e-comm & POS event streams |
| `crm` | CRM | ext | advise | customer 360 · pipeline & quotes · service SLAs · churn/CLV models | connected-product telemetry, app events |
| `ful` | FULFILL | core | act | WMS · TMS · wave planning · route optimization · OTIF | fleet GPS, geo-fences, cold-chain probes |
| `ret` | RETURN | core | advise | RMA · reverse logistics · disposition · warranty & recovery | smart RMA kiosks, return-drop scans |
| `inv` | INVENTORY | ext | act | multi-echelon optimization · ABC/XYZ · cycle counts | RFID, smart shelves, drone counts |
| `srm` | SUPPLIERS | ext | advise | qualification · scorecards · OTIF/PPM · risk monitoring | supplier port telemetry, cert feeds |
| `ctr` | CONTRACTS | ext | observe | bitemporal terms · sole writer of `ctr.commercial_terms` · SLAs | usage meters, SLA probes, e-sign pads |
| `fin` | FINANCE | ext | advise | costing & landed cost · cash-to-cash · AP/AR · FX & tax engines | POS terminals, metered-usage events |
| `tsk` | TASKS | ext | advise | push target of every spoke · SLAs · escalation evidence | shop-floor badges, wearable pings |
| `prj` | PROJECTS | ext | advise | portfolio · critical path · budgets · stage gates | site sensors, asset & crew trackers |
| `net` | CONNECTORS | ext | act | ERP connectors (SAP/Oracle/NetSuite) · idempotent replay | edge agent fleet, device shadows |

Every spoke: plugin lifecycle (`pause / stop / update`), own dashboard + KPIs,
own spoke log, multi-tenant with local FX + USD reserve, and **external
egress** — it may expose *select* data to external hubs & spokes through the
grid. One spine at any scale: every capability scopes to tenant size.

### Five-layer permission engine (deny wins everywhere)

```
L-1  party boundary      no relationship + signed sharing contract -> no disclosure
L0  spoke state          a disabled spoke gates all — even ORIGIN re-enables first
L1  agent tier           observe -> advise -> act (allowlist + ORIGIN approval)
L2  spoke grant          spoke-to-spoke reads only, granted by ORIGIN, never writes
L3  column role          glob rules down to the column; unknown column = deny
```

Passing L-1 grants nothing: the request re-enters L0–L3 inside the owning
company. ORIGIN bypasses L2/L3 only — never L0, never the ledger, never an
agent.

### Agents, built-in and bring-your-own

- Every spoke carries a **serious domain agent** — it runs its spoke's craft
  (STEERS THE LINE, REBALANCES STOCK, CLOSES THE LOOP), bounded by its L1
  tier. It signals up to the leader AI **and briefs spoke users** scoped by
  role: dashboards, alerts, briefs.
- The **leader agent (AIL)** correlates across spokes, ranks on severity ·
  evidence quality · corroboration, and raises tasks & proposals — never
  commands. No agent holds ORIGIN. Leader agents never talk to each other.
- **Bring your own agents**: register user-defined agents via **MCP** or the
  **REST API** — sandboxed, policy-scoped, audited like any actor.

### IoT, native — not bolted on

Every spoke declares its meaningful telemetry (see table). `hub.ingest`
accepts MQTT/edge streams with store-and-forward buffers, a device registry,
and device shadows; every arrival is a master-log entry.

### The grid & external egress

Any spoke may expose **select data** to **external hubs and spokes** — beyond
hub & peer grants. Exposure is contract-shaped: versioned sharing contracts,
signed by both parties, notarized, one clause per field + direction, with
exact/banded/aggregate granularity and default opacity. Revocation is
retroactive. Benchmarking publishes p10/p50/p90 + party count only
(k-anonymity, k = 5; no party above ~1/3 of a sample).

## Quickstart

```bash
git clone https://github.com/tessera-scm/tessera.git
cd tessera
docker compose up          # hub + postgres + bus + plugin host (dev profile)
open http://localhost:8080 # setup wizard = the adaptive spine
```

Reference CLI (python):

```bash
pip install -e reference/python
tessera init --tenant demo --blueprint retail
tessera spoke list
tessera ingest csv ./demo/orders.csv --into ord
```

## Repository layout

```
hub/                 rust workspace: hub services (origin, access, ledger, ...)
spokes/              one crate per spoke (pln, src, trf, ord, crm, ful, ret,
                     inv, srm, ctr, fin, tsk, prj, net)
agents/              built-in spoke agents + leader agent (AIL)
sdk/                 MCP + REST client SDKs, plugin API
reference/python     executable spec & conformance vectors
schemas/             manifest.schema.json, sharing-contract, signals
docs/                architecture docs + canonical diagrams
.github/             CI, issue & PR templates
```

The repo is **docs-first**: the layout above is the target shape. Code
directories land after the design freeze (see [ROADMAP.md](ROADMAP.md)).

## Roadmap — next spoke candidates

Voted on per minor release; details in [ROADMAP.md](ROADMAP.md). Current
shortlist:

1. **AFTERMARKET** — service parts, warranties, field service, install-base lifecycle
2. **SUSTAIN** — emissions, circularity, supplier ESG scoring
3. **WORKFORCE** — labor planning, shift capacity, skills & certifications feeding TASKS and PROJECTS

(COMPLIANCE, ASSETS, and PRODUCT remain on the longer backlog.)

Propose your own via [feature template](.github/ISSUE_TEMPLATE/feature_request.md).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Vector-first: behavior changes ship
with conformance vectors. Design tokens: monotone zinc + exactly one accent.

## License & attribution

Apache-2.0 — **plus the Tessera naming requirement** (see
[LICENSE Appendix](LICENSE#appendix--tessera-naming-requirement) and
[NOTICE](NOTICE)). In short: free to use, study, modify, and ship —
commercially or not — but you must name Tessera where your users can see it.
The one-line credit:

```
Powered by Tessera - https://github.com/tessera-scm/tessera
```

Vendors may run it commercially; nobody may strip the credit. Compliance
recipes for apps, forks, SaaS, and embedded builds: [ATTRIBUTION.md](ATTRIBUTION.md).
