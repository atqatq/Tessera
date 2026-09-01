//! The safety-stock engine's behavioural spec, as sentences (Part A1).
//! Written before the implementation exists: the first run fails to
//! compile because the API does not exist — that is the red state.
//! The numbers are the committed vectors' numbers; these tests mirror
//! the Python reference's own suite.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tessera_inv::{ConfigError, Echelon, recommend, z_from_service_level};

fn leaf(name: &str, mu: f64, sd: f64, l: f64, sl: f64) -> Echelon {
    Echelon {
        name: name.to_string(),
        mean_demand: mu,
        sd_demand: sd,
        mean_lead_time: l,
        sd_lead_time: 0.0,
        service_level: sl,
        parent: None,
    }
}

#[test]
fn z_is_zero_at_a_zero_service_level() {
    assert_eq!(z_from_service_level(0.0).expect("sl=0 is fine"), 0.0);
}

#[test]
fn a_service_level_of_one_is_refused() {
    assert!(matches!(
        z_from_service_level(1.0),
        Err(ConfigError::UnachievableServiceLevel { .. })
    ));
    assert!(matches!(
        z_from_service_level(1.000_000_1),
        Err(ConfigError::UnachievableServiceLevel { .. })
    ));
}

#[test]
fn z_at_the_median_is_near_zero() {
    let z = z_from_service_level(0.5).expect("sl=0.5 is fine");
    assert!(z.abs() < 1e-5, "z(0.5) = {z}");
}

#[test]
fn z_matches_known_quantiles_within_the_documented_tolerance() {
    // A-S 26.2.23 carries |epsilon| < 4.5e-4 — documented, not hidden.
    for (p, expected) in [
        (0.90, 1.2816),
        (0.95, 1.6449),
        (0.975, 1.9600),
        (0.99, 2.3263),
    ] {
        let z = z_from_service_level(p).expect("valid sl");
        assert!(
            (z - expected).abs() < 5e-4,
            "z({p}) = {z}, expected ≈ {expected}"
        );
    }
}

#[test]
fn a_single_echelon_degenerates_to_the_classic_formula() {
    // sigma_DL = sqrt(4 * 30^2) = 60; z(0.95) ~= 1.6451 -> ceil = 99
    let r = recommend(&[leaf("dc", 100.0, 30.0, 4.0, 0.95)]).expect("valid tree");
    assert_eq!(r.method, "staged-service-level-meio/1");
    assert_eq!(r.echelons[0].ss_units, 99);
    assert_eq!(
        r.explain(0),
        "echelon dc: safety stock 99 units — staged service-level MEIO, \
         sigma_DL 60.0 from lead time 4±0, service level 95%"
    );
}

#[test]
fn zero_demand_means_zero_stock() {
    let r = recommend(&[leaf("dc", 0.0, 0.0, 14.0, 0.99)]).expect("valid tree");
    assert_eq!(r.echelons[0].ss_units, 0);
}

#[test]
fn a_zero_service_level_means_zero_stock() {
    let r = recommend(&[leaf("dc", 100.0, 30.0, 4.0, 0.0)]).expect("valid tree");
    assert_eq!(r.echelons[0].ss_units, 0);
}

#[test]
fn a_negative_lead_time_is_refused() {
    assert!(matches!(
        recommend(&[leaf("dc", 100.0, 30.0, -1.0, 0.95)]),
        Err(ConfigError::NegativeLeadTime { .. })
    ));
}

#[test]
fn a_hundred_percent_target_is_refused_as_a_configuration() {
    assert!(matches!(
        recommend(&[leaf("dc", 100.0, 30.0, 4.0, 1.0)]),
        Err(ConfigError::UnachievableServiceLevel { .. })
    ));
}

#[test]
fn two_echelons_aggregate_demand_downward() {
    // dc (root) under staged sl 0.95; two retailers at 0.90
    let dc = Echelon {
        name: "dc".to_string(),
        mean_demand: 0.0,
        sd_demand: 0.0,
        mean_lead_time: 6.0,
        sd_lead_time: 1.0,
        service_level: 0.95,
        parent: None,
    };
    let mk_store = |name: &str| Echelon {
        name: name.to_string(),
        mean_demand: 50.0,
        sd_demand: 12.0,
        mean_lead_time: 2.0,
        sd_lead_time: 0.0,
        service_level: 0.90,
        parent: Some(0),
    };
    let r = recommend(&[dc, mk_store("ret-1"), mk_store("ret-2")]).expect("valid tree");
    let d = &r.echelons[0];
    assert!((d.mean_demand - 100.0).abs() < 1e-9);
    assert!((d.sd_demand - 16.970_562_748).abs() < 1e-8);
    assert!((d.sigma_dl - 108.295_890_965).abs() < 1e-6);
    assert_eq!(d.ss_units, 179);
    // each retailer: sigma_DL = sqrt(2*144) = 16.9706; z(0.90) ~= 1.28173
    for e in &r.echelons[1..] {
        assert_eq!(e.ss_units, 22, "{}", e.name);
    }
}

#[test]
fn a_parent_own_demand_inputs_are_ignored() {
    let junk = Echelon {
        name: "p".to_string(),
        mean_demand: 999.0,
        sd_demand: 999.0,
        mean_lead_time: 3.0,
        sd_lead_time: 0.0,
        service_level: 0.95,
        parent: None,
    };
    let a = Echelon {
        name: "a".to_string(),
        mean_demand: 40.0,
        sd_demand: 5.0,
        mean_lead_time: 2.0,
        sd_lead_time: 0.0,
        service_level: 0.90,
        parent: Some(0),
    };
    let b = Echelon {
        name: "b".to_string(),
        mean_demand: 60.0,
        sd_demand: 7.0,
        mean_lead_time: 2.0,
        sd_lead_time: 0.0,
        service_level: 0.90,
        parent: Some(0),
    };
    let r = recommend(&[junk, a, b]).expect("valid tree");
    assert!((r.echelons[0].mean_demand - 100.0).abs() < 1e-9);
}

#[test]
fn a_parent_index_must_point_backwards() {
    let self_cycle = Echelon {
        name: "a".to_string(),
        mean_demand: 1.0,
        sd_demand: 0.0,
        mean_lead_time: 1.0,
        sd_lead_time: 0.0,
        service_level: 0.9,
        parent: Some(0),
    };
    assert!(matches!(
        recommend(&[self_cycle]),
        Err(ConfigError::InvalidTree { .. })
    ));
    let forward = Echelon {
        name: "a".to_string(),
        mean_demand: 50.0,
        sd_demand: 5.0,
        mean_lead_time: 2.0,
        sd_lead_time: 0.0,
        service_level: 0.90,
        parent: Some(1),
    };
    let b = Echelon {
        name: "b".to_string(),
        mean_demand: 50.0,
        sd_demand: 5.0,
        mean_lead_time: 2.0,
        sd_lead_time: 0.0,
        service_level: 0.90,
        parent: None,
    };
    assert!(matches!(
        recommend(&[forward, b]),
        Err(ConfigError::InvalidTree { .. })
    ));
}

#[test]
fn duplicate_echelon_names_are_refused() {
    assert!(matches!(
        recommend(&[leaf("x", 1.0, 0.0, 1.0, 0.9), leaf("x", 1.0, 0.0, 1.0, 0.9)]),
        Err(ConfigError::DuplicateName { .. })
    ));
}
