//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Column-level access control.
//!
//! Permission is evaluated at the column, not the object. Tenants add
//! their own fields to shared objects, and those fields carry their own
//! sensitivity: two users can both read `srm.supplier` while only one of
//! them may read `srm.supplier.negotiated_floor_usd`.
//!
//! Every decision passes independent layers, and deny beats allow in all
//! of them:
//!
//! 0. **State layer.** The owning spoke's lifecycle state decides what is
//!    possible at all. State is a machine fact, not a permission.
//! 1. **Agent layer.** AI principals are constrained further than the
//!    humans they work for. An agent proposes; it does not commit, unless
//!    origin has granted it a narrow, reversible act allowlist.
//! 2. **Spoke layer.** If the calling spoke does not own the field, origin
//!    must have granted that spoke read access to the owning spoke. This
//!    is the layer that stops a spoke reading another spoke behind the
//!    hub's back.
//! 3. **Principal layer.** The caller's roles must permit the action on
//!    that specific column.
//!
//! A missing grant is a deny.
//!
//! Behaviour is held to `reference/python/tests/test_policy.py`; the Rust
//! and Python engines must stay in lockstep.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Action being attempted on a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Read a column.
    Read,
    /// Write a column.
    Write,
    /// Propose a change (the agent path; a human applies it).
    Propose,
}

impl Action {
    /// Stable machine spelling, matching the Python reference.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Propose => "propose",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Agent capability tiers, least to most privileged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// See, never touch.
    Observe,
    /// Propose changes for humans to apply.
    Advise,
    /// Write within a narrow, origin-approved allowlist.
    Act,
}

impl Tier {
    /// Parses a tier spelling (`observe` / `advise` / `act`).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "observe" => Some(Self::Observe),
            "advise" => Some(Self::Advise),
            "act" => Some(Self::Act),
            _ => None,
        }
    }

    /// Stable machine spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Advise => "advise",
            Self::Act => "act",
        }
    }
}

/// The outcome of an access check. Always carries a reason: a denial the
/// operator cannot explain becomes a support ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// Whether the action is permitted.
    pub allowed: bool,
    /// Stable machine code, e.g. `no_spoke_grant`.
    pub code: String,
    /// Human-readable explanation.
    pub reason: String,
    /// True when the value comes from a paused or disabled spoke.
    pub stale: bool,
}

impl Decision {
    fn deny(code: &str, reason: String) -> Self {
        Self {
            allowed: false,
            code: code.to_string(),
            reason,
            stale: false,
        }
    }
}

/// Who is asking.
///
/// `agent` and `origin_session` are mutually exclusive. No model instance
/// holds origin, under any configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Principal {
    /// Stable subject identifier.
    pub subject: String,
    /// Tenant the session is scoped to.
    pub tenant: String,
    /// Role codes held by the subject, in evaluation order.
    pub roles: Vec<String>,
    /// True only inside a verified origin session.
    pub origin_session: bool,
    /// Recorded intent. Mandatory for origin sessions.
    pub intent: Option<String>,
    /// True for model principals.
    pub agent: bool,
    /// Capability tier for model principals.
    pub agent_tier: Option<Tier>,
}

/// Rule effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Permit the matched action.
    Allow,
    /// Forbid the matched action; deny wins.
    Deny,
}

/// A single column rule attached to a role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Exact name or glob, e.g. `srm.*` or `srm.supplier.tier`.
    pub field: String,
    /// Actions the rule speaks about.
    pub actions: BTreeSet<Action>,
    /// Allow or deny.
    pub effect: Effect,
}

impl Rule {
    /// Builds an allow rule for the given actions.
    #[must_use]
    pub fn allow(field: &str, actions: &[Action]) -> Self {
        Self {
            field: field.to_string(),
            actions: actions.iter().copied().collect(),
            effect: Effect::Allow,
        }
    }

    /// Builds a deny rule for the given actions.
    #[must_use]
    pub fn deny(field: &str, actions: &[Action]) -> Self {
        Self {
            field: field.to_string(),
            actions: actions.iter().copied().collect(),
            effect: Effect::Deny,
        }
    }

    /// True when this rule speaks about `action` on `field`.
    #[must_use]
    pub fn matches(&self, field: &str, action: Action) -> bool {
        self.actions.contains(&action) && glob_match(&self.field, field)
    }
}

/// A role: a code plus its column rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Role {
    /// Role code, e.g. `SRM`.
    pub code: String,
    /// Column rules, evaluated in order; deny wins immediately.
    pub rules: Vec<Rule>,
}

impl Role {
    /// Builds a role from rules.
    #[must_use]
    pub fn new(code: &str, rules: Vec<Rule>) -> Self {
        Self {
            code: code.to_string(),
            rules,
        }
    }
}

/// The policy engine: role registry plus the origin-granted spoke graph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyEngine {
    /// Role registry by code.
    pub roles: BTreeMap<String, Role>,
    /// Origin-granted spoke-to-spoke read access: `ord -> {ctr, inv}`.
    pub spoke_access: BTreeMap<String, BTreeSet<String>>,
    /// Current lifecycle state per spoke.
    pub spoke_states: BTreeMap<String, String>,
    /// Origin-approved act allowlist per spoke.
    pub agent_act_allowlist: BTreeMap<String, Vec<String>>,
}

/// Spoke states that still serve reads, and which of them are stale.
fn state_allows(state: &str, action: Action) -> bool {
    matches!(
        (action, state),
        (Action::Read, "active" | "paused" | "disabled")
            | (Action::Write, "active")
            | (Action::Propose, "active")
    )
}

fn state_is_stale(state: &str) -> bool {
    state == "paused" || state == "disabled"
}

impl PolicyEngine {
    /// Evaluate a single access request.
    #[must_use]
    pub fn decide(
        &self,
        principal: &Principal,
        calling_spoke: &str,
        field: &str,
        action: Action,
    ) -> Decision {
        if !field.contains('.') {
            return Decision::deny("unqualified_field", format!("{field:?} is not namespaced"));
        }

        let owner = field.split_once('.').map_or("", |(o, _)| o);
        let state = self
            .spoke_states
            .get(owner)
            .map(String::as_str)
            .unwrap_or("planned");

        // -- Layer 0: does the owning spoke's state permit this at all? --
        if !state_allows(state, action) {
            return Decision::deny(
                "spoke_state",
                format!(
                    "spoke {owner:?} is {state}; {} is not available",
                    action.as_str()
                ),
            );
        }
        let stale = state_is_stale(state);

        // -- Layer 1: agent constraints ----------------------------------
        if principal.agent {
            if principal.origin_session {
                return Decision::deny(
                    "agent_origin_forbidden",
                    "no model principal may hold an origin session".to_string(),
                );
            }
            if let Some(refusal) = self.check_agent(principal, calling_spoke, field, action) {
                return refusal;
            }
        } else if action == Action::Propose {
            return Decision::deny(
                "propose_is_for_agents",
                "human principals write directly; propose is the agent path".to_string(),
            );
        }

        // Origin bypasses the role layer but not the state layer or the
        // ledger.
        if principal.origin_session {
            let Some(_intent) = &principal.intent else {
                return Decision::deny(
                    "origin_no_intent",
                    "origin sessions must record an intent statement before acting".to_string(),
                );
            };
            return Decision {
                allowed: true,
                code: "origin".to_string(),
                reason: "origin session, logged to the ledger".to_string(),
                stale,
            };
        }

        // -- Layer 2: spoke to spoke --------------------------------------
        if calling_spoke != owner {
            let granted = self
                .spoke_access
                .get(calling_spoke)
                .is_some_and(|set| set.contains(owner));
            if !granted {
                return Decision::deny(
                    "no_spoke_grant",
                    format!("spoke {calling_spoke:?} has no origin grant to read {owner:?}"),
                );
            }
            if action != Action::Read {
                return Decision::deny(
                    "cross_spoke_write",
                    format!("spoke {calling_spoke:?} may read {owner:?} but never change it"),
                );
            }
        }

        // -- Layer 3: principal roles, column level, deny wins ------------
        let mut matched_allow: Option<&str> = None;
        for role_code in &principal.roles {
            let Some(role) = self.roles.get(role_code) else {
                continue;
            };
            for rule in &role.rules {
                if !rule.matches(field, action) {
                    continue;
                }
                if rule.effect == Effect::Deny {
                    return Decision::deny(
                        "role_deny",
                        format!(
                            "role {role_code:?} explicitly denies {} on {field:?}",
                            action.as_str()
                        ),
                    );
                }
                matched_allow = Some(role_code);
            }
        }

        let Some(role_code) = matched_allow else {
            return Decision::deny(
                "no_role_grant",
                format!(
                    "no role held by {:?} permits {} on {field:?}",
                    principal.subject,
                    action.as_str()
                ),
            );
        };
        Decision {
            allowed: true,
            code: "allowed".to_string(),
            reason: format!("granted by role {role_code:?}"),
            stale,
        }
    }

    /// Return a refusal, or `None` to continue through the normal layers.
    fn check_agent(
        &self,
        principal: &Principal,
        calling_spoke: &str,
        field: &str,
        action: Action,
    ) -> Option<Decision> {
        let Some(tier) = principal.agent_tier else {
            return Some(Decision::deny(
                "agent_tier",
                format!("agent {:?} has no valid capability tier", principal.subject),
            ));
        };
        if action == Action::Propose && tier == Tier::Observe {
            return Some(Decision::deny(
                "agent_tier",
                format!(
                    "agent {:?} is observe-tier and cannot propose",
                    principal.subject
                ),
            ));
        }
        if action == Action::Write {
            if tier != Tier::Act {
                return Some(Decision::deny(
                    "agent_write_forbidden",
                    format!(
                        "agent {:?} is {}-tier; agents propose rather than write, and a \
                         human applies the change",
                        principal.subject,
                        tier.as_str()
                    ),
                ));
            }
            let listed = self
                .agent_act_allowlist
                .get(calling_spoke)
                .into_iter()
                .flatten()
                .any(|pattern| glob_match(pattern, field));
            if !listed {
                return Some(Decision::deny(
                    "agent_not_in_allowlist",
                    format!(
                        "{field:?} is outside the origin-approved act allowlist for agent {:?}",
                        principal.subject
                    ),
                ));
            }
        }
        None
    }

    /// Filter a projection down to what the principal may actually read.
    #[must_use]
    pub fn visible_columns(
        &self,
        principal: &Principal,
        calling_spoke: &str,
        fields: &[String],
    ) -> Vec<String> {
        fields
            .iter()
            .filter(|f| {
                self.decide(principal, calling_spoke, f, Action::Read)
                    .allowed
            })
            .cloned()
            .collect()
    }
}

/// Minimal `fnmatch`-style glob: `*` matches any sequence, `?` matches one
/// character, everything else is literal. Case-sensitive, like Python's
/// `fnmatch.fnmatchcase` on ASCII patterns.
#[must_use]
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    glob_iter(&p, &t)
}

fn glob_iter(p: &[char], t: &[char]) -> bool {
    let Some((pc, prest)) = p.split_first() else {
        return t.is_empty();
    };
    match *pc {
        '*' => (0..=t.len())
            .filter_map(|i| t.get(i..))
            .any(|suffix| glob_iter(prest, suffix)),
        '?' => match t.split_first() {
            Some((_, trest)) => glob_iter(prest, trest),
            None => false,
        },
        c => match t.split_first() {
            Some((tc, trest)) => c == *tc && glob_iter(prest, trest),
            None => false,
        },
    }
}
