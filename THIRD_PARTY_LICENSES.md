# Third-party licences

Everything in this repository is Apache-2.0 unless listed here. The
Rust allowlist is enforced in CI by `cargo deny` (`deny.toml`); this
file is the human-readable mirror. A dependency or vendored asset
added without a row here is a review failure.

## Rust dependencies

| Crate | Licence | Why it exists | Removal cost |
|---|---|---|---|
| `thiserror` | Apache-2.0 OR MIT | typed errors for the libraries (A4) | trivial: hand-write `Display`/`Error` impls |
| `sha2` | Apache-2.0 OR MIT | SHA-256 from RustCrypto — Part 0.4/0.8 forbids hand-rolled primitives (ADR 0009) | one file behind `entry_hash`; vectors prove the swap |
| `serde_json` | Apache-2.0 OR MIT | conformance-vector consumption in test targets only | test-only; delete the harnesses |
| `proptest` | Apache-2.0 OR MIT | property tests for all-inputs invariants (A2) | dev-only; delete the properties |

## Vendored assets

| Asset | Licence | Source |
|---|---|---|
| Inter (text face) | SIL OFL 1.1 | https://github.com/rsms/inter |
| JetBrains Mono (mono face) | SIL OFL 1.1 | https://github.com/JetBrains/JetBrainsMono |

Font files live under `brand/fonts/` with their OFL licence text
alongside, per the SIL licence terms.

## Python reference

The reference (`reference/python/`) is stdlib-only by design — zero
runtime dependencies — so the conformance vectors it generates are
reproducible anywhere.
