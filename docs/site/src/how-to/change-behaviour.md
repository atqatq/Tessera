# Change behaviour safely

The honest sequence, as actually used in this codebase — see
[CONTRIBUTING](https://github.com/atqatq/Tessera/blob/main/CONTRIBUTING.md)
for the full story, including the design flaw the properties caught.

## 1. Vector or test first — red

Behaviour that the vectors pin: extend `gen_vectors.py`, regenerate,
watch the Rust harness fail the new case. Behaviour internal to one
crate: write the sentence-named test first and watch it fail — for new
APIs, "fails to compile" is the red state; the test defines the API.

Never write the implementation first and backfill a test. If the code
came first, delete it and start from the test.

## 2. Green — minimum

The smallest change that makes the failing test pass. In this codebase
that discipline produced `RuleAction`: a "propose rule" was a design
flaw the property test exposed, and the fix was making it
unrepresentable — not more tests.

## 3. Refactor — tests untouched

A pure refactor changes no test. If you edited a test, it was not a
refactor; say so in the commit.

## 4. Prove it everywhere

```bash
make check
```

fmt, clippy `-D warnings`, the full suite, and vector freshness — the
same bar as CI, in one command, locally.

## 5. Write down what moved

- Behaviour change → the vector diff is in your commit, with the why.
- Kernel invariant touched → an RFC first
  ([process](../RFC_PROCESS.md)), then a superseding ADR.
- New invariant pinned → a row in [the testing map](../TESTING.md).

## What never happens

- A TODO without an issue number.
- A dependency without a justification line and a removal cost.
- A claim in the docs that no test or vector pins.
