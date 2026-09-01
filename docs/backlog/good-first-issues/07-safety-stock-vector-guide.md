# 07 — Document the safety-stock vectors: a reader's guide page

- Labels: `good-first-issue`, `area-inv`, `area-docs`
- Size: small (one book page)

## Why this matters

`modules/inv` proves the depth bar with multi-echelon safety stock
under staged service levels, and its conformance vectors are the
specification. The vectors are readable — inputs, expected outputs,
edge cases — but nobody has written the page that *walks a novice
through one*, which is the difference between a contract and a
tar pit.

## Acceptance criteria

- A new page in the docs book under "How-to guides":
  "Read a safety-stock vector" — pick one case, walk every field,
  explain where the number comes from (the z-approximation, the
  σ_DL formula, the ceil to whole units), and show how to verify the
  case by hand in both languages.
- Honest tone: name the assumptions (independent demand, given lead
  times) — the page teaches the spec, it does not sell it.
- Linked from the tutorial and from the inv section of the README's
  status table (one line each).

## Where to start

`reference/python/vectors/safety_stock.vectors.json`,
`reference/python/tools/gen_vectors.py` (the generator documents the
algorithm), and the book's Diátaxis how-to siblings for voice.

## Definition of done

mdBook builds; a reviewer can follow the page without the source
open; no number appears that the vectors do not produce.
