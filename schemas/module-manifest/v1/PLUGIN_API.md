# The plugin API contract — schema version 1 (frozen artefact)

This document and `module-manifest.schema.json` are the versioned
contract between the kernel and out-of-tree modules. They are frozen:
schema v1 does not change in place — evolution ships as v2 with a
migration note. The kernel *refuses* manifests whose
`schema_version` it does not implement: forward compatibility is a
refusal, not a guess.

This pass **specifies** the contract; it does not build the host
(ADR-0006 — one first-party module first, no scaffolding for a
runtime nobody loads yet). Every statement here is normative for the
host when it lands.

## 1. What a module is

A module is a manifest plus code. The manifest (frozen schema) names
the module's identity, its kernel-service `requires` (kernel services
only — ADR-0002), its declarative permission surface, its agent tier
and craft, its KPI pack, its telemetry declarations, its egress
candidates, and its compatibility envelope.

## 2. How the kernel verifies a module

At install time, in order, each step refusing with a reason:

1. **Schema validation** — the manifest parses against its declared
   `schema_version`. Unknown versions refuse. Unknown fields inside
   v1 refuse (`additionalProperties: false`).
2. **Identity checks** — module id matches the identifier grammar and
   never the `kernel.` prefix; version is valid SemVer.
3. **Requires resolution** — every `requires` entry is an implemented
   kernel service, and (at runtime) a *running* one. A module may
   require a service the host has disabled: install succeeds, start
   refuses, and the reason names the missing service.
4. **Permission admission** — the declarative `permissions` are
   recorded as L3 rules and L2 grant *requests*; peer reads still
   require the owning side's grant (denied by default). A module is
   never trusted to enforce its own matrix.
5. **Egress registration** — `egress_candidates` bound what the grid
   may ever share for this module. Nothing else is shareable, ever.

## 3. How the kernel sandboxes a module

Out-of-tree code is untrusted (ADR-0011 is the trust boundary):

- **No ambient authority.** Every capability flows through a handle
  the kernel issued: reads via `kernel.access` (deny wins, default
  deny), writes via the proposal path a human commits, time via an
  injected clock, identity via the subject the host binds at call
  time. A module cannot open a socket, read the filesystem, or see
  another tenant — not because we asked nicely, but because the API
  surface has no such call.
- **Resource bounds** — CPU and memory budgets per module instance,
  declared in the host configuration, enforced by the runtime that
  loads the module (a WASM host or an OS process boundary; the
  mechanism is the host's choice, the *guarantee* is this contract).
- **Telemetry is data** — device and sensor input arrives as
  permission-checked, ledger-stamped facts like everything else.

## 4. Lifecycle

`install → start → pause → resume → stop → update → archive`

- **install**: the five verification steps; the fact is ledger-stamped.
- **pause / resume**: the module stops receiving events and calls;
  its log freezes read-only; references still resolve (ADR-0006).
- **stop**: as pause, plus running work is cancelled at the next
  await/cancellation point — never mid-write (writes are idempotent;
  A7).
- **update**: a new manifest version goes through verification as a
  *new* install; the kernel computes the permission diff and refuses
  silent escalations — a widened `permissions` or new `requires`
  needs an operator's explicit approval, ledger-stamped.
- **archive**: read-only tombstone; the module's historical facts stay
  queryable forever (bitemporal); the code does not load again.

Disableability is a contract: a disabled module must not break the
kernel or the other modules — that is what "plugins, not
microservices" buys.

## 5. Versioning and compatibility

- Manifest schema: integer versions; v1 is frozen. v2 must ship with
  a migration note and a host that still reads v1 for the deprecation
  window (RELEASES.md: two minor cycles minimum).
- `compatibility.kernel_api`: a SemVer range against the plugin API
  itself. Breaking the plugin API is a major kernel version.
- Module versions are SemVer; the kernel treats a major-bump as a new
  update requiring the same approval flow as a permission widening.

## 6. The conformance a third-party module must pass

`reference/python/vectors/module_conformance.vectors.json` pins the
manifest gate: valid manifests accepted, each invalid shape refused
with its reason. The vectors are the contract for whichever
component validates manifests (the Python reference validates them
today; the host lands later and replays the same files).
