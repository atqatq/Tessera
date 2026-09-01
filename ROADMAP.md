# Roadmap

**v0.x success criterion, in one sentence:** a person can install the
kernel, install `inv`, ingest a CSV of stock positions, get a
safety-stock recommendation, and read the ledger entry recording it —
written as an end-to-end test in `tests/e2e` that fails today and
flips to required when it passes.

Fourteen modules at genuine industry depth is 500+ engineer-years;
the previous version of this file presented that wish list as a plan.
It is not a plan. v0.x targets **exactly one module** plus the minimum
kernel, and proves the depth bar with one real algorithm.

## v0.x — the one module and the minimum kernel

| Piece | Scope | State |
|---|---|---|
| `kernel.access` | five-layer permission engine; fourteen decision codes | **built + tested** (`kernel/access`) |
| `kernel.ledger` | per-tenant hash chains, append-only, idempotent replay | **built + tested** (`kernel/ledger`) |
| `kernel.ids` | typed identifiers | **built + tested** (`kernel/ids`) |
| `kernel.master_data` | bitemporal master data, expression DSL | specified (docs, ADR-0004) |
| `kernel.events` | the bus | specified (ADR-0002 puts it inside the star) |
| `kernel.plugin_host` | module lifecycle | specified (ADR-0006) |
| `inv` | multi-echelon safety stock under staged service levels | **the one red-green-refactor cycle of this pass** (`modules/inv`) |

The depth bar is proved by one algorithm with its conformance vectors
written **before** the implementation — inputs, expected outputs, and
the edge cases (zero demand, negative lead time, single echelon,
service level at 0 and at 1). The vectors are the specification.

## Design intent, not scheduled

The rest of the original fourteen are specifications, kept as
specifications. They are not a roadmap, carry no dates, and no
labour is promised against them in v0.x:

PLAN, SOURCE, TRANSFORM, ORDER, CRM, FULFILL, RETURN, SUPPLIERS,
CONTRACTS, FINANCE, TASKS, PROJECTS, CONNECTORS — plus the adaptive
spine (configuration, not a module), the grid (federation spec),
agents (spec), and notarisation (optional appendix; the kernel runs
with no notary configured).

A module graduates from this section only by: a proposal issue, an
RFC, a spec PR (manifest, permission matrix, KPI pack, agent tier,
vectors), and a maintainer decision that v0.x is done — in that
order, with the [success criterion](#v0x--the-one-module-and-the-minimum-kernel)
still true.

## After v0.x, in the order value would arrive

1. **Ingest + CLI + persistence** — the v0.1 criterion demands it;
   design lives in the docs (ingest framework, three stores).
2. **`kernel.master_data`** — bitemporal facts underpin every
   recommendation's assumptions.
3. **`kernel.plugin_host`** — the module contract is already frozen
   (`schemas/`); the host makes it loadable.
4. **`kernel.events`** — the star's delivery mechanism, loom-tested.

Each gets its own red-green cycles and its own conformance vectors;
none is announced before its first vector exists.

## Next module candidates

Voted per minor release, only after v0.x. A candidate becomes a
proposal issue, then a spec PR (manifest, permission matrix, KPI set,
agent tier justification, conformance vectors — see CONTRIBUTING).
AFTERMARKET, SUSTAIN, WORKFORCE, COMPLIANCE, ASSETS and PRODUCT remain
in the idea file at `docs/backlog/` — demand, not date, moves them.

## Non-goals

- Tessera never forks the reference-model anchors — niches attach as
  configuration and extension modules.
- No module-to-module direct calls, ever; the kernel brokers everything.
- No agent, built-in or user-defined, ever holds ORIGIN.
- No performance claim without a committed benchmark and a regression
  gate (Part F.3).
