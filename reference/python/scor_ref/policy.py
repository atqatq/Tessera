# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Column-level access control.

Permission is evaluated at the column, not the object. Tenants add their
own fields to shared objects, and those fields carry their own
sensitivity: two users can both read `srm.supplier` while only one of them
may read `srm.supplier.negotiated_floor_usd`.

Every decision passes independent layers, and deny beats allow in all of
them:

0. **State layer.** The owning spoke's lifecycle state decides what is
   possible at all. State is a machine fact, not a permission.
1. **Agent layer.** AI principals are constrained further than the humans
   they work for. An agent proposes; it does not commit, unless origin has
   granted it a narrow, reversible act allowlist.
2. **Spoke layer.** If the calling spoke does not own the field, origin
   must have granted that spoke read access to the owning spoke. This is
   the layer that stops a spoke reading another spoke behind the hub's back.
3. **Principal layer.** The caller's roles must permit the action on that
   specific column.

A missing grant is a deny.
"""

from __future__ import annotations

import fnmatch
from dataclasses import dataclass, field as dc_field
from typing import Dict, List, Optional, Sequence

READ, WRITE, PROPOSE = "read", "write", "propose"
ACTIONS = frozenset({READ, WRITE, PROPOSE})

#: Agent capability tiers, least to most privileged.
OBSERVE, ADVISE, ACT = "observe", "advise", "act"
AGENT_TIERS = frozenset({OBSERVE, ADVISE, ACT})

#: Spoke lifecycle states and what they permit.
STATE_READ = {"active": True, "paused": True, "disabled": True, "archived": False,
              "installed": False, "planned": False}
STATE_WRITE = {"active": True, "paused": False, "disabled": False, "archived": False,
               "installed": False, "planned": False}
#: A proposal needs a live spoke; there is nothing to apply it to otherwise.
STATE_PROPOSE = dict(STATE_WRITE)
STATE_STALE = {"paused", "disabled"}


@dataclass(frozen=True)
class Rule:
    """A single column rule attached to a role."""

    field: str  # exact name or glob, e.g. 'srm.*' or 'srm.supplier.tier'
    actions: frozenset
    effect: str = "allow"  # allow | deny

    def matches(self, field: str, action: str) -> bool:
        return action in self.actions and fnmatch.fnmatchcase(field, self.field)


@dataclass(frozen=True)
class Role:
    code: str  # three letters, e.g. 'SRM'
    rules: Sequence[Rule] = dc_field(default=())


@dataclass(frozen=True)
class Principal:
    """A human user, a spoke agent, or the leader agent.

    `agent` and `origin_session` are mutually exclusive. No model instance
    holds origin, under any configuration.
    """

    subject: str
    tenant: str
    roles: Sequence[str] = dc_field(default=())
    origin_session: bool = False
    intent: Optional[str] = None
    agent: bool = False
    agent_tier: Optional[str] = None


@dataclass(frozen=True)
class Decision:
    allowed: bool
    code: str
    reason: str
    stale: bool = False

    def __bool__(self) -> bool:
        return self.allowed


@dataclass
class PolicyEngine:
    roles: Dict[str, Role] = dc_field(default_factory=dict)
    #: origin-granted spoke-to-spoke read access: {'ord': {'ctr', 'inv'}}
    spoke_access: Dict[str, frozenset] = dc_field(default_factory=dict)
    #: current lifecycle state per spoke
    spoke_states: Dict[str, str] = dc_field(default_factory=dict)
    #: origin-approved act allowlist per spoke: {'inv': ('inv.*.reorder_point',)}
    agent_act_allowlist: Dict[str, Sequence[str]] = dc_field(default_factory=dict)

    def decide(
        self,
        principal: Principal,
        calling_spoke: str,
        field: str,
        action: str,
    ) -> Decision:
        if action not in ACTIONS:
            return Decision(False, "unknown_action", f"{action!r} is not a known action")
        if "." not in field:
            return Decision(False, "unqualified_field", f"{field!r} is not namespaced")

        owner = field.split(".", 1)[0]
        state = self.spoke_states.get(owner, "planned")

        # -- Layer 0: does the owning spoke's state permit this at all? ----
        table = {READ: STATE_READ, WRITE: STATE_WRITE, PROPOSE: STATE_PROPOSE}[action]
        if not table.get(state, False):
            return Decision(
                False,
                "spoke_state",
                f"spoke {owner!r} is {state}; {action} is not available",
            )
        stale = state in STATE_STALE

        # -- Layer 1: agent constraints ------------------------------------
        if principal.agent:
            if principal.origin_session:
                return Decision(
                    False,
                    "agent_origin_forbidden",
                    "no model principal may hold an origin session",
                )
            refusal = self._check_agent(principal, calling_spoke, field, action)
            if refusal is not None:
                return refusal
        elif action == PROPOSE:
            return Decision(
                False,
                "propose_is_for_agents",
                "human principals write directly; propose is the agent path",
            )

        # Origin bypasses the role layer but not the state layer or the ledger.
        if principal.origin_session:
            if not principal.intent:
                return Decision(
                    False,
                    "origin_no_intent",
                    "origin sessions must record an intent statement before acting",
                )
            return Decision(True, "origin", "origin session, logged to the ledger", stale)

        # -- Layer 2: spoke to spoke ---------------------------------------
        if calling_spoke != owner:
            granted = self.spoke_access.get(calling_spoke, frozenset())
            if owner not in granted:
                return Decision(
                    False,
                    "no_spoke_grant",
                    f"spoke {calling_spoke!r} has no origin grant to read {owner!r}",
                )
            if action in (WRITE, PROPOSE):
                return Decision(
                    False,
                    "cross_spoke_write",
                    f"spoke {calling_spoke!r} may read {owner!r} but never change it",
                )

        # -- Layer 3: principal roles, column level, deny wins -------------
        matched_allow = None
        for role_code in principal.roles:
            role = self.roles.get(role_code)
            if role is None:
                continue
            for rule in role.rules:
                if not rule.matches(field, action):
                    continue
                if rule.effect == "deny":
                    return Decision(
                        False,
                        "role_deny",
                        f"role {role_code!r} explicitly denies {action} on {field!r}",
                    )
                matched_allow = role_code

        if matched_allow is None:
            return Decision(
                False,
                "no_role_grant",
                f"no role held by {principal.subject!r} permits {action} on {field!r}",
            )
        return Decision(True, "allowed", f"granted by role {matched_allow!r}", stale)

    def _check_agent(
        self, principal: Principal, calling_spoke: str, field: str, action: str
    ) -> Optional[Decision]:
        """Return a refusal, or None to continue through the normal layers."""
        tier = principal.agent_tier
        if tier not in AGENT_TIERS:
            return Decision(
                False,
                "agent_tier",
                f"agent {principal.subject!r} has no valid capability tier",
            )
        if action == PROPOSE and tier == OBSERVE:
            return Decision(
                False,
                "agent_tier",
                f"agent {principal.subject!r} is observe-tier and cannot propose",
            )
        if action == WRITE:
            if tier != ACT:
                return Decision(
                    False,
                    "agent_write_forbidden",
                    f"agent {principal.subject!r} is {tier}-tier; agents propose rather "
                    "than write, and a human applies the change",
                )
            allowlist = self.agent_act_allowlist.get(calling_spoke, ())
            if not any(fnmatch.fnmatchcase(field, pattern) for pattern in allowlist):
                return Decision(
                    False,
                    "agent_not_in_allowlist",
                    f"{field!r} is outside the origin-approved act allowlist for "
                    f"agent {principal.subject!r}",
                )
        return None

    def visible_columns(
        self, principal: Principal, calling_spoke: str, fields: Sequence[str]
    ) -> List[str]:
        """Filter a projection down to what the principal may actually read."""
        return [f for f in fields if self.decide(principal, calling_spoke, f, READ).allowed]
