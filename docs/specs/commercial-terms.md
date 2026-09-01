# Commercial terms object

*SCOR platform › hub master data › `ctr.commercial_terms`*

> Contracts owns the negotiation. The hub owns the answer to "what price was in force on 14 March". Those are different jobs, so they live in different places.

## 📘 Why this object exists

Five spokes need contract terms. Source needs prices. Order needs rebate tiers. Fulfill needs incoterms and lead times. Return needs warranty windows. Finance needs penalty exposure.

If they all read the Contracts spoke directly, then disabling Contracts breaks five spokes and the independence rule is dead. So Contracts publishes an immutable, effective-dated version into hub master data, and everyone else reads that.

The Contracts spoke is the only writer. Every other spoke is a reader. The hub enforces both halves.

## 🧱 Structure

| Field | Type | Notes |
|---|---|---|
| `terms_id` | uuid | Stable across versions of the same contract |
| `tenant_id` | uuid | Terms never cross a tenant boundary |
| `contract_ref` | string | Human reference, e.g. `CT-2026-0188` |
| `supplier_id` | uuid | Resolves to `srm.supplier` when SRM is installed |
| `version` | integer | Monotonic per `terms_id`, starts at 1 |
| `status` | enum | `active`, `superseded`, `expired`, `terminated` |
| `valid_from` | date | Business time: when the terms take commercial effect |
| `valid_to` | date, nullable | Null means open-ended |
| `tx_from` | timestamp | System time: when the hub learned this |
| `tx_to` | timestamp, nullable | Null means current record |
| `currency` | iso4217 | Contract currency |
| `fx_rate_to_usd` | decimal | Rate captured at publication, never recalculated |
| `incoterm` | enum | Incoterms 2020 code |
| `payment_terms_days` | integer | Net days |
| `lead_time_days` | integer | Contractual, not observed |
| `price_lines` | array | See below |
| `rebate_tiers` | array | Threshold and percentage pairs |
| `sla` | array | Metric, target, measurement window |
| `penalties` | array | Trigger, formula reference, cap |
| `warranty_days` | integer | Feeds the Return spoke |
| `renewal` | object | Mode, notice days, auto-renew flag |
| `published_by` | principal | The Contracts spoke principal that published |
| `ledger_ref` | hash | Entry in the tenant hash chain |

### Price line

```yaml
price_lines:
  - sku: SKU-40192
    uom: case
    min_qty: 1
    max_qty: 499
    unit_price: 12.400
    currency: KWD
    unit_price_usd: 40.390        # captured at publish, frozen
  - sku: SKU-40192
    uom: case
    min_qty: 500
    max_qty: null
    unit_price: 11.900
    currency: KWD
    unit_price_usd: 38.760
```

Price bands must not overlap and must not leave gaps. The hub rejects a publication that does either, because a gap becomes a runtime "no price found" during order entry.

## ⏳ Effective dating

The object is bitemporal. Two independent axes:

- **Business time** (`valid_from` / `valid_to`) — when the terms applied commercially.
- **System time** (`tx_from` / `tx_to`) — when the platform knew about them.

The two differ constantly. A contract signed on 20 March backdated to 1 March has `valid_from = 2026-03-01` and `tx_from = 2026-03-20`. Both questions are answerable:

- *What price applied on 5 March?* → business time query.
- *What price did we think applied on 5 March, as at 10 March?* → system time query. This is the one auditors ask.

### Resolution rule

```python
def resolve_terms(rows, sku, as_of_business, as_of_system=None):
    """Return the single price line in force. Deterministic, no I/O."""
    candidates = [
        r for r in rows
        if r["valid_from"] <= as_of_business
        and (r["valid_to"] is None or r["valid_to"] > as_of_business)
        and (
            as_of_system is None
            or (r["tx_from"] <= as_of_system
                and (r["tx_to"] is None or r["tx_to"] > as_of_system))
        )
    ]
    if not candidates:
        return None
    return max(candidates, key=lambda r: (r["valid_from"], r["version"]))
```

```excel
=INDEX(price_usd,
   MATCH(1,
     (sku_col = target_sku) *
     (valid_from <= as_of) *
     ((valid_to = "") + (valid_to > as_of)),
   0))
```

Exactly one row must match. Two matches means overlapping validity, which is a publication bug, and the hub raises rather than picking one.

## 🔌 Behaviour when Contracts is disabled

| Aspect | Behaviour |
|---|---|
| Existing terms | Readable, frozen at last published version |
| New versions | Cannot be published; there is no writer |
| Reads by other spokes | Succeed, flagged `source_state: disabled` and `stale_as_of` |
| Expiry | Still processed by the hub from `valid_to`; expiry is a date, not a workflow |
| Time travel | Unaffected; the ledger is hub-owned |

No other spoke changes behaviour. That is the whole point of publishing into the hub rather than exposing a Contracts API.

## 🤖 Agents and terms

The Contracts agent is `observe`-tier by default and should stay there. Published terms are a legal artefact: an agent may flag a renewal exposure, a price band gap, or an SLA breach pattern, and it may propose an amendment for a human to negotiate. It never publishes a version.

Other spokes' agents read `commercial_terms` from the hub like any other principal, subject to their spoke-to-spoke grant. That is how the Source agent can notice it is paying above the contracted band without Contracts being involved at all.

Every terms publication and every agent signal referencing terms is a master log entry with the `ledger_ref` attached. See `docs/logging.md`.

## ⚠️ Pitfalls

- **Recalculating `fx_rate_to_usd` on read.** Never. The rate is captured at publish. Recalculating makes every historical report move, and reconciliation becomes impossible.
- **Letting a spoke read `ctr.*` directly.** Block it at the gateway, not by convention. Once one team does it the guarantee is gone and nobody notices until Contracts is disabled.
- **Letting an agent publish.** An `act`-tier allowlist must never include `ctr.commercial_terms.*`. The validator cannot know this is a legal artefact; you do.
- **Mutable versions.** A published version is immutable. A correction is a new version with a new `tx_from`, and the old one gets a `tx_to`. Editing in place destroys the audit answer.
- **Overlapping price bands.** Validate at publish. A gap or overlap discovered at order entry is a production incident.

```yaml
id: commercial-terms
type: object-definition
owner-spoke: ctr
readers: [src, ord, ful, ret, inv]
temporality: bitemporal
status: 🟡 draft
```
