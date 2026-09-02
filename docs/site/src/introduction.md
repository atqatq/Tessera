# Tessera documentation

Tessera is a supply chain **kernel**: a permission engine that assumes
nobody is trusted, an append-only ledger for everything that happens,
and modules that plug in and out — with every recommendation auditable
and every deviation from a recommendation recorded.

This site is organised by [Diátaxis](https://diataxis.fr/), because the
four kinds of documentation answer four different questions and mixing
them is how docs rot:

- **Tutorials** — *learning*: follow along, achieve something concrete.
- **How-to guides** — *doing*: you know what you want; here are the steps.
- **Reference** — *consulting*: exact, complete, no narrative.
- **Explanation** — *understanding*: why it is the way it is.

## The one-paragraph status

The kernel crates are real and tested: typed identifiers, the
five-layer permission engine (fourteen decision codes, deny wins,
default deny), the per-tenant hash-chained ledger, and the `inv`
safety-stock core — with a stdlib-only Python reference that must
reproduce the same committed conformance vectors byte-for-byte.
Everything else in these pages is **specified, not built**, and the
prose says so on every page where it matters. [ROADMAP](https://github.com/atqatq/Tessera/blob/main/ROADMAP.md)
separates schedule from intent.

## Where to start

- New to the code: [your first passing suite](tutorials/first-suite.md).
- Want to know whether this is for you: [why Tessera](explanation/why-tessera.md)
  and, more importantly, [when not to use Tessera](explanation/when-not-to-use-tessera.md).
- Evaluating trust: the [ADR index](adr/index.md) and the
  [invariant → test map](TESTING.md).
