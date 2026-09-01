# Vocabulary

Words mean one thing each in these docs and this code. When prose and
code disagree, the code and its vectors win and the prose gets fixed.

| Term | Meaning |
|---|---|
| **Kernel** | the trusted core: access, ledger, master data, events, plugin host, tenancy. Modules require kernel services and nothing else. |
| **Module** | a pluggable domain engine (`inv`, `pln`, …). Never calls another module; peer reads are brokered. |
| **ORIGIN** | the superuser principal above root. Hardware-key identity; records intent before effect; bypasses L2/L3 only — never L0, never the ledger. No agent holds it, ever. |
| **L0–L3** | the four in-company permission layers: module state, agent tier, peer grants, column role rules. See [decision codes](../reference/decision-codes.md). |
| **L-1 (party boundary)** | the inter-company layer above the kernel: no relationship plus contract, no disclosure. Passing it grants nothing inside the owning company. |
| **Deny wins** | an explicit deny beats any allow on the same column; uncovered and unknown columns deny. |
| **Ledger** | the append-only, hash-chained, per-tenant record of what a value was. Never rewritten. |
| **Master log** | the gapless record of who did what. Separate from the ledger and from module logs. |
| **Bitemporal** | every fact carries `valid_time` × `system_time`; queries name their clock. |
| **Conformance vectors** | committed JSON: inputs and expected outputs both implementations must reproduce byte-for-byte. The contract, not decoration. |
| **The reference** | the stdlib-only Python package that generates and replays the vectors. A spec, not a runtime. |
| **Staged service levels** | per-echelon service-level targets in multi-echelon inventory optimization; the method `inv`'s safety-stock core implements. |
| **Adaptive spine** | setup capability: entities, flows, KPIs, vocabulary scoped to the tenant. Structure is configurable; method is not. |
| **Role pack** | a configuration artefact encoding a job: KPIs, cadence, dashboards, escalation paths, agent briefs. Not an HR module. |
| **Delegation** | ORIGIN approval made delegable: scoped (module × action × columns), time-boxed, rate-limited, revocable, ledger-stamped at issue and every use, never re-delegable — [ADR-0012](https://github.com/atqatq/Tessera/blob/main/docs/adr/0012-origin-delegation.md) |
| **The competence thesis** | the correct path is the default path and every deviation leaves a record — [docs/OPERATOR_MODEL.md](https://github.com/atqatq/Tessera/blob/main/docs/OPERATOR_MODEL.md) |
| **Grid** | the inter-company layer: sharing contracts, node tiers, benchmark gates. External to the kernel's trust model. |
| **Module log** | a module's verbose log, referenced by write-through pointers from the master log. Freezes read-only when the module is disabled. |
