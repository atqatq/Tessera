# 03 — Property: adjacent stars in a glob behave like one star

- Labels: `good-first-issue`, `area-access`
- Size: small (one proptest)

## Why this matters

`Glob::matches` (`kernel/access/src/lib.rs`) treats `*` as "any
sequence", and splitting on `*` naturally makes adjacent stars
(`a**b`) behave identically to `a*b` — empty pattern parts are
skipped. That equivalence is load-bearing (it keeps patterns sane
when generated or concatenated), but nothing pins it. Unpinned
equivalences are where regressions hide.

## Acceptance criteria

- A proptest in `kernel/access/tests/properties.rs`: for arbitrary
  pattern strings `p` built from the documented alphabet, and
  arbitrary subject strings `s`,
  `Glob::new(p with every '*ⁿ' collapsed).matches(s)` equals
  `Glob::new(p).matches(s)` — i.e. `a**b ≡ a*b`, `** ≡ *`.
- If the property exposes a real divergence, that is a bug: fix the
  engine minimally and say so in the commit message (and expect the
  maintainers to ask for a vector case too).

## Where to start

`glob_strategy()` in the properties file shows how patterns are
generated today; you will need a custom strategy that can emit
adjacent stars (the current one cannot).

## Definition of done

Green CI, property added without touching existing tests, one
sentence in the commit body explaining which invariant now holds.
