"""Mirror of kernel/ledger — per-tenant append-only hash chains.

Same hash construction, byte for byte, as the Rust implementation:

    SHA-256("tessera-ledger/1" || tenant || 0x00 || prev
            || u64be(height) || u64be(valid_ms) || u64be(system_ms)
            || u32be(payload_len) || payload)

The digest uses hashlib (OpenSSL-backed in CPython); the Rust side uses
the audited ``sha2`` crate. Neither side ever hand-rolls a primitive.
"""

from __future__ import annotations

import hashlib

DOMAIN = b"tessera-ledger/1"
GENESIS_PREV = bytes(32)


def entry_hash(tenant: str, prev: bytes, height: int, valid_ms: int,
               system_ms: int, payload: bytes) -> bytes:
    """The one hash function both implementations must agree on."""
    h = hashlib.sha256()
    h.update(DOMAIN)
    h.update(tenant.encode("utf-8"))
    h.update(b"\x00")  # tenant boundary separator
    h.update(prev)
    h.update(height.to_bytes(8, "big"))
    h.update(valid_ms.to_bytes(8, "big"))
    h.update(system_ms.to_bytes(8, "big"))
    h.update(len(payload).to_bytes(4, "big"))
    h.update(payload)
    return h.digest()


def build_chain(tenant: str, entries: list[dict]) -> list[dict]:
    """Append entries (height, valid_ms, system_ms, payload_hex) in order.

    Returns records shaped like the vectors: prev_hex / hash_hex pairs.
    Heights must be contiguous from zero — the reference does not model
    conflict errors, only the honest chain.
    """
    records: list[dict] = []
    prev = GENESIS_PREV
    for i, e in enumerate(entries):
        height = e["height"]
        if height != i:
            raise ValueError(f"chain gap: expected height {i}, got {height}")
        payload = bytes.fromhex(e["payload_hex"])
        digest = entry_hash(tenant, prev, height, e["valid_ms"],
                            e["system_ms"], payload)
        records.append({
            "prev_hex": prev.hex(),
            "hash_hex": digest.hex(),
            "height": height,
        })
        prev = digest
    return records


def verify(records: list[dict], tenant: str, entries: list[dict]) -> int | None:
    """Recompute the chain; return the first broken height or None.

    ``records`` may have been tampered with; ``entries`` are the tampered
    entries as stored (the tamper is already applied by the generator, so
    verification sees the tampered world exactly as a verifier would).
    """
    prev = GENESIS_PREV
    for i, (record, entry) in enumerate(zip(records, entries)):
        payload = bytes.fromhex(entry["payload_hex"])
        digest = entry_hash(tenant, prev, entry["height"], entry["valid_ms"],
                            entry["system_ms"], payload)
        if (entry["height"] != i
                or bytes.fromhex(record["prev_hex"]) != prev
                or bytes.fromhex(record["hash_hex"]) != digest):
            return i
        prev = bytes.fromhex(record["hash_hex"])
    return None
