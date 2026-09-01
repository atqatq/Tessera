# The Adaptive Spine

The kernel's setup superpower: the system is industry- and niche-agnostic, and
the spine morphs it to *your* supply chain during onboarding — like a spine
the whole organism hangs on.

## First-class, not a wizard bolted on

The adaptive spine is the same class of kernel capability as roles, accounts,
and permissions: versioned, permission-checked, ledger-stamped. Setup
decisions are facts with provenance, not disposable wizard state.

## What the blueprint wizard fits

For a blank tenant, the wizard walks:

1. **Entities** — what things exist (items, batches, lots, containers,
   runs, loads, sites) and which bitemporal columns they carry
2. **Flows** — how things move (which SCOR anchors are active; which
   modules are installed; hand-offs between them)
3. **KPIs** — what "good" means (OTIF? OEE? cash-to-cash? PPM?)
   and the widget layout per role
4. **Terms** — commercial vocabulary and pricing ladders that
   `ctr.commercial_terms` will govern
5. **Scale profile** — tenant size and reach: concurrency, retention, depth of
   planning runs, IoT fleet size — the spine scopes every capability

## Verticals

Retail, pharma, automotive, food, chemicals, 3PL are configuration packs,
not forks: **SCOR anchors never fork** — niches attach as configuration
and extension modules. A pharma tenant adds cold-chain probes and
quarantine dispositions; a two-person shop disables eight modules and runs
PLAN + ORDER + FULFILL from a phone.

## Guarantees

- The spine never writes outside its setup scope; every change is an
  audited proposal -> commit on master data.
- Re-running the wizard on a live tenant produces a diff plan, never an
  in-place surprise.
- Export/import: a tenant blueprint is a signed artifact (portable,
  diffable, reviewable).
