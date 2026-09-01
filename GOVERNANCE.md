# Governance

Tessera is independent and single-maintainer today. This file states
how decisions get made, how that is meant to change, and what happens
if the maintainer stops — stated plainly, because vague governance is
the kind contributors only discover when they are hurt by it.

## How decisions get made

- **Code changes**: PRs, reviewed by the maintainer (and reviewers, as
  they join). Every change passes the gates in `make check` — the same
  bar CI runs.
- **Kernel invariants** (permission layers, ledger hash, store
  separation, module contract): an RFC first — [docs/RFC_PROCESS.md](docs/RFC_PROCESS.md).
  Structural outcomes graduate into ADRs in [docs/adr/](docs/adr/).
- **Scope and roadmap**: the maintainer, with the community's input
  through Discussions and proposal issues. The roadmap lives in
  [ROADMAP.md](ROADMAP.md) and nowhere else.

## Contributor ladder

| Step | You are… | Criteria | Powers |
|---|---|---|---|
| User | running or evaluating Tessera | none | everything the licence grants |
| Contributor | PRs merged, issue triage, review comments | one substantive merged PR or two accepted reviews | PR authorship, Discussions voice |
| Reviewer | trusted to judge changes | ~3 months of consistent, review-quality contributions; deep familiarity with at least one kernel crate | approves PRs in your areas; RFC co-authorship |
| Maintainer | owns a kernel crate or module with the maintainer | nominated by the current maintainer with no unresolved objection from existing maintainers; succession-grade bus factor | merge rights, release cutting, governance votes |

Every promotion is public — a PR to this file naming the person and
the criteria met.

## The maintainer and succession

Tessera today has one maintainer: [@atqatq](https://github.com/atqatq).
That is a bus factor of one, and this paragraph is the honest plan
while it is true:

- The ADRs, RFCs, TESTING map, and conformance vectors are written so
  that a successor can reconstruct *why* without archaeology. That is
  the primary succession asset, and it is already in the repo.
- The intended long-term home is the **Eclipse Foundation**,
  conditional on (a) real adoption — multiple organisations running
  the kernel in production — and (b) a second maintainer. Eclipse
  because the dataspace work this grid must interoperate with already
  lives there (Tractus-X and the EDC ecosystem), and because its IP,
  trademark, and vendor-neutrality processes are exactly what an
  infrastructure project needs when it outgrows one person.
- If the maintainer becomes unreachable for six months with no
  delegated authority: reviewers may fork the project as `tessera-*`
  (TRADEMARK.md governs the name), and the repo states this fact in
  the README rather than letting it be discovered.

## Disputes

Technical disputes resolve in RFC/PR threads by evidence — benchmarks,
vectors, security arguments. Two reviewer-maintainers disagreeing goes
to the maintainer; a dispute *involving* the maintainer is escalated
to the Eclipse Foundation's process once the project is under Eclipse,
and until then to a neutral external reviewer chosen by the parties.
The [Code of Conduct](CODE_OF_CONDUCT.md) governs conduct, separately
from technical decisions.
