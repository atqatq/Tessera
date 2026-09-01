# Permission model

*SCOR platform › hub access › column-level authorisation*

> Object-level permission is not enough. Tenants add their own columns to shared objects, and those columns carry their own sensitivity.

## 📘 Why columns, not objects

Two buyers both need `srm.supplier`. Only one of them may see `srm.supplier.negotiated_floor_usd`. If permission were evaluated at the object, the choice would be all or nothing, and the tenant would end up duplicating the object to work around it.

Because tenants define their own custom fields at runtime, the set of columns is not known at design time. The model has to handle columns it has never seen, and the default for an unknown column is deny.

## 🧱 Four layers

Every request passes all of them. Deny wins in each.

### Layer 0 — spoke state

Before anything else, the owning spoke's lifecycle state decides what is possible at all.

| State | Read | Write | Result flagged stale |
|---|---|---|---|
| `active` | yes | yes | no |
| `paused` | yes | no | yes |
| `disabled` | yes | no | yes |
| `installed` | no | no | — |
| `planned` | no | no | — |
| `archived` | no | no | — |

State is a machine fact, not a permission. Even origin cannot write a disabled spoke; it re-enables first. That keeps "what is running" and "who may act" as separate questions.

### Layer 1 — agent constraints

AI principals are constrained further than the humans they work for. This layer applies only when `principal.agent` is true.

| Tier | Read | Propose | Write |
|---|---|---|---|
| `observe` | yes | no | no |
| `advise` | yes | yes | no |
| `act` | yes | yes | only fields matching the origin-approved allowlist |

- **No agent holds origin.** A principal that is both `agent` and `origin_session` is refused with `agent_origin_forbidden`. Origin is a hardware key plus a threshold signature; there is nothing for a model to hold.
- **`propose` is the agent path.** A human using it gets `propose_is_for_agents`; humans write directly.
- **Write from a non-`act` agent** is refused with `agent_write_forbidden`, never silently downgraded to a proposal.
- **An `act` agent outside its allowlist** gets `agent_not_in_allowlist`. An empty allowlist blocks every write, which is the correct default.

Passing this layer does not grant anything. The agent still goes through layers 2 and 3 exactly as a human would, holding roles like anyone else. See `docs/ai-fabric.md`.

### Layer 2 — spoke to spoke

If the calling spoke does not own the field, origin must have granted that spoke read access to the owning spoke.

- Cross-spoke **reads** need an explicit grant.
- Cross-spoke **writes and proposals** are never permitted, grant or no grant. A spoke owns its own data, and an agent cannot propose its way around that. The leader agent routes cross-spoke work as a task instead.
- A spoke reading its own namespace skips this layer.

### Layer 3 — principal roles

The caller's roles must permit the action on that exact column. Rules use glob patterns:

```text
role SRM         allow read,write  on srm.*
role CSR         allow read,write  on ord.*
role CSR         allow read        on srm.supplier.tier
role AUD         allow read        on *
role RESTRICTED  allow read        on srm.*
role RESTRICTED  deny  read        on srm.supplier.negotiated_floor_usd
```

Deny beats allow regardless of role order. Holding both `SRM` and `RESTRICTED` gives you the intersection, not the union.

A missing grant is a deny. There is no implicit inheritance from the object to its columns.

## 🔐 Origin

An origin session bypasses layers 2 and 3 but not layer 0, and not the ledger. It is never available to an agent.

- No intent statement recorded means no action. The check returns `origin_no_intent`.
- The intent, the action and the result are all written before the effect lands.
- Origin is an account, not a role. `ORG` is never assignable.

## 📋 Decision codes

Every decision carries a machine code and a human reason, because a denial an operator cannot explain becomes a support ticket.

| Code | Layer | Meaning |
|---|---|---|
| `allowed` | — | Granted, with the granting role named |
| `origin` | — | Origin session, logged |
| `spoke_state` | 0 | Owning spoke's state forbids it |
| `agent_origin_forbidden` | 1 | A model principal claimed an origin session |
| `agent_tier` | 1 | Missing or insufficient capability tier |
| `agent_write_forbidden` | 1 | Non-`act` agent attempted a direct write |
| `agent_not_in_allowlist` | 1 | `act` agent reached outside its approved fields |
| `propose_is_for_agents` | 1 | A human used the agent proposal path |
| `no_spoke_grant` | 2 | Calling spoke has no origin grant |
| `cross_spoke_write` | 2 | A spoke may read another, never change it |
| `no_role_grant` | 3 | No held role permits it |
| `role_deny` | 3 | A held role explicitly denies it |
| `origin_no_intent` | — | Origin session without a recorded intent |
| `unqualified_field` | — | Field name is not namespaced |
| `unknown_action` | — | Action is neither read nor write |

## 🎭 Roles across the eleven spokes

Three letters, supply chain meaning, namespaced as `role.XXX` to avoid collision with spoke codes.

| Spoke | Roles |
|---|---|
| Plan | `SCP` planner, `DMP` demand, `SPP` supply |
| Source | `SRC` sourcing, `PRC` procurement |
| Transform | `MFG` production, `QAI` quality |
| Order | `OTC` order-to-cash, `CSR` customer service |
| Fulfill | `LOG` logistics, `WHM` warehouse, `TRP` transport |
| Return | `RMA` returns, `RVL` reverse logistics |
| Inventory | `INV` control, `MDM` data steward |
| Supplier relations | `SRM` manager, `SPD` development, `SQM` supplier quality |
| Contracts | `CTR` contract owner, `LGL` legal, `FIN` finance |
| Tasks | `TKO` task owner, `TMG` task manager |
| Projects | `PMO` programme office, `PRT` project team |
| Cross-cutting | `AUD` auditor (read-only, includes the ledger and master log), `TNA` tenant admin, `RSK` risk |
| Agents | `AIS` spoke agent, `AIL` leader agent — held by model principals only, never assignable to a person |

Disabling a spoke suspends its roles from the assignable list. Existing assignments are kept, never deleted, so re-enabling restores the previous state rather than requiring a re-grant.

## ⚠️ Pitfalls

- **Projections that assume a fixed column set.** Any column can be absent for a given user. Build the projection from `visible_columns`, not from a hard-coded list.
- **Treating a spoke grant as a user grant.** Both layers must pass. A spoke having read access to `srm` does not mean every user of that spoke does.
- **Leaking existence through error messages.** A denied column and a non-existent column should be indistinguishable to the caller.
- **Wildcard `allow read on *`.** Only `AUD` and `AIL` should have it, and neither should ever have write.
- **Granting an agent a human's role wholesale.** An agent holding `SRM` inherits every column `SRM` can reach. Give agents their own role with a narrower rule set, then let the tier layer narrow it further.
- **Treating `act` as a convenience.** Each allowlist entry is a standing permission for a model to change production data. It needs the same review cadence as a service account.

```yaml
id: permission-model
type: security-reference
layers: 4
default: deny
agent-tiers: [observe, advise, act]
reference-impl: reference/scor_ref/policy.py
status: 🟡 draft
```
