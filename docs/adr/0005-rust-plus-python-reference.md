# ADR 0005 — Rust kernel plus a stdlib-only Python reference

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

A permission engine and a hash chain are trust anchors: a subtle bug is
a security event, and "the code is right because we reviewed it hard"
is not evidence. Two independent implementations of one written
specification, compared against committed vectors, turn divergence into
a test failure instead of a discovery.

## Options considered

1. Rust only, with thorough unit tests.
2. Rust plus a second Rust implementation (two crates, same language).
3. Rust plus a Python reference, stdlib-only, that generates and
   verifies the conformance vectors.

## Decision

Option 3. A second Rust implementation shares its ecosystem's blind
spots; Python does not. The reference stays dependency-free so vectors
are reproducible anywhere, and the Rust side consumes the same vector
files byte-identically (`docs/adr/0008-vectors-are-the-contract.md`).

## Consequences

- Every behavioural invariant is tested twice, in two languages, with
  two dependency trees — divergence is a failing CI job, not a debate.
- The reference is slow and incomplete by design: it is a spec, not a
  runtime. Feature parity is scoped to what the vectors pin.
- Two implementations must be updated in lockstep for any pinned
  behaviour; the vectors make the required order explicit (vector
  change first, then both implementations).
