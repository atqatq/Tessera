# The fourteen decision codes

`kernel/access` (and its Python mirror) answer with exactly one of
these codes. Four allow, ten deny — every deny path fails closed.

## Allow

| Code | Layer | Meaning |
|---|---|---|
| `allow_origin` | origin | ORIGIN with recorded intent; bypasses L2/L3 only, never L0 |
| `allow_tier` | l1 | allowed on the agent's tier path (read at any tier; propose ≥ advise; write = act + allowlist + ORIGIN approval) |
| `allow_grant` | l2 | allowed by a covering, unexpired peer-read grant |
| `allow_rule` | l3 | allowed by column role rules, every requested column covered |

## Deny

| Code | Layer | Meaning |
|---|---|---|
| `deny_module_disabled` | l0 | the target module is disabled; gates everyone, including ORIGIN |
| `deny_intent_required` | origin | ORIGIN acted without recorded intent |
| `deny_tier_insufficient` | l1 | the agent's tier is below what the action requires |
| `deny_agent_not_allowlisted` | l1 | act-tier operation with no allowlist entry |
| `deny_origin_approval_required` | l1 | allowlisted, but no ORIGIN approval for this subject/module/action |
| `deny_grant_missing` | l2 | no grant covers this peer read (grants are column-exact; peer reads must name columns) |
| `deny_grant_expired` | l2 | a covering grant exists but is expired — checked inclusively at the injected instant |
| `deny_rule_explicit` | l3 | an explicit deny matched a requested column; deny always wins |
| `deny_column_unknown` | l3 | a requested column is not declared by the module; denied for every actor except ORIGIN |
| `deny_default` | l3 | nothing allowed it. The default is deny |

## Properties that hold across all inputs

- Deny always wins: one matching deny beats any number of allows.
- Decisions are independent of rule/grant ordering.
- Propose and write verdicts agree for users — proposals are judged by
  the write rules; there is no such thing as a propose rule.
- Tiers are monotonic: observe ⊆ advise ⊆ act on the agent surface.
- Expired grants never allow, at the expiry instant or after.
- Unknown columns deny for everyone except ORIGIN (which bypasses L3).
- ORIGIN never passes a disabled module.

Each property is a proptest in `kernel/access/tests/properties.rs` and
pinned by committed vectors; see the
[testing map](../TESTING.md).
