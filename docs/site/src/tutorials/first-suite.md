# Your first passing suite

Goal: from a fresh clone to a fully passing test suite, on any machine,
in under two minutes. Everything on this page is executed by CI; if a
command here stops working, CI fails and we fix the page.

## 1. Clone and enter

```bash
git clone https://github.com/atqatq/Tessera.git
cd Tessera
```

## 2. Set up

```bash
make setup
```

This installs the pinned Rust toolchain (via rustup, if cargo is
missing) and installs the Python reference as an editable package.
The reference has **zero runtime dependencies** — stdlib only — so
there is nothing else to fetch.

## 3. Run the suite

```bash
make test
```

You should see the Rust suites pass (`tessera-ids`, `tessera-access`,
`tessera-ledger`, `tessera-inv`) followed by the Python reference
unittests. Around sixty tests, well under two minutes on a laptop.

## 4. Prove the contract to yourself

```bash
make vectors
```

This regenerates the conformance vectors from the Python reference and
checks that regeneration produced **no diff** against the committed
files. The vectors are the contract between the two implementations
([why](../explanation/why-tessera.md)); a diff here means behaviour
changed on purpose, and the commit that changed it has to say so.

## What you just ran

- `cargo test --workspace` — unit, property, and vector-replay tests
  for every kernel crate.
- `python3 -m unittest discover -s reference/python/tests` — the
  reference's own drift guards.
- `gen_vectors.py` + `git diff --exit-code` — the freshness gate.

## Where to next

- Change some behaviour safely: [change behaviour](../how-to/change-behaviour.md).
- Understand why the permission engine cannot answer "allow" without
  evidence: [why deny wins](../explanation/why-deny-wins.md).
