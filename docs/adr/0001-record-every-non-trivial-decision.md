# ADR 0001 — Record every non-trivial decision

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

Tessera's specification documents make strong claims. Without a decision
record, a reader cannot tell which claims are chosen policy, which are
defaults nobody examined, and which are aspirations. Contributors re-litigate
settled questions because the settlement is nowhere written down, and the
reasons travel with nobody.

## Options considered

1. Decisions live in chat and commit messages — free, but unsearchable and
   unattributed.
2. A single DESIGN.md — becomes a dumping ground; entries cannot be
   individually accepted or superseded.
3. Architecture Decision Records, one file per decision, numbered,
   immutable once accepted, superseded explicitly.

## Decision

Option 3. Every non-trivial decision gets an ADR in `docs/adr/NNNN-title.md`
with the fields this file uses: context, options considered, decision,
consequences. ADRs are never edited after acceptance except to change
Status to superseded, with a link to the replacement.

## Consequences

- New contributors can read ten short files and know why the kernel looks
  the way it does, without archaeology.
- Changing a settled invariant means writing an ADR that says why the old
  one was wrong — a deliberate speed bump, and that is the point.
- Anything NOT recorded here is not a settled decision, however confident
  its prose sounds elsewhere.
