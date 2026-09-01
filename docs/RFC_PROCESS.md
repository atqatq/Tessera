# RFCs

Anything that touches a kernel invariant — the permission engine's
layers, the ledger hash construction, the store separation, the module
contract — starts as an RFC, modelled on the Rust RFC process.

## When an RFC is required

- A change to any invariant listed in `docs/adr/` (a change there also
  requires a superseding ADR).
- A new module manifest capability or a change to the frozen manifest
  schema.
- Any new dependency in a kernel crate.
- Anything that would change or re-scope committed conformance vectors.

## Process

1. Copy `docs/adr/rfc-template.md` to `docs/rfc/NNNN-title.md`.
2. Open a PR with the RFC; discussion happens on the PR, in the open.
3. After feedback settles, the maintainer accepts, rejects, or defers
   with reasons recorded in the file itself.
4. Accepted RFCs produce implementation PRs; each cites its RFC.
5. When the decision is structural, it graduates into an ADR (0001)
   and the RFC notes its ADR number.

Light process is not no process: a one-paragraph RFC beats a design
that lives only in someone's head.
