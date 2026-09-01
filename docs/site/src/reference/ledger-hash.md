# Ledger hash construction

The ledger commits every record with one function, identical in Rust
(`tessera_ledger::entry_hash`, using the audited `sha2` crate) and in
the stdlib-only Python reference (`hashlib`):

```text
entry_hash = SHA-256( "tessera-ledger/1" || tenant || 0x00 || prev
                      || u64be(height) || u64be(valid_ms)
                      || u64be(system_ms) || u32be(payload_len)
                      || payload )
```

- The domain string separates these hashes from any other SHA-256 use.
- The **tenant** is inside the hash: chains from different tenants can
  neither be spliced together nor correlated by hash equality —
  identical event sequences under two tenants produce different hashes.
- `prev` is the previous record's hash; genesis anchors to the zero
  hash (`0x00 × 32`).
- Timestamps are big-endian u64 milliseconds: `valid_ms` (when the
  fact was true) and `system_ms` (when the kernel learned it) —
  bitemporal by construction (ADR 0004).
- `payload_len` is a big-endian u32 guard against length ambiguity;
  payloads are opaque bytes the ledger never interprets.

## Verification

`verify(records)` recomputes the whole chain from genesis. On any
mismatch — wrong height, broken link, or a hash that does not
recompute — it returns the **first broken height**, so a verifier can
quarantine exactly where history was tampered with. The tamper
properties pin this: any single-byte change to a payload, a stored
hash, or a prev-link is caught at exactly the height where it
happened.

## Policy

The digest comes from an audited crate — never from this repository.
See [ADR 0009](https://github.com/atqatq/Tessera/blob/main/docs/adr/0009-cryptographic-primitives.md)
for the dependency decision, its honest audit-status caveat, and why
the vectors make a future swap a one-file change.
