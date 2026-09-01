"""Mirror of kernel/access — the pure permission engine.

Same layers, same fourteen decision codes, same deny-wins semantics as
the Rust implementation. This module is the executable spec: it operates
on JSON-shaped envs and requests (the same shapes the conformance
vectors store) so there is no translation layer between the spec and
the contract.
"""

from __future__ import annotations

# ---------------------------------------------------------------- actions

READ = "read"
PROPOSE = "propose"
WRITE = "write"

ALLOW = "allow"
DENY = "deny"

OBSERVE = "observe"
ADVISE = "advise"
ACT = "act"

# Rule actions: there is deliberately no propose rule. Proposals are
# judged by the write rules (agents propose, humans commit).
RULE_READ = "read"
RULE_WRITE = "write"

# ---------------------------------------------------------------- codes

ALLOW_ORIGIN = "allow_origin"
ALLOW_TIER = "allow_tier"
ALLOW_GRANT = "allow_grant"
ALLOW_RULE = "allow_rule"
DENY_MODULE_DISABLED = "deny_module_disabled"
DENY_INTENT_REQUIRED = "deny_intent_required"
DENY_TIER_INSUFFICIENT = "deny_tier_insufficient"
DENY_AGENT_NOT_ALLOWLISTED = "deny_agent_not_allowlisted"
DENY_ORIGIN_APPROVAL_REQUIRED = "deny_origin_approval_required"
DENY_GRANT_MISSING = "deny_grant_missing"
DENY_GRANT_EXPIRED = "deny_grant_expired"
DENY_RULE_EXPLICIT = "deny_rule_explicit"
DENY_COLUMN_UNKNOWN = "deny_column_unknown"
DENY_DEFAULT = "deny_default"

CODES = [
    ALLOW_ORIGIN, ALLOW_TIER, ALLOW_GRANT, ALLOW_RULE,
    DENY_MODULE_DISABLED, DENY_INTENT_REQUIRED, DENY_TIER_INSUFFICIENT,
    DENY_AGENT_NOT_ALLOWLISTED, DENY_ORIGIN_APPROVAL_REQUIRED,
    DENY_GRANT_MISSING, DENY_GRANT_EXPIRED, DENY_RULE_EXPLICIT,
    DENY_COLUMN_UNKNOWN, DENY_DEFAULT,
]

LAYER_ORIGIN = "origin"
LAYER_L0 = "l0"
LAYER_L1 = "l1"
LAYER_L2 = "l2"
LAYER_L3 = "l3"

SENTINEL = "*"


def glob_matches(pattern: str, s: str) -> bool:
    """Same semantics as tessera-access's Glob: '*' matches any sequence,
    everything else is literal, so exact 'qty' never matches 'qty_reserved'."""
    parts = pattern.split("*")
    if len(parts) == 1:
        return s == pattern
    rest = s
    last = len(parts) - 1
    for i, part in enumerate(parts):
        if part == "":
            continue
        if i == 0:
            if not rest.startswith(part):
                return False
            rest = rest[len(part):]
        elif i == last:
            if not rest.endswith(part):
                return False
        else:
            pos = rest.find(part)
            if pos < 0:
                return False
            rest = rest[pos + len(part):]
    return True


def _rule_applies(rule_action: str, request_action: str) -> bool:
    if rule_action == RULE_READ:
        return request_action == READ
    if rule_action == RULE_WRITE:
        return request_action in (WRITE, PROPOSE)
    return False


def _grant_verdict(env: dict, request: dict, home: str) -> str | None:
    """None when a valid covering grant exists; a deny code otherwise."""
    covered_valid = False
    covered_expired = False
    columns = request.get("columns") or []
    # Peer reads must name columns; grants are column-exact, so an empty
    # (sentinel) request is never covered.
    if columns:
        for grant in env.get("grants", []):
            if grant["owner"] != request["target"] or grant["granted_to"] != home:
                continue
            if not all(c in grant["columns"] for c in columns):
                continue
            expires_at = grant.get("expires_at")
            if expires_at is not None and env["now"] >= expires_at:
                covered_expired = True  # at the expiry instant: already expired
            else:
                covered_valid = True
    if covered_valid:
        return None
    if covered_expired:
        return DENY_GRANT_EXPIRED
    return DENY_GRANT_MISSING


def _l3(env: dict, request: dict, role: str) -> tuple[str, str]:
    columns = request.get("columns") or []
    names_columns = bool(columns)
    for col in columns if names_columns else [SENTINEL]:
        saw_allow = False
        saw_deny = False
        for rule in env.get("rules", []):
            if rule["role"] != role or rule["module"] != request["target"]:
                continue
            if not _rule_applies(rule["action"], request["action"]):
                continue
            if not glob_matches(rule["column"], col):
                continue
            if rule["effect"] == ALLOW:
                saw_allow = True
            else:
                saw_deny = True
        if saw_deny:
            return DENY_RULE_EXPLICIT, LAYER_L3  # deny wins
        if not saw_allow:
            return DENY_DEFAULT, LAYER_L3  # default deny
    return ALLOW_RULE, LAYER_L3


def _is_peer(env: dict, request: dict) -> bool:
    home = request.get("home")
    return home is not None and home != request["target"]


def _user_path(env: dict, request: dict, role: str) -> tuple[str, str]:
    if _is_peer(env, request):
        if request["action"] != READ:
            # Cross-module writes do not exist as an operation.
            return DENY_DEFAULT, LAYER_L2
        verdict = _grant_verdict(env, request, request["home"])
        if verdict is not None:
            return verdict, LAYER_L2
        # Passing L2 grants nothing by itself: the owning company's rules
        # still apply.
        code, layer = _l3(env, request, role)
        if code != ALLOW_RULE:
            return code, layer
        return ALLOW_GRANT, LAYER_L2
    return _l3(env, request, role)


def _agent_path(env: dict, request: dict, subject: str, tier: str) -> tuple[str, str]:
    if _is_peer(env, request):
        if request["action"] != READ:
            return DENY_DEFAULT, LAYER_L2
        verdict = _grant_verdict(env, request, request["home"])
        if verdict is not None:
            return verdict, LAYER_L2
        # Agents hold no role, so no L3 applies after the grant.
        return ALLOW_GRANT, LAYER_L2
    action = request["action"]
    if action == READ:
        return ALLOW_TIER, LAYER_L1
    if action == PROPOSE:
        if tier in (ADVISE, ACT):
            return ALLOW_TIER, LAYER_L1
        return DENY_TIER_INSUFFICIENT, LAYER_L1
    # WRITE: tier, then allowlist, then ORIGIN approval.
    if tier != ACT:
        return DENY_TIER_INSUFFICIENT, LAYER_L1
    if (request["target"], WRITE) not in {(m, a) for (m, a) in env.get("agent_allowlist", [])}:
        return DENY_AGENT_NOT_ALLOWLISTED, LAYER_L1
    approvals = {(s, m, a) for (s, m, a) in env.get("origin_approvals", [])}
    if (subject, request["target"], WRITE) not in approvals:
        return DENY_ORIGIN_APPROVAL_REQUIRED, LAYER_L1
    return ALLOW_TIER, LAYER_L1


def evaluate(env: dict, request: dict) -> tuple[str, str]:
    """Pure decision: (code, layer). Mirrors kernel/access exactly."""
    # L0 — module state gates everyone, including ORIGIN.
    if not env.get("module_enabled", True):
        return DENY_MODULE_DISABLED, LAYER_L0

    # Unknown columns deny for every actor except ORIGIN. The sentinel
    # (an empty column set) is exempt — it is a module-level request.
    if request.get("actor", {}).get("kind") != "origin":
        known = set(env.get("known_columns", []))
        for col in request.get("columns", []):
            if col not in known:
                return DENY_COLUMN_UNKNOWN, LAYER_L3

    actor = request["actor"]
    kind = actor["kind"]
    if kind == "origin":
        if request.get("origin_intent") is not None:
            return ALLOW_ORIGIN, LAYER_ORIGIN
        return DENY_INTENT_REQUIRED, LAYER_ORIGIN
    if kind == "user":
        return _user_path(env, request, actor["role"])
    if kind == "agent":
        return _agent_path(env, request, actor["subject"], actor["tier"])
    raise ValueError(f"unknown actor kind: {kind!r}")
