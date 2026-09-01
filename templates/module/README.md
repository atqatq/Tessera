# Module template

Copy this directory to start a third-party module. The manifest here
validates against the frozen v1 schema — keep it that way and add
your domain inside the contract.

## The three steps

1. **Rename the identity.** `module.id` follows the identifier grammar
   (lowercase, 1–64 chars, never `kernel.`), `version` is SemVer,
   `requires` names kernel services only.
2. **Declare your surface honestly.** `permissions.known_columns` and
   `peer_reads` are evaluated deny-wins by the kernel — a peer read
   without the owning side's grant is refused, whatever you declare.
   `egress_candidates` bounds what may *ever* leave the company;
   anything not listed is not shareable.
3. **Run the conformance vectors.**
   `python3 -m unittest reference.python tests for manifests` — the
   committed vectors in `reference/python/vectors/module_conformance.vectors.json`
   are the gate your manifest passes before any kernel ever sees it
   (see `schemas/module-manifest/v1/PLUGIN_API.md`).

## What a module is not

- Not a service (ADR-0006): plugins, one lifecycle, the kernel
  brokers everything.
- Not trusted (ADR-0011): no ambient authority — every capability is
  a kernel-issued handle.
- Not exempt from the rules: deny wins, append-only, and the vectors
  are the contract, for out-of-tree code too.
