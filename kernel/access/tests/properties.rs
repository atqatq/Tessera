//! Property tests for the invariants that must hold across all inputs
//! (Part A2 makes these mandatory for the permission engine).
//!
//! - deny always wins
//! - decisions are independent of rule/grant order
//! - propose and write verdicts agree for users (propose matches write rules)
//! - agent tiers are monotonic
//! - expired grants never allow, at any instant at or after expiry
//! - unknown columns are denied for everyone except ORIGIN
//! - ORIGIN never passes a disabled module

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use proptest::prelude::*;
use tessera_access::{
    Action, Actor, DecisionCode, Effect, Env, Glob, Grant, Request, Rule, RuleAction, Tier,
    evaluate,
};
use tessera_ids::{EpochMs, ModuleId, RoleId, SubjectId, TenantId};

// ------------------------------------------------------------ strategies

fn module_strategy() -> impl Strategy<Value = ModuleId> {
    prop_oneof![
        Just(module("inv")),
        Just(module("ord")),
        Just(module("ful"))
    ]
}

fn role_strategy() -> impl Strategy<Value = RoleId> {
    prop_oneof![
        Just(role("planner")),
        Just(role("auditor")),
        Just(role("op"))
    ]
}

fn column_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("sku".to_string()),
        Just("qty".to_string()),
        Just("price".to_string()),
        Just("mystery".to_string()), // intentionally unknown
    ]
}

fn glob_strategy() -> impl Strategy<Value = Glob> {
    prop_oneof![
        Just(Glob::new("*").expect("valid")),
        Just(Glob::new("sku").expect("valid")),
        Just(Glob::new("qty").expect("valid")),
        Just(Glob::new("qty_*").expect("valid")),
        Just(Glob::new("price").expect("valid")),
        Just(Glob::new("*_reserved").expect("valid")),
    ]
}

fn effect_strategy() -> impl Strategy<Value = Effect> {
    prop_oneof![Just(Effect::Allow), Just(Effect::Deny)]
}

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Read),
        Just(Action::Propose),
        Just(Action::Write)
    ]
}

fn rule_strategy() -> impl Strategy<Value = Rule> {
    // Rules govern Read or Write only — there is no propose rule (proposals
    // are judged by the write rules; the type makes that unrepresentable).
    (
        role_strategy(),
        module_strategy(),
        prop_oneof![Just(RuleAction::Read), Just(RuleAction::Write)],
        glob_strategy(),
        effect_strategy(),
    )
        .prop_map(|(role, module, action, column, effect)| Rule {
            role,
            module,
            action,
            column,
            effect,
        })
}

fn grant_strategy() -> impl Strategy<Value = Grant> {
    (
        module_strategy(),
        module_strategy(),
        prop::collection::vec(column_strategy(), 0..3),
        proptest::option::of(0u64..5).prop_map(|off| off.map(|o| EpochMs::new(100 + o))),
    )
        .prop_map(|(owner, granted_to, columns, expires_at)| Grant {
            owner,
            granted_to,
            columns: columns.into_iter().collect(),
            expires_at,
        })
}

#[allow(clippy::type_complexity)]
fn actor_strategy()
-> impl Strategy<Value = (Actor, Option<ModuleId>, ModuleId, Action, Vec<String>)> {
    prop_oneof![
        // user, own module
        (
            role_strategy(),
            module_strategy(),
            action_strategy(),
            prop::collection::vec(column_strategy(), 0..3)
        )
            .prop_map(|(r, m, a, cols)| {
                (
                    Actor::User {
                        subject: SubjectId::new("u-1").expect("valid"),
                        role: r,
                    },
                    Some(m.clone()),
                    m,
                    a,
                    cols,
                )
            }),
        // user, kernel-level (no home)
        (
            role_strategy(),
            module_strategy(),
            action_strategy(),
            prop::collection::vec(column_strategy(), 0..3)
        )
            .prop_map(|(r, m, a, cols)| {
                (
                    Actor::User {
                        subject: SubjectId::new("u-1").expect("valid"),
                        role: r,
                    },
                    None,
                    m,
                    a,
                    cols,
                )
            }),
        // agent, own module
        (
            prop_oneof![Just(Tier::Observe), Just(Tier::Advise), Just(Tier::Act)],
            module_strategy(),
            action_strategy(),
            prop::collection::vec(column_strategy(), 0..3)
        )
            .prop_map(|(t, m, a, cols)| {
                (
                    Actor::Agent {
                        subject: SubjectId::new("ag-1").expect("valid"),
                        tier: t,
                    },
                    Some(m.clone()),
                    m,
                    a,
                    cols,
                )
            }),
        // user peer read between two different modules
        (
            role_strategy(),
            module_strategy(),
            prop::collection::vec(column_strategy(), 1..3)
        )
            .prop_flat_map(|(r, home, cols)| {
                let home_for_filter = home.clone();
                (
                    module_strategy()
                        .prop_filter("modules must differ", move |t| t != &home_for_filter),
                    Just(home),
                    Just(r),
                    Just(cols),
                )
            })
            .prop_map(|(target, home, r, cols)| {
                (
                    Actor::User {
                        subject: SubjectId::new("u-1").expect("valid"),
                        role: r,
                    },
                    Some(home),
                    target,
                    Action::Read,
                    cols,
                )
            }),
        // agent peer read between two different modules
        (
            prop_oneof![Just(Tier::Observe), Just(Tier::Advise), Just(Tier::Act)],
            module_strategy(),
            prop::collection::vec(column_strategy(), 1..3),
        )
            .prop_flat_map(|(t, home, cols)| {
                let home_for_filter = home.clone();
                (
                    module_strategy()
                        .prop_filter("modules must differ", move |t2| t2 != &home_for_filter),
                    Just(home),
                    Just(t),
                    Just(cols),
                )
            })
            .prop_map(|(target, home, tier, cols)| {
                (
                    Actor::Agent {
                        subject: SubjectId::new("ag-1").expect("valid"),
                        tier,
                    },
                    Some(home),
                    target,
                    Action::Read,
                    cols,
                )
            }),
    ]
}

fn env_strategy() -> impl Strategy<Value = Env> {
    (
        0u64..5,
        prop::collection::vec(rule_strategy(), 0..6),
        prop::collection::vec(grant_strategy(), 0..3),
        prop::collection::vec(column_strategy(), 0..3),
    )
        .prop_map(|(now, rules, grants, known)| {
            let mut env = Env::new(EpochMs::new(now))
                .with_known_columns(["sku", "qty", "price"])
                .with_agent_allowlist(module("inv"), Action::Write)
                .with_origin_approval(
                    SubjectId::new("ag-1").expect("valid"),
                    module("inv"),
                    Action::Write,
                );
            for r in rules {
                env = env.with_rule(r);
            }
            for g in grants {
                env = env.with_grant(g);
            }
            for c in known {
                env = env.with_known_columns([c]);
            }
            env
        })
}

fn request_strategy() -> impl Strategy<Value = Request> {
    actor_strategy().prop_map(|(actor, home, target, action, cols)| Request {
        tenant: TenantId::new("acme").expect("valid"),
        actor,
        home,
        target,
        action,
        columns: cols.into_iter().collect(),
        origin_intent: None,
    })
}

fn module(s: &str) -> ModuleId {
    ModuleId::new(s).expect("valid module id")
}
fn role(s: &str) -> RoleId {
    RoleId::new(s).expect("valid role id")
}

// ------------------------------------------------------------ properties

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Deny always wins: if a rule denies any requested column, the whole
    /// request is denied, no matter what allow rules exist.
    #[test]
    fn explicit_deny_always_wins(
        env in env_strategy(),
        target in module_strategy(),
        r in role_strategy(),
        action in action_strategy(),
        col in column_strategy(),
    ) {
        // Find an allow-only env for this request by stripping deny rules
        // that match; if it allows, adding a matching deny must flip it.
        let req = Request {
            tenant: TenantId::new("acme").expect("valid"),
            actor: Actor::User { subject: SubjectId::new("u-1").expect("valid"), role: r.clone() },
            home: Some(target.clone()),
            target: target.clone(),
            action,
            columns: [col.clone()].into_iter().collect(),
            origin_intent: None,
        };
        // Only meaningful when the column is known; unknown columns deny anyway.
        let mut allow_env = env.clone();
        allow_env = allow_env.with_known_columns([col.clone()]);
        allow_env.rules.retain(|rule| rule.effect != Effect::Deny);
        let d_allow = evaluate(&req, &allow_env);
        if matches!(d_allow.code, DecisionCode::AllowRule) {
            // A deny rule in the same family as the request action must flip
            // the verdict: read denies read; write denies write and propose.
            let rule_action = match action {
                tessera_access::Action::Read => tessera_access::RuleAction::Read,
                tessera_access::Action::Propose | tessera_access::Action::Write => {
                    tessera_access::RuleAction::Write
                }
            };
            let deny_rule = Rule {
                role: r,
                module: target.clone(),
                action: rule_action,
                column: Glob::new("*").expect("valid"),
                effect: Effect::Deny,
            };
            let deny_env = allow_env.with_rule(deny_rule);
            let d_deny = evaluate(&req, &deny_env);
            prop_assert_eq!(d_deny.code, DecisionCode::DenyRuleExplicit);
        }
    }

    /// Decisions are independent of the order of rules and grants.
    #[test]
    fn decision_is_order_independent(
        env in env_strategy(),
        req in request_strategy(),
    ) {
        let mut reversed = env.clone();
        reversed.rules.reverse();
        reversed.grants.reverse();
        let a = evaluate(&req, &env);
        let b = evaluate(&req, &reversed);
        prop_assert_eq!(a.code, b.code);
        prop_assert_eq!(a.layer, b.layer);
    }

    /// Propose and write verdicts agree for users: proposals are judged by
    /// the write rules, so they can never diverge.
    #[test]
    fn propose_and_write_agree_for_users(
        env in env_strategy(),
        target in module_strategy(),
        r in role_strategy(),
        cols in prop::collection::vec(column_strategy(), 0..3),
    ) {
        let mk = |action| Request {
            tenant: TenantId::new("acme").expect("valid"),
            actor: Actor::User { subject: SubjectId::new("u-1").expect("valid"), role: r.clone() },
            home: Some(target.clone()),
            target: target.clone(),
            action,
            columns: cols.clone().into_iter().collect(),
            origin_intent: None,
        };
        // compare only own-module requests; peer writes do not exist
        let write = evaluate(&mk(Action::Write), &env);
        let propose = evaluate(&mk(Action::Propose), &env);
        let same = matches!(write.code, DecisionCode::AllowRule) == matches!(propose.code, DecisionCode::AllowRule);
        prop_assert!(same, "write {:?} vs propose {:?} on {}", write.code, propose.code, target.as_str());
    }

    /// Agent tiers are monotonic: whatever an Observe agent may do, Advise
    /// and Act may do; whatever Advise may do, Act may do.
    #[test]
    fn agent_tiers_are_monotonic(
        env in env_strategy(),
        target in module_strategy(),
        cols in prop::collection::vec(column_strategy(), 0..3),
    ) {
        let mk = |tier: Tier, action: Action| Request {
            tenant: TenantId::new("acme").expect("valid"),
            actor: Actor::Agent { subject: SubjectId::new("ag-1").expect("valid"), tier },
            home: Some(target.clone()),
            target: target.clone(),
            action,
            columns: cols.clone().into_iter().collect(),
            origin_intent: None,
        };
        for action in [Action::Read, Action::Propose] {
            let o = evaluate(&mk(Tier::Observe, action), &env);
            let a = evaluate(&mk(Tier::Advise, action), &env);
            let t = evaluate(&mk(Tier::Act, action), &env);
            let allows = |d: &tessera_access::Decision| matches!(d.code, DecisionCode::AllowTier);
            if allows(&o) {
                prop_assert!(allows(&a), "advise must do what observe does");
                prop_assert!(allows(&t), "act must do what observe does");
            }
            if allows(&a) {
                prop_assert!(allows(&t), "act must do what advise does");
            }
        }
    }

    /// A grant expired at instant t never allows at t or any later instant.
    #[test]
    fn expired_grants_never_allow(
        case in (env_strategy(), module_strategy(), column_strategy(), 5u64..50, 0u64..10)
            .prop_flat_map(|(env, target, col, expiry, off)| {
                let t = target.clone();
                (
                    Just(env),
                    Just(target),
                    module_strategy().prop_filter("home must differ", move |m| m != &t),
                    Just(col),
                    Just(expiry),
                    Just(expiry + off),
                )
            }),
    ) {
        let (env, target, home, col, expiry, now) = case;
        // The property isolates the one expiring grant: if the environment
        // already contains a covering grant for this request, allowing is
        // correct behaviour and says nothing about expiry.
        prop_assume!(!env.grants.iter().any(|g| {
            g.owner == target && g.granted_to == home && g.columns.contains(&col)
        }));
        let g = Grant {
            owner: target.clone(),
            granted_to: home.clone(),
            columns: [col.clone()].into_iter().collect(),
            expires_at: Some(EpochMs::new(expiry)),
        };
        let env = env.with_grant(g);
        let req = Request {
            tenant: TenantId::new("acme").expect("valid"),
            actor: Actor::User { subject: SubjectId::new("u-1").expect("valid"), role: role("planner") },
            home: Some(home),
            target,
            action: Action::Read,
            columns: [col].into_iter().collect(),
            origin_intent: None,
        };
        let d = evaluate(&req, &env.with_now(EpochMs::new(now)));
        prop_assert_ne!(d.code, DecisionCode::AllowGrant);
    }

    /// Unknown columns are denied for every non-ORIGIN actor, whatever the
    /// rules say.
    #[test]
    fn unknown_columns_are_denied(
        env in env_strategy(),
        req in request_strategy(),
    ) {
        prop_assume!(!matches!(req.actor, Actor::Origin { .. }));
        // every column not in known_columns must deny
        let unknown: Vec<String> = req
            .columns
            .iter()
            .filter(|c| !env.known_columns.contains(*c))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            let d = evaluate(&req, &env);
            prop_assert!(
                matches!(d.code, DecisionCode::DenyColumnUnknown | DecisionCode::DenyDefault),
                "unknown columns {:?} produced {:?}",
                unknown,
                d.code
            );
        }
    }

    /// ORIGIN never passes a disabled module — across every env.
    #[test]
    fn origin_never_passes_a_disabled_module(
        env in env_strategy(),
        target in module_strategy(),
        action in action_strategy(),
        intent in "[a-z0-9][a-z0-9._-]{0,10}",
    ) {
        let env = env.with_module_enabled(false);
        let req = Request {
            tenant: TenantId::new("acme").expect("valid"),
            actor: Actor::Origin { subject: SubjectId::new("root").expect("valid") },
            home: None,
            target,
            action,
            columns: BTreeSet::new(),
            origin_intent: Some(tessera_access::IntentRef::new(&intent).expect("valid")),
        };
        let d = evaluate(&req, &env);
        prop_assert_eq!(d.code, DecisionCode::DenyModuleDisabled);
    }
}
