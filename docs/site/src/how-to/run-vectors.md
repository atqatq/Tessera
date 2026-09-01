# Run the conformance vectors

The vectors under `reference/python/vectors/` are the contract between
the Python reference and the Rust implementation. Both sides must
consume the same files and agree byte-for-byte.

## Replay everything

```bash
cargo test -p tessera-access --test vectors
cargo test -p tessera-ledger --test vectors
python3 -m unittest discover -s reference/python/tests
```

The Rust harnesses replay every case and assert the same code, layer,
hash, and broken height; the Python tests replay the same files
against the reference.

## Check the vectors are current

```bash
make vectors
```

Regenerates from the reference and fails if the committed files would
change. CI runs this too — a stale vector file cannot be merged.

## Read a case

Access vectors are JSON: an `env` (clock, module state, known columns,
rules, grants, allowlists), a `request` (actor, target, action,
columns), and an `expected` decision — one of the
[fourteen codes](../reference/decision-codes.md). Ledger vectors are
entries plus expected prev/hash pairs, and tamper cases with the exact
first broken height.

## Add a case (real behaviour change only)

1. Extend `reference/python/tools/gen_vectors.py` with the scenario.
2. Regenerate: `python3 reference/python/tools/gen_vectors.py`.
3. The diff **is** the specification change — put the reason in your
   commit message.
4. Both implementations must now pass the new case; if one does not,
   you have found a real divergence, and that is the point.
