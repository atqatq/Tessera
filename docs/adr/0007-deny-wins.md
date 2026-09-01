# ADR 0007 — Deny wins, default deny, everywhere

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer

## Context

Every permission system has gaps between rules. Two postures exist:
allow by default and patch the gaps with deny rules, or deny by default
and require positive permission. The first fails open under change; the
second fails closed.

## Options considered

1. Default allow with deny rules for exceptions.
2. Default deny with explicit allows; an explicit deny always beats any
   allow on the same column; a column the module does not declare is
   denied for every actor except ORIGIN.

## Decision

Option 2, structurally, not by convention:

- The engine (`kernel.access`, mirrored by the Python reference) has no
  code path that returns allow without positive evidence.
- Unknown columns deny before rules are even consulted.
- L0 (module state) gates everyone, including ORIGIN.
- Peer reads pass an L2 grant *and* the owning company's L3 rules;
  passing one layer grants nothing by itself.

## Consequences

- New features fail closed until someone writes the rule — friction by
  design, and the right side to err on for infrastructure.
- The invariants are property-tested across all inputs and pinned in
  conformance vectors (order independence, deny-wins, tier
  monotonicity, expiry fail-closed at the boundary instant).
- Legitimate broad access (audit, break-glass) must be modelled
  explicitly as roles and rules — ORIGIN exists for the remainder, and
  every ORIGIN action records intent before effect.
