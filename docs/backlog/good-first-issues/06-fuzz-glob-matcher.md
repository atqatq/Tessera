# 06 — Fuzz target: the glob matcher vs arbitrary bytes

- Labels: `good-first-issue`, `area-access`, `help-wanted`
- Size: medium (a fuzz/ crate + docs)

## Why this matters

The testing taxonomy (A2, `docs/site/src/TESTING.md`) is honest about
a gap: fuzzing lands "with the first parser that sees untrusted
bytes". The glob matcher is the closest thing the kernel has today —
tenant-configured patterns, `*` semantics, byte-wise scanning. A
libFuzzer target against `Glob::matches` plus a property oracle
(matches must be consistent with a naive reference implementation for
short inputs) closes part of that gap early and cheaply.

## Acceptance criteria

- `kernel/access/fuzz/` (cargo-fuzz layout) with one target:
  arbitrary (pattern, subject) byte pairs into `Glob::matches`.
- An oracle: for ASCII short inputs, compare against a brute-force
  expansion matcher (document that the oracle is slow by design).
- `docs/site/src/TESTING.md` taxonomy section updated: the fuzz row
  moves from "does not exist" to "exists for the glob matcher" with
  the run command.
- Running locally documented (`cargo fuzz run glob -- -max_total_time=60`).

## Where to start

[ cargo-fuzz book ](https://rust-fuzz.github.io/book/). Note: fuzz
targets build with nightly; that is fine — CI does *not* run fuzzing
yet, and the docs must say so honestly.

## Definition of done

60 local fuzz run with zero crashes (or crashes fixed with a
regression test first); docs updated; no new dependencies outside the
fuzz crate.
