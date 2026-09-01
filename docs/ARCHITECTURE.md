# Architecture

The canonical diagram: [scor-hub-spoke-architecture.svg](scor-hub-spoke-architecture.svg)
(rendered in the repo root README).

## Topology

```
            THE GRID (inter-company, zero-trust)
   other hubs & spokes  <-sharing contracts->  THIS COMPANY
                            +----------------+
                            |     THE HUB    |   AI-native control plane
                            |  12 services   |   sole authority, all events
                            +----------------+
                               |  |  |  |        1,000,000 msg/sec duplex
        +----------+-----------+  |  +----------+----------+
        |          |              |             |          |
      14 SPOKES   ... hub-brokered peer access ...  spoke logs
```

## Invariants

1. **Star, not mesh.** Spokes never call spokes. `hub.access` brokers every
   peer read; cross-spoke writes do not exist.
2. **Three stores, deliberately separate.**
   - *ledger* — what a value was (bitemporal, hash-chained per tenant)
   - *master log* — who did what (gapless seq, hub-owned ordering)
   - *spoke logs* — verbose payloads via write-through pointers
3. **Bitemporal master data.** Every fact carries `valid_time` x
   `system_time`; the expression DSL (decimal arithmetic, units,
   effective-dated formulas) computes on demand — values are never stored
   ahead of time.
4. **Plugins, not microservices.** Spokes install, pause, stop, and update
   independently. A disabled spoke freezes its log read-only; references
   still resolve.
5. **Two trust models, never conflated.** Inside a company the hub is
   trusted (L0-L3). Across the grid nobody trusts anybody's hub (L-1 +
   sharing contracts + notary).

## Message bus

Duplex, per-tenant hash-chaining of event streams, backpressure-aware.
Every transaction, action, and event — hub or spoke — is recorded; the
ledger is the system of record for state, the master log for agency.

## Why Rust + Python

Hub services and spokes are Rust crates (one workspace, one manifest
schema). `reference/python` is the executable spec: conformance vectors
live there and CI proves the Rust implementation matches it.
