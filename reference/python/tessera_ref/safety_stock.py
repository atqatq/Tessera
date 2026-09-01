"""Mirror of modules/inv — multi-echelon safety stock under staged service levels.

This is the executable specification (the vectors are its output). The
algorithm is fixed and every arithmetic step is stated, because the Rust
implementation must reproduce it bit-for-bit on the committed vectors:

1.  Validate inputs. Refusals are part of the contract:
    - negative lead time, negative demand stats -> ConfigError
    - service level > 1 -> UnachievableServiceLevel (a 100% target is a
      known-bad configuration, not an edge case)
    - service level < 0 -> NegativeServiceLevel
    - parent index not strictly before the node -> InvalidTree (cycles
      and forward references are unrepresentable by construction)
2.  Derive demand top-down (retailers first). A node with children is an
    aggregation point: mean = sum of children's means; deviation =
    root-sum-square of children's deviations (independent-demand
    assumption). A leaf's demand is its own input. A parent's own demand
    inputs are ignored — demand is derived, never mixed.
3.  sigma_DL per node (King): sqrt(L * sd_demand^2 + mean_demand^2 * sd_L^2)
4.  z from the staged service level via Abramowitz & Stegun 26.2.23 —
    |epsilon| < 4.5e-4, three orders below unit rounding at the scales
    this engine targets, and the approximation (with its published
    constants) is identical in both implementations. Swapping in a
    higher-order quantile function is a one-function change behind the
    vectors.
5.  safety stock = ceil(z * sigma_DL) whole units.

Determinism: floating-point `ln` is not guaranteed bit-identical across
platform libms, so vectors store z and sigma_DL rounded to 9 decimals;
safety stock is an integer. The generator refuses inputs whose
z * sigma_DL sits within 1e-3 of an integer (a ceil flip under a 1-ulp
drift), so committed cases are far from rounding boundaries.

Service level 0 means "stock nothing": z := 0, SS := 0. Service level 1
is refused — unachievable, and promising it would be the system lying.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

METHOD = "staged-service-level-meio/1"

# A-S 26.2.23 coefficients (published constants, stated here so the Rust
# side can copy them byte-for-byte from the same source).
_A_COEF = (2.515517, 0.802853, 0.010328)
_B_COEF = (1.432788, 0.189269, 0.001308)


class ConfigError(ValueError):
    """A configuration the engine refuses to save (E2, mechanism 2)."""


@dataclass(frozen=True)
class Echelon:
    """One echelon in the staging tree, listed topologically (parents first)."""

    name: str
    mean_demand: float
    sd_demand: float
    mean_lead_time: float
    sd_lead_time: float
    service_level: float
    parent: int | None  # index into the echelon list; None for roots


@dataclass(frozen=True)
class EchelonStock:
    """One echelon's recommendation, with its method and assumptions."""

    name: str
    ss_units: int
    z: float
    sigma_dl: float
    service_level: float
    mean_demand: float
    sd_demand: float
    mean_lead_time: float
    sd_lead_time: float


@dataclass(frozen=True)
class Recommendation:
    """The full recommendation: never a bare number."""

    method: str
    echelons: tuple[EchelonStock, ...]

    def explain(self, index: int) -> str:
        e = self.echelons[index]
        return (
            f"echelon {e.name}: safety stock {e.ss_units} units — "
            f"staged service-level MEIO, sigma_DL {e.sigma_dl:.1f} "
            f"from lead time {e.mean_lead_time:g}±{e.sd_lead_time:g}, "
            f"service level {e.service_level:.0%}"
        )


def z_from_service_level(p: float) -> float:
    """Inverse standard normal CDF, A-S 26.2.23 (|eps| < 4.5e-4); z(0) := 0."""
    if p <= 0.0:
        return 0.0
    if p >= 1.0:
        raise ConfigError("service level 1.0 is unachievable")
    q = p if p <= 0.5 else 1.0 - p
    t = math.sqrt(math.log(1.0 / (q * q)))
    num = _A_COEF[0] + _A_COEF[1] * t + _A_COEF[2] * t * t
    den = 1.0 + _B_COEF[0] * t + _B_COEF[1] * t * t + _B_COEF[2] * t * t * t
    z = t - num / den
    return z if p > 0.5 else -z


def _validate(echelons: list[Echelon]) -> None:
    if not echelons:
        raise ConfigError("at least one echelon is required")
    names = set()
    for i, e in enumerate(echelons):
        if e.name in names:
            raise ConfigError(f"duplicate echelon name: {e.name}")
        names.add(e.name)
        if e.mean_demand < 0 or e.sd_demand < 0:
            raise ConfigError(f"{e.name}: negative demand statistics")
        if e.mean_lead_time < 0 or e.sd_lead_time < 0:
            raise ConfigError(f"{e.name}: negative lead time")
        if e.service_level >= 1.0:
            raise ConfigError(f"{e.name}: service level >= 1 is unachievable")
        if e.service_level < 0:
            raise ConfigError(f"{e.name}: negative service level")
        if e.parent is not None and e.parent >= i:
            raise ConfigError(
                f"{e.name}: parent index {e.parent} is not before {i} (cycle or gap)"
            )


def recommend(echelons: list[Echelon]) -> Recommendation:
    """The staged-service-level MEIO recommendation for a staging tree."""
    _validate(echelons)
    n = len(echelons)
    children: list[list[int]] = [[] for _ in range(n)]
    for i, e in enumerate(echelons):
        if e.parent is not None:
            children[e.parent].append(i)

    # derive demand top-down (index order is topological by validation)
    demand: list[tuple[float, float]] = [(0.0, 0.0)] * n
    for i in range(n - 1, -1, -1):
        e = echelons[i]
        if children[i]:
            mean = sum(demand[c][0] for c in children[i])
            var = sum(demand[c][1] ** 2 for c in children[i])
            demand[i] = (mean, math.sqrt(var))
        else:
            demand[i] = (e.mean_demand, e.sd_demand)

    out = []
    for i, e in enumerate(echelons):
        mu_d, sd_d = demand[i]
        sigma_dl = math.sqrt(
            e.mean_lead_time * sd_d * sd_d + mu_d * mu_d * e.sd_lead_time * e.sd_lead_time
        )
        z = z_from_service_level(e.service_level)
        ss = math.ceil(z * sigma_dl)
        if ss < 0:
            ss = 0
        out.append(
            EchelonStock(
                name=e.name,
                ss_units=ss,
                z=round(z, 9),
                sigma_dl=round(sigma_dl, 9),
                service_level=e.service_level,
                mean_demand=round(mu_d, 9),
                sd_demand=round(sd_d, 9),
                mean_lead_time=e.mean_lead_time,
                sd_lead_time=e.sd_lead_time,
            )
        )
    return Recommendation(method=METHOD, echelons=tuple(out))
