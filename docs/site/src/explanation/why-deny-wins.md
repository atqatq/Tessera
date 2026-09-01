# Why deny wins

Every permission system has gaps between its rules. There are exactly
two postures about those gaps, and only one of them survives contact
with change:

- **Default allow** means a new module, a new column, or a refactor
  that renames a rule starts life fully open until someone writes the
  exception. The failure mode is silent: nothing tells you the gap
  exists until someone walks through it.
- **Default deny** means the same new thing starts life closed, and
  the first failed access is the notification. Friction is the
  feature: it converts an invisible gap into a visible request.

Tessera enforces this structurally, not by convention — the engine has
no code path that returns allow without positive evidence. The full
statement of the layers and their properties is
[ADR 0007](https://github.com/atqatq/Tessera/blob/main/docs/adr/0007-deny-wins.md);
the properties that hold across all inputs are listed with the
[decision codes](../reference/decision-codes.md).

Two consequences are worth naming honestly:

1. Legitimate broad access must be modelled explicitly — auditor
   roles, break-glass rules — instead of being improvised. ORIGIN
   exists for the remainder, and every ORIGIN action records intent
   before effect and never bypasses module state or the ledger.
2. The system is annoying in exactly the direction that protects the
   people whose data it holds. A permission engine that is convenient
   to extend loosely is a liability with good documentation.

## What "deny wins" adds on top

Default deny alone is not enough: two allow rules must never outvote
one deny rule. In Tessera an explicit deny on any requested column
denies the whole request, whatever allows exist. Rules are
order-independent — shuffling them cannot change a decision — because
the outcome is computed from set semantics, not from first-match
scanning. Both properties are proptested across generated inputs, not
asserted on examples.
