# 10 — Safety-stock vector: a three-echelon chain with unequal staged service levels

- Labels: `good-first-issue`, `area-inv`, `area-conformance`
- Size: small (one case in gen_vectors.py)

## Why this matters

The inv safety-stock vectors include edge cases the brief demanded —
zero demand, negative lead time, single echelon, service level at 0
and at 1 — but the *interesting* production shape is deeper than the
edge cases: a real chain where the staged service levels differ per
echelon and the upstream stage inherits aggregated demand from two
downstream children. One such case in the committed set pins the
aggregation rule (sum of children's mean demand, root-sum-square of
their demand deviations) between both implementations.

## Acceptance criteria

- One new case in the safety-stock vector set: three echelons — two
  independent retailers under one regional DC — service levels e.g.
  0.90 / 0.90 / 0.95 (retailers / DC), distinct lead-time means and
  deviations.
- Expected outputs computed by the Python reference; both
  implementations replay green (`make test`).
- The vector's `note` field names the aggregation rule so a reader
  knows what is being pinned.

## Where to start

`reference/python/tools/gen_vectors.py` (safety-stock section) and
the existing two-echelon case next to it. Keep arithmetic exactly as
the generator does — every step is documented because it must
reproduce bit-for-bit across languages.

## Definition of done

Vector diff in the PR with the reason; `make check` green; the new
case is referenced from the vector-reader guide (issue #07) if that
has landed.
