# ADR 0011 — The trust boundary for out-of-tree modules

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer
- Requires: ADR-0002 (star), ADR-0006 (plugins), ADR-0007 (deny wins)

## Context

Third-party modules are what make Tessera a platform rather than an
application — but out-of-tree code in the same process as the ledger
is out-of-tree code that can read the ledger. The frozen plugin
contract (`schemas/module-manifest/v1/PLUGIN_API.md`) needs an
explicit trust model, stated before the host exists so the API
surface is designed around it rather than retrofitted.

## Options considered

1. **Trusted in-process plugins** — simplest, fastest; a malicious or
   buggy module can read any tenant, forge facts, or wedge the loop.
   Equivalent to npm's supply-chain posture, which is a cautionary
   tale, not a model.
2. **Everything sandboxed (WASM only)** — strong isolation; heavy for
   first-party modules and hostile to the plugin model's promise that
   modules are boring to write.
3. **Capability-based, no ambient authority** — modules run in-process
   (or in a host-chosen sandbox) but the *API surface carries no
   ambient power*: every capability is a kernel-issued handle, every
   read is permission-checked, every side effect is ledger-stamped,
   and the host may additionally isolate any module it does not trust.

## Decision

Option 3, with the boundary stated as invariants rather than
mechanisms:

- **No ambient authority.** The plugin API has no call that opens a
  socket, reads a filesystem, reads another tenant, or mints identity.
  Time comes from an injected clock; identity comes from the host at
  call time; storage comes through the kernel stores.
- **The permission engine is not bypassable from inside the process.**
  Deny wins for modules exactly as for humans and agents — L0 gates
  the module itself (pause it and its log freezes), L1 bounds its
  agent, L2 requires the owning side's grant, L3 denies unknown
  columns. A module's declarative manifest matrix is an *application*,
  not an authority.
- **The host may isolate anything.** The contract guarantees the API
  carries no ambient power; the host decides per module whether to
  additionally sandbox (WASM, OS process). First-party modules run
  unsandboxed; unknown publishers may be forced into isolation by
  policy, and the manifest's publisher identity is ledger-stamped at
  install.
- **Supply chain is part of the boundary.** A module's dependency
  closure is its problem to justify (the same rule as the kernel's);
  the SBOM and signature story applies to module releases like any
  other release artefact.

## Consequences

- The plugin API is small, because every method must justify itself
  against "is this a capability handle?" — a review question with a
  clear answer.
- A compromised module can do damage only through capabilities it was
  granted, which the ledger records — incident response becomes
  reading, not archaeology.
- First-party modules prove the model: they use the same handles,
  which is why the kernel crates already consume no powers beyond
  what the engine grants them.
- Full memory isolation is explicitly *not* claimed: a module that
  the host runs unsandboxed shares the process with the kernel. The
  sandbox knob exists for exactly that, and the docs say which mode
  is running. Honesty here is the point of stating a boundary.
