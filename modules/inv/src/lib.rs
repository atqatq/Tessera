//! Multi-echelon safety stock under staged service levels — the `inv`
//! module's core, and the one complete red-green-refactor cycle of this
//! hardening pass (Part E1).
//!
//! The vectors are the specification
//! (`reference/python/vectors/safety_stock.vectors.json`); this
//! implementation mirrors the stdlib-only Python reference statement by
//! statement, because both must reproduce the committed cases
//! byte-for-byte. The algorithm, stated so a practitioner can argue
//! with it:
//!
//! 1. Validate; refuse known-bad configurations (a 100% service-level
//!    target is a refusal, not an edge case — see OPERATOR_MODEL).
//! 2. Derive demand top-down: a node with children aggregates them
//!    (mean = sum; deviation = root-sum-square, independent-demand
//!    assumption); a leaf uses its own inputs. A parent's own demand
//!    inputs are ignored.
//! 3. King's formula per echelon: sigma_DL = sqrt(L·σ_d² + μ_d²·σ_L²).
//! 4. z from the staged service level via Abramowitz & Stegun 26.2.23
//!    (|ε| < 4.5e-4 — three orders below unit rounding at sane scales;
//!    published constants, identical in both languages).
//! 5. Safety stock = ceil(z · sigma_DL) whole units; never negative.
//!
//! Recommendations are never bare numbers: each echelon's entry carries
//! z, sigma_DL, the derived demand statistics, and the method name —
//! the human-readable [`Recommendation::explain`] is pinned by the
//! vectors too (OPERATOR_MODEL mechanism 3).
//!
//! Determinism (Part A3): pure function, no clock, no randomness.
//! Cross-language bit-identity: vectors store z and sigma_DL rounded
//! to 9 decimals (ties-to-even, identical rounding both sides); the
//! generator refuses inputs whose z·sigma_DL sits within 1e-3 of an
//! integer, so no committed case depends on ceil() at a boundary.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// The method name carried by every recommendation (never a bare number).
pub const METHOD: &str = "staged-service-level-meio/1";

/// One echelon in the staging tree, listed topologically: a parent's
/// index is always strictly less than the child's.
#[derive(Debug, Clone, PartialEq)]
pub struct Echelon {
    /// Echelon name, unique within a recommendation.
    pub name: String,
    /// Mean demand per period (leaf only; ignored on parents).
    pub mean_demand: f64,
    /// Demand deviation per period (leaf only; ignored on parents).
    pub sd_demand: f64,
    /// Mean replenishment lead time, in periods.
    pub mean_lead_time: f64,
    /// Lead-time deviation, in periods.
    pub sd_lead_time: f64,
    /// The staged service-level target for this echelon.
    pub service_level: f64,
    /// Index of the parent echelon, or `None` for a root.
    pub parent: Option<usize>,
}

/// One echelon's recommendation: the number, plus its method and
/// assumptions (a bare number is not a recommendation).
#[derive(Debug, Clone, PartialEq)]
pub struct EchelonStock {
    /// Echelon name.
    pub name: String,
    /// Safety stock, whole units (demand never ships fractional units).
    pub ss_units: u64,
    /// The quantile factor used (9-decimal-rounded, as the vectors store).
    pub z: f64,
    /// Demand over lead time deviation, 9-decimal-rounded.
    pub sigma_dl: f64,
    /// The staged service level applied.
    pub service_level: f64,
    /// Derived mean demand at this echelon.
    pub mean_demand: f64,
    /// Derived demand deviation at this echelon.
    pub sd_demand: f64,
    /// Mean lead time, echoed for the explainer.
    pub mean_lead_time: f64,
    /// Lead-time deviation, echoed for the explainer.
    pub sd_lead_time: f64,
}

/// A complete recommendation: method plus every echelon's stock.
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    /// The method that produced this recommendation.
    pub method: String,
    /// One entry per echelon, input order preserved.
    pub echelons: Vec<EchelonStock>,
}

impl Recommendation {
    /// Human-readable, deterministic explanation of one echelon's
    /// recommendation — the string the vectors pin, and the string an
    /// operator reads (OPERATOR_MODEL: every recommendation carries its
    /// method and assumptions).
    pub fn explain(&self, index: usize) -> String {
        let e = &self.echelons[index];
        format!(
            "echelon {}: safety stock {} units — staged service-level MEIO, \
             sigma_DL {:.1} from lead time {}±{}, service level {:.0}%",
            e.name,
            e.ss_units,
            e.sigma_dl,
            e.mean_lead_time,
            e.sd_lead_time,
            e.service_level * 100.0
        )
    }
}

/// Why a configuration was refused. Refusals are part of the contract:
/// the messages are pinned by the conformance vectors (OPERATOR_MODEL
/// mechanism 2 — the system declines to save a setup that guarantees
/// failure, and each refusal is a test).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// An empty staging tree protects nothing.
    #[error("at least one echelon is required")]
    EmptyEchelons,
    /// Two echelons share a name.
    #[error("duplicate echelon name: {0}")]
    DuplicateName(String),
    /// Negative demand mean or deviation.
    #[error("{0}: negative demand statistics")]
    NegativeDemand(String),
    /// Negative lead-time mean or deviation.
    #[error("{0}: negative lead time")]
    NegativeLeadTime(String),
    /// A 100%+ service-level target is unachievable; promising it would
    /// be the system lying.
    #[error("{0}: service level >= 1 is unachievable")]
    UnachievableServiceLevel(String),
    /// A negative service level protects nothing either.
    #[error("{0}: negative service level")]
    NegativeServiceLevel(String),
    /// A parent index that is not strictly before the node: a cycle or
    /// a gap. The tree shape makes this a configuration error, not a
    /// runtime surprise.
    #[error("{0}: parent index {1} is not before {2} (cycle or gap)")]
    InvalidTree(String, usize, usize),
}

/// Inverse standard normal CDF, Abramowitz & Stegun 26.2.23.
/// `z(0) := 0`; a service level ≥ 1 is refused.
pub fn z_from_service_level(p: f64) -> Result<f64, ConfigError> {
    if p <= 0.0 {
        return Ok(0.0);
    }
    if p >= 1.0 {
        return Err(ConfigError::UnachievableServiceLevel(String::from("z")));
    }
    const A: [f64; 3] = [2.515_517, 0.802_853, 0.010_328];
    const B: [f64; 3] = [1.432_788, 0.189_269, 0.001_308];
    let q = if p <= 0.5 { p } else { 1.0 - p };
    let t = (1.0 / (q * q)).ln().sqrt();
    let num = A[0] + A[1] * t + A[2] * t * t;
    let den = 1.0 + B[0] * t + B[1] * t * t + B[2] * t * t * t;
    let z = t - num / den;
    Ok(if p > 0.5 { z } else { -z })
}

/// Rounds to 9 decimals, ties-to-even — the same rounding Python's
/// `round(x, 9)` applies, so both sides store and compare identical
/// doubles.
fn round9(x: f64) -> f64 {
    (x * 1e9).round_ties_even() / 1e9
}

/// Computes the staged-service-level safety stock for a whole tree.
pub fn recommend(echelons: &[Echelon]) -> Result<Recommendation, ConfigError> {
    let n = echelons.len();
    if n == 0 {
        return Err(ConfigError::EmptyEchelons);
    }
    let mut names = std::collections::BTreeSet::new();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, e) in echelons.iter().enumerate() {
        if !names.insert(e.name.as_str()) {
            return Err(ConfigError::DuplicateName(e.name.clone()));
        }
        if e.mean_demand < 0.0 || e.sd_demand < 0.0 {
            return Err(ConfigError::NegativeDemand(e.name.clone()));
        }
        if e.mean_lead_time < 0.0 || e.sd_lead_time < 0.0 {
            return Err(ConfigError::NegativeLeadTime(e.name.clone()));
        }
        if e.service_level >= 1.0 {
            return Err(ConfigError::UnachievableServiceLevel(e.name.clone()));
        }
        if e.service_level < 0.0 {
            return Err(ConfigError::NegativeServiceLevel(e.name.clone()));
        }
        if let Some(p) = e.parent {
            if p >= i {
                return Err(ConfigError::InvalidTree(e.name.clone(), p, i));
            }
            children[p].push(i);
        }
    }

    // derive demand from the leaves upward (index order is topological
    // by validation, so reverse iteration always has children ready)
    let mut demand: Vec<(f64, f64)> = vec![(0.0, 0.0); n];
    for i in (0..n).rev() {
        if children[i].is_empty() {
            demand[i] = (echelons[i].mean_demand, echelons[i].sd_demand);
        } else {
            let mean: f64 = children[i].iter().map(|&c| demand[c].0).sum();
            let var: f64 = children[i].iter().map(|&c| demand[c].1 * demand[c].1).sum();
            demand[i] = (mean, var.sqrt());
        }
    }

    let mut out = Vec::with_capacity(n);
    for (i, e) in echelons.iter().enumerate() {
        let (mu_d, sd_d) = demand[i];
        let sigma_dl =
            (e.mean_lead_time * sd_d * sd_d + mu_d * mu_d * e.sd_lead_time * e.sd_lead_time).sqrt();
        let z = z_from_service_level(e.service_level).map_err(|_| {
            // already validated >= 1 above; the map keeps z's error type
            // honest without a second message format
            ConfigError::UnachievableServiceLevel(e.name.clone())
        })?;
        let ss_raw = z * sigma_dl;
        let ss = if ss_raw <= 0.0 {
            0
        } else {
            ss_raw.ceil() as u64
        };
        out.push(EchelonStock {
            name: e.name.clone(),
            ss_units: ss,
            z: round9(z),
            sigma_dl: round9(sigma_dl),
            service_level: e.service_level,
            mean_demand: round9(mu_d),
            sd_demand: round9(sd_d),
            mean_lead_time: e.mean_lead_time,
            sd_lead_time: e.sd_lead_time,
        });
    }
    Ok(Recommendation {
        method: METHOD.to_string(),
        echelons: out,
    })
}

impl fmt::Display for EchelonStock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} units", self.name, self.ss_units)
    }
}
