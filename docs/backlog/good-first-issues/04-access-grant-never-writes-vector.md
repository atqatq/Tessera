# 04 — Access vector: a covering grant never confers a write

- Labels: `good-first-issue`, `area-access`, `area-conformance`
- Size: small (one case in gen_vectors.py)

## Why this matters

The spec test `peer_writes_do_not_exist_as_an_operation` proves that
cross-module writes deny even when a peer grant covers the exact
column — grants are read-only by construction. But the committed
vector set has no case where a grant exists *and* a write is
attempted, so the contract between the two implementations is silent
on the most tempting shortcut in the whole permission model. (This
is the exact loophole an implementation under deadline pressure
would cut.)

## Acceptance criteria

- One new access vector case: env with a covering grant, request is
  a peer `write`, expected `deny_default` at layer `l2`.
- Regenerated vectors; `make check` green; the Python reference
  already agrees (it should — verify, do not assume).

## Where to start

`reference/python/tools/gen_vectors.py` → `access_cases()`; follow
the `peer_writes…` example in `kernel/access/tests/spec.rs` for the
env shape.

## Definition of done

Vector diff in the PR; commit message states which loophole the case
pins shut.
