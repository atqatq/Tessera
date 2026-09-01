# ADR 0010 — Structure is configurable, method is not

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer
- Context: docs/OPERATOR_MODEL.md (the competence thesis)

## Context

Tessera's adaptive spine promises industry-agnostic configuration: a
tenant morphs entities, flows, KPIs, and vocabulary during setup. The
operator model promises opinionated defaults, refusal on known-bad
configurations, and recommendations that carry a specific method.
Read carelessly, these promise opposite things — one says "the system
bends to you", the other says "the system knows better". Without an
explicit boundary, every module debate re-litigates it, and product
pressure will always push toward "just make it configurable".

## Options considered

1. **Everything configurable.** Maximum flexibility; method becomes
   tenant data. The defaults stop meaning anything (each tenant's is
   different), refusals become suggestions, and a recommendation can
   no longer name a method a practitioner could look up. The
   competence thesis collapses at all three pinned mechanisms.
2. **Nothing configurable.** Method integrity at the cost of the
   spine's reason to exist; the two-person workshop and the pharma
   distributor model the same world, which is simply false.
3. **The split: structure is configurable, method is not.** A tenant
   defines entities, flows, KPIs, and vocabulary — the shape of their
   world. Nobody redefines how safety stock is computed, what makes a
   configuration known-bad, or which arithmetic a recommendation ran.

## Decision

Option 3. Concretely:

- Configurable: entity and column definitions, module installation,
  role packs, KPI selection and layout, vocabulary, scale profile,
  policy values **within their stated bands**.
- Not configurable: algorithm selection and internals (safety stock is
  staged-service-level MEIO, full stop), the refusal rules, the
  decision codes, the ledger hash construction, the conformance
  vectors' meaning.
- The boundary itself changes only by RFC, because it is a kernel
  invariant (this ADR).

## Consequences

- "Can we ship a tenant's custom planning method?" has a standing
  answer: no — propose it as an upstream method with vectors, or run
  it outside Tessera. The answer is now policy, not a negotiation.
- Vendors get a defensible line for product decisions that sales will
  not love; this ADR is the citation.
- The adaptive spine's docs needed no softening: configuration of
  structure remains first-class, ledger-stamped, and portable.
- Method changes still happen — upstream, via vectors and RFCs, where
  both implementations must agree and the whole community inherits
  the improvement. That is the pressure valve, and it points the same
  direction for everyone.
