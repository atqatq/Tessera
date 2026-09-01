# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

import pytest

from scor_ref.policy import READ, WRITE, PolicyEngine, Principal, Role, Rule

ALL_ACTIONS = frozenset({READ, WRITE})
READ_ONLY = frozenset({READ})


def engine(**overrides) -> PolicyEngine:
    base = PolicyEngine(
        roles={
            "SRM": Role("SRM", [Rule("srm.*", ALL_ACTIONS)]),
            "AUD": Role("AUD", [Rule("*", READ_ONLY)]),
            "CSR": Role(
                "CSR",
                [
                    Rule("ord.*", ALL_ACTIONS),
                    Rule("srm.supplier.tier", READ_ONLY),
                ],
            ),
            "RESTRICTED": Role(
                "RESTRICTED",
                [
                    Rule("srm.*", READ_ONLY),
                    Rule("srm.supplier.negotiated_floor_usd", READ_ONLY, effect="deny"),
                ],
            ),
        },
        spoke_access={"ord": frozenset({"ctr", "srm"})},
        spoke_states={
            "srm": "active",
            "ord": "active",
            "ctr": "active",
            "src": "active",
            "prj": "disabled",
            "trf": "paused",
            "ret": "archived",
        },
    )
    for key, value in overrides.items():
        setattr(base, key, value)
    return base


def user(*roles, **kwargs) -> Principal:
    return Principal(subject="atique", tenant="acme_gulf", roles=roles, **kwargs)


class TestPrincipalLayer:
    def test_matching_role_allows_read(self):
        assert engine().decide(user("SRM"), "srm", "srm.supplier.tier", READ)

    def test_no_role_is_a_deny(self):
        decision = engine().decide(user(), "srm", "srm.supplier.tier", READ)
        assert not decision and decision.code == "no_role_grant"

    def test_unknown_role_is_ignored_not_trusted(self):
        decision = engine().decide(user("SUPERUSER"), "srm", "srm.supplier.tier", READ)
        assert not decision

    def test_read_only_role_cannot_write(self):
        decision = engine().decide(user("AUD"), "srm", "srm.supplier.tier", WRITE)
        assert not decision and decision.code == "no_role_grant"


class TestColumnLevelGranularity:
    def test_object_access_does_not_imply_every_column(self):
        e = engine()
        principal = user("RESTRICTED")
        assert e.decide(principal, "srm", "srm.supplier.tier", READ)
        assert not e.decide(principal, "srm", "srm.supplier.negotiated_floor_usd", READ)

    def test_deny_beats_allow_regardless_of_role_order(self):
        e = engine()
        forward = user("RESTRICTED", "SRM")
        reverse = user("SRM", "RESTRICTED")
        field = "srm.supplier.negotiated_floor_usd"
        assert not e.decide(forward, "srm", field, READ)
        assert not e.decide(reverse, "srm", field, READ)

    def test_visible_columns_filters_a_projection(self):
        e = engine()
        columns = [
            "srm.supplier.name",
            "srm.supplier.tier",
            "srm.supplier.negotiated_floor_usd",
        ]
        visible = e.visible_columns(user("RESTRICTED"), "srm", columns)
        assert visible == ["srm.supplier.name", "srm.supplier.tier"]

    def test_unqualified_field_is_rejected(self):
        decision = engine().decide(user("AUD"), "srm", "supplier", READ)
        assert not decision and decision.code == "unqualified_field"


class TestSpokeLayer:
    def test_owning_spoke_needs_no_grant(self):
        assert engine().decide(user("SRM"), "srm", "srm.supplier.tier", READ)

    def test_cross_spoke_read_needs_an_origin_grant(self):
        assert engine().decide(user("CSR"), "ord", "srm.supplier.tier", READ)

    def test_cross_spoke_read_without_a_grant_is_denied(self):
        decision = engine().decide(user("SRM"), "src", "srm.supplier.tier", READ)
        assert not decision and decision.code == "no_spoke_grant"

    def test_a_spoke_may_never_write_another_spokes_column(self):
        decision = engine().decide(user("SRM"), "ord", "srm.supplier.tier", WRITE)
        assert not decision and decision.code == "cross_spoke_write"

    def test_spoke_grant_does_not_bypass_the_role_layer(self):
        """Both layers must pass. A spoke grant is not a user grant."""
        decision = engine().decide(user(), "ord", "srm.supplier.tier", READ)
        assert not decision and decision.code == "no_role_grant"


class TestSpokeState:
    def test_disabled_spoke_still_serves_reads(self):
        e = engine(roles={"AUD": Role("AUD", [Rule("*", READ_ONLY)])})
        decision = e.decide(user("AUD"), "prj", "prj.project.status", READ)
        assert decision and decision.stale

    def test_disabled_spoke_refuses_writes(self):
        e = engine()
        e.roles["ALL"] = Role("ALL", [Rule("*", ALL_ACTIONS)])
        decision = e.decide(user("ALL"), "prj", "prj.project.status", WRITE)
        assert not decision and decision.code == "spoke_state"

    def test_paused_spoke_reads_are_flagged_stale(self):
        e = engine()
        assert e.decide(user("AUD"), "trf", "trf.workorder.qty", READ).stale

    def test_archived_spoke_is_not_readable_live(self):
        decision = engine().decide(user("AUD"), "ret", "ret.rma.status", READ)
        assert not decision and decision.code == "spoke_state"

    def test_active_spoke_reads_are_not_stale(self):
        assert not engine().decide(user("SRM"), "srm", "srm.supplier.tier", READ).stale

    def test_unknown_spoke_defaults_to_denied(self):
        decision = engine().decide(user("AUD"), "xyz", "xyz.thing.value", READ)
        assert not decision and decision.code == "spoke_state"


class TestOriginSession:
    def test_origin_bypasses_role_and_spoke_layers(self):
        principal = Principal("origin", "acme_gulf", (), origin_session=True, intent="rotate keys")
        assert engine().decide(principal, "hub", "srm.supplier.negotiated_floor_usd", WRITE)

    def test_origin_without_intent_is_refused(self):
        principal = Principal("origin", "acme_gulf", (), origin_session=True)
        decision = engine().decide(principal, "hub", "srm.supplier.tier", READ)
        assert not decision and decision.code == "origin_no_intent"

    def test_origin_cannot_write_a_disabled_spoke(self):
        """State is a machine fact, not a permission. Even origin re-enables first."""
        principal = Principal("origin", "acme_gulf", (), origin_session=True, intent="fix data")
        decision = engine().decide(principal, "hub", "prj.project.status", WRITE)
        assert not decision and decision.code == "spoke_state"


class TestInputValidation:
    def test_unknown_action_is_refused(self):
        decision = engine().decide(user("SRM"), "srm", "srm.supplier.tier", "delete")
        assert not decision and decision.code == "unknown_action"

    @pytest.mark.parametrize("action", [READ, WRITE])
    def test_every_decision_carries_a_reason(self, action):
        decision = engine().decide(user(), "srm", "srm.supplier.tier", action)
        assert decision.reason


# ---------------------------------------------------------------------------
# AI agent principals: spoke agents and the leader agent.
# ---------------------------------------------------------------------------

from scor_ref.policy import ACT, ADVISE, OBSERVE, PROPOSE  # noqa: E402


def agent(subject, spoke_roles, tier=ADVISE) -> Principal:
    return Principal(
        subject=subject,
        tenant="acme_gulf",
        roles=spoke_roles,
        agent=True,
        agent_tier=tier,
    )


def agent_engine(**overrides) -> PolicyEngine:
    e = engine(**overrides)
    e.roles["AIS"] = Role("AIS", [Rule("srm.*", frozenset({READ, PROPOSE, WRITE}))])
    e.roles["AIL"] = Role("AIL", [Rule("*", frozenset({READ, PROPOSE}))])
    e.agent_act_allowlist = {"srm": ("srm.scorecard.*",)}
    return e


class TestAgentsProposeRatherThanWrite:
    def test_advise_tier_agent_may_read(self):
        assert agent_engine().decide(agent("srm-ai", ("AIS",)), "srm", "srm.supplier.tier", READ)

    def test_advise_tier_agent_may_propose(self):
        assert agent_engine().decide(agent("srm-ai", ("AIS",)), "srm", "srm.supplier.tier", PROPOSE)

    def test_advise_tier_agent_cannot_write(self):
        decision = agent_engine().decide(agent("srm-ai", ("AIS",)), "srm", "srm.supplier.tier", WRITE)
        assert not decision and decision.code == "agent_write_forbidden"

    def test_observe_tier_agent_cannot_even_propose(self):
        principal = agent("srm-ai", ("AIS",), tier=OBSERVE)
        decision = agent_engine().decide(principal, "srm", "srm.supplier.tier", PROPOSE)
        assert not decision and decision.code == "agent_tier"

    def test_agent_without_a_tier_is_refused(self):
        principal = Principal("rogue-ai", "acme_gulf", ("AIS",), agent=True)
        decision = agent_engine().decide(principal, "srm", "srm.supplier.tier", READ)
        assert not decision and decision.code == "agent_tier"


class TestActTierAllowlist:
    def test_act_tier_agent_may_write_inside_the_allowlist(self):
        principal = agent("srm-ai", ("AIS",), tier=ACT)
        assert agent_engine().decide(principal, "srm", "srm.scorecard.health_index", WRITE)

    def test_act_tier_agent_cannot_write_outside_the_allowlist(self):
        principal = agent("srm-ai", ("AIS",), tier=ACT)
        decision = agent_engine().decide(principal, "srm", "srm.supplier.tier", WRITE)
        assert not decision and decision.code == "agent_not_in_allowlist"

    def test_an_empty_allowlist_blocks_every_write(self):
        e = agent_engine()
        e.agent_act_allowlist = {}
        principal = agent("srm-ai", ("AIS",), tier=ACT)
        decision = e.decide(principal, "srm", "srm.scorecard.health_index", WRITE)
        assert not decision and decision.code == "agent_not_in_allowlist"


class TestAgentsInheritEveryOtherLayer:
    def test_agent_cannot_read_a_spoke_its_spoke_cannot_read(self):
        decision = agent_engine().decide(agent("src-ai", ("AIS",)), "src", "srm.supplier.tier", READ)
        assert not decision and decision.code == "no_spoke_grant"

    def test_agent_cannot_propose_into_another_spoke(self):
        """The leader routes cross-spoke work as tasks, never as direct change."""
        decision = agent_engine().decide(agent("ord-ai", ("AIL",)), "ord", "srm.supplier.tier", PROPOSE)
        assert not decision and decision.code == "cross_spoke_write"

    def test_agent_is_bound_by_column_level_denies(self):
        e = agent_engine()
        principal = agent("srm-ai", ("AIS", "RESTRICTED"))
        field = "srm.supplier.negotiated_floor_usd"
        assert not e.decide(principal, "srm", field, READ)

    def test_agent_of_a_disabled_spoke_gets_the_state_refusal(self):
        e = agent_engine()
        e.roles["PRJ_AI"] = Role("PRJ_AI", [Rule("prj.*", frozenset({READ, PROPOSE}))])
        principal = agent("prj-ai", ("PRJ_AI",))
        decision = e.decide(principal, "prj", "prj.project.status", PROPOSE)
        assert not decision and decision.code == "spoke_state"

    def test_agent_reads_of_a_paused_spoke_are_flagged_stale(self):
        e = agent_engine()
        e.roles["TRF_AI"] = Role("TRF_AI", [Rule("trf.*", frozenset({READ}))])
        assert e.decide(agent("trf-ai", ("TRF_AI",)), "trf", "trf.workorder.qty", READ).stale


class TestNoAgentHoldsOrigin:
    def test_agent_claiming_origin_is_refused(self):
        principal = Principal(
            "leader-ai", "acme_gulf", (), origin_session=True, intent="optimise",
            agent=True, agent_tier=ACT,
        )
        decision = agent_engine().decide(principal, "hub", "srm.supplier.tier", WRITE)
        assert not decision and decision.code == "agent_origin_forbidden"


class TestProposeIsAnAgentPath:
    def test_humans_do_not_use_propose(self):
        decision = agent_engine().decide(user("SRM"), "srm", "srm.supplier.tier", PROPOSE)
        assert not decision and decision.code == "propose_is_for_agents"

    def test_humans_still_write_directly(self):
        assert agent_engine().decide(user("SRM"), "srm", "srm.supplier.tier", WRITE)
