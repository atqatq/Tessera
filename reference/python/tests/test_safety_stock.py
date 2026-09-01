"""The safety-stock reference's own tests, written first (red until the
reference exists — and the reference IS this module, so these tests pin
the behaviour the vectors will carry)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from tessera_ref.safety_stock import (  # noqa: E402
    ConfigError,
    Echelon,
    METHOD,
    recommend,
    z_from_service_level,
)


def leaf(name, mu, sd, L, sl, sd_L=0.0):
    return Echelon(name, mu, sd, L, sd_L, sl, None)


class ZFromServiceLevel(unittest.TestCase):
    def test_z_zero_at_zero_service_level(self):
        self.assertEqual(z_from_service_level(0.0), 0.0)

    def test_service_level_one_is_refused(self):
        with self.assertRaises(ConfigError):
            z_from_service_level(1.0)
        with self.assertRaises(ConfigError):
            z_from_service_level(1.0000001)

    def test_z_at_half_is_near_zero(self):
        self.assertAlmostEqual(z_from_service_level(0.5), 0.0, places=5)

    def test_z_matches_known_quantiles_within_as_tolerance(self):
        # A-S 26.2.23 carries |epsilon| < 4.5e-4 — three orders below
        # unit rounding at sane scales; documented, not hidden
        for p, expected in [(0.90, 1.2816), (0.95, 1.6449), (0.975, 1.9600), (0.99, 2.3263)]:
            self.assertAlmostEqual(z_from_service_level(p), expected, delta=5e-4)

    def test_symmetry(self):
        self.assertAlmostEqual(
            z_from_service_level(0.05), -z_from_service_level(0.95), places=12
        )


class SingleEchelon(unittest.TestCase):
    def test_classic_kings_formula_degenerates(self):
        # single echelon: mean 100/day, sd 30/day, L=4, sl=95% ->
        # sigma_DL = sqrt(4*900 + 10000*0) = 60; SS = ceil(1.6451*60) = 99
        r = recommend([leaf("dc", 100.0, 30.0, 4.0, 0.95)])
        self.assertEqual(r.method, METHOD)
        e = r.echelons[0]
        self.assertEqual(e.ss_units, math_ceil_check(1.6451 * 60, 99))
        self.assertEqual(r.explain(0),
                         "echelon dc: safety stock 99 units — staged service-level MEIO, "
                         "sigma_DL 60.0 from lead time 4±0, service level 95%")

    def test_zero_demand_means_zero_stock(self):
        r = recommend([leaf("dc", 0.0, 0.0, 14.0, 0.99)])
        self.assertEqual(r.echelons[0].ss_units, 0)

    def test_service_level_zero_means_zero_stock(self):
        r = recommend([leaf("dc", 100.0, 30.0, 4.0, 0.0)])
        self.assertEqual(r.echelons[0].ss_units, 0)

    def test_negative_lead_time_is_refused(self):
        with self.assertRaises(ConfigError):
            recommend([leaf("dc", 100.0, 30.0, -1.0, 0.95)])

    def test_service_level_one_is_refused_at_config_level(self):
        with self.assertRaises(ConfigError):
            recommend([leaf("dc", 100.0, 30.0, 4.0, 1.0)])


class MultiEchelon(unittest.TestCase):
    def test_two_echelons_staged_service_levels(self):
        # two retailers under a DC; staged: retailers 0.90, DC 0.95
        retailers = [
            Echelon(f"ret-{i}", 50.0, 12.0, 2.0, 0.0, 0.90, parent=0)
            for i in (1, 2)
        ]
        dc = Echelon("dc", 0.0, 0.0, 6.0, 1.0, 0.95, parent=None)  # the root
        # order: parents first -> dc at index 0, retailers after
        r = recommend([dc, retailers[0], retailers[1]])
        dc_e = r.echelons[0]
        # DC demand: mean 100, sd sqrt(12^2+12^2)=16.9706
        self.assertAlmostEqual(dc_e.mean_demand, 100.0, places=9)
        self.assertAlmostEqual(dc_e.sd_demand, 16.970562748, places=8)
        # sigma_DL = sqrt(6*16.9706^2 + 100^2*1) = sqrt(11728) = 108.2959
        self.assertAlmostEqual(dc_e.sigma_dl, 108.295890965, places=6)
        self.assertEqual(dc_e.ss_units, math_ceil_check(1.64506 * 108.295891, 179))

    def test_parent_demand_inputs_are_ignored(self):
        a = Echelon("a", 40.0, 5.0, 2.0, 0.0, 0.90, parent=0)
        b = Echelon("b", 60.0, 7.0, 2.0, 0.0, 0.90, parent=0)
        parent_with_junk = Echelon("p", 999.0, 999.0, 3.0, 0.0, 0.95, parent=None)
        r = recommend([parent_with_junk, a, b])
        self.assertAlmostEqual(r.echelons[0].mean_demand, 100.0, places=9)

    def test_parent_index_must_point_backwards(self):
        with self.assertRaises(ConfigError):
            recommend([Echelon("a", 1, 0, 1, 0, 0.9, parent=1)])
        with self.assertRaises(ConfigError):
            recommend([Echelon("a", 1, 0, 1, 0, 0.9, parent=0)])  # self-cycle

    def test_duplicate_names_are_refused(self):
        with self.assertRaises(ConfigError):
            recommend([leaf("x", 1, 0, 1, 0.9), leaf("x", 1, 0, 1, 0.9)])


def math_ceil_check(estimate: float, expected: int) -> int:
    """Assert the pre-computed expected value matches the estimate's ceil
    (guards against hand-arithmetic slips in these tests)."""
    import math
    actual = math.ceil(estimate - 7.5e-6)  # allowance for the A-S epsilon
    assert abs(actual - expected) <= 1, f"ceil({estimate})={actual} vs expected {expected}"
    return expected


if __name__ == "__main__":
    unittest.main()
