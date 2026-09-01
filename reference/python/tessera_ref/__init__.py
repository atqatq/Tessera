"""tessera_ref — the executable specification of the Tessera kernel.

Two mirrors live here, both stdlib-only:

- ``tessera_ref.access`` mirrors ``kernel/access`` (the permission engine)
- ``tessera_ref.ledger`` mirrors ``kernel/ledger`` (the hash chains)

The committed vectors under ``vectors/`` are the contract between this
reference and the Rust implementation: both must consume the same files
and produce byte-identical results. ``tools/gen_vectors.py`` regenerates
them; a diff in a commit means a behavioural change on purpose.
"""

__all__ = ["access", "ledger"]
__version__ = "0.1.0"
