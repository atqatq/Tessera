# Architecture

> **Design intent.** This page explains the target design. What is
> built today is stated in the README's status table — the two must
> never be confused, and where this page describes an unbuilt part,
> that is specification, not a claim.

The canonical diagram: [tessera-system-architecture.svg](../assets/tessera-system-architecture.svg)
(a design-intent view; the built v0.x picture is in the README).

## Topology

```
            THE GRID (inter-company, zero-trust)
   other kernels & modules  <-sharing contracts->  THIS COMPANY
                            +----------------+
                            |     THE KERNEL    |   AI-native control plane
                            |  12 services   |   sole authority, all events
                            +----------------+
                               |  |  |  |        the event bus (specified — no
        +----------+-----------+  |  +----------+----------+
        |          |              |             |          |
      14 MODULES   ... kernel-brokered peer access ...  module logs
```

## Invariants

1. **Star, not mesh.** Modules never call modules. `kernel.access` brokers every
   peer read; cross-module writes do not exist.
2. **Three stores, deliberately separate.**
   - *ledger* — what a value was (bitemporal, hash-chained per tenant)
   - *master log* — who did what (gapless seq, kernel-owned ordering)
   - *module logs* — verbose payloads via write-through pointers
3. **Bitemporal master data.** Every fact carries `valid_time` x
   `system_time`; the expression DSL (decimal arithmetic, units,
   effective-dated formulas) computes on demand — values are never stored
   ahead of time.
4. **Plugins, not microservices.** Modules install, pause, stop, and update
   independently. A disabled module freezes its log read-only; references
   still resolve.
5. **Two trust models, never conflated.** Inside a company the kernel is
   trusted (L0-L3). Across the grid nobody trusts anybody's kernel (L-1 +
   sharing contracts + notary).

## Message bus (specified, not built)

The design calls for a duplex, per-tenant hash-chaining of event
streams, backpressure-aware. No throughput number appears here on
purpose: a performance claim returns only with a committed benchmark
and a CI regression gate (Part F.3), and the bus does not exist yet.
Every transaction, action, and event — kernel or module — is to be
recorded; the ledger is the system of record for state, the master
log for agency.

## Why Rust + Python

Kernel services and modules are Rust crates (one workspace, one manifest
schema). `reference/python` is the executable spec: conformance vectors
live there and CI proves the Rust implementation matches it.
