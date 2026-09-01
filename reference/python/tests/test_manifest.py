# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

import copy

from scor_ref.manifest import HUB_SERVICES, validate_all, validate_manifest

VALID = {
    "spoke": "srm",
    "version": "2.4.0",
    "requires": ["hub.master_data", "hub.events", "hub.ledger"],
    "enhances": ["spoke.src", "spoke.ctr"],
    "provides": {
        "objects": ["srm.supplier", "srm.scorecard"],
        "events": ["srm.supplier_rated"],
        "kpis": ["srm.otif_pct"],
    },
    "consumes": [
        {"field": "ctr.commercial_terms.penalty_exposure_usd", "on_missing": "hold_last"},
        {"field": "src.spend_usd_ttm", "on_missing": "null"},
    ],
}


def manifest(**overrides):
    m = copy.deepcopy(VALID)
    m.update(overrides)
    return m


def codes(result):
    return {f.code for f in result.findings}


class TestTheIndependenceRule:
    """A hard requires may name a hub service and nothing else."""

    def test_valid_manifest_passes(self):
        assert validate_manifest(VALID).ok

    def test_hard_requiring_another_spoke_is_rejected(self):
        result = validate_manifest(manifest(requires=["hub.ledger", "spoke.src"]))
        assert not result.ok
        assert "requires.spoke_dependency" in codes(result)

    def test_the_rejection_explains_the_fix(self):
        result = validate_manifest(manifest(requires=["spoke.ctr"]))
        message = " ".join(f.message for f in result.errors)
        assert "enhances" in message

    def test_soft_dependency_on_a_spoke_is_fine(self):
        assert validate_manifest(manifest(enhances=["spoke.src"])).ok

    def test_unknown_hub_service_is_rejected(self):
        result = validate_manifest(manifest(requires=["hub.teleporter"]))
        assert "requires.unknown_service" in codes(result)

    def test_every_declared_hub_service_is_accepted(self):
        assert validate_manifest(manifest(requires=sorted(HUB_SERVICES))).ok

    def test_no_hub_services_is_a_warning_not_an_error(self):
        result = validate_manifest(manifest(requires=[]))
        assert result.ok
        assert "requires.empty" in codes(result)


class TestIdentity:
    def test_unregistered_spoke_code_is_rejected(self):
        assert "spoke.unregistered" in codes(validate_manifest(manifest(spoke="wharehouse")))

    def test_uppercase_spoke_code_is_rejected(self):
        assert "spoke.code" in codes(validate_manifest(manifest(spoke="SRM")))

    def test_non_semver_version_is_rejected(self):
        assert "spoke.version" in codes(validate_manifest(manifest(version="2.4")))

    def test_prerelease_version_is_accepted(self):
        assert validate_manifest(manifest(version="2.4.0-rc.1")).ok


class TestProvides:
    def test_unnamespaced_object_is_rejected(self):
        m = manifest(provides={"objects": ["supplier"]})
        assert "provides.namespace" in codes(validate_manifest(m))

    def test_publishing_under_another_spokes_namespace_is_rejected(self):
        m = manifest(provides={"objects": ["ctr.commercial_terms"]})
        assert "provides.foreign_namespace" in codes(validate_manifest(m))

    def test_collision_across_manifests_is_reported(self):
        a = manifest(spoke="srm", provides={"objects": ["srm.supplier"]}, consumes=[])
        b = manifest(
            spoke="src",
            enhances=[],
            provides={"objects": ["srm.supplier"]},
            consumes=[],
        )
        results = validate_all([a, b])
        assert "provides.collision" in codes(results["src"])


class TestConsumes:
    def test_cross_spoke_read_needs_a_missing_policy(self):
        m = manifest(consumes=[{"field": "src.spend_usd_ttm"}])
        assert "consumes.on_missing" in codes(validate_manifest(m))

    def test_invalid_missing_policy_is_rejected(self):
        m = manifest(consumes=[{"field": "src.spend_usd_ttm", "on_missing": "guess"}])
        assert "consumes.on_missing" in codes(validate_manifest(m))

    def test_fail_policy_requires_origin_approval(self):
        m = manifest(consumes=[{"field": "src.spend_usd_ttm", "on_missing": "fail"}])
        result = validate_manifest(m)
        assert "consumes.fail_needs_origin" in codes(result)

    def test_fail_policy_with_origin_approval_is_accepted(self):
        m = manifest(
            consumes=[
                {
                    "field": "src.spend_usd_ttm",
                    "on_missing": "fail",
                    "origin_approval": "ORIG-2026-0114",
                }
            ]
        )
        assert validate_manifest(m).ok

    def test_consuming_your_own_field_is_rejected(self):
        m = manifest(consumes=[{"field": "srm.otif_pct", "on_missing": "null"}])
        assert "consumes.own_namespace" in codes(validate_manifest(m))

    def test_absent_owner_produces_a_warning_only(self):
        """Disabling ctr must not fail srm's install."""
        srm = manifest()
        results = validate_all([srm])
        assert results["srm"].ok
        assert "consumes.absent_owner" in codes(results["srm"])


class TestReportingQuality:
    def test_all_findings_are_returned_not_just_the_first(self):
        m = manifest(spoke="SRM", version="2.4", requires=["spoke.src"])
        assert len(validate_manifest(m).errors) >= 3

    def test_findings_render_readably(self):
        result = validate_manifest(manifest(requires=["spoke.src"]))
        assert str(result.errors[0]).startswith("[error] requires.spoke_dependency:")


# ---------------------------------------------------------------------------
# Agent and dashboard declarations.
# ---------------------------------------------------------------------------

AI_ADVISE = {"enabled": True, "tier": "advise"}
AI_ACT = {
    "enabled": True,
    "tier": "act",
    "act_allowlist": ["srm.scorecard.*"],
    "origin_approval": "ORIG-2026-0221",
}
FULL_REQUIRES = ["hub.master_data", "hub.events", "hub.ledger", "hub.ai_core"]


def with_ai(ai, **overrides):
    return manifest(requires=list(FULL_REQUIRES), ai=ai, **overrides)


class TestAgentDeclaration:
    def test_advise_tier_agent_is_valid(self):
        assert validate_manifest(with_ai(AI_ADVISE)).ok

    def test_act_tier_agent_with_allowlist_and_approval_is_valid(self):
        assert validate_manifest(with_ai(AI_ACT)).ok

    def test_missing_ai_block_is_a_warning_not_an_error(self):
        result = validate_manifest(manifest())
        assert result.ok
        assert "ai.absent" in codes(result)

    def test_disabled_agent_needs_nothing_else(self):
        assert validate_manifest(manifest(ai={"enabled": False})).ok

    def test_unknown_tier_is_rejected(self):
        m = with_ai({"enabled": True, "tier": "autonomous"})
        assert "ai.tier" in codes(validate_manifest(m))

    def test_agent_spoke_must_require_the_ai_core(self):
        m = manifest(requires=["hub.master_data", "hub.ledger"], ai=AI_ADVISE)
        assert "ai.missing_service" in codes(validate_manifest(m))

    def test_agent_spoke_must_require_the_ledger(self):
        m = manifest(requires=["hub.master_data", "hub.ai_core"], ai=AI_ADVISE)
        assert "ai.missing_service" in codes(validate_manifest(m))


class TestActTierGuardrails:
    def test_act_tier_without_an_allowlist_is_rejected(self):
        m = with_ai({"enabled": True, "tier": "act", "origin_approval": "ORIG-2026-0221"})
        assert "ai.act_needs_allowlist" in codes(validate_manifest(m))

    def test_act_tier_without_origin_approval_is_rejected(self):
        m = with_ai({"enabled": True, "tier": "act", "act_allowlist": ["srm.scorecard.*"]})
        assert "ai.act_needs_origin" in codes(validate_manifest(m))

    def test_allowlist_cannot_reach_into_another_spoke(self):
        ai = dict(AI_ACT, act_allowlist=["ctr.commercial_terms.*"])
        assert "ai.foreign_allowlist" in codes(validate_manifest(with_ai(ai)))

    def test_allowlist_on_a_lower_tier_is_flagged_as_dead_config(self):
        ai = {"enabled": True, "tier": "advise", "act_allowlist": ["srm.scorecard.*"]}
        result = validate_manifest(with_ai(ai))
        assert result.ok
        assert "ai.allowlist_unused" in codes(result)


class TestDashboardDeclaration:
    def test_dashboards_referencing_published_kpis_are_valid(self):
        m = manifest(dashboards=[{"slug": "supplier-health", "kpis": ["srm.otif_pct"]}])
        assert validate_manifest(m).ok

    def test_non_kebab_slug_is_rejected(self):
        m = manifest(dashboards=[{"slug": "Supplier_Health"}])
        assert "dashboards.slug" in codes(validate_manifest(m))

    def test_duplicate_slug_is_rejected(self):
        m = manifest(dashboards=[{"slug": "overview"}, {"slug": "overview"}])
        assert "dashboards.duplicate" in codes(validate_manifest(m))

    def test_unpublished_own_kpi_is_rejected(self):
        m = manifest(dashboards=[{"slug": "overview", "kpis": ["srm.invented_metric"]}])
        assert "dashboards.unpublished_kpi" in codes(validate_manifest(m))

    def test_cross_spoke_kpi_is_allowed_with_a_staleness_warning(self):
        """This is the whole point of routing dashboards through the hub."""
        m = manifest(dashboards=[{"slug": "overview", "kpis": ["src.spend_usd_ttm"]}])
        result = validate_manifest(m)
        assert result.ok
        assert "dashboards.cross_spoke_kpi" in codes(result)
