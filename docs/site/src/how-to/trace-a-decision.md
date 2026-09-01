# Trace a permission decision

You have a request in mind — "can this agent write to `inv.qty`?" —
and you want to know exactly how the engine answers it, without
guessing.

## 1. Name the actors and layers

The engine (`kernel/access`) evaluates in a fixed order. Write down
which layer your question lives in:

1. **L0** — is the target module enabled? (gates everyone, even ORIGIN)
2. **ORIGIN** — only if the actor is the origin principal; needs
   recorded intent; bypasses L2/L3 only.
3. **L1** — agents: tier vs action (read ⊂ propose ⊂ write; write needs
   allowlist + ORIGIN approval).
4. **L2** — peer reads: a covering grant, not expired at the injected
   instant.
5. **L3** — column role rules: deny beats allow, uncovered columns fall
   to default deny, unknown columns deny outright.

## 2. Reproduce it as a test

Write the request in `kernel/access/tests/spec.rs` using the existing
builders — sentence-named test, one assertion on `decision.code` and,
where it matters, `decision.layer`:

```rust
#[test]
fn act_agent_write_needs_origin_approval_even_when_allowlisted() { /* … */ }
```

If your trace disagrees with the engine, the test tells you which
layer answered differently.

## 3. Check it against the reference

The same env/request as JSON through
`tessera_ref.access.evaluate` — if the Python reference disagrees with
Rust on your case, stop and raise it: that is a contract divergence
([the vectors](run-vectors.md) are the arbiter).

## 4. Name it in the map

If your case pins a *new* invariant, add a row to
[the testing map](../TESTING.md). An invariant without a failing test
is a claim, and this project does not publish claims.
