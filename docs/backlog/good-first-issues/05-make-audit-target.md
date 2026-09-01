# 05 — `make audit`: mirror the CI cargo-audit gate locally

- Labels: `good-first-issue`, `area-ci`
- Size: small (Makefile + one docs line)

## Why this matters

CI runs `cargo audit` (the `cargo-audit` job), but `make check` — the
local mirror of CI, per the E7 rule "whatever CI runs, `make check`
runs locally" — does not. Gates that exist only in CI are surprises,
not gates.

## Acceptance criteria

- A `make audit` target that runs `cargo audit` (installing it via
  the same mechanism the CI job's tool uses, or instructing rustup-
  installed cargo to fetch it — your call, documented in the target's
  comments).
- Decision, made explicitly and written in the PR: should `check`
  depend on `audit`? Arguments both ways (network dependency on the
  advisory DB vs mirror completeness). The maintainers lean **not** —
  `audit` stays an explicit target because advisory-DB fetches should
  not gate every local build — but argue if you disagree.
- `make help` lists the new target.

## Where to start

`Makefile` — follow the shape of `mutation` and `coverage`.

## Definition of done

`make audit` works on a clean machine; the Makefile comment states
the decision and why.
