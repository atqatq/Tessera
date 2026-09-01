# 08 — `Display` doctest for a ledger record

- Labels: `good-first-issue`, `area-ledger`, `area-docs`
- Size: small (one doctest)

## Why this matters

`Record` implements `Display` (`kernel/ledger/src/lib.rs`), but its
doc comment shows no output. `missing_docs` forces documentation to
*exist*; only doctests force it to be *true*. A runnable example that
shows the exact `tenant#height prev=… hash=…` shape is the cheapest
possible documentation that cannot rot.

## Acceptance criteria

- The `impl Display for Record` doc comment gains a doctest that
  builds a one-record chain (tenant `acme`, payload `order#1`), prints
  it, and asserts the exact rendered string.
- The hash shown is the real one — compute it by running the test,
  then paste it (that is the point: the test pins it).
- `cargo test --doc -p tessera-ledger` passes; no other test touched.

## Where to start

The `Display` impl and the `Chain::new`/`append` API two screens
above. Keep the doctest unwraps — doctests are allowed them.

## Definition of done

Green CI; the doctest output is byte-exact; commit message notes that
the example is the first pinned rendering of a record.
