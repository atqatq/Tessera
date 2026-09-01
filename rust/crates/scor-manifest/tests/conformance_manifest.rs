//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Port of the key scenarios from the Python reference's
//! `test_manifest.py`, so both validators are held to the same behaviour.
//!
//! Test files use `serde_json`'s `IndexMut` for surgical fixture edits;
//! it inserts on missing keys and cannot panic on objects, so slicing is
//! scoped out here.
#![allow(clippy::indexing_slicing)]

use scor_manifest::{validate_all, validate_manifest, Severity, ValidationResult};
use serde_json::{json, Value};

fn valid() -> Value {
    // Mirrors `VALID` in the Python reference's test_manifest.py.
    json!({
        "spoke": "srm",
        "version": "2.4.0",
        "requires": ["hub.master_data", "hub.events", "hub.ledger"],
        "enhances": ["spoke.src", "spoke.ctr"],
        "provides": {
            "objects": ["srm.supplier", "srm.scorecard"],
            "events": ["srm.supplier_rated"],
            "kpis": ["srm.otif_pct"]
        },
        "consumes": [
            {"field": "ctr.commercial_terms.penalty_exposure_usd", "on_missing": "hold_last"},
            {"field": "src.spend_usd_ttm", "on_missing": "null"}
        ]
    })
}

fn codes(manifest: &Value) -> Vec<(Severity, String)> {
    validate_manifest(manifest)
        .findings
        .into_iter()
        .map(|f| (f.severity, f.code))
        .collect()
}

/// An absent key yields an empty result, so the assertions below fail with
/// a clear message and no `expect`/`unwrap` (denied by workspace lints).
fn of(map: &std::collections::BTreeMap<String, ValidationResult>, key: &str) -> ValidationResult {
    map.get(key).cloned().unwrap_or_default()
}

#[test]
fn valid_manifest_has_no_errors() {
    let result = validate_manifest(&valid());
    assert!(result.ok(), "unexpected findings: {:?}", result.findings);
}

#[test]
fn hard_spoke_dependency_is_the_cardinal_error() {
    let mut m = valid();
    m["requires"] = json!(["hub.access", "spoke.ord"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "requires.spoke_dependency".to_string())));
}

#[test]
fn unknown_hub_service_is_rejected() {
    let mut m = valid();
    m["requires"] = json!(["hub.access", "hub.nuclear"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "requires.unknown_service".to_string())));
}

#[test]
fn empty_requires_warns_but_does_not_block() {
    let mut m = valid();
    m["requires"] = json!([]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Warning, "requires.empty".to_string())));
    assert!(validate_manifest(&m).ok());
}

#[test]
fn unregistered_spoke_code_is_rejected() {
    let mut m = valid();
    m["spoke"] = json!("xxx");
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "spoke.unregistered".to_string())));
}

#[test]
fn bad_spoke_code_shape_is_rejected() {
    let mut m = valid();
    m["spoke"] = json!("Srm");
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "spoke.code".to_string())));
}

#[test]
fn non_semver_version_is_rejected() {
    for bad in ["1.2", "v1.2.0", "1.2.3.4", "1.2.3-", "1.2.3\n"] {
        let mut m = valid();
        m["version"] = json!(bad);
        let found = codes(&m);
        assert!(
            found.contains(&(Severity::Error, "spoke.version".to_string())),
            "{bad:?} should not be semver"
        );
    }
}

#[test]
fn semver_prerelease_is_accepted() {
    let mut m = valid();
    m["version"] = json!("1.2.3-rc.1");
    assert!(validate_manifest(&m).ok());
}

#[test]
fn enhances_must_reference_registered_spokes() {
    let mut m = valid();
    m["enhances"] = json!(["spoke.xxx"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "enhances.unregistered".to_string())));
}

#[test]
fn enhances_cannot_target_self() {
    let mut m = valid();
    m["enhances"] = json!(["spoke.srm"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "enhances.self".to_string())));
}

#[test]
fn provides_must_stay_in_namespace() {
    let mut m = valid();
    m["provides"]["objects"] = json!(["ord.order"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "provides.foreign_namespace".to_string())));
}

#[test]
fn provides_must_be_namespaced() {
    let mut m = valid();
    m["provides"]["objects"] = json!(["supplier"]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "provides.namespace".to_string())));
}

#[test]
fn consumes_needs_a_missing_value_policy() {
    let mut m = valid();
    m["consumes"] = json!([{"field": "ctr.baseline"}]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "consumes.on_missing".to_string())));
}

#[test]
fn consumes_cannot_target_own_namespace() {
    let mut m = valid();
    m["consumes"] = json!([{"field": "srm.supplier", "on_missing": "null"}]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "consumes.own_namespace".to_string())));
}

#[test]
fn fail_policy_needs_origin_approval() {
    let mut m = valid();
    m["consumes"] = json!([{ "field": "ctr.baseline", "on_missing": "fail" }]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "consumes.fail_needs_origin".to_string())));

    m["consumes"][0]["origin_approval"] = json!({"by": "origin", "at": "2026-01-01"});
    assert!(validate_manifest(&m).ok());
}

#[test]
fn agent_needs_leader_and_ledger_services() {
    let mut m = valid();
    m["requires"] = json!(["hub.access"]);
    m["ai"] = json!({"enabled": true, "tier": "advise"});
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "ai.missing_service".to_string())));
}

#[test]
fn agent_tier_must_be_known() {
    let mut m = valid();
    m["ai"] = json!({"enabled": true, "tier": "autonomous"});
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "ai.tier".to_string())));
}

#[test]
fn act_tier_needs_allowlist_and_origin() {
    let mut m = valid();
    m["ai"] = json!({"enabled": true, "tier": "act"});
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "ai.act_needs_allowlist".to_string())));
    assert!(found.contains(&(Severity::Error, "ai.act_needs_origin".to_string())));

    m["ai"]["act_allowlist"] = json!(["srm.supplier.tier"]);
    m["ai"]["origin_approval"] = json!({"by": "origin"});
    let found = codes(&m);
    assert!(!found.contains(&(Severity::Error, "ai.act_needs_allowlist".to_string())));
    assert!(!found.contains(&(Severity::Error, "ai.act_needs_origin".to_string())));
}

#[test]
fn foreign_allowlist_is_rejected() {
    let mut m = valid();
    m["ai"] = json!({"enabled": true, "tier": "advise", "act_allowlist": ["ord.order.qty"]});
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "ai.foreign_allowlist".to_string())));
}

#[test]
fn allowlist_without_act_tier_warns() {
    let mut m = valid();
    m["ai"] = json!({"enabled": true, "tier": "advise", "act_allowlist": ["srm.supplier.tier"]});
    let found = codes(&m);
    assert!(found.contains(&(Severity::Warning, "ai.allowlist_unused".to_string())));
}

#[test]
fn absent_agent_warns() {
    let mut m = valid();
    m["ai"] = Value::Null;
    let found = codes(&m);
    assert!(found.contains(&(Severity::Warning, "ai.absent".to_string())));
}

#[test]
fn dashboards_must_use_kebab_slugs() {
    let mut m = valid();
    m["dashboards"] = json!([{"slug": "Supplier Health", "kpis": []}]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "dashboards.slug".to_string())));
}

#[test]
fn dashboards_cannot_repeat_a_slug() {
    let mut m = valid();
    m["dashboards"] = json!([
        {"slug": "health", "kpis": []},
        {"slug": "health", "kpis": []}
    ]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "dashboards.duplicate".to_string())));
}

#[test]
fn dashboards_cannot_show_unpublished_own_kpis() {
    let mut m = valid();
    m["dashboards"] = json!([{"slug": "health", "kpis": ["srm.secret_pct"]}]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "dashboards.unpublished_kpi".to_string())));
}

#[test]
fn cross_spoke_kpi_warns() {
    let mut m = valid();
    m["dashboards"] = json!([{"slug": "health", "kpis": ["ord.atp_pct"]}]);
    let found = codes(&m);
    assert!(found.contains(&(Severity::Warning, "dashboards.cross_spoke_kpi".to_string())));
}

#[test]
fn validate_all_flags_collisions() {
    let a = valid();
    let mut b = valid();
    b["spoke"] = json!("ctr");
    b["provides"]["objects"] = json!(["srm.supplier"]);
    let results = validate_all(&[a, b]);
    let ctr = of(&results, "ctr");
    assert!(ctr.findings.iter().any(|f| f.code == "provides.collision"));
}

#[test]
fn validate_all_warns_when_owner_is_not_installed() {
    let mut m = valid();
    m["consumes"] = json!([{"field": "src.spend_usd_ttm", "on_missing": "null"}]);
    let results = validate_all(&[m]);
    let srm = of(&results, "srm");
    assert!(srm
        .findings
        .iter()
        .any(|f| f.code == "consumes.absent_owner"));
}

#[test]
fn json_side_channel_string_patterns_stay_rejected() {
    // A trailing newline must not sneak past the anchored regexes.
    let mut m = valid();
    m["spoke"] = json!("srm\n");
    let found = codes(&m);
    assert!(found.contains(&(Severity::Error, "spoke.code".to_string())));
}
