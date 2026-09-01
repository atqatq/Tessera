# Why Tessera

Supply chain software forces a trade most companies quietly lose.
Enterprise suites assume ten thousand SKUs, a systems-integration
budget, and a team to run it — everything smaller is underserved.
Bolt-together spreadsheets assume nothing will go wrong. Tessera
declines the trade with three commitments, each of which is enforced
by the kernel rather than promised by the marketing:

## 1. One kernel, any scale

The same spine runs a global manufacturer and a two-person workshop:
capabilities scope to the tenant, chosen during setup. The unit of
deployment is a plugin, not a microservice, so the two-person shop can
actually run it ([ADR 0006](https://github.com/atqatq/Tessera/blob/main/docs/adr/0006-plugins-not-microservices.md)).

## 2. Recommendations you can audit

When the system recommends a safety stock of 412, the recommendation
carries its method and its assumptions — which algorithm, which
history, which targets. Override it and nothing is blocked, but the
ledger keeps what was recommended, what you did instead, who you are,
and what happened afterwards. Trust is built from records, not from
confidence scores. (This is the competence thesis; it gets its own
page as the operator model lands.)

## 3. Deny wins, and history never rewrites

The permission engine assumes nobody is trusted — including agents,
including ORIGIN ([why](why-deny-wins.md)). The ledger is append-only,
hash-chained per tenant, and bitemporal: what a value *was* is a
query, not an investigation ([ADR 0003](https://github.com/atqatq/Tessera/blob/main/docs/adr/0003-three-stores-deliberately-separate.md),
[ADR 0004](https://github.com/atqatq/Tessera/blob/main/docs/adr/0004-bitemporal-everything.md)).

## The honest part

None of that is a reason to adopt software that does not exist yet.
The kernel crates are tested and the rest is specified —
[start here](../tutorials/first-suite.md) to verify that claim
yourself, and read [when not to use Tessera](when-not-to-use-tessera.md)
before deciding anything.
