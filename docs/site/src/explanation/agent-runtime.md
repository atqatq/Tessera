# Agent Runtime

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
  FULFILL, INVENTORY, CONNECTORS)

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

## Guarantee

Agents hold no passwords and no origin key. Proposals are logged even
when rejected. `raise -> review -> commit` is the only path to side
effects outside an act allowlist.
