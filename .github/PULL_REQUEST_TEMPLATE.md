## What

<!-- one paragraph: what changes and why -->

## Which rule does it touch?

- [ ] kernel-service dependency rule (module requires = kernel.* only)
- [ ] permission engine (deny wins)
- [ ] append-only ledger / master log
- [ ] design tokens (monotone + one accent)
- [ ] none of the above

## Conformance vectors

- [ ] vectors added/updated under `reference/python/vectors/` (and regenerated cleanly)
- vector ids:

## Gates (`make check` mirrors CI)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `python3 -m unittest discover -s reference/python/tests`
- [ ] docs updated (including the TESTING.md invariant map if an invariant changed)

## Contribution terms

- [ ] every commit is DCO signed off (`git commit -s`)
- [ ] contributions are Apache-2.0 (no CLA — see LICENSING.md)
