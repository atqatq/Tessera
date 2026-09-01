# 02 — Vectorize the height-field ledger tamper

- Labels: `good-first-issue`, `area-ledger`, `area-conformance`
- Size: small (gen_vectors.py + regenerate)

## Why this matters

The Rust spec test `verify_detects_a_wrong_height_field`
(`kernel/ledger/tests/spec.rs`) pins that a mutated `entry.height` is
caught at the exact height — but the committed conformance vectors
(`reference/python/vectors/ledger.vectors.json`) have no such case:
they cover payload, hash, and prev tampers only. The vectors are the
contract (ADR 0008); a behaviour proven only in one implementation's
test file is not yet pinned *between* the implementations.

## Acceptance criteria

- `gen_vectors.py` grows a `height` tamper kind (flip a byte of the
  stored height for one record).
- The regenerated vector set includes the case with the expected
  first broken height.
- Both implementations replay it green: `make test` (the Rust
  vectors harness needs the new kind handled — that is part of the
  work, small by design).

## Where to start

`reference/python/tools/gen_vectors.py` → `build()` and the tamper
application; then `kernel/ledger/tests/vectors.rs` → the tamper
match arms. Follow how `prev` tamper flows through both sides.

## Definition of done

`make check` green; the vector diff is in the PR; the commit message
says why the contract grew.
