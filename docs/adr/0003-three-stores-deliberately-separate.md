# ADR 0003 — Three stores, deliberately separate

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

The kernel must answer three different questions: *what was a value*,
*who did what*, and *what happened in detail*. One store serving all
three either bloats the audit trail with payloads or starves the audit
of evidence.

## Options considered

1. One unified event store.
2. Two stores: ledger plus a log.
3. Three separate stores: **ledger** (bitemporal, hash-chained per
   tenant — the system of record for state), **master log** (gapless
   sequence of who did what, kernel-owned ordering), **module logs**
   (verbose payloads, written through pointers into the master log).

## Decision

Option 3. Each store has one writer shape, one retention profile, and
one query surface. A pointer in the master log references the module
log entry that carries the payload; nothing is duplicated.

## Consequences

- Tamper-evidence applies where it matters: the ledger chain (see
  `tessera-ledger`) is hash-chained per tenant; the master log is
  gapless by construction.
- Verbose payloads never bloat the ledger; retention differs per store
  by policy, not by accident.
- Three stores must be reconciled on read; the pointers make that a
  join, not a guess. Reconciliation tests are part of the kernel's
  contract (Part F.4).
