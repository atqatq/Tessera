"""Drift guard: the manifest validator refuses exactly the committed
module-conformance vectors' invalid fixtures and accepts the valid ones."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from tessera_ref import manifest  # noqa: E402

VECTORS = HERE.parent / "vectors" / "module_conformance.vectors.json"


@unittest.skipUnless(VECTORS.exists(), "module conformance vectors missing")
class ModuleConformance(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = json.loads(VECTORS.read_text(encoding="utf-8"))
        self.base = VECTORS.parent

    def test_every_case_is_replayed(self) -> None:
        for case in self.doc["cases"]:
            with self.subTest(case=case["name"]):
                if "manifest_file" in case:
                    manifest.validate_file(self.base / case["manifest_file"])
                    self.assertTrue(case["expect_valid"])
                    continue
                m = case["manifest"]
                if case["expect_valid"]:
                    manifest.validate(m)
                else:
                    with self.assertRaises(manifest.ManifestInvalid):
                        manifest.validate(m)
                    self.assertTrue(case.get("reason"), "every refusal names its reason")

    def test_the_template_is_among_the_valid_fixtures(self) -> None:
        names = [c["name"] for c in self.doc["cases"]]
        self.assertIn("the_template_manifest_is_valid", names)


if __name__ == "__main__":
    unittest.main()
