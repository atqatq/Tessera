# Design system brief

*SCOR platform › design › handoff spec for Claude Design*

> This is an operator's tool, not a marketing surface. Colour means state, never decoration. If a colour appears and does not encode a state, it is a bug.

Feed this whole file to Claude Design. It is written to be actionable without further context.

## 📘 Product in one paragraph

A multi-tenant supply chain platform built as a hub with eleven pluggable spokes. Users are supply chain operators: planners, buyers, warehouse managers, contract owners. They work in dense tables for hours, compare numbers across tenants and currencies, and need to know at a glance whether a number is live, stale, or frozen. Sessions are long. Screens are wide. Speed of scanning beats delight.

Each spoke also runs its own AI agent, and the hub runs a leader agent that reads all of them. Agents propose; people decide. Every dashboard is user-composable, and every action in the system is logged. Those three things drive most of what follows.

## 🎨 Visual direction

Monotone base with a single accent. Neutral greys carry the interface; colour is reserved for four semantic states plus one accent for interactive affordances. No gradients, no shadows beyond a single hairline elevation, no illustration, no rounded-everything.

The reference feeling is a well-set financial terminal: quiet, dense, legible at 13px, nothing competing for attention until something is wrong.

### Tokens

```css
:root {
  --neutral-0:   #ffffff;
  --neutral-25:  #fafafa;
  --neutral-50:  #f4f4f5;
  --neutral-100: #e8e8ea;
  --neutral-200: #d4d4d8;
  --neutral-400: #a1a1aa;
  --neutral-600: #52525b;
  --neutral-800: #27272a;
  --neutral-900: #18181b;

  --accent:        #3f6ad8;
  --accent-subtle: #eef2fd;

  --state-ok:      #1d7a55;
  --state-warn:    #a86a12;
  --state-risk:    #b3382f;
  --state-stale:   #6b6b76;

  --radius-control: 6px;
  --radius-card:    10px;
  --hairline:       1px solid var(--neutral-200);

  --space-1: 4px;  --space-2: 8px;  --space-3: 12px;
  --space-4: 16px; --space-6: 24px; --space-8: 32px;
}
```

Dark mode is not optional. Invert the neutral ramp, keep the four state hues at the same semantic meaning, lift their lightness by roughly 15% so they hold contrast on `--neutral-900`.

### Typography

| Role | Size | Weight | Use |
|---|---|---|---|
| Page title | 20px | 500 | One per screen |
| Section | 16px | 500 | Card and panel headers |
| Body | 14px | 400 | Forms, prose, labels |
| Table cell | 13px | 400 | Default table density |
| Numeric | 13px | 400 | Tabular figures, right aligned |
| Caption | 12px | 400 | Timestamps, staleness, help |

Two weights only, 400 and 500. Every number uses tabular figures and right alignment. Currency codes sit next to the number in caption size, never as a prefix symbol.

## 🧩 Component inventory

Built on shadcn/ui + TanStack. Where a shadcn primitive exists, extend it rather than replacing it.

| Component | Base | Notes |
|---|---|---|
| Proposal card | Card | An agent's proposed change: summary, evidence links, confidence, accept / reject / defer |
| Proposal queue | Sheet | Per spoke, filterable, with the rejection history visible |
| Agent badge | Badge | Marks any value or annotation an agent produced; shows tier on hover |
| Signal feed | Sheet | Leader-agent view: correlated findings across spokes, ranked by severity |
| Dashboard builder | Grid + Sheet | Drag widgets, bind to KPIs, set filters, window, currency |
| Widget config | Sheet | KPI picker (own spoke and, where granted, others), chart type, `on_missing` |
| KPI editor | Form + Dialog | Reuses the field designer; shows dependency preview and fan-out estimate |
| Log viewer | Data table | Master log for `AUD`, spoke-scoped for everyone else; denials highlighted |
| Data table | TanStack Table + shadcn Table | Column pinning, virtualised rows, server-side sort and filter, column-level visibility driven by permissions |
| KPI tile | Card | Value, delta, sparkline, freshness badge |
| Freshness badge | Badge | `live` / `stale` / `frozen` / `disabled` |
| Spoke switcher | Command | Keyboard-first, shows disabled spokes greyed with reason |
| Tenant switcher | Select | Always visible, always shows active currency |
| Currency toggle | Toggle group | Local ↔ USD reserve, never both at once |
| Time travel control | Date picker + toggle | Business time and system time as separate inputs |
| Field designer | Form + Dialog | Kind, type, unit, currency, expression, `on_missing` |
| Expression editor | Textarea + inline validate | Live parse errors with character position |
| Dependency preview | Read-only graph | Shows inputs and fan-out estimate before save |
| Role matrix | Table | Roles as rows, columns as columns, allow/deny/inherit cells |
| Task push | Sheet | Any spoke pushes into Tasks without leaving context |
| Ledger drawer | Sheet | Entry list for the current record, opens from any row |
| Empty state | Card | Distinguishes "no data" from "spoke disabled" from "no permission" |

## 🤖 Agent surfaces

The rule that governs all of them: **an operator must always be able to tell, without effort, whether a human or a model produced what they are looking at.**

- Any value, annotation, or suggestion originating from an agent carries the agent badge. Never inferred from context, never omitted for tidiness.
- Proposals are never auto-applied in the UI. Accept is an explicit act with a visible actor.
- Evidence is always one click away. A proposal without resolvable evidence links is a bug, not a design variation.
- Confidence renders as a coarse band (low / medium / high), never as a decimal. Agent confidences are not calibrated across spokes and a precise number implies otherwise.
- `act`-tier agents write directly within their allowlist. Those writes carry a distinct, persistent marker — not a transient toast — because nobody approved them in the moment.
- Rejected proposals stay visible in history. Hiding them makes it impossible to judge whether an agent is worth its noise.

Do not design an agent persona, avatar, or conversational surface. This is an analyst tool, not an assistant.

## 📓 Log surfaces

- The log viewer is a dense table, not a timeline. Operators filter and scan; they do not scroll through narrative.
- Denials render in `--state-warn` and are filterable in one click. They are the entries investigations start from.
- Every log row links to its ledger entry where state changed, and to the spoke log record for the verbose payload.
- Sequence gaps and hash mismatches render in `--state-risk` at the top of the view, not buried in a row.
- Agent entries show the model identifier and tier as columns, not as hover detail.

## 🚦 State semantics

This is the part that matters most and the part generic design systems get wrong.

| State | Colour | Badge | Meaning to the operator |
|---|---|---|---|
| Live | none | none | Value is current. Absence of decoration is the signal. |
| Stale | `--state-stale` | `stale · 14:02` | Value is real but the source is paused or disabled. Show the timestamp always. |
| Frozen | `--state-stale` | `frozen` | Historical view via time travel. Whole screen gets a top border in the same tone. |
| Warning | `--state-warn` | contextual | Threshold breached, action optional. |
| Risk | `--state-risk` | contextual | Threshold breached, action required. |
| Healthy | `--state-ok` | contextual | Only where the absence of a problem is itself the news. |

Rules:

- A stale value is never hidden and never silently substituted. Show the number and show its age.
- Time travel changes the entire viewport chrome, not one widget. An operator must never mistake a historical screen for a live one.
- Disabled spokes stay visible in navigation, greyed, with the reason on hover. Hiding them makes users think the feature was removed.
- Never use colour alone. Every state carries a text label or an icon as well.
- A widget whose source spoke has no grant shows *not available for your organisation* — visibly different from *no data yet*. Conflating the two sends people to support.
- A widget the viewer lacks column permission for is omitted entirely and the layout reflows. No empty frame, no lock icon: its absence is not their business.

## 📐 Layout

Three regions. A fixed left rail for spoke navigation, a top bar for tenant, currency, time travel and origin indicator, and the work area.

- Work area max width: none. These are wide screens with wide tables.
- Table density default: comfortable at 36px rows, with a compact 28px option persisted per user.
- Forms: single column, 480px max. Two-column forms slow down accurate entry.
- Modals for destructive confirmation only. Everything else is a sheet or an inline panel.
- The origin indicator is a persistent top-bar element when an origin session is active, in `--state-risk`, with the recorded intent visible. It cannot be dismissed.

## ♿ Accessibility

- Contrast: 4.5:1 minimum for all text, including the 12px caption tier and all four state hues in both modes.
- Keyboard: every table action reachable without a mouse. Command palette on `⌘K`.
- Focus: 2px `--accent` ring, never removed, never relying on colour change alone.
- Motion: transitions capped at 150ms, all wrapped in `prefers-reduced-motion`.
- Screen readers: staleness and permission-denied states announced, not conveyed only visually.

## 📦 What to hand back

1. Token file as CSS custom properties and as a JSON file for the Tailwind config.
2. Component specs for the inventory above, each with default, hover, focus, disabled, loading, empty, and error states.
3. Five worked screens: the Inventory spoke dashboard, the dashboard builder mid-edit with a cross-spoke widget, the field designer with a live expression error, the proposal queue with one agent proposal expanded, and the role matrix.
4. A dark mode pass of all five screens.
5. A one-page state semantics reference the engineering team can pin up, including the agent-provenance rules.

## ⚠️ Constraints to respect

- Column-level permissions mean any table column can be absent for a given user. Layouts must not assume a fixed column set.
- Tenants add custom fields, so tables have unknown column counts at design time. Design for horizontal overflow as the normal case.
- Numbers appear in local currency and USD reserve. Never show both in the same cell.
- Eleven spokes, of which any subset may be disabled. Navigation must look correct with three spokes and with eleven.
- Dashboards are user-composed, so no layout can assume a known widget count or arrangement. Design the empty dashboard and the forty-widget dashboard.
- A spoke may run no agent at all. Every agent surface must degrade to simply not being there, without leaving a gap in the shell.

```yaml
id: design-system-brief
type: design-handoff
target: claude-design
stack: [react, typescript, tailwind, shadcn-ui, tanstack]
surfaces: [dashboards, agent-proposals, log-viewer]
status: 🟡 draft
```
