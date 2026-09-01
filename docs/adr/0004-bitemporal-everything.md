# ADR 0004 — Bitemporal everything

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

Supply chain records live under two clocks: when something was true in
the world (`valid_time`) and when the system learned it (`system_time`).
A supplier address changes; a price was retroactively renegotiated; an
ingest arrives late. Single-time models overwrite history and make the
audit trail a reconstruction.

## Options considered

1. Append-only single timeline with correction events.
2. Bitemporal facts: every fact carries `valid_time` × `system_time`;
   queries name one or both clocks.

## Decision

Option 2. Master data and ledger records are bitemporal; the ledger's
`Entry` carries both instants (`valid_ms`, `system_ms`). Values are
computed on demand by the expression DSL — never stored ahead of time —
so a retroactive correction changes future answers without rewriting
past facts.

## Consequences

- "What did we believe on March 3 about the March 1 stock position?" is
  a query, not an investigation.
- A fact is never visible before its `valid_time` under a
  `valid_time`-pinned query — an invariant with a property test in the
  kernel's test taxonomy (Part A2), enforced as the stores land.
- Every read path must name its clock or accept the default (`system_time`
  now, `valid_time` now); ambiguity is a type error, not a guess.
