//! The Tessera permission engine: a pure decision function.
//!
//! `evaluate(&Request, &Env) -> Decision` is the whole API surface. There is
//! no I/O, no clock, no randomness (Part A3): the caller injects the current
//! instant as [`Env::now`], and every collection is ordered or order-agnostic,
//! so the same inputs always produce the same decision. That purity is what
//! makes the deny-wins properties testable across all inputs and what lets
//! the Python reference reproduce this engine byte-for-byte from conformance
//! vectors.
//!
//! # Layers
//!
//! | Layer | Concern | Denies with |
//! |---|---|---|
//! | L0 | module state | [`DecisionCode::DenyModuleDisabled`] — gates everyone, including ORIGIN |
//! | ORIGIN | superuser path | [`DecisionCode::DenyIntentRequired`] without recorded intent; otherwise bypasses L2/L3 only |
//! | L1 | agent tiers | [`DecisionCode::DenyTierInsufficient`], [`DecisionCode::DenyAgentNotAllowlisted`], [`DecisionCode::DenyOriginApprovalRequired`] |
//! | L2 | peer-read grants | [`DecisionCode::DenyGrantMissing`], [`DecisionCode::DenyGrantExpired`] (expiry instant fails closed) |
//! | L3 | column role rules | [`DecisionCode::DenyRuleExplicit`] (deny wins), [`DecisionCode::DenyColumnUnknown`], [`DecisionCode::DenyDefault`] |
//!
//! Two invariants are structural, not incidental:
//!
//! - **Deny wins.** An explicit deny on any requested column denies the whole
//!   request, whatever allow rules exist.
//! - **Default deny.** Anything not positively allowed is denied. Unknown
//!   columns are denied for every actor except ORIGIN — a column the module
//!   does not declare cannot be read, written, or proposed.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

use tessera_ids::{EpochMs, ModuleId, RoleId, SubjectId};

// ---------------------------------------------------------------- actions

/// What the actor wants to do to the target module.
///
/// [`Action::Propose`] is judged by the same rules as [`Action::Write`]:
/// agents propose, humans commit, and a proposal can therefore never be
/// allowed where a write would be denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// Read columns of the target module.
    Read,
    /// Draft a change, judged by the write rules; commits are human writes.
    Propose,
    /// Commit a change to the target module.
    Write,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Read => f.write_str("read"),
            Action::Propose => f.write_str("propose"),
            Action::Write => f.write_str("write"),
        }
    }
}

/// Effect of a column rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Positively allow matching columns.
    Allow,
    /// Deny matching columns; deny always wins.
    Deny,
}

/// Agent capability tier (L1). Ordered: `Observe < Advise < Act`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Read + report only.
    Observe,
    /// Draft, rank, propose; humans commit.
    Advise,
    /// Execute within an allowlist plus ORIGIN approval.
    Act,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tier::Observe => f.write_str("observe"),
            Tier::Advise => f.write_str("advise"),
            Tier::Act => f.write_str("act"),
        }
    }
}

// ---------------------------------------------------------------- actors

/// The principal making the request.
///
/// ORIGIN is a distinct variant, not a flag on a user or agent — an agent
/// holding ORIGIN is unrepresentable by construction (no agent can be
/// named as an [`Actor::Origin`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// A human (or service identity) with a role from the tenancy registry.
    User {
        /// Who acts; audited with every decision.
        subject: SubjectId,
        /// The role whose L3 column rules apply.
        role: RoleId,
    },
    /// A built-in or user-defined agent, bounded by its tier.
    Agent {
        /// Who acts; audited with every decision.
        subject: SubjectId,
        /// The agent's capability tier.
        tier: Tier,
    },
    /// The superuser path above root. Requires recorded intent on the
    /// request, bypasses L2/L3 only, never L0.
    Origin {
        /// Who holds the origin key; audited with every decision.
        subject: SubjectId,
    },
}

/// A reference to the recorded intent that must precede every ORIGIN
/// action (intent logged before effect). Validated with the shared
/// identifier grammar via [`tessera_ids::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentRef(Box<str>);

impl IntentRef {
    /// Validates and wraps an intent reference.
    pub fn new(s: &str) -> Result<Self, tessera_ids::InvalidId> {
        tessera_ids::validate(s)?;
        Ok(Self(s.into()))
    }

    /// The validated intent reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IntentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------- rules

/// A column glob: the only wildcard is `*`, matching any sequence
/// (including the empty one). All other characters are literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Glob(Box<str>);

impl Glob {
    /// Validates and wraps a glob pattern. Patterns must be non-empty.
    pub fn new(pattern: &str) -> Result<Self, InvalidGlob> {
        if pattern.is_empty() {
            return Err(InvalidGlob::Empty);
        }
        Ok(Self(pattern.into()))
    }

    /// Whether `s` matches this pattern.
    ///
    /// `*` matches any sequence; everything else is literal, so the exact
    /// pattern `qty` matches only `qty` and never `qty_reserved` — widen
    /// deliberately with `qty_*`, never accidentally.
    pub fn matches(&self, s: &str) -> bool {
        let pattern = &self.0;
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 1 {
            return s == self.0.as_ref();
        }
        let mut rest = s;
        let last = parts.len() - 1;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i == 0 {
                if !rest.starts_with(part) {
                    return false;
                }
                rest = &rest[part.len()..];
            } else if i == last {
                if !rest.ends_with(part) {
                    return false;
                }
            } else {
                match rest.find(part) {
                    Some(pos) => rest = &rest[pos + part.len()..],
                    None => return false,
                }
            }
        }
        true
    }
}

/// Why a glob pattern was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidGlob {
    /// The empty pattern matches nothing and is always a mistake.
    #[error("glob pattern must not be empty")]
    Empty,
}

/// The action a rule governs.
///
/// Only `Read` and `Write` exist. There is deliberately no propose rule:
/// proposals are judged by the write rules (agents propose, humans commit),
/// so a "propose rule" would be an illegal state — this type makes it
/// unrepresentable (Part A4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleAction {
    /// Governs reads (and only reads).
    Read,
    /// Governs writes and proposals.
    Write,
}

impl RuleAction {
    /// Whether this rule action applies to the request action.
    fn applies_to(self, action: Action) -> bool {
        matches!(
            (self, action),
            (RuleAction::Read, Action::Read)
                | (RuleAction::Write, Action::Write)
                | (RuleAction::Write, Action::Propose)
        )
    }
}

impl fmt::Display for RuleAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleAction::Read => f.write_str("read"),
            RuleAction::Write => f.write_str("write"),
        }
    }
}

/// An L3 column rule: one role, one module, one action, one column glob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// The role this rule applies to.
    pub role: RoleId,
    /// The module whose columns this rule governs.
    pub module: ModuleId,
    /// The action this rule governs. A write rule also governs proposals.
    pub action: RuleAction,
    /// The column pattern this rule matches.
    pub column: Glob,
    /// Allow or deny. Deny always wins.
    pub effect: Effect,
}

/// An L2 peer-read grant: the owning module lets `granted_to` read exactly
/// these columns. Grants are column-exact (no globs — globs are an L3
/// concern), issued by ORIGIN, and never confer writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    /// The module that owns the data.
    pub owner: ModuleId,
    /// The module whose actors may read it.
    pub granted_to: ModuleId,
    /// The exact readable columns.
    pub columns: BTreeSet<String>,
    /// Expiry instant, inclusive: at `expires_at` the grant is already
    /// expired (fail closed). `None` means the grant lives until revoked.
    pub expires_at: Option<EpochMs>,
}

// ---------------------------------------------------------------- request

/// A permission request: who wants to do what to which columns of which
/// module, inside which tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Tenant scope; the kernel evaluates within one tenancy.
    pub tenant: tessera_ids::TenantId,
    /// Who acts.
    pub actor: Actor,
    /// The module the actor belongs to. `None` marks a kernel-level actor
    /// (an auditor role, ORIGIN itself); such actors are judged by L3 rules
    /// alone — L2 grants exist between modules, not between a person and
    /// the kernel.
    pub home: Option<ModuleId>,
    /// The module being accessed.
    pub target: ModuleId,
    /// What they want to do.
    pub action: Action,
    /// The requested columns. An empty set is a module-level request,
    /// evaluated against the sentinel column `*` — only the `*` glob
    /// covers it.
    pub columns: BTreeSet<String>,
    /// Recorded intent, required for ORIGIN (intent logged before effect).
    pub origin_intent: Option<IntentRef>,
}

// ---------------------------------------------------------------- env

/// Everything the engine needs to know about the world, injected by the
/// caller. No clock, no I/O: the same `Env` plus the same `Request` always
/// produces the same `Decision` (Part A3 determinism).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    /// The current instant, injected. Grant expiry is judged against it.
    pub now: EpochMs,
    /// Whether the target module is enabled (L0). A disabled module gates
    /// everyone — even ORIGIN re-enables first.
    pub module_enabled: bool,
    /// Columns the target module actually declares. Unknown columns deny.
    pub known_columns: BTreeSet<String>,
    /// L3 column role rules.
    pub rules: Vec<Rule>,
    /// L2 peer-read grants.
    pub grants: Vec<Grant>,
    /// L1 act-tier allowlist: (module, action) pairs where act agents may
    /// operate at all.
    pub agent_allowlist: BTreeSet<(ModuleId, Action)>,
    /// ORIGIN approvals for act agents: (subject, module, action).
    pub origin_approvals: BTreeSet<(SubjectId, ModuleId, Action)>,
}

impl Env {
    /// An enabled, empty environment at instant `now`.
    pub fn new(now: EpochMs) -> Self {
        Self {
            now,
            module_enabled: true,
            known_columns: BTreeSet::new(),
            rules: Vec::new(),
            grants: Vec::new(),
            agent_allowlist: BTreeSet::new(),
            origin_approvals: BTreeSet::new(),
        }
    }

    /// Sets the module state (L0).
    pub fn with_module_enabled(mut self, enabled: bool) -> Self {
        self.module_enabled = enabled;
        self
    }

    /// Adds declared columns to the target module.
    pub fn with_known_columns<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.known_columns
            .extend(columns.into_iter().map(Into::into));
        self
    }

    /// Adds an L3 rule.
    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Adds an L2 grant.
    pub fn with_grant(mut self, grant: Grant) -> Self {
        self.grants.push(grant);
        self
    }

    /// Allowlists an act-tier (module, action) pair.
    pub fn with_agent_allowlist(mut self, module: ModuleId, action: Action) -> Self {
        self.agent_allowlist.insert((module, action));
        self
    }

    /// Records an ORIGIN approval for an act agent.
    pub fn with_origin_approval(
        mut self,
        subject: SubjectId,
        module: ModuleId,
        action: Action,
    ) -> Self {
        self.origin_approvals.insert((subject, module, action));
        self
    }

    /// Moves the injected clock; used to test expiry boundaries.
    pub fn with_now(mut self, now: EpochMs) -> Self {
        self.now = now;
        self
    }
}

// ---------------------------------------------------------------- decision

/// The layer that produced a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// The ORIGIN path.
    Origin,
    /// Module state.
    L0,
    /// Agent tiers.
    L1,
    /// Peer-read grants.
    L2,
    /// Column role rules.
    L3,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::Origin => f.write_str("origin"),
            Layer::L0 => f.write_str("l0"),
            Layer::L1 => f.write_str("l1"),
            Layer::L2 => f.write_str("l2"),
            Layer::L3 => f.write_str("l3"),
        }
    }
}

/// The fourteen decision codes. Four allow, ten deny — and every deny path
/// fails closed by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionCode {
    /// ORIGIN with recorded intent; L2/L3 bypassed, L0 never bypassed.
    AllowOrigin,
    /// Allowed by the agent's tier path (L1).
    AllowTier,
    /// Allowed by a peer-read grant (L2).
    AllowGrant,
    /// Allowed by column role rules (L3).
    AllowRule,
    /// The target module is disabled; gates everyone, even ORIGIN (L0).
    DenyModuleDisabled,
    /// ORIGIN acted without recorded intent.
    DenyIntentRequired,
    /// The agent's tier is below what the action requires.
    DenyTierInsufficient,
    /// Act-tier operation without an allowlist entry.
    DenyAgentNotAllowlisted,
    /// Allowlisted but no ORIGIN approval for this subject/module/action.
    DenyOriginApprovalRequired,
    /// No grant covers this peer read.
    DenyGrantMissing,
    /// A covering grant exists but is expired (checked at the expiry
    /// instant, inclusively).
    DenyGrantExpired,
    /// An explicit deny rule matched a requested column.
    DenyRuleExplicit,
    /// A requested column is not declared by the module. Denied for every
    /// actor except ORIGIN.
    DenyColumnUnknown,
    /// Nothing allowed it. The default is deny.
    DenyDefault,
}

impl fmt::Display for DecisionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            DecisionCode::AllowOrigin => "allow_origin",
            DecisionCode::AllowTier => "allow_tier",
            DecisionCode::AllowGrant => "allow_grant",
            DecisionCode::AllowRule => "allow_rule",
            DecisionCode::DenyModuleDisabled => "deny_module_disabled",
            DecisionCode::DenyIntentRequired => "deny_intent_required",
            DecisionCode::DenyTierInsufficient => "deny_tier_insufficient",
            DecisionCode::DenyAgentNotAllowlisted => "deny_agent_not_allowlisted",
            DecisionCode::DenyOriginApprovalRequired => "deny_origin_approval_required",
            DecisionCode::DenyGrantMissing => "deny_grant_missing",
            DecisionCode::DenyGrantExpired => "deny_grant_expired",
            DecisionCode::DenyRuleExplicit => "deny_rule_explicit",
            DecisionCode::DenyColumnUnknown => "deny_column_unknown",
            DecisionCode::DenyDefault => "deny_default",
        };
        f.write_str(name)
    }
}

/// The outcome of [`evaluate`]: a code plus the layer that decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Decision {
    /// What was decided.
    pub code: DecisionCode,
    /// Which layer decided it.
    pub layer: Layer,
}

// ---------------------------------------------------------------- engine

/// Evaluates a request against an environment. Pure, total, deterministic.
pub fn evaluate(req: &Request, env: &Env) -> Decision {
    // L0 — module state gates everyone, including ORIGIN.
    if !env.module_enabled {
        return Decision {
            code: DecisionCode::DenyModuleDisabled,
            layer: Layer::L0,
        };
    }

    // Unknown columns deny for every actor except ORIGIN (which bypasses
    // L3 entirely). A column the module does not declare cannot be
    // requested at all — this fails fast, before grants or rules.
    if !matches!(req.actor, Actor::Origin { .. }) {
        for col in &req.columns {
            if !env.known_columns.contains(col) {
                return Decision {
                    code: DecisionCode::DenyColumnUnknown,
                    layer: Layer::L3,
                };
            }
        }
    }

    match &req.actor {
        Actor::Origin { .. } => match &req.origin_intent {
            Some(_) => Decision {
                code: DecisionCode::AllowOrigin,
                layer: Layer::Origin,
            },
            None => Decision {
                code: DecisionCode::DenyIntentRequired,
                layer: Layer::Origin,
            },
        },
        Actor::User { role, .. } => evaluate_user(req, env, role),
        Actor::Agent { subject, tier } => evaluate_agent(req, env, subject, *tier),
    }
}

/// L2 grant lookup shared by users and agents. Returns the deny code when
/// no valid grant covers the request, or `None` when a valid grant exists.
fn grant_verdict(req: &Request, env: &Env, home: &ModuleId) -> Option<DecisionCode> {
    let mut covered_valid = false;
    let mut covered_expired = false;
    // Peer reads must name columns; grants are column-exact, so an empty
    // (sentinel) request is never covered.
    let names_columns = !req.columns.is_empty();
    if names_columns {
        for g in &env.grants {
            if g.owner != req.target || g.granted_to != *home {
                continue;
            }
            let covers = req.columns.iter().all(|c| g.columns.contains(c));
            if !covers {
                continue;
            }
            match g.expires_at {
                Some(exp) if env.now >= exp => covered_expired = true,
                _ => covered_valid = true,
            }
        }
    }
    if covered_valid {
        None
    } else if covered_expired {
        Some(DecisionCode::DenyGrantExpired)
    } else {
        Some(DecisionCode::DenyGrantMissing)
    }
}

/// The user path: peer reads through L2 grants then L3 rules; own-module
/// and kernel-level operations through L3 rules; proposals judged by the
/// write rules.
fn evaluate_user(req: &Request, env: &Env, role: &RoleId) -> Decision {
    match &req.home {
        Some(home) if home != &req.target => evaluate_user_peer(req, env, role, home),
        _ => evaluate_l3(req, env, role),
    }
}

/// The user peer-read path. `home` is proven distinct from the target by
/// the caller's match, so no `expect` is needed to narrow it (Part A4).
fn evaluate_user_peer(req: &Request, env: &Env, role: &RoleId, home: &ModuleId) -> Decision {
    if req.action != Action::Read {
        // Cross-module writes do not exist as an operation (star, not
        // mesh); the peer layer is where this fails.
        return Decision {
            code: DecisionCode::DenyDefault,
            layer: Layer::L2,
        };
    }
    if let Some(code) = grant_verdict(req, env, home) {
        return Decision {
            code,
            layer: Layer::L2,
        };
    }
    // The grant carried us here, but the owning company's rules still
    // apply — passing L2 grants nothing by itself.
    let Decision { code, .. } = evaluate_l3(req, env, role);
    if !matches!(code, DecisionCode::AllowRule) {
        return Decision {
            code,
            layer: Layer::L3,
        };
    }
    Decision {
        code: DecisionCode::AllowGrant,
        layer: Layer::L2,
    }
}

/// L3 column rules. Deny wins; default deny; proposals are judged by the
/// write rules.
fn evaluate_l3(req: &Request, env: &Env, role: &RoleId) -> Decision {
    let sentinel;
    let columns: Vec<&str> = if req.columns.is_empty() {
        sentinel = "*";
        vec![sentinel]
    } else {
        req.columns.iter().map(String::as_str).collect()
    };
    for col in columns {
        let mut saw_allow = false;
        let mut saw_deny = false;
        for rule in &env.rules {
            if rule.role != *role || rule.module != req.target {
                continue;
            }
            let action_matches = rule.action.applies_to(req.action);
            if !action_matches || !rule.column.matches(col) {
                continue;
            }
            match rule.effect {
                Effect::Allow => saw_allow = true,
                Effect::Deny => saw_deny = true,
            }
        }
        if saw_deny {
            return Decision {
                code: DecisionCode::DenyRuleExplicit,
                layer: Layer::L3,
            };
        }
        if !saw_allow {
            return Decision {
                code: DecisionCode::DenyDefault,
                layer: Layer::L3,
            };
        }
    }
    Decision {
        code: DecisionCode::AllowRule,
        layer: Layer::L3,
    }
}

/// The agent path: tier gates on the own-module surface, grants on the
/// peer-read surface, allowlist plus ORIGIN approval for act writes.
fn evaluate_agent(req: &Request, env: &Env, subject: &SubjectId, tier: Tier) -> Decision {
    if let Some(home) = req.home.as_ref() {
        if home != &req.target {
            return evaluate_agent_peer(req, env, home);
        }
    }
    match req.action {
        Action::Read => Decision {
            code: DecisionCode::AllowTier,
            layer: Layer::L1,
        },
        Action::Propose => {
            if tier >= Tier::Advise {
                Decision {
                    code: DecisionCode::AllowTier,
                    layer: Layer::L1,
                }
            } else {
                Decision {
                    code: DecisionCode::DenyTierInsufficient,
                    layer: Layer::L1,
                }
            }
        }
        Action::Write => {
            if tier < Tier::Act {
                return Decision {
                    code: DecisionCode::DenyTierInsufficient,
                    layer: Layer::L1,
                };
            }
            if !env
                .agent_allowlist
                .contains(&(req.target.clone(), Action::Write))
            {
                return Decision {
                    code: DecisionCode::DenyAgentNotAllowlisted,
                    layer: Layer::L1,
                };
            }
            if !env
                .origin_approvals
                .contains(&(subject.clone(), req.target.clone(), Action::Write))
            {
                return Decision {
                    code: DecisionCode::DenyOriginApprovalRequired,
                    layer: Layer::L1,
                };
            }
            Decision {
                code: DecisionCode::AllowTier,
                layer: Layer::L1,
            }
        }
    }
}

/// The agent peer-read path. Agents have no role, so after a covering
/// grant the decision is made at L2 — no L3 applies.
fn evaluate_agent_peer(req: &Request, env: &Env, home: &ModuleId) -> Decision {
    if req.action != Action::Read {
        return Decision {
            code: DecisionCode::DenyDefault,
            layer: Layer::L2,
        };
    }
    if let Some(code) = grant_verdict(req, env, home) {
        return Decision {
            code,
            layer: Layer::L2,
        };
    }
    Decision {
        code: DecisionCode::AllowGrant,
        layer: Layer::L2,
    }
}
