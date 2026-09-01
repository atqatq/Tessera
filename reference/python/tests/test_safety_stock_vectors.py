"""Drift guard: the Python safety-stock reference must reproduce the
committed safety-stock vectors (the Rust side replays the same files)."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

from tessera_ref import safety_stock as ss  # noqa: E402

VECTORS = HERE.parent / "vectors" / "safety_stock.vectors.json"


@unittest.skipUnless(VECTORS.exists(), "vectors not generated yet")
class SafetyStockVectors(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = json.loads(VECTORS.read_text(encoding="utf-8"))

    def test_domain_and_method_are_declared(self) -> None:
        self.assertEqual(self.doc["domain"], "safety-stock/1")
        self.assertEqual(self.doc["method"], ss.METHOD)

    def test_reference_reproduces_every_vector(self) -> None:
        for case in self.doc["cases"]:
            with self.subTest(case=case["name"]):
                echelons = [ss.Echelon(**e) for e in case["echelons"]]
                if case["expected"]["ok"]:
                    rec = ss.recommend(echelons)
                    self.assertEqual(rec.method, case["expected"]["method"])
                    for got, want in zip(rec.echelons, case["expected"]["echelons"]):
                        self.assertEqual(got.name, want["name"], case["name"])
                        self.assertEqual(got.ss_units, want["ss_units"], case["name"])
                        self.assertEqual(got.z, want["z"], case["name"])
                        self.assertEqual(got.sigma_dl, want["sigma_dl"], case["name"])
                        self.assertEqual(got.mean_demand, want["mean_demand"], case["name"])
                        self.assertEqual(got.sd_demand, want["sd_demand"], case["name"])
                    if "explain_0" in case["expected"]:
                        self.assertEqual(rec.explain(0), case["expected"]["explain_0"])
                else:
                    with self.assertRaises(ss.ConfigError) as ctx:
                        ss.recommend(echelons)
                    self.assertEqual(str(ctx.exception), case["expected"]["message"])

    def test_refusals_are_part_of_the_contract(self) -> None:
        # E2 mechanism 2: known-bad configurations are refused, and the
        # refusal is itself pinned data.
        kinds = [c["name"] for c in self.doc["cases"] if not c["expected"]["ok"]]
        self.assertTrue(any("service_level_one" in k for k in kinds))
        self.assertTrue(any("negative_lead_time" in k for k in kinds))


if __name__ == "__main__":
    unittest.main()
