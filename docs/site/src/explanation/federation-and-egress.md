# Federation and egress — on existing standards, not invented formats

The inter-company layer. Two trust models, never conflated:

- **Inside a company** the kernel is trusted: five permission layers,
  the star, the ledger.
- **Across the grid** nobody trusts anybody's kernel: contract
  negotiation and policy expression ride existing standards, events
  speak an existing vocabulary, and anchoring is optional plumbing.

A federation layer that only talks to itself has no value, and the
standards this layer needs are already shipping. Tessera's rule: where
a standard exists, speak it and map to it; where it genuinely does
not, say so and justify the extension — never invent a format by
inattention.

## Contract negotiation and transfer: IDSA Dataspace Protocol

The [IDSA Dataspace Protocol](https://international-dataspaces.org/dataspace-protocol/)
(DSP) covers the parts of the grid that are somebody else's solved
problem: how two participants discover each other, negotiate a
contract, and transfer data under it. Tessera's grid adopts DSP for
exactly that surface — catalog, negotiation, transfer — and a Tessera
node behaves as a DSP participant, because the dataspace work this
must interoperate with (Tractus-X and the EDC ecosystem) already
speaks it.

## Policy expression: ODRL

Sharing-contract clauses are [ODRL](https://www.w3.org/TR/odrl-model/)
policies: Permissions, Duties, Prohibitions, and Constraints on
W3C-defined operands. Writing clause semantics in ODRL means policy
engines that already exist can evaluate them, and the contracts stop
being prose.

## The mapping — every grid concept to its standard

| Tessera concept | Standard home | Notes |
|---|---|---|
| Sharing contract | DSP Contract Agreement, expressed as an ODRL Agreement | versioning: DSP negotiates offers; Tessera keeps each version as ledger-stamped facts |
| Clause (one field + direction) | ODRL Rule (Permission/Prohibition) with a Constraint naming the field | one clause per field survives as one Rule per field — auditable and diffable |
| Granularity: exact | ODRL constraint, standard operands | nothing to extend |
| Granularity: banded / aggregate (k-anonymity) | **no ODRL equivalent** — extension | ODRL constrains *usage* but has no semantics for "only publish p10/p50/p90 with k ≥ 5". Tessera defines custom RightOperands (`tessera:aggregate`, `tessera:kAnonymity`) with published shapes; the benchmark gates stay enforced **in code** (see below), because policy text alone must never be the gate |
| Retroactive revocation | **no DSP/ODRL equivalent** — extension | both standards enforce prospectively; Tessera's revocation re-reads history through the ledger (a disclosure becomes invisible for *new* queries and the revocation is itself ledger-stamped). This is a deliberate, justified divergence: dataspaces audit forward, supply chains audit backward too |
| Party, relationship | DSP Participant; the standing **relationship** is a Tessera extension | DSP has no long-lived relationship registry; Tessera keeps it, ledger-stamped, because suppliers onboard once and are reused by every buyer |
| Node tiers (full / light / observer) | **no equivalent** — Tessera participation profile | declared in the participant's catalog metadata; observer tier is time-boxed and clause-scoped by contract |
| Consent per metric | ODRL Duty/Prohibition per metric | one Rule per metric, as above |
| Agents reading across a boundary | out of DSP scope; governed by the contract's ODRL rules + the kernel's L1 tiers | agents may read across a boundary, never act; leaders never talk to each other |
| Cross-party writes | **do not exist** — in neither the grid nor the standards | changes travel as requests a human accepts |

Every extension above is named `tessera:*` in ODRL vocabulary terms,
documented in the sharing-contract schema when it freezes, and
carried by an RFC before it freezes.

## Events crossing a company boundary: GS1 EPCIS 2.0 and CBV

Anything that crosses a company boundary speaks
[EPCIS 2.0](https://www.gs1.org/standards/epcis) (the GS1 event
standard, now a ratified ISO/IEC standard) with the
[Core Business Vocabulary](https://www.gs1.org/standards/core-business-vocabulary):
*what object, what event (commission/observe/transform/decommission),
when, where, why*. The grid does not invent an event envelope; a
shipment seen across a boundary is an EPCIS `ObjectEvent`, a
transformation is a `TransformationEvent`, and the vocabulary for
business steps and dispositions comes from the CBV rather than a
Tessera dictionary.

**Conformance target:** the [OpenEPCIS](https://openepcis.io/)
implementation — Apache-2.0, aligned with GS1 — is the reference
tooling, and its published test resources become the conformance
vectors for boundary events. Vectors before implementation, as
everywhere else in this project.

## Ingest and identity: EDI and the GS1 identifier set

No real integration avoids EDI, and no cross-company identity survives
without the GS1 identifier set:

- **EDI ingest** — `kernel.ingest` treats EDIFACT and X12 as ingest
  formats beside CSV/XLSX/API: parsed to bitemporal facts with the
  same dead-letter discipline (unparsable segments are quarantined,
  never dropped). Mapping sets are versioned configuration, because a
  partner's EDI dialect is structure — and structure is configurable
  ([OPERATOR_MODEL](../../OPERATOR_MODEL.md)) — not method.
- **GS1 identifiers** — GTIN (products), GLN (locations/parties), SSCC
  (logistic units) are the identity layer for anything crossing a
  boundary. Tessera's internal identifiers stay typed and internal;
  cross-boundary payloads carry GS1 identifiers so the other side can
  resolve them without knowing anything about Tessera.

## Node tiers

- **Full node** — your company: this kernel + your modules.
- **Light node (free)** — one identity, docs, orders, scorecards; a
  supplier onboards once and is reused by every buyer. Light → full
  migrates nothing.
- **Observer tier** — auditor/regulator, time-boxed, clause-scoped.

## Appendix: notarisation, demoted to optional

Notarisation is an optional appendix to this document, not a kernel
service and not a dependency:

- The context that made it seem central is gone: **Hyperledger Grid
  reached end-of-life in March 2023, and Sawtooth was archived in
  2024**. The projects Tessera's original design borrowed its anchoring
  story from no longer exist as going concerns.
- Tessera therefore treats anchoring as **pluggable, never a
  dependency**: an optional adapter may anchor salted Merkle roots to
  external consensus (`submit(topic, root) -> receipt`), tiered from
  none → local chain → anchored → attested. Salted, because the root
  must not leak per-tenant correlation — the same reasoning that puts
  the tenant inside the ledger hash.
- **The kernel runs fully with no notary configured — that is
  pinned, not promised**: nothing in `kernel.*` depends on a notary
  (check `cargo tree`: no such dependency exists), and the end-to-end
  v0.1 test in `tests/e2e` runs its whole flow with `notary: none` and
  asserts the node reports it as healthy configuration.
