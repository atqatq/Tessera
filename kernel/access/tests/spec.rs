//! The permission engine's behavioural spec, as sentences (Part A1).
//!
//! Each test names one behaviour; the names are the readable contract.
//! The engine is pure: `evaluate(Request, Env) -> Decision` — no clock,
//! no I/O, no randomness (Part A3). Layer order is L0 module state, then
//! the actor path (ORIGIN / L1 tiers / L2 grants / L3 column rules).
//!
//! Written before the implementation: the first run fails to compile
//! because the API does not exist — that is the red state.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use tessera_access::{
    Action, Actor, DecisionCode, Effect, Env, Glob, Grant, Layer, Request, Rule, RuleAction, Tier,
    evaluate,
};
use tessera_ids::{EpochMs, ModuleId, RoleId, SubjectId, TenantId};

const T0: u64 = 1_700_000_000_000;

fn id(s: &str) -> SubjectId {
    SubjectId::new(s).expect("valid subject id")
}
fn module(s: &str) -> ModuleId {
    ModuleId::new(s).expect("valid module id")
}
fn role(s: &str) -> RoleId {
    RoleId::new(s).expect("valid role id")
}
fn tenant(s: &str) -> TenantId {
    TenantId::new(s).expect("valid tenant id")
}
fn at(ms: u64) -> EpochMs {
    EpochMs::new(ms)
}

fn base_env() -> Env {
    Env::new(at(T0))
        .with_known_columns(["sku", "qty", "price"])
        .with_rule(Rule {
            role: role("planner"),
            module: module("inv"),
            action: RuleAction::Read,
            column: Glob::new("*").expect("valid glob"),
            effect: Effect::Allow,
        })
}

fn user_request(target: &str, action: Action, columns: &[&str]) -> Request {
    Request {
        tenant: tenant("acme"),
        actor: Actor::User {
            subject: id("u-1"),
            role: role("planner"),
        },
        home: Some(module("inv")),
        target: module(target),
        action,
        columns: columns.iter().map(|c| (*c).to_string()).collect(),
        origin_intent: None,
    }
}

fn agent_request(tier: Tier, target: &str, action: Action, columns: &[&str]) -> Request {
    Request {
        tenant: tenant("acme"),
        actor: Actor::Agent {
            subject: id("ag-inv-1"),
            tier,
        },
        home: Some(module("inv")),
        target: module(target),
        action,
        columns: columns.iter().map(|c| (*c).to_string()).collect(),
        origin_intent: None,
    }
}

// ---------------------------------------------------------------- L0

#[test]
fn a_disabled_module_gates_everyone_including_origin() {
    let env = base_env().with_module_enabled(false);
    let mut req = user_request("inv", Action::Read, &["sku"]);
    req.actor = Actor::Origin {
        subject: id("root"),
    };
    req.origin_intent = Some(tessera_access::IntentRef::new("int-1").expect("valid intent"));
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::DenyModuleDisabled);
    assert_eq!(d.layer, Layer::L0);
}

// ---------------------------------------------------------------- ORIGIN

#[test]
fn origin_without_recorded_intent_is_refused() {
    let env = base_env();
    let mut req = user_request("inv", Action::Read, &["sku"]);
    req.actor = Actor::Origin {
        subject: id("root"),
    };
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::DenyIntentRequired);
    assert_eq!(d.layer, Layer::Origin);
}

#[test]
fn origin_with_intent_bypasses_column_rules_and_grants() {
    // no rules cover `not_a_real_column` (it is even unknown), yet ORIGIN reads.
    let env = base_env();
    let mut req = user_request("inv", Action::Read, &["not_a_real_column"]);
    req.actor = Actor::Origin {
        subject: id("root"),
    };
    req.origin_intent = Some(tessera_access::IntentRef::new("int-1").expect("valid intent"));
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::AllowOrigin);
    assert_eq!(d.layer, Layer::Origin);
}

// ---------------------------------------------------------------- L3 rules

#[test]
fn unknown_column_is_denied_even_when_a_star_rule_allows() {
    let env = base_env();
    let d = evaluate(&user_request("inv", Action::Read, &["nope"]), &env);
    assert_eq!(d.code, DecisionCode::DenyColumnUnknown);
    assert_eq!(d.layer, Layer::L3);
}

#[test]
fn explicit_deny_beats_allow_on_the_same_column() {
    let env = base_env().with_rule(Rule {
        role: role("planner"),
        module: module("inv"),
        action: RuleAction::Read,
        column: Glob::new("price").expect("valid glob"),
        effect: Effect::Deny,
    });
    let d = evaluate(&user_request("inv", Action::Read, &["price"]), &env);
    assert_eq!(d.code, DecisionCode::DenyRuleExplicit);
}

#[test]
fn uncovered_column_falls_through_to_default_deny() {
    // `qty` is known but no rule covers it (the star rule was removed).
    let env = Env::new(at(T0)).with_known_columns(["sku", "qty", "price"]);
    let d = evaluate(&user_request("inv", Action::Read, &["qty"]), &env);
    assert_eq!(d.code, DecisionCode::DenyDefault);
    assert_eq!(d.layer, Layer::L3);
}

#[test]
fn propose_is_judged_by_the_write_rules() {
    let env = base_env().with_rule(Rule {
        role: role("planner"),
        module: module("inv"),
        action: RuleAction::Write,
        column: Glob::new("qty").expect("valid glob"),
        effect: Effect::Allow,
    });
    // proposals are judged against the write rules, so this is allowed
    // even though no Read rule covers qty.
    let d = evaluate(&user_request("inv", Action::Propose, &["qty"]), &env);
    assert_eq!(d.code, DecisionCode::AllowRule);
}

#[test]
fn a_column_set_is_allowed_only_when_every_column_is_covered() {
    let env = base_env().with_rule(Rule {
        role: role("planner"),
        module: module("inv"),
        action: RuleAction::Write,
        column: Glob::new("qty").expect("valid glob"),
        effect: Effect::Allow,
    });
    // star rule covers reads of sku/price; qty covered explicitly for writes;
    // a write across all three must fail because sku/price have no write rule.
    let d = evaluate(
        &user_request("inv", Action::Write, &["sku", "qty", "price"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::DenyDefault);
    let d = evaluate(&user_request("inv", Action::Write, &["qty"]), &env);
    assert_eq!(d.code, DecisionCode::AllowRule);
}

#[test]
fn a_module_level_read_needs_the_star_rule() {
    // empty column set = module-level request, matched against the
    // sentinel column `*`. The star rule in base_env covers it.
    let env = base_env();
    let d = evaluate(&user_request("inv", Action::Read, &[]), &env);
    assert_eq!(d.code, DecisionCode::AllowRule);
}

#[test]
fn glob_exact_rules_do_not_match_longer_names() {
    let env = Env::new(at(T0))
        .with_known_columns(["qty", "qty_reserved"])
        .with_rule(Rule {
            role: role("planner"),
            module: module("inv"),
            action: RuleAction::Read,
            column: Glob::new("qty").expect("valid glob"),
            effect: Effect::Allow,
        });
    // exact rule `qty` must not cover `qty_reserved`
    let d = evaluate(&user_request("inv", Action::Read, &["qty_reserved"]), &env);
    assert_eq!(d.code, DecisionCode::DenyDefault);
}

#[test]
fn rules_for_other_roles_or_modules_do_not_apply() {
    let env = Env::new(at(T0))
        .with_known_columns(["sku"])
        .with_rule(Rule {
            role: role("auditor"),
            module: module("inv"),
            action: RuleAction::Read,
            column: Glob::new("*").expect("valid glob"),
            effect: Effect::Allow,
        });
    let d = evaluate(&user_request("inv", Action::Read, &["sku"]), &env);
    assert_eq!(d.code, DecisionCode::DenyDefault);
}

// ---------------------------------------------------------------- L1 agents

#[test]
fn agent_read_of_own_module_is_allowed_at_any_tier() {
    let env = base_env();
    for tier in [Tier::Observe, Tier::Advise, Tier::Act] {
        let d = evaluate(&agent_request(tier, "inv", Action::Read, &["sku"]), &env);
        assert_eq!(d.code, DecisionCode::AllowTier, "tier {tier:?} must read");
        assert_eq!(d.layer, Layer::L1);
    }
}

#[test]
fn observe_agent_may_not_propose() {
    let env = base_env();
    let d = evaluate(
        &agent_request(Tier::Observe, "inv", Action::Propose, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::DenyTierInsufficient);
}

#[test]
fn advise_agent_may_propose_but_not_write() {
    let env = base_env();
    let d = evaluate(
        &agent_request(Tier::Advise, "inv", Action::Propose, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::AllowTier);
    let d = evaluate(
        &agent_request(Tier::Advise, "inv", Action::Write, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::DenyTierInsufficient);
}

#[test]
fn act_agent_write_needs_an_allowlist_entry() {
    let env = base_env();
    let d = evaluate(
        &agent_request(Tier::Act, "inv", Action::Write, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::DenyAgentNotAllowlisted);
}

#[test]
fn act_agent_write_needs_origin_approval_even_when_allowlisted() {
    let env = base_env().with_agent_allowlist(module("inv"), Action::Write);
    let d = evaluate(
        &agent_request(Tier::Act, "inv", Action::Write, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::DenyOriginApprovalRequired);
}

#[test]
fn allowlisted_and_approved_act_agent_may_write() {
    let env = base_env()
        .with_agent_allowlist(module("inv"), Action::Write)
        .with_origin_approval(id("ag-inv-1"), module("inv"), Action::Write);
    let d = evaluate(
        &agent_request(Tier::Act, "inv", Action::Write, &["qty"]),
        &env,
    );
    assert_eq!(d.code, DecisionCode::AllowTier);
}

// ---------------------------------------------------------------- L2 grants

#[test]
fn peer_read_requires_a_grant() {
    let env = base_env();
    let mut req = user_request("ord", Action::Read, &["sku"]);
    req.home = Some(module("ful")); // ful reads ord, no grant exists
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::DenyGrantMissing);
    assert_eq!(d.layer, Layer::L2);
}

#[test]
fn a_grant_must_cover_every_requested_column() {
    let env = base_env().with_grant(Grant {
        owner: module("ord"),
        granted_to: module("ful"),
        columns: BTreeSet::from(["sku".to_string()]),
        expires_at: None,
    });
    let mut req = user_request("ord", Action::Read, &["sku", "qty"]);
    req.home = Some(module("ful"));
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::DenyGrantMissing);
}

#[test]
fn an_expired_grant_is_denied_at_the_expiry_instant() {
    // The grant opens the peer door; the owning company's L3 rules still
    // govern what may be read — so this env also allows planner reads on ord.
    let env = base_env()
        .with_grant(Grant {
            owner: module("ord"),
            granted_to: module("ful"),
            columns: BTreeSet::from(["sku".to_string()]),
            expires_at: Some(at(T0 + 100)),
        })
        .with_rule(Rule {
            role: role("planner"),
            module: module("ord"),
            action: RuleAction::Read,
            column: Glob::new("*").expect("valid glob"),
            effect: Effect::Allow,
        });
    let mut req = user_request("ord", Action::Read, &["sku"]);
    req.home = Some(module("ful"));
    // one tick before expiry: allowed
    let d = evaluate(&req, &env.clone().with_now(at(T0 + 99)));
    assert_eq!(d.code, DecisionCode::AllowGrant);
    // at the expiry instant: denied — fail closed
    let d = evaluate(&req, &env.with_now(at(T0 + 100)));
    assert_eq!(d.code, DecisionCode::DenyGrantExpired);
}

#[test]
fn peer_reads_honour_deny_wins_rules_after_the_grant() {
    // grant covers price, but the owner company denies `price` to `planner`
    let env = base_env()
        .with_grant(Grant {
            owner: module("ord"),
            granted_to: module("ful"),
            columns: BTreeSet::from(["price".to_string()]),
            expires_at: None,
        })
        .with_rule(Rule {
            role: role("planner"),
            module: module("ord"),
            action: RuleAction::Read,
            column: Glob::new("price").expect("valid glob"),
            effect: Effect::Deny,
        });
    let mut req = user_request("ord", Action::Read, &["price"]);
    req.home = Some(module("ful"));
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::DenyRuleExplicit);
    assert_eq!(d.layer, Layer::L3);
}

#[test]
fn peer_writes_do_not_exist_as_an_operation() {
    let env = base_env().with_grant(Grant {
        owner: module("ord"),
        granted_to: module("ful"),
        columns: BTreeSet::from(["qty".to_string()]),
        expires_at: None,
    });
    for action in [Action::Write, Action::Propose] {
        let mut req = user_request("ord", action, &["qty"]);
        req.home = Some(module("ful"));
        let d = evaluate(&req, &env);
        assert_eq!(
            d.code,
            DecisionCode::DenyDefault,
            "peer {action:?} must deny"
        );
    }
}

// ---------------------------------------------------------------- actors

#[test]
fn kernel_level_user_without_home_is_judged_by_rules_alone() {
    // an auditor with home=None: no grants apply; L3 rules decide.
    let env = base_env().with_rule(Rule {
        role: role("auditor"),
        module: module("inv"),
        action: RuleAction::Read,
        column: Glob::new("price").expect("valid glob"),
        effect: Effect::Allow,
    });
    let mut req = user_request("inv", Action::Read, &["price"]);
    req.actor = Actor::User {
        subject: id("u-audit"),
        role: role("auditor"),
    };
    req.home = None;
    let d = evaluate(&req, &env);
    assert_eq!(d.code, DecisionCode::AllowRule);
}
