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
  reference must agree. The full map lives in [docs/TESTING.md](docs/TESTING.md).

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
[docs/RFC_PROCESS.md](docs/RFC_PROCESS.md). Structural decisions
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
