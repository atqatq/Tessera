# 01 — Extend the identifier round-trip property to all four id kinds

- Labels: `good-first-issue`, `area-ids`
- Size: small (one test file, ~20 lines)

## Why this matters

`kernel/ids/tests/roundtrip.rs` proves the parse → display → parse
round trip and the corruption rejection as *properties*, but only for
`TenantId` and `ModuleId`. `SubjectId` and `RoleId` are covered only
by the curated table in `validation.rs`. The grammar is shared, but
the *proof* should not be partial — properties exist precisely so we
do not argue from examples.

## Acceptance criteria

- The `round_trip_through_display` and `corrupted_ids_are_rejected`
  proptests in `kernel/ids/tests/roundtrip.rs` cover all four string
  id types (`TenantId`, `ModuleId`, `SubjectId`, `RoleId`).
- All cases still pass, and `make check` is green.

## Where to start

Read `kernel/ids/tests/roundtrip.rs` — the two proptests at the
bottom. A tiny helper that maps an id string through a constructor
generic over the four types keeps it tidy (or just duplicate the
three assertions — clarity beats cleverness in tests).

## Definition of done

PR green on CI, no test renamed or weakened, commit message explains
why the proof had to cover all four kinds.
