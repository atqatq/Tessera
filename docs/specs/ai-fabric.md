# AI fabric

*SCOR platform › hub AI core › leader and spoke agents*

> Every spoke has an analyst. The hub has the one who reads all the analysts. Neither of them commits anything a human has not approved, unless origin has said otherwise in writing.

## 📘 Shape

Two tiers, and only two.

- **Spoke agents.** One per spoke. Reads its own spoke's data, analyses it, proposes changes, optimises within its domain, assesses risk, and reports upward. It sees other spokes' data only through the hub, under the same permission rules as a human in that spoke.
- **Leader agent.** Lives in the hub AI core. Receives signals from every spoke agent, correlates across them, spots what no single spoke can see, and holds the complete oversight picture.

There is no third tier and no agent-to-agent channel. Spoke agents do not talk to each other. If `srm` needs something from `ctr`, it goes through the hub, exactly like every other cross-spoke read.

## 🤖 What a spoke agent does

| Capability | Meaning |
|---|---|
| Read | Its own spoke's data, plus whatever the hub grants it |
| Analyse | Trend, variance, outlier, and pattern detection within its domain |
| Propose | Raise a change for approval; never apply one |
| Optimise | Search its own decision space, e.g. reorder points, lane mix, buffer sizing |
| Assess risk | Score exposures and flag them with evidence |
| Report | Emit signals upward to the leader |

The agent is declared in the spoke manifest. A spoke with no `ai` block runs no agent and reports nothing.

```yaml
ai:
  enabled: true
  tier: advise
  signal_kinds: [finding, risk, proposal]
```

## 🎚️ Capability tiers

This is the control that matters most.

| Tier | Read | Propose | Write | Requires |
|---|---|---|---|---|
| `observe` | yes | no | no | nothing |
| `advise` | yes | yes | no | nothing |
| `act` | yes | yes | allowlist only | non-empty `act_allowlist` **and** recorded origin approval |

Default is `advise`. Most spokes should stay there.

`act` exists because some loops are too fast for a human — recomputing a safety stock every hour, say. It is deliberately painful to configure: the manifest must name the exact field patterns, all inside the spoke's own namespace, and origin must have approved it with a ledger reference. Unbounded autonomous write is not a supported configuration, and the validator rejects it rather than warning about it.

## 🚧 Hard limits on every agent

These hold at every tier, enforced in the permission engine rather than by convention.

- **No agent holds origin.** A principal that is both `agent` and `origin_session` is refused outright. Origin is a hardware key plus a threshold signature; there is nothing for a model to hold.
- **Agents propose, humans commit.** `write` from an `observe` or `advise` agent is refused with `agent_write_forbidden`, not silently downgraded.
- **An agent never changes another spoke's data.** Cross-spoke `propose` is refused the same way cross-spoke `write` is. The leader routes cross-spoke work as a task into the Tasks spoke, which a human or the owning spoke's agent then picks up.
- **Column denies apply to agents.** An agent inherits the full column-level rule set. If a role denies a column, the agent holding that role cannot read it either.
- **Spoke state applies to agents.** A disabled spoke's agent is disabled with it. The leader keeps its last signals, flagged stale.

## 📡 Signals

A signal is what a spoke agent sends the leader. It is a record, not a message: it lands in the hub, is logged, and stays queryable.

```yaml
signal:
  id: sig_01JQ...
  tenant: acme_gulf
  spoke: srm
  agent_tier: advise
  kind: risk                      # finding | risk | proposal | forecast | anomaly
  subject: srm.supplier.SUP-0194
  summary: single-source exposure on a rising-volume part
  severity: high
  confidence: 0.72
  evidence:
    - srm.scorecard.SUP-0194@2026-08-14
    - inv.coverage_days.SKU-40192@2026-08-14
  proposed_action: qualify a second source
  model: <model-id>
  context_hash: sha256:...
  emitted_at: 2026-08-14T09:12:04Z
```

`evidence` is mandatory and must be resolvable field references with a timestamp. A signal whose evidence cannot be resolved is rejected at the hub boundary. That single rule is what stops the leader building a picture on assertions nobody can check.

`confidence` is the agent's own, not calibrated across agents. The leader does not compare raw confidence between spokes; it ranks on severity, evidence quality, and corroboration count.

## 🧠 What the leader does

- **Deduplicate.** Three spokes noticing the same supplier problem is one issue, not three.
- **Correlate.** A `ret` quality signal, an `srm` scorecard drop and a `ctr` penalty exposure on the same supplier become one correlated finding with higher severity than any of them alone. This is the job no spoke agent can do, because no spoke agent can see the other two.
- **Rank and escalate.** Into the Tasks spoke, addressed to a role, with the evidence attached.
- **Hold oversight.** Answer "what is the state of this tenant's supply chain right now" across every enabled spoke.

What the leader does **not** do:

- Issue commands to spokes. It creates tasks and proposals; it does not reach into a spoke and change it.
- Become a dependency. No spoke may hard-require `hub.ai_core` for its core function — it requires it to *report*. A spoke whose operations stop when the leader is unavailable has been built wrong.
- Override a spoke agent's refusal. If a spoke agent lacks permission for something, the leader asking on its behalf does not create the permission.

## 📓 Everything is logged

Agent activity is a first-class action class in the master log. Each entry carries the model identifier, the tier in force, the context hash, the inputs read, the output produced, and the outcome of any proposal. See `docs/logging.md`.

An agent proposal that a human rejects is logged as thoroughly as one that is accepted. The rejection history is the only way to tell, six months in, whether an agent is actually useful or merely busy.

## ⚠️ Pitfalls

- **Tier creep.** `act` gets requested for convenience and granted for one field, then another. Review the allowlists on a schedule; the origin approval reference makes that auditable.
- **Evidence rot.** Signals reference field values at a timestamp. If those references are not resolved against the ledger they degrade into prose. Resolve at read time, not at emit time.
- **Correlation on stale signals.** A disabled spoke's last signals stay in the leader's view. They must carry `stale_as_of` into every correlated finding, or the leader will report a problem that was resolved weeks ago.
- **Confidence arithmetic.** Do not average confidences across agents. They are not on a shared scale and never will be.

```yaml
id: ai-fabric
type: architecture-spec
tiers: [leader, spoke]
capability-tiers: [observe, advise, act]
default-tier: advise
enforcement: reference/scor_ref/policy.py
status: 🟡 draft
```
