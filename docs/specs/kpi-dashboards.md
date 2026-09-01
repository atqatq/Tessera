# KPI dashboards

*SCOR platform › spokes › dashboards and KPI definitions*

> A dashboard is tenant data, not spoke code. The spoke ships defaults; the tenant rebuilds them. Any widget can reach any spoke, because every read goes through the hub anyway.

## 📘 The idea

Each spoke has its own dashboards. They are highly modifiable — tenants add widgets, define new KPIs, rearrange layouts, and pull figures from other spokes.

That last part is only possible because the platform already routes every cross-spoke read through the hub with permission and staleness attached. A dashboard widget pointing at another spoke is not a special case; it is an ordinary hub read with a chart on top.

## 🧱 Three layers

| Layer | Owner | Lifecycle |
|---|---|---|
| KPI definition | Hub master data | Versioned, effective-dated |
| Widget | Tenant | Freely edited |
| Dashboard | Tenant | Freely created, shared by role |

### KPI definition

A KPI is a `formulated` master data field. It reuses the whole machinery already specified: the expression DSL, the validation gates, cycle detection, the fan-out budget, and the `on_missing` policy.

```yaml
kpi: srm.health_index
tenant: acme_gulf
kind: formulated
type: decimal
unit: index_0_100
inputs:
  - srm.otif_pct
  - srm.quality_ppm
  - ctr.commercial_terms.penalty_exposure_usd
expression: >
  (otif_pct * 0.4)
  + ((1 - min(quality_ppm / 10000, 1)) * 100 * 0.3)
  + (price_realisation_pct * 0.3)
on_missing: hold_last
refresh: on_dependency_change
```

Because KPI definitions are effective-dated, changing a formula does not rewrite last quarter's chart. The chart for March uses March's formula. This is the single most common thing dashboard products get wrong.

### Widget

```yaml
widget:
  id: wdg_01JQ...
  type: line | bar | number | table | gauge | sparkline
  kpi: srm.health_index
  filters:
    supplier_tier: [strategic, preferred]
  window: last_12_months
  currency: reserve            # reserve | local
  on_missing: hold_last
  compare_to: previous_period
```

### Dashboard

```yaml
dashboard:
  slug: supplier-health
  spoke: srm
  tenant: acme_gulf
  origin: shipped | tenant | user
  visible_to: [SRM, SPD, PMO]
  layout: [...]
  widgets: [...]
```

`origin: shipped` dashboards come from the spoke manifest. A tenant editing one forks it to `origin: tenant`; the shipped version stays intact so a spoke update can still improve it.

## 🔀 Cross-spoke widgets

A widget on the `srm` dashboard can display `src.spend_usd_ttm`. The read goes through the hub and passes every layer:

1. **Spoke layer.** Origin must have granted `srm` read access to `src`. No grant, no widget — the widget renders as *not permitted*, distinct from *no data*.
2. **Principal layer.** The viewing user's roles must permit that specific column. Two people opening the same dashboard can legitimately see different widgets.
3. **State layer.** If `src` is paused or disabled, the value still renders, flagged stale with its timestamp.

The manifest validator warns when a shipped dashboard references another spoke's KPI, because that widget's behaviour depends on a grant the spoke cannot guarantee.

### When the source spoke is off

| Situation | Widget shows |
|---|---|
| Source spoke active | Live value |
| Source spoke paused or disabled | Last value, `stale · 14 Aug`, greyed |
| No spoke-to-spoke grant | *Not available for your organisation* |
| No column permission for this user | Widget omitted entirely |
| KPI's `on_missing` is `fail` | Widget shows an error, dashboard still renders |

A dashboard never breaks because one widget's source went away. That is the same graceful-degradation contract the spokes themselves follow.

## 🤖 Agents on dashboards

The spoke agent reads the same KPIs the dashboard does, so its findings line up with what the operator is looking at rather than a parallel set of numbers.

- Agent signals attach to the KPI they concern and surface as an annotation on the widget.
- An agent may **propose** a new KPI definition or a dashboard change. It lands in the proposal queue like any other agent output; a human applies it.
- An `act`-tier agent may refresh a KPI's value inside its allowlist. It may not create, delete, or re-scope a dashboard at any tier.

## 📊 Performance

Dashboard reads hit the analytics store, not the transactional one. KPI values are materialised on dependency change rather than computed per view — which is exactly why the fan-out budget exists. A widely referenced input feeding forty tenant dashboards is precisely the fan-out the budget is there to catch before it ships.

Cross-spoke widgets are the expensive ones. Cache per tenant with the staleness marker attached to the cache entry, so a stale cache and a stale source are indistinguishable to the viewer, which is the honest outcome.

## ⚠️ Pitfalls

- **Editing a shipped dashboard in place.** Fork on first edit, always. Otherwise the next spoke update either clobbers the tenant's work or gets skipped.
- **Assuming a fixed column set.** Column-level permission means widgets disappear per user. Layouts must reflow, not leave holes.
- **Mixing currencies in one widget.** Local or reserve, never both in the same chart. The toggle is per widget.
- **Unversioned KPI formulas.** Change the formula without effective dating and every historical chart silently moves.
- **Cross-spoke widgets on shipped dashboards.** Ship them referencing your own KPIs. Let tenants add the cross-spoke ones, where the grant is their decision.

```yaml
id: kpi-dashboards
type: feature-spec
layers: [kpi-definition, widget, dashboard]
cross-spoke: via-hub
degradation: stale-not-broken
status: 🟡 draft
```
