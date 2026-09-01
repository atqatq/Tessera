"""The Python reference must reproduce the committed ledger vectors.

These tests are the drift guard on the reference side: if gen_vectors.py
output and the reference ever disagree, this fails. The Rust side runs
the same files — that is the cross-implementation contract.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from tessera_ref import ledger as led  # noqa: E402

VECTORS = HERE.parent / "vectors" / "ledger.vectors.json"


def _flip(hexstr: str, byte: int) -> str:
    raw = bytearray(bytes.fromhex(hexstr))
    raw[byte % len(raw)] ^= 0x01
    return raw.hex()


def _apply_tamper(records: list[dict], entries: list[dict],
                  kind: str, index: int, byte: int) -> tuple[list[dict], list[dict]]:
    """Mirror of the generator's tamper application — same order, same bytes."""
    stored = [dict(r) for r in records]
    stored_entries = [dict(e) for e in entries]
    if kind == "payload":
        raw = bytearray(bytes.fromhex(stored_entries[index]["payload_hex"]))
        if raw:
            raw[byte % len(raw)] ^= 0x01
            stored_entries[index]["payload_hex"] = raw.hex()
        else:
            stored_entries[index]["payload_hex"] = "00"
    elif kind == "hash":
        stored[index]["hash_hex"] = _flip(stored[index]["hash_hex"], byte)
    elif kind == "prev":
        stored[index]["prev_hex"] = _flip(stored[index]["prev_hex"], byte)
    else:
        raise ValueError(kind)
    return stored, stored_entries


@unittest.skipUnless(VECTORS.exists(), "vectors not generated yet")
class LedgerVectors(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = json.loads(VECTORS.read_text(encoding="utf-8"))

    def test_domain_is_declared(self) -> None:
        self.assertEqual(self.doc["domain"], "tessera-ledger/1")

    def test_reference_reproduces_every_vector(self) -> None:
        for case in self.doc["cases"]:
            with self.subTest(case=case["name"]):
                records = led.build_chain(case["tenant"], case["entries"])
                if case["tamper"]:
                    stored, entries = _apply_tamper(
                        records, case["entries"],
                        case["tamper"]["kind"],
                        case["tamper"]["record"],
                        case["tamper"]["byte"],
                    )
                else:
                    stored, entries = records, case["entries"]

                # The reference's committed records must match the vectors
                # field for field...
                for got, want in zip(stored, case["expected"]["records"]):
                    self.assertEqual(got["prev_hex"], want["prev_hex"],
                                     case["name"])
                    self.assertEqual(got["hash_hex"], want["hash_hex"],
                                     case["name"])
                # ...and verification must pin the same first broken height.
                first_broken = led.verify(stored, case["tenant"], entries)
                self.assertEqual(
                    first_broken,
                    case["expected"]["first_broken_height"],
                    case["name"],
                )

    def test_clean_chains_verify(self) -> None:
        for case in self.doc["cases"]:
            if case["tamper"]:
                continue
            records = led.build_chain(case["tenant"], case["entries"])
            self.assertIsNone(led.verify(records, case["tenant"], case["entries"]))


if __name__ == "__main__":
    unittest.main()
