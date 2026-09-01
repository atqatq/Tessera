# Contributing to Tessera

Thanks for helping build supply chain infrastructure a global enterprise
and a two-person shop can both trust. The rules below are gates, not
aspirations — a commit that violates any of them is not ready, however
good the code is.

## The five rules

1. **Vector-first.** Behaviour changes ship with conformance vectors
   under `reference/python/vectors/` — no vector, no merge. The vectors
   are the contract between the Python reference and the Rust
   implementation (ADR 0008).
2. **Kernel services are the only legal dependency.** A module's
   `requires` may name `kernel.*` services and nothing else.
   Module-to-module calls do not exist; peer reads go through
   `kernel.access` grants (ADR 0002).
3. **Deny wins.** Every permission path fails closed. If your change
   makes any path default-allow, it is wrong (ADR 0007).
4. **Append-only.** The ledger and master log never rewrite history.
   Corrections are new facts with provenance.
5. **Monotone + one accent.** UI follows the design tokens: zinc ramp,
   single accent `#6E96E8`. No decorative colour.

## The workflow: red, green, refactor — strictly

No production line exists before a test demanding it. This includes
scaffolding, glue, and "obvious" code. If you are writing code with no
failing test in front of you, stop and go back. Never write a test to
fit code you already wrote — if code came first, delete it and start
from the test.

A worked cycle from this codebase (`kernel/access`):

1. **Red — write the failing test first.** `tests/spec.rs` was written
   against an API that did not exist; the suite failed to compile, and
   that compile failure *was* the red state: the test named the exact
   functions and types the code would need.
2. **Green — minimum code to pass.** The engine was implemented only
   against those tests, not ahead of them.
3. **The properties earn their keep.** Mid-cycle, the property
   `propose_and_write_agree_for_users` caught a real design flaw: rules
   with an action of `Propose` made propose and write verdicts diverge.
   The fix was not more tests — it was making the flaw unrepresentable:
   rules now govern `Read` or `Write` only (`RuleAction`), and a propose
   rule cannot exist. That is what "tests describe behaviour" means:
   the failing property changed the design.
4. **Refactor with the tests green.** A pure refactor must not change a
   single test. If it does, it was not a refactor.

### Naming and structure

- Tests are sentences: `rejects_ids_over_64_characters`, not `test_2`.
  Sentence names let a test failure tell you which behaviour broke.
- Unit tests for pure logic; `proptest` for every invariant that must
  hold across all inputs; conformance vectors where the Python
  reference must agree. The full map lives in [docs/TESTING.md](docs/site/src/TESTING.md).


## Your first PR — one full cycle

This walks a real change end to end: `make setup` through a red test
to a merged PR. Budget: about an hour the first time.

1. **Set up** (`make setup`) and prove the suite green (`make test`).
2. **Pick a change small enough to finish.** The list in
   [`docs/backlog/good-first-issues/`](docs/backlog/good-first-issues/)
   is curated for exactly this; each one states its acceptance
   criteria.
3. **Branch** (`git checkout -b docs/gfi-04-subject-role-roundtrip`).
4. **Write the failing test first.** For example, extending an
   existing roundtrip property to cover `SubjectId`/`RoleId`:
   change the proptest, run `cargo test -p tessera-ids`, and watch it
   fail — that failure is your red, and it must fail for the reason
   you expect (read the assertion message).
5. **Green — minimum change.** Implement only what the test demands.
6. **Gates:** `make check` — fmt, clippy `-D warnings`, the full
   suite, vector freshness. If you touched behaviour the vectors pin,
   regenerate them and explain the diff in the commit message.
7. **Commit** with a Conventional Commit that explains *why*, signed
   off: `git commit -s`. Unsigned commits fail the DCO gate; note that
   commits do **not** need to be cryptographically signed — branch
   protection enforces signed commits on `main` at merge time, and the
   maintainer signs the merge.
8. **Open the PR** against `main`. Use the template; tick what you
   actually did. CI runs the same gates you ran; green on both sides
   is the review request.
9. **Review.** A maintainer (or reviewer) responds — questions are
   normal, requests to re-run gates are normal, "the test you wrote
   changed during refactor" is not allowed to anyone, including you.

That is the whole loop. Everything else in this document is the same
loop with sharper edges.

## Triage labels

`good-first-issue` (scoped, real, reachable), `help-wanted`
(maintainer capacity is the constraint), `needs-design` (RFC or
proposal first). The set lives in
[`.github/labels.yml`](.github/labels.yml); new areas get a label in
the same PR that creates the area.

## Community channels

- **Discussions** for questions and ideas (issues are tracked work).
- Response times are honestly stated in SUPPORT.md — this project has
  one maintainer today, and pretending otherwise helps nobody.

## Gates (enforced before every commit)

```bash
make check        # lint + test + vector freshness — the same bar as CI
```

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `python3 -m unittest discover -s reference/python/tests`
- Regenerated vectors produce an empty diff.

`#![forbid(unsafe_code)]` stands in every crate. No `unwrap`, `expect`,
or `panic!` on any path reachable from input — enforced by lints, not
vigilance; test code opts out locally, with a comment, because a
crashing test is a signal.

## Commits and review

- **Conventional Commits** (`feat:`, `fix:`, `docs:`, `chore:`,
  `module(inv):`) with a body explaining *why*, not what.
- **DCO sign-off** on every commit (`git commit -s`) — no CLA (see
  LICENSING.md).
- One logical change per PR; each commit is green and revertible on
  its own.
- Signed commits are enforced on `main` by branch protection;
  contributors' commits do not need to be signed — the maintainer
  signs the merge.
- No commented-out code, no dead code, no TODO without an issue number.

## Kernel invariant changes need an RFC

Anything touching a kernel invariant, the frozen module manifest
schema, or a kernel dependency starts as an RFC:
[docs/RFC_PROCESS.md](docs/site/src/RFC_PROCESS.md). Structural decisions
graduate into ADRs ([docs/adr/](docs/adr/)).

## Adding a module — checklist

- [ ] `schemas/module-manifest.schema.json` validates
- [ ] `requires` only kernel services
- [ ] permission matrix (L0–L3) documented, deny-wins tests
- [ ] 3+ industry-standard capability lines (MRP-grade depth, not CRUD)
- [ ] agent tier stated (observe/advise/act) + craft description
- [ ] meaningful IoT telemetry declared + ingest mappings
- [ ] egress candidates (which fields may ever leave, at what granularity)
- [ ] dashboard/KPI pack + module log events
- [ ] disableability: pause/stop/update tested, log freezes read-only

## Reporting bugs

Use the bug template; include the failing test or vector that
reproduces. Security issues: see [SECURITY.md](SECURITY.md) — do not
open public issues.
