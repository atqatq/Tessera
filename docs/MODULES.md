# The Module Registry

Fourteen modules ship in-tree. All follow one contract; they differ in
domain depth, not in plumbing.

## The module contract

Every module must provide:

- `manifest` (validated by `schemas/manifest.schema.json`), whose `requires`
  lists **kernel services only**
- a permission matrix (L0-L3) with deny-wins tests
- an agent with a declared tier (`observe | advise | act`) and craft
- a dashboard + KPI pack; module-log event list
- multi-tenant isolation, local FX + USD reserve
- plugin lifecycle: install, pause, stop, update, archive
- IoT telemetry declaration + ingest mapping
- egress candidates: which fields may ever leave, at which granularity

## Registry

| Code | Module | Echelon | Agent tier | Craft (agent line) | Industry-standard depth | IoT telemetry |
|---|---|---|---|---|---|---|
| `pln` | PLAN | core | advise | RUNS S&OP DRAFTS | MRP - MPS - ML demand forecasting - supply design & planning - S&OP/S&OE - inventory policy | POS & shelf sensors, weather feeds |
| `src` | SOURCE | core | advise | SCORES SUPPLY RISK | strategic sourcing - RFx/auctions - PO lifecycle - receiving - 3-way match handoff | inbound GPS, gate scans, cold-chain probes |
| `trf` | TRANSFORM | core | act | STEERS THE LINE | MES - finite-capacity scheduling - BOM/where-used - WIP - SPC - OEE | line PLCs, OEE counters, vision QA |
| `ord` | ORDER | core | advise | PROMISES & PROTECTS | OMS - ATP/CTP - order-to-cash - backorder & allocation - credit checks | e-comm & POS event streams |
| `crm` | CRM | ext | advise | OWNS CHURN CRAFT | customer 360 - pipeline & quotes - service & SLAs - churn & CLV models - campaigns | connected-product telemetry, app events |
| `ful` | FULFILL | core | act | OPTIMIZES EVERY ROUTE | WMS - TMS - pick-pack-ship - wave planning - carrier selection - OTIF/POD | fleet GPS, geo-fences, cold-chain probes |
| `ret` | RETURN | core | advise | DISPOSITIONS FAST | RMA - reverse logistics - inspection & disposition - warranty - recovery accounting | smart RMA kiosks, return-drop scans |
| `inv` | INVENTORY | ext | act | REBALANCES STOCK | multi-echelon optimization (MEIO) - safety stock - ABC/XYZ - cycle counts | RFID, smart shelves, drone counts |
| `srm` | SUPPLIERS | ext | advise | WATCHES SUPPLIERS | qualification - scorecards - OTIF/PPM - risk monitoring - development | supplier port telemetry, cert feeds |
| `ctr` | CONTRACTS | ext | observe | WATCHES CLAUSES | bitemporal terms - sole writer of `ctr.commercial_terms` - SLAs - pricing ladders | usage meters, SLA probes, e-sign pads |
| `fin` | FINANCE | ext | advise | CLOSES THE LOOP | costing & landed cost - margin waterfalls - cash-to-cash - AP/AR - FX & tax | POS terminals, metered-usage events |
| `tsk` | TASKS | ext | advise | PRIORITIZES WORK | push target of every module + leader - SLAs - escalation curves - evidence | shop-floor badges, wearable pings |
| `prj` | PROJECTS | ext | advise | GUARDS CRITICAL PATH | portfolio - milestones - critical path - budgets - stage gates | site sensors, asset & crew trackers |
| `net` | CONNECTORS | ext | act | KEEPS SYNCS HONEST | ERP connectors (SAP/Oracle/NetSuite) - bidirectional sync - idempotent replay | edge agent fleet, device shadows |

## Depth bar (why "industry standard" is a test, not a slogan)

A capability line is accepted only if a practitioner from that industry
would recognize it as the real thing: MRP means netted, low-level-coded,
time-phased requirements; MEIO means multi-echelon optimization with
staged service levels; OTIF means on-time-in-full with POD. CRUD wrapped
in supply chain vocabulary is rejected in review.

## Roadmap candidates

WORKFORCE (labor planning) - COMPLIANCE (ESG/trade) - ASSETS (PdM/OEE) -
PRODUCT (NPI/BOM governance). See README roadmap to vote or propose.
