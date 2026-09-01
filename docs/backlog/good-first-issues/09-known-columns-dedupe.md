# 09 — State and pin the `known_columns` dedupe semantics

- Labels: `good-first-issue`, `area-access`
- Size: small (one test + one doc sentence)

## Why this matters

`Env::with_known_columns` feeds a `BTreeSet`, so duplicate inserts
silently dedupe — fine, but *unstated*. Undocumented lenience in an
engine whose entire job is precise refusal is exactly where future
confusion grows: does a duplicate throw? Warn? Nothing? Today:
nothing, on purpose. Say so, and pin it.

## Acceptance criteria

- A unit test in `kernel/access/tests/spec.rs`:
  `with_known_columns_called_twice_with_duplicates_still_denies_the_unknown`
  — build an env in two `with_known_columns` calls with overlapping
  columns, assert a decision on a known column still resolves and an
  unknown one still denies.
- One sentence in the `with_known_columns` doc comment stating the
  dedupe semantics and why (sets, not bags: a column is known or it
  is not).
- The decision NOT to warn is recorded in the doc comment too — a
  reviewer in a year should not re-litigate it.

## Where to start

`Env::with_known_columns` in `kernel/access/src/lib.rs`; the sentence-
named tests around it.

## Definition of done

Green CI; test and doc carry the decision; nothing else changed.
