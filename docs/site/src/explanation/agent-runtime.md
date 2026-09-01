# Agent Runtime

> **Design intent.** This page explains target behaviour; the README's
> status table says what is built today. Specification, not claim.

Two tiers of built-in agents, one runtime for user-defined agents, and
hard rules about who may do what.

## Built-in: module agents

Every module ships a **serious domain operator** — not a chat wrapper. It
runs its module's craft (see modules.md): TRANSFORM's agent steers the
line within finite-capacity rules; INVENTORY's agent rebalances stock
under MEIO policy; FINANCE's agent closes the loop on costing. Agents are
bounded by their L1 tier:

- `observe` — read + report only (CONTRACTS)
- `advise` — draft, rank, propose; humans commit
- `act` — execute within an allowlist + ORIGIN approval (TRANSFORM,
  FULFILL, INVENTORY, CONNECTORS). The approval is **delegable** —
  see "ORIGIN delegation" below — so the key-holder is not a
  synchronous bottleneck on every act-tier action.

Dual reporting: agents signal **up** to the leader AI *and* **brief module
users** scoped by role — dashboards, alerts, briefs. Signals carry
mandatory evidence and are ranked on severity, evidence quality, and
corroboration — never raw confidence.

## Built-in: the leader agent (AIL)

Correlates across every module agent. Raises tasks and proposals — never
commands, never holds ORIGIN. No agent-to-agent channel exists; leader
agents across companies never talk (grid rule).

## Bring your own agents

`kernel.agents` registers **user-defined agents** via:

- **MCP** — expose your agent as an MCP client; the kernel publishes tools
  per module, scoped by the agent's granted role
- **REST API** — webhook-style registration with signed callbacks

Every user agent gets: a sandboxed execution context, an explicit scope
set (modules x actions x columns), rate limits, and a full audit trail.
They rank and are ranked exactly like built-in agents. An agent that
needs to *act* needs an allowlist and ORIGIN approval — there is no other
door.

## ORIGIN delegation

As first specified, act-tier agents needed ORIGIN approval *per
action* — in practice that makes the key-holder a synchronous
bottleneck, and bottlenecks get routed around within a week. The
delegation primitive fixes the bottleneck without loosening the
model. A delegation is:

- **scoped** — module × action × column set, enumerated, never `*`;
- **time-boxed** — hard expiry, checked inclusively like every grant
  (at the expiry instant it is already expired);
- **rate-limited** — a use count and window, enforced before the
  action runs;
- **revocable** — revocation is a ledger-stamped fact; the next use
  fails closed;
- **ledger-stamped twice** — at issue and at *every* use (who, what,
  which delegation, which columns);
- **non-re-delegable** — a delegate cannot delegate. There is no
  chain, by construction.

The four deny-wins properties are the spec, written before any
implementation and pinned as pending vectors
(`reference/python/vectors/access_delegation.pending.vectors.json`;
they activate when `kernel.delegation` lands): **a delegation cannot
widen its own scope, cannot outlive its expiry, cannot survive
revocation, and cannot be used past its rate limit.** The full
rationale and the executable proptest source live in
[ADR-0012](https://github.com/atqatq/Tessera/blob/main/docs/adr/0012-origin-delegation.md).

Until the primitive ships, act-tier writes keep the per-action
approval path in `kernel.access` (`deny_origin_approval_required`) —
annoying by design, never silent.

## Guarantee

Agents hold no passwords and no origin key. Proposals are logged even
when rejected. `raise -> review -> commit` is the only path to side
effects outside an act allowlist.
