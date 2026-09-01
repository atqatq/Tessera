# ADR index

Architecture Decision Records live in the repository at
[`docs/adr/`](https://github.com/atqatq/Tessera/tree/main/docs/adr).
An ADR is immutable once accepted; superseding one requires a new ADR
that says why the old one was wrong.

| ADR | Decision |
|---|---|
| [0001](https://github.com/atqatq/Tessera/blob/main/docs/adr/0001-record-every-non-trivial-decision.md) | Record every non-trivial decision |
| [0002](https://github.com/atqatq/Tessera/blob/main/docs/adr/0002-star-not-mesh.md) | Star topology, not mesh |
| [0003](https://github.com/atqatq/Tessera/blob/main/docs/adr/0003-three-stores-deliberately-separate.md) | Three stores, deliberately separate |
| [0004](https://github.com/atqatq/Tessera/blob/main/docs/adr/0004-bitemporal-everything.md) | Bitemporal everything |
| [0005](https://github.com/atqatq/Tessera/blob/main/docs/adr/0005-rust-plus-python-reference.md) | Rust kernel plus a stdlib-only Python reference |
| [0006](https://github.com/atqatq/Tessera/blob/main/docs/adr/0006-plugins-not-microservices.md) | Plugins, not microservices |
| [0007](https://github.com/atqatq/Tessera/blob/main/docs/adr/0007-deny-wins.md) | Deny wins, default deny, everywhere |
| [0008](https://github.com/atqatq/Tessera/blob/main/docs/adr/0008-vectors-are-the-contract.md) | Vectors are the contract |
| [0009](https://github.com/atqatq/Tessera/blob/main/docs/adr/0009-cryptographic-primitives.md) | Cryptographic primitives come from audited crates |

Kernel invariant changes start as an RFC:
[the process](https://github.com/atqatq/Tessera/blob/main/docs/site/src/RFC_PROCESS.md).
