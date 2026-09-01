# ADR 0002 — Star topology, not mesh: modules never call modules

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

Fourteen domain modules exchanging data create N×(N−1) potential
integration points. A mesh lets each pair optimise for itself; it also
means no single place enforces permissions, tenancy, or audit, and a
module outage propagates unpredictably.

## Options considered

1. Module-to-module mesh with per-pair contracts.
2. An event bus as the only inter-module channel.
3. Star topology: every peer access is brokered by `kernel.access`,
   permission-checked, and ledger-stamped; cross-module writes do not
   exist as an operation — changes travel as proposals a human commits.

## Decision

Option 3, with the event bus as a delivery mechanism inside the star,
never as an authority. The permission engine (L2 peer grants) and the
ledger are the only path by which one module's data reaches another.

## Consequences

- One enforcement point for access, tenancy, and audit — the permission
  engine's deny-wins properties are testable in one crate.
- Peer reads cost one hop through the kernel; measured performance is a
  benchmark question (Part F: no number without its benchmark), not a
  design assumption.
- Modules stay ignorant of each other: a module's manifest names kernel
  services and nothing else.
