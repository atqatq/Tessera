# Python reference — the executable spec

This package is the executable specification of two kernel components:
the permission engine (`tessera_ref.access`) and the ledger hash chains
(`tessera_ref.ledger`). It is **stdlib-only** so the vectors it generates
are reproducible anywhere without a dependency graph.

## The contract

The committed files under `vectors/` are the contract between this
reference and the Rust implementation. Both sides consume the same files
and must produce byte-identical results:

- Python: `python3 -m unittest discover tests`
- Rust: `cargo test -p tessera-access --test vectors -p tessera-ledger --test vectors`

Regenerating vectors and getting a diff means a behavioural change on
purpose — it belongs in your commit message.

## Commands

```bash
python3 tools/gen_vectors.py          # regenerate vectors (deterministic)
python3 -m unittest discover tests    # drift guard: reference vs vectors
pip install -e .                      # install as `tessera_ref` (no deps)
```
