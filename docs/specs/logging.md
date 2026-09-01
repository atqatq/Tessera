# Logging

*SCOR platform › hub ledger › master log and spoke logs*

> The ledger says what a value was. The log says who did what. Two different questions, two different structures, and confusing them is how audit trails end up useless.

## 📘 Three stores, clearly separated

| Store | Answers | Owner | Shape |
|---|---|---|---|
| Ledger | What was this value on 14 March? | Hub | Bitemporal rows, hash-chained per tenant |
| Master log | Who did what, anywhere in the system? | Hub | Append-only action records, hash-chained per tenant |
| Spoke log | What exactly happened inside this spoke? | Spoke | Verbose local records, retention-bound |

The master log is the system of record for *action*. The ledger is the system of record for *state*. A field change produces one entry in each: the ledger records the new value and its validity, the master log records who changed it, why, and under what session.

## 🏛️ Master log

Every action anywhere in the system produces a master log entry. No exceptions and no opt-out — a spoke cannot suppress its own entries, because the hub writes them, not the spoke.

```yaml
entry:
  id: log_01JQ...
  tenant: acme_gulf
  seq: 84102993                    # per-tenant, monotonic, gapless
  at: 2026-08-14T09:12:04.881Z
  actor:
    kind: user | agent | origin | system
    subject: atique | srm-agent | origin | plugin_host
    roles: [SRM]
    agent_tier: advise             # agents only
    model: <model-id>              # agents only
  spoke: srm                       # or 'hub'
  action: field.write
  target: srm.supplier.SUP-0194.tier
  outcome: allowed | denied
  decision_code: allowed           # from the permission engine
  intent: "quarterly re-tiering"   # mandatory for origin
  ledger_ref: sha256:...           # when state changed
  spoke_log_ref: srm:log_01JQ...   # pointer to the verbose record
  prev_hash: sha256:...
  hash: sha256:...
```

### Action classes

| Class | Examples |
|---|---|
| `field.*` | read (sampled), write, propose |
| `master_data.*` | field defined, formula versioned, schema materialised |
| `spoke.*` | installed, enabled, paused, disabled, updated, archived |
| `access.*` | role granted, spoke-to-spoke grant issued or revoked |
| `origin.*` | session opened, intent recorded, ledger replayed |
| `agent.*` | signal emitted, proposal raised, proposal accepted or rejected, act performed |
| `ingest.*` | file accepted, rows rejected, dead-lettered |
| `dashboard.*` | created, modified, shared, deleted |

**Denials are logged as loudly as successes.** A denied read is the entry that matters most in an investigation, and it is the one systems most often drop.

Reads are sampled rather than logged individually — at a million messages a second, logging every read would be larger than the data. Writes, proposals, denials, and everything an agent does are logged in full, never sampled.

## 🧩 Spoke logs

Each spoke keeps its own log with the detail the master log deliberately omits: request payloads, intermediate calculations, retry attempts, the reasoning trace behind an agent proposal.

The relationship is **write-through, not duplication**:

1. The spoke writes the verbose record locally and gets a local id.
2. The spoke emits a compact action record to the hub.
3. The hub assigns the sequence number, chains the hash, and stores the pointer back to the spoke record.

The hub owns ordering and tamper-evidence. The spoke owns the payload. That split is what makes the throughput target survivable: hash-chaining is serial, so you chain small records per tenant, not verbose payloads globally.

### Behaviour when a spoke is disabled

- The spoke log freezes read-only. Nothing is deleted.
- Master log entries about that spoke stay live and queryable, because they are hub-owned.
- `spoke_log_ref` pointers still resolve; the payload is still there.
- New inbound events go to the hub dead-letter queue, and their arrival is itself a master log entry.

Disabling a spoke never creates a gap in the master log sequence. If it did, the chain would be unverifiable, and the whole point of chaining would be lost.

## 🔗 Verification

Entries are chained per tenant: each `hash` covers the record plus `prev_hash`. Tenant chains are rolled into a global Merkle root on a short interval.

```python
entry_hash = sha256(canonical_json(entry_without_hash) + prev_hash)
```

```excel
=IF(computed_hash = stored_hash, "ok", "BROKEN")
```

Verification walks a tenant chain and recomputes. A gap in `seq` or a hash mismatch is a hard alert, not a warning.

## 🔐 Who can read the log

The master log is a resource like any other and goes through the same permission layers.

- `AUD` (auditor) holds read on the whole master log, and nothing else in the system.
- A spoke role reads master log entries scoped to its own spoke.
- Agents read the log for the spoke they belong to. The leader agent reads across spokes only where origin has granted it, and its own reads are logged like everyone else's.
- Nobody writes the log. There is no update path and no delete path; corrections are new entries that reference the old one.

## ⚠️ Pitfalls

- **Logging the payload twice.** If the verbose record goes into the master log as well as the spoke log, the master log becomes the bottleneck and the retention bill. Pointer only.
- **Sampling the wrong things.** Sample reads. Never sample writes, denials, or agent activity.
- **Per-spoke sequence numbers.** Sequence is per tenant, not per spoke. Per-spoke sequences cannot be interleaved into a verifiable order.
- **Treating the ledger as the audit trail.** The ledger tells you the value changed. It does not tell you who, from where, under what role, or whether three denied attempts preceded it.
- **Unbounded spoke log retention.** Set it per tenant, and make the master log entry outlive the spoke record it points to. A dangling pointer with a reason is better than an unbounded store.

```yaml
id: logging
type: architecture-spec
stores: [ledger, master-log, spoke-log]
chaining: per-tenant
sampling: reads-only
status: 🟡 draft
```
