# The Grid: Federation & External Egress

The inter-company layer. Two trust models, never conflated:

- **Inside a company** the kernel is trusted: five permission layers, duplex
  bus, ledger.
- **Across the grid** nobody trusts anybody's kernel: sharing contracts,
  notarization, and benchmark gates.

## External egress — every module's capability

Any module may expose **select data to external kernels and modules** — beyond
kernel & peer grants. Egress is not a data dump; it is contract-shaped:

- **Sharing contract**: versioned, signed by both parties, notarized.
  One clause per field + direction.
- **Granularity**: exact - banded - aggregate. Default is opacity;
  disclosure is clause-scoped.
- **Consent**: per-metric, revocable — retroactively.
- **Writes**: cross-party write does not exist as an operation. Changes
  travel as requests a human on the other side accepts.
- **Agents**: may read across a boundary, never act. Leader agents never
  talk to each other.

## Node tiers

- **Full node** — your company: this kernel + your modules.
- **Light node (free)** — one identity, docs, orders, scorecards; a
  supplier onboards once and is reused by every buyer. Light -> full
  migrates nothing.
- **Observer tier** — auditor/regulator, time-boxed, clause-scoped.

## Notarization (anchor, never store)

`kernel.notary` anchors salted Merkle roots to external consensus
(Hedera today, swappable): fair ordering, consensus timestamps, seconds
finality. `submit(topic, root) -> receipt`. Tiers: none - local chain -
anchored (batched root) - attested (per-event). A node runs fully with
the notary unreachable.

## Benchmark network

Cross-company metrics with gates enforced in code, not policy: per-metric
consent, k-anonymity (k = 5 parties), no party above ~1/3 of a sample;
publishes p10/p50/p90 + party count only — never names, never ranks.
