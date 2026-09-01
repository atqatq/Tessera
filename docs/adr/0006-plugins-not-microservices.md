# ADR 0006 — Plugins, not microservices

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

Fourteen domain modules plus tenant-specific extensions could be
deployed as separate services (independent scaling, polyglot freedom)
or as in-process plugins (no network hops, shared trust, one binary).

## Options considered

1. Microservices per module: independent scaling and deployment;
   network auth on every call, distributed failure modes, and an
   operational floor (mesh, service discovery, tracing) a two-person
   shop will never run.
2. In-process plugins with a lifecycle (install, pause, stop, update,
   archive), sandboxed by the permission engine rather than by a
   network boundary.
3. Hybrid: plugins by default, services for scale-out hot paths later.

## Decision

Option 2 for v0.x, with 3 kept open explicitly. The scale profile a
tenant picks during setup scopes capability; a two-person shop and a
global enterprise run the same spine, which is only possible if the
unit of deployment is a plugin, not a service.

## Consequences

- A disabled module freezes its log read-only; references still
  resolve. Lifecycle transitions are ledger-stamped like anything else.
- The trust boundary for out-of-tree code is a kernel concern, not a
  network concern — see the module manifest schema freeze (E4) and its
  trust-boundary ADR when that lands.
- Hot paths that outgrow the plugin model will need the service escape
  hatch; that decision gets its own ADR when a benchmark demands it,
  not before.
