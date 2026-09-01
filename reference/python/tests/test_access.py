"""The Python reference must reproduce the committed access vectors.

Drift guard on the reference side; the Rust side runs the same files.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from tessera_ref import access as acc  # noqa: E402

VECTORS = HERE.parent / "vectors" / "access.vectors.json"


@unittest.skipUnless(VECTORS.exists(), "vectors not generated yet")
class AccessVectors(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = json.loads(VECTORS.read_text(encoding="utf-8"))

    def test_domain_is_declared(self) -> None:
        self.assertEqual(self.doc["domain"], "tessera-access/1")

    def test_reference_reproduces_every_vector(self) -> None:
        for case in self.doc["cases"]:
            with self.subTest(case=case["name"]):
                code, layer = acc.evaluate(case["env"], case["request"])
                self.assertEqual(
                    (code, layer),
                    (case["expected"]["code"], case["expected"]["layer"]),
                    case["name"],
                )

    def test_all_fourteen_codes_are_covered(self) -> None:
        covered = {c["expected"]["code"] for c in self.doc["cases"]}
        self.assertEqual(covered, set(acc.CODES))

    def test_reference_matches_itself_deterministically(self) -> None:
        # evaluate is pure: same env+request, same answer, twice.
        for case in self.doc["cases"][:5]:
            a = acc.evaluate(case["env"], case["request"])
            b = acc.evaluate(case["env"], case["request"])
            self.assertEqual(a, b)


if __name__ == "__main__":
    unittest.main()
