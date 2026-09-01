//! Property tests for the safety-stock invariants (Part A2): safety
//! stock is monotone in the service level, never negative, and exactly
//! zero when there is nothing to protect.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use tessera_inv::{Echelon, recommend};

fn echelon(mu: f64, sd: f64, l: f64, sd_l: f64, sl: f64) -> Echelon {
    Echelon {
        name: "n".to_string(),
        mean_demand: mu,
        sd_demand: sd,
        mean_lead_time: l,
        sd_lead_time: sd_l,
        service_level: sl,
        parent: None,
    }
}

fn sane_inputs() -> impl Strategy<Value = (f64, f64, f64, f64)> {
    (1.0f64..1_000.0, 0.0f64..300.0, 0.0f64..60.0, 0.0f64..5.0)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Safety stock is monotone non-decreasing in the service level.
    #[test]
    fn safety_stock_is_monotone_in_service_level(
        (mu, sd, l, sd_l) in sane_inputs(),
        sl_lo in 0.01f64..0.98,
        sl_step in 0.005f64..0.05,
    ) {
        let sl_hi = (sl_lo + sl_step).min(0.985);
        let lo = recommend(&[echelon(mu, sd, l, sd_l, sl_lo)]).expect("valid").echelons[0].ss_units;
        let hi = recommend(&[echelon(mu, sd, l, sd_l, sl_hi)]).expect("valid").echelons[0].ss_units;
        prop_assert!(hi >= lo, "ss({sl_hi})={hi} < ss({sl_lo})={lo}");
    }

    /// With deterministic lead times, safety stock depends on demand
    /// variability, not volume: sigma_DL = sqrt(L)*sigma_d has no mean
    /// term. (Non-negativity needs no property — ss_units is unsigned;
    /// the type system is the proof, which is the A4 lesson.)
    #[test]
    fn deterministic_lead_times_ignore_volume(
        sd in 0.0f64..300.0,
        l in 0.0f64..60.0,
        sl in 0.01f64..0.99,
        scale in proptest::sample::select(vec![1.0f64, 10.0, 1000.0]),
    ) {
        prop_assume!(sd > 0.0);
        let small = recommend(&[echelon(1.0 * scale, sd, l, 0.0, sl)]).expect("valid");
        let large = recommend(&[echelon(1000.0 * scale, sd, l, 0.0, sl)]).expect("valid");
        prop_assert_eq!(
            small.echelons[0].ss_units,
            large.echelons[0].ss_units,
            "volume must not change ss when lead time is deterministic"
        );
    }

    /// Zero demand and zero deviation mean exactly zero stock, at any
    /// achievable service level.
    #[test]
    fn nothing_to_protect_means_zero_stock(sl in 0.01f64..0.99) {
        let r = recommend(&[echelon(0.0, 0.0, 42.0, 9.0, sl)]).expect("valid");
        prop_assert_eq!(r.echelons[0].ss_units, 0);
    }

    /// The same inputs produce the same recommendation, twice — the
    /// engine is pure (A3).
    #[test]
    fn recommendations_are_deterministic(
        (mu, sd, l, sd_l) in sane_inputs(),
        sl in 0.01f64..0.99,
    ) {
        let a = recommend(&[echelon(mu, sd, l, sd_l, sl)]).expect("valid");
        let b = recommend(&[echelon(mu, sd, l, sd_l, sl)]).expect("valid");
        prop_assert_eq!(a.echelons[0].ss_units, b.echelons[0].ss_units);
        prop_assert_eq!(a.echelons[0].z, b.echelons[0].z);
        prop_assert_eq!(a.echelons[0].sigma_dl, b.echelons[0].sigma_dl);
    }
}
