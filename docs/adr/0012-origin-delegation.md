# ADR 0012 — ORIGIN delegation: scoped, time-boxed, rate-limited, revocable, ledger-stamped, non-re-delegable

- Status: accepted
- Date: 2026-09-02
- Deciders: maintainer
- Requires: ADR-0007 (deny wins), the frozen access vectors (ADR-0008)
- Changes: the act-tier approval path described in AGENT_RUNTIME

## Context

As specified, act-tier agents needed ORIGIN approval *per action*.
That makes the key-holder a synchronous bottleneck: every write by
TRANSFORM, FULFILL, INVENTORY, or CONNECTORS queues behind a human.
In practice that kills the automation — or gets routed around, which
is worse, because a routed-around control leaves no record. The
choice is not "bottleneck or loosen"; it is "bottleneck or design the
delegation properly".

## Options considered

1. **Keep per-action approval.** Safe and unusable; the control gets
   routed around and the record disappears.
2. ** blanket pre-approval** (allowlist grants standing authority) —
   the allowlist already exists in the engine, but making it
   sufficient removes ORIGIN from the loop entirely; the approval
   would be a rubber stamp at allowlist-editing time, with no
   per-use stamping.
3. **Delegation primitive**: scoped (module × action × column set),
   time-boxed, rate-limited, revocable, ledger-stamped at issue and
   at every use, non-re-delegable.

## Decision

Option 3. The primitive, specified now and built later (Part 0.3:
this pass specifies; E1's cycle was the one build):

- A delegation names one subject, one module, one action, and an
  explicit column set — never `*`. Scope is a subset of what the
  allowlist permits; it can never exceed it.
- Expiry follows the L2 grant rule exactly: inclusive instant, fail
  closed — at `expires_at` the delegation is already expired.
- A use budget (`max_uses`) is enforced before the action; the count
  lives with the delegation and increments only on successful,
  ledger-stamped use.
- Revocation is a ledger-stamped fact; the *next* use fails with
  `deny_delegation_revoked`. No caching horizon is permitted — the
  check reads committed state.
- Every use stamps the ledger: delegation id, subject, module,
  action, columns, and the request's outcome. Issue is stamped too.
- Delegation chains do not exist. A delegate is a subject bound at
  issue; it cannot hand the delegation on, and the code has no way
  to express it (the delegation's subject is fixed at creation).

### The four deny-wins properties (the spec, written first)

1. A delegation cannot widen its own scope — a request outside the
   enumerated columns/module/action is refused
   (`deny_delegation_scope`), even when the underlying rules would
   allow it.
2. A delegation cannot outlive its expiry —
   `deny_delegation_expired` at and after the instant.
3. A delegation cannot survive revocation —
   `deny_delegation_revoked` from the revocation onward.
4. A delegation cannot be used past its rate limit —
   `deny_delegation_exhausted` once the budget is spent.

Pinned as committed data in
`reference/python/vectors/access_delegation.pending.vectors.json`
(status: pending-implementation — neither implementation consumes it
yet), and to be replayed byte-identically by both when
`kernel.delegation` lands. The Rust proptest source that must ship
with the implementation:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// A delegation cannot widen its own scope: for every env and every
    /// request outside the delegation's enumerated (module, action,
    /// columns), the decision is a delegation deny — never an allow.
    #[test]
    fn delegation_never_widens_scope(/* env, delegation, off-scope request */) {
        // assert: matches!(code, DenyDelegationScope | DenyDefault | ..)
        //        && !matches!(code, AllowDelegation | AllowTier)
    }

    /// A delegation at or past its expiry never allows, at any instant.
    #[test]
    fn delegation_never_outlives_expiry(/* now >= expires_at */) {
        // assert: code == DenyDelegationExpired
    }

    /// A revoked delegation never allows again, at any later instant.
    #[test]
    fn delegation_never_survives_revocation(/* revoked, any now */) {
        // assert: code == DenyDelegationRevoked
    }

    /// A delegation with uses >= max_uses never allows, whatever else
    /// is true.
    #[test]
    fn delegation_never_exceeds_its_rate_limit(/* uses >= max_uses */) {
        // assert: code == DenyDelegationExhausted
    }
}
```

(Commented placeholders, not compiled code — the types do not exist
yet, and a test that cannot compile cannot guard anything. The
vectors above are the compiled-equivalent: data the implementation
must satisfy.)

## Consequences

- The key-holder signs delegations, not actions: synchronous
  bottleneck → asynchronous, auditable grant. The worst case is a
  delegation that should never have been issued — visible at issue,
  bounded in scope, time, and uses, and revocable in one step.
- `deny_origin_approval_required` remains the path for un-delegated
  act writes; nothing loosens until the primitive ships.
- Two new facts per delegation issue and one per use lands on the
  ledger; volume is bounded by the rate limits themselves.
- The delegation cache some future maintainer will propose is
  forbidden by the revocation rule — reads must see committed state.
  This ADR is the citation for that "no".
