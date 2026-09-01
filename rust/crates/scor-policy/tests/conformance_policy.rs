//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Port of the key scenarios from the Python reference's
//! `test_policy.py`: layered evaluation, deny-wins, state gating, agent
//! tiers and allowlists.

#![allow(clippy::indexing_slicing)]

use std::collections::{BTreeMap, BTreeSet};

use scor_policy::{Action, Decision, PolicyEngine, Principal, Role, Rule, Tier};

fn read_only() -> Vec<Rule> {
    vec![Rule::allow("*", &[Action::Read])]
}

fn all_actions() -> Vec<Rule> {
    vec![Rule::allow("*", &[Action::Read, Action::Write])]
}

fn engine() -> PolicyEngine {
    let mut roles = BTreeMap::new();
    roles.insert(
        "SRM".to_string(),
        Role::new(
            "SRM",
            vec![Rule::allow("srm.*", &[Action::Read, Action::Write])],
        ),
    );
    roles.insert("AUD".to_string(), Role::new("AUD", read_only()));
    roles.insert(
        "CSR".to_string(),
        Role::new(
            "CSR",
            vec![
                Rule::allow("ord.*", &[Action::Read, Action::Write]),
                Rule::allow("srm.supplier.tier", &[Action::Read]),
            ],
        ),
    );
    roles.insert(
        "RESTRICTED".to_string(),
        Role::new(
            "RESTRICTED",
            vec![
                Rule::allow("ord.*", &[Action::Read, Action::Write]),
                Rule::allow("srm.supplier.tier", &[Action::Read]),
                Rule::allow("srm.*", &[Action::Read]),
                Rule::deny("srm.supplier.negotiated_floor_usd", &[Action::Read]),
            ],
        ),
    );
    let mut spoke_access = BTreeMap::new();
    spoke_access.insert(
        "ord".to_string(),
        BTreeSet::from(["ctr".to_string(), "srm".to_string()]),
    );
    let mut spoke_states = BTreeMap::new();
    for (k, v) in [
        ("srm", "active"),
        ("ord", "active"),
        ("ctr", "active"),
        ("src", "active"),
        ("prj", "disabled"),
        ("trf", "paused"),
        ("ret", "archived"),
    ] {
        spoke_states.insert(k.to_string(), v.to_string());
    }
    PolicyEngine {
        roles,
        spoke_access,
        spoke_states,
        agent_act_allowlist: BTreeMap::new(),
    }
}

fn agent_engine() -> PolicyEngine {
    let mut e = engine();
    e.roles.insert(
        "AIS".to_string(),
        Role::new(
            "AIS",
            vec![Rule::allow(
                "srm.*",
                &[Action::Read, Action::Propose, Action::Write],
            )],
        ),
    );
    e.roles.insert(
        "AIL".to_string(),
        Role::new(
            "AIL",
            vec![Rule::allow("*", &[Action::Read, Action::Propose])],
        ),
    );
    e.agent_act_allowlist
        .insert("srm".to_string(), vec!["srm.scorecard.*".to_string()]);
    e
}

fn human(subject: &str, roles: &[&str]) -> Principal {
    Principal {
        subject: subject.to_string(),
        tenant: "acme_gulf".to_string(),
        roles: roles.iter().map(|s| (*s).to_string()).collect(),
        ..Principal::default()
    }
}

fn agent(subject: &str, roles: &[&str], tier: Tier) -> Principal {
    Principal {
        subject: subject.to_string(),
        tenant: "acme_gulf".to_string(),
        roles: roles.iter().map(|s| (*s).to_string()).collect(),
        agent: true,
        agent_tier: Some(tier),
        ..Principal::default()
    }
}

fn assert_allowed(d: Decision) {
    assert!(d.allowed, "expected allow, got {}: {}", d.code, d.reason);
}

fn assert_denied_code(d: Decision, code: &str) {
    assert!(
        !d.allowed,
        "expected deny {code}, got allowed: {}",
        d.reason
    );
    assert_eq!(d.code, code);
}

#[test]
fn matching_role_allows_read() {
    assert_allowed(engine().decide(
        &human("atique", &["SRM"]),
        "srm",
        "srm.supplier.tier",
        Action::Read,
    ));
}

#[test]
fn no_role_is_a_deny() {
    assert_denied_code(
        engine().decide(
            &human("ghost", &[]),
            "srm",
            "srm.supplier.tier",
            Action::Read,
        ),
        "no_role_grant",
    );
}

#[test]
fn unknown_role_is_ignored_not_trusted() {
    assert_denied_code(
        engine().decide(
            &human("ghost", &["GHOST"]),
            "srm",
            "srm.supplier.tier",
            Action::Read,
        ),
        "no_role_grant",
    );
}

#[test]
fn read_only_role_cannot_write() {
    assert_denied_code(
        engine().decide(
            &human("aud", &["AUD"]),
            "srm",
            "srm.supplier.tier",
            Action::Write,
        ),
        "no_role_grant",
    );
}

#[test]
fn object_access_does_not_imply_every_column() {
    assert_denied_code(
        engine().decide(
            &human("csr", &["CSR"]),
            "srm",
            "srm.supplier.negotiated_floor_usd",
            Action::Read,
        ),
        "no_role_grant",
    );
}

#[test]
fn deny_beats_allow_regardless_of_rule_order() {
    assert_denied_code(
        engine().decide(
            &human("r", &["RESTRICTED"]),
            "srm",
            "srm.supplier.negotiated_floor_usd",
            Action::Read,
        ),
        "role_deny",
    );
    // Even when a second, wider role also grants the column.
    assert_denied_code(
        engine().decide(
            &human("r", &["RESTRICTED", "SRM"]),
            "srm",
            "srm.supplier.negotiated_floor_usd",
            Action::Read,
        ),
        "role_deny",
    );
}

#[test]
fn visible_columns_filters_a_projection() {
    let e = engine();
    let fields: Vec<String> = ["srm.supplier.tier", "srm.supplier.negotiated_floor_usd"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        e.visible_columns(&human("csr", &["CSR"]), "srm", &fields),
        vec!["srm.supplier.tier".to_string()]
    );
}

#[test]
fn unqualified_field_is_rejected() {
    assert_denied_code(
        engine().decide(&human("sr", &["SRM"]), "srm", "supplier", Action::Read),
        "unqualified_field",
    );
}

#[test]
fn owning_spoke_needs_no_grant() {
    assert_allowed(engine().decide(
        &human("sr", &["SRM"]),
        "srm",
        "srm.supplier.tier",
        Action::Write,
    ));
}

#[test]
fn cross_spoke_read_without_a_grant_is_denied() {
    // `src` is active but not in ord's origin grant list.
    assert_denied_code(
        engine().decide(
            &human("aud", &["AUD"]),
            "ord",
            "src.spend_usd_ttm",
            Action::Read,
        ),
        "no_spoke_grant",
    );
}

#[test]
fn cross_spoke_read_with_a_grant_then_role_layer() {
    // ord has an origin grant to srm; the role layer still decides.
    assert_allowed(engine().decide(
        &human("aud", &["AUD"]),
        "ord",
        "srm.supplier.tier",
        Action::Read,
    ));
    assert_denied_code(
        engine().decide(
            &human("nobody", &[]),
            "ord",
            "srm.supplier.tier",
            Action::Read,
        ),
        "no_role_grant",
    );
}

#[test]
fn a_spoke_may_never_write_another_spokes_column() {
    assert_denied_code(
        engine().decide(
            &human("sr", &["SRM"]),
            "ord",
            "srm.supplier.tier",
            Action::Write,
        ),
        "cross_spoke_write",
    );
}

#[test]
fn disabled_spoke_still_serves_reads_flagged_stale() {
    let d = engine().decide(
        &human("aud", &["AUD"]),
        "prj",
        "prj.milestone",
        Action::Read,
    );
    assert_allowed(d.clone());
    assert!(d.stale);
}

#[test]
fn disabled_spoke_refuses_writes_even_for_origin() {
    let mut e = engine();
    e.roles
        .insert("ALL".to_string(), Role::new("ALL", all_actions()));
    assert_denied_code(
        e.decide(
            &human("all", &["ALL"]),
            "prj",
            "prj.milestone",
            Action::Write,
        ),
        "spoke_state",
    );
    let origin = Principal {
        subject: "origin".to_string(),
        tenant: "acme_gulf".to_string(),
        origin_session: true,
        intent: Some("re-enable prj".to_string()),
        ..Principal::default()
    };
    assert_denied_code(
        e.decide(&origin, "hub", "prj.milestone", Action::Write),
        "spoke_state",
    );
}

#[test]
fn paused_spoke_reads_are_flagged_stale() {
    let d = engine().decide(&human("aud", &["AUD"]), "trf", "trf.oee", Action::Read);
    assert_allowed(d.clone());
    assert!(d.stale);
}

#[test]
fn archived_spoke_is_not_readable_live() {
    assert_denied_code(
        engine().decide(&human("aud", &["AUD"]), "ret", "ret.rma", Action::Read),
        "spoke_state",
    );
}

#[test]
fn unknown_spoke_defaults_to_denied() {
    // `inv` has no recorded state: it defaults to planned, and a planned
    // spoke serves nothing.
    assert_denied_code(
        engine().decide(&human("aud", &["AUD"]), "inv", "inv.on_hand", Action::Read),
        "spoke_state",
    );
}

#[test]
fn origin_bypasses_role_and_spoke_layers() {
    let origin = Principal {
        subject: "origin".to_string(),
        tenant: "acme_gulf".to_string(),
        origin_session: true,
        intent: Some("rotate keys".to_string()),
        ..Principal::default()
    };
    assert_allowed(engine().decide(
        &origin,
        "hub",
        "srm.supplier.negotiated_floor_usd",
        Action::Write,
    ));
}

#[test]
fn origin_without_intent_is_refused() {
    let origin = Principal {
        subject: "origin".to_string(),
        tenant: "acme_gulf".to_string(),
        origin_session: true,
        ..Principal::default()
    };
    assert_denied_code(
        engine().decide(&origin, "hub", "srm.supplier.tier", Action::Read),
        "origin_no_intent",
    );
}

#[test]
fn propose_is_the_agent_path() {
    assert_denied_code(
        engine().decide(
            &human("sr", &["SRM"]),
            "srm",
            "srm.supplier.tier",
            Action::Propose,
        ),
        "propose_is_for_agents",
    );
}

#[test]
fn advise_tier_agent_may_read_and_propose() {
    let e = agent_engine();
    let a = agent("srm-ai", &["AIS"], Tier::Advise);
    assert_allowed(e.decide(&a, "srm", "srm.supplier.tier", Action::Read));
    assert_allowed(e.decide(&a, "srm", "srm.supplier.tier", Action::Propose));
}

#[test]
fn advise_tier_agent_cannot_write() {
    let d = agent_engine().decide(
        &agent("srm-ai", &["AIS"], Tier::Advise),
        "srm",
        "srm.supplier.tier",
        Action::Write,
    );
    assert_denied_code(d, "agent_write_forbidden");
}

#[test]
fn observe_tier_agent_cannot_even_propose() {
    let d = agent_engine().decide(
        &agent("srm-ai", &["AIS"], Tier::Observe),
        "srm",
        "srm.supplier.tier",
        Action::Propose,
    );
    assert_denied_code(d, "agent_tier");
}

#[test]
fn agent_without_a_tier_is_refused() {
    let mut a = agent("srm-ai", &["AIS"], Tier::Advise);
    a.agent_tier = None;
    assert_denied_code(
        agent_engine().decide(&a, "srm", "srm.supplier.tier", Action::Read),
        "agent_tier",
    );
}

#[test]
fn agent_cannot_hold_an_origin_session() {
    let mut a = agent("srm-ai", &["AIS"], Tier::Act);
    a.origin_session = true;
    a.intent = Some("i should not exist".to_string());
    assert_denied_code(
        agent_engine().decide(&a, "srm", "srm.supplier.tier", Action::Read),
        "agent_origin_forbidden",
    );
}

#[test]
fn act_tier_agent_writes_only_inside_the_allowlist() {
    let e = agent_engine();
    let a = agent("srm-ai", &["AIS"], Tier::Act);
    assert_allowed(e.decide(&a, "srm", "srm.scorecard.health_index", Action::Write));
    assert_denied_code(
        e.decide(&a, "srm", "srm.supplier.tier", Action::Write),
        "agent_not_in_allowlist",
    );
}

#[test]
fn agent_act_allowlist_never_crosses_spokes() {
    let e = agent_engine();
    let a = agent("ord-ai", &["AIL"], Tier::Act);
    // Act tier passes the write gate, but there is no allowlist for ord:
    // the agent is refused before the role layer is even consulted.
    assert_denied_code(
        e.decide(&a, "ord", "ord.line.qty", Action::Write),
        "agent_not_in_allowlist",
    );
}

#[test]
fn every_decision_carries_a_reason() {
    let e = engine();
    let principals = vec![
        human("sr", &["SRM"]),
        human("nobody", &[]),
        agent("srm-ai", &["AIS"], Tier::Advise),
    ];
    let fields = [
        "srm.supplier.tier",
        "supplier",
        "ret.rma",
        "srm.supplier.negotiated_floor_usd",
    ];
    for p in &principals {
        for field in fields {
            for action in [Action::Read, Action::Write, Action::Propose] {
                let d = e.decide(p, "srm", field, action);
                assert!(!d.reason.is_empty(), "empty reason for {field} {action}");
                assert!(!d.code.is_empty(), "empty code for {field} {action}");
            }
        }
    }
}

#[test]
fn glob_semantics_match_fnmatchcase_subset() {
    assert!(scor_policy::glob_match("srm.*", "srm.supplier.tier"));
    assert!(scor_policy::glob_match("*", "anything.at.all"));
    assert!(scor_policy::glob_match("srm.?upplier", "srm.supplier"));
    assert!(!scor_policy::glob_match("srm.*", "ord.order"));
    assert!(!scor_policy::glob_match(
        "srm.supplier",
        "srm.supplier.tier"
    ));
}
