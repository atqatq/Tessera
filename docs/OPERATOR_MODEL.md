# The operator model — the competence thesis

Tessera's least-stated and most differentiating claim: **the operator's
competence should not be the limiting factor, because the correct path
is the default path and every deviation leaves a record.**

State it again without the abstractions, because this document will be
read by people who have run warehouses for thirty years: the planner
with a week of training and the planner with thirty years of scars
should both end up doing the right thing — one because they know it,
the other because the system's defaults already encode it and the
system makes the cheap path and the correct path the same path. When
they disagree with the system, they are never blocked; they override,
and the override ships with a receipt and, later, an outcome. Everyone
learns or everyone proves a point — the ledger does not care which.

Nothing in this document reads operators as the problem. Bad outcomes
in supply chain software are usually architecture that made the wrong
path easy. The architecture is the thing we can fix.

## Mechanism 1 — Opinionated defaults with narrow bands

Every policy ships with the default a competent practitioner would
choose. Deviating **outside a sane band** requires a recorded reason —
recorded, not blocked.

- **Default:** each configurable policy carries a named default
  (`service_level: 0.95` for A-class make-to-stock items, cadence
  weekly, horizon twice the lead time). Defaults are picked by the
  module team and argued in the module's RFC, not per tenant.
- **Band:** the range within which the policy may move without
  explanation. Outside the band, the configuration still saves — with
  the reason attached, versioned, and visible to audit.
- **Invariants:** a default exists for every policy knob; a reason is
  mandatory for out-of-band values; the reason is ledger-stamped like
  any other fact.
- **Vectors:** the hard edges of the bands *are* the refusal vectors
  (mechanism 2). The band edges themselves land with the policy
  engine — specified, not pinned, until then. Nothing here claims to
  be tested before its vector exists.

## Mechanism 2 — Refusal on known-bad configurations

The system declines to save a setup that guarantees failure. Refusals
are failure modes of the *configuration*, addressed to the operator as
facts, not as scolding:

- zero safety stock against variable lead time,
- 100% service-level targets,
- reorder points below lead-time demand,
- forecast horizons shorter than the replenishment lead time.

**Invariants:** a refused configuration is never persisted; the refusal
message names the violated arithmetic, not the operator's judgement;
the same refusal fires identically in both implementations.

**Pinned today:** the safety-stock conformance vectors carry the
refusals as data — `service_level_one_is_refused`,
`negative_lead_time_is_refused`, `negative_demand_deviation_is_refused`,
plus the tree refusals (cycles, duplicates). Each refusal is a test
that fails when the refusal stops firing. The remaining refusals in
the list above land with reorder points and forecast configuration;
each gets its vector before its implementation, same as this one did.

## Mechanism 3 — Every recommendation carries its method and assumptions

Not a bare number:

> echelon dc: safety stock 179 units — staged service-level MEIO,
> sigma_DL 108.3 from lead time 6±1, service level 95%

A novice learns from it: they can see *which* method ran, and what it
assumed. Nobody can dispute the number without disputing something
specific — the service level, the lead-time deviation, the
aggregation. Disagreement becomes configuration, not argument.

**Invariants:** a recommendation without method and assumptions cannot
be constructed (the type has no such constructor); the explainer string
is deterministic; both is pinned by committed vectors.

**Pinned today:** `Recommendation::explain` is byte-pinned in the
safety-stock vectors (`explain_0`), and the `EchelonStock` struct makes
a method-less recommendation unrepresentable.

## Mechanism 4 — Overrides are free but never silent

Nobody is blocked from overriding. The ledger keeps:

1. what the system recommended,
2. what the human did instead,
3. who they are,
4. what happened afterwards.

**Invariants:** an override without a receipt cannot exist — the
override *is* a ledger entry referencing the recommendation entry it
deviates from; the outcome attaches as a later fact, never by editing
either entry (append-only, ADR-0003).

**Pinned today:** the receipt's *shape* is pinned by the end-to-end
v0.1 test (`tests/e2e`), which refuses a flow where an override leaves
no readable ledger receipt. The ledger machinery itself is built and
tamper-tested; the runtime that writes receipts through it is the v0.1
work the criterion describes.

## Mechanism 5 — Role packs, not an HR module

A role pack encodes the *job*: KPIs, cadence, dashboards, escalation
paths, and the brief the module's agent uses when it works for that
role. The competency framework becomes a configuration artefact — a
versioned, reviewable, portable file — not a fifteenth module and not a
consulting engagement.

- **Invariants:** a role pack is data (validated against the frozen
  manifest schema family); it grants nothing by itself — permissions
  still flow through `kernel.access` and deny wins; installing one is
  a ledger-stamped act like every other configuration change.
- **Vectors:** land with the schema freeze (module manifest v1 family)
  — specified here, pinned there, and this section will link them when
  they exist.

## The tension with the adaptive spine, resolved

The spine lets a tenant configure *structure*: entities, flows, KPIs,
vocabulary, scale. It must never let a tenant configure *method*:

> **Structure is configurable, method is not.**

A tenant defines their entities and flows; a tenant does not redefine
how safety stock is computed. The moment method became configuration,
the defaults would stop encoding competence (mechanism 1 collapses),
the refusals would become negotiable (mechanism 2 collapses), and the
recommendations would stop carrying a method anyone can look up
(mechanism 3 collapses). [ADR-0010](adr/0010-structure-configurable-method-not.md)
records the decision and its consequences.

## Tone, stated as a rule

The correct path is the easy path and every deviation leaves a record.
Any sentence in this project's docs or UI that reads as contempt for
the people who will use it is a bug — report it like one.
