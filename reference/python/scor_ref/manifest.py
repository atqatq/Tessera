# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Spoke manifest validation.

One rule carries the whole independence guarantee:

    A hard `requires` entry may name a hub service and nothing else.

If a spoke could hard-require another spoke, disabling that other spoke
would cascade, and the promise that any organisation can switch off the
spokes it does not need would be false. Everything else in this module
exists to make that rule impossible to sidestep.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field as dc_field
from typing import Dict, List, Sequence

SEMVER = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?\Z")
CODE = re.compile(r"^[a-z][a-z0-9_]{1,15}\Z")
NAMESPACED = re.compile(r"^[a-z][a-z0-9_]{1,15}\.[a-z][a-z0-9_.]*\Z")

#: The complete set of hub services a spoke is allowed to hard-require.
#: Adding to this list is an origin-level decision, not a spoke's.
HUB_SERVICES = frozenset(
    {
        "hub.origin",
        "hub.access",
        "hub.master_data",
        "hub.ingest",
        "hub.ledger",
        "hub.events",
        "hub.plugin_host",
        "hub.tenancy",
        "hub.ai_core",
    }
)

#: Spoke codes in the register. A manifest for an unregistered code is
#: rejected so that typos cannot create a shadow spoke.
REGISTERED_SPOKES = frozenset(
    {
        "pln",
        "src",
        "trf",
        "ord",
        "ful",
        "ret",
        "inv",
        "srm",
        "ctr",
        "tsk",
        "prj",
    }
)

MISSING_POLICIES = frozenset({"hold_last", "default", "null", "fail"})

#: Agent capability tiers, least to most privileged.
AGENT_TIERS = frozenset({"observe", "advise", "act"})

SLUG = re.compile(r"^[a-z][a-z0-9-]{1,39}\Z")


@dataclass
class Finding:
    severity: str  # error | warning
    code: str
    message: str

    def __str__(self) -> str:
        return f"[{self.severity}] {self.code}: {self.message}"


@dataclass
class ValidationResult:
    findings: List[Finding] = dc_field(default_factory=list)

    @property
    def errors(self) -> List[Finding]:
        return [f for f in self.findings if f.severity == "error"]

    @property
    def warnings(self) -> List[Finding]:
        return [f for f in self.findings if f.severity == "warning"]

    @property
    def ok(self) -> bool:
        return not self.errors

    def error(self, code: str, message: str) -> None:
        self.findings.append(Finding("error", code, message))

    def warn(self, code: str, message: str) -> None:
        self.findings.append(Finding("warning", code, message))


def validate_manifest(manifest: dict) -> ValidationResult:
    """Validate a single spoke manifest. Returns every finding, not just the first."""
    result = ValidationResult()

    spoke = manifest.get("spoke")
    if not isinstance(spoke, str) or not CODE.match(spoke):
        result.error("spoke.code", f"spoke code {spoke!r} must be lowercase snake_case")
        spoke = spoke if isinstance(spoke, str) else ""
    elif spoke not in REGISTERED_SPOKES:
        result.error("spoke.unregistered", f"spoke {spoke!r} is not in the register")

    version = manifest.get("version")
    if not isinstance(version, str) or not SEMVER.match(version):
        result.error("spoke.version", f"version {version!r} is not semver")

    _validate_requires(manifest.get("requires", []), result)
    _validate_enhances(manifest.get("enhances", []), spoke, result)
    _validate_provides(manifest.get("provides", {}), spoke, result)
    _validate_consumes(manifest.get("consumes", []), spoke, result)
    _validate_ai(manifest.get("ai"), spoke, manifest.get("requires", []), result)
    _validate_dashboards(manifest.get("dashboards", []), spoke, manifest.get("provides", {}), result)

    return result


def _validate_ai(ai, spoke: str, requires, result: ValidationResult) -> None:
    """Every spoke may run an agent; none of them may run one unsupervised.

    The agent is a principal in its own right, so the manifest has to say
    what it is allowed to do before the plugin host will start it.
    """
    if ai is None:
        result.warn("ai.absent", "spoke declares no agent; it will report nothing to the leader")
        return
    if not isinstance(ai, dict):
        result.error("ai.type", "ai must be an object")
        return
    if not ai.get("enabled", False):
        return

    tier = ai.get("tier")
    if tier not in AGENT_TIERS:
        result.error(
            "ai.tier",
            f"agent tier {tier!r} must be one of " + ", ".join(sorted(AGENT_TIERS)),
        )

    required = set(requires) if isinstance(requires, list) else set()
    for service in ("hub.ai_core", "hub.ledger"):
        if service not in required:
            result.error(
                "ai.missing_service",
                f"an agent-bearing spoke must require {service!r}: the leader cannot "
                "receive signals, and the agent's actions cannot be logged, without it",
            )

    allowlist = ai.get("act_allowlist", []) or []
    if not isinstance(allowlist, list):
        result.error("ai.act_allowlist", "act_allowlist must be a list of field patterns")
        allowlist = []
    for pattern in allowlist:
        if not isinstance(pattern, str) or not pattern.startswith(f"{spoke}."):
            result.error(
                "ai.foreign_allowlist",
                f"{pattern!r} is outside this spoke's namespace; an agent never writes "
                "another spoke's data",
            )

    if tier == "act":
        if not allowlist:
            result.error(
                "ai.act_needs_allowlist",
                "act-tier agents must declare a non-empty act_allowlist; unbounded "
                "autonomous write is not a supported configuration",
            )
        if not ai.get("origin_approval"):
            result.error(
                "ai.act_needs_origin",
                "act-tier autonomy requires recorded origin approval",
            )
    elif allowlist:
        result.warn(
            "ai.allowlist_unused",
            f"act_allowlist is declared but tier is {tier!r}, so it has no effect",
        )


def _validate_dashboards(dashboards, spoke: str, provides, result: ValidationResult) -> None:
    """Dashboards ship with the spoke; tenants then modify them freely."""
    if not isinstance(dashboards, list):
        result.error("dashboards.type", "dashboards must be a list")
        return
    published_kpis = set((provides or {}).get("kpis", []) or [])
    seen = set()
    for entry in dashboards:
        if not isinstance(entry, dict):
            result.error("dashboards.type", f"dashboard entry {entry!r} must be an object")
            continue
        slug = entry.get("slug")
        if not isinstance(slug, str) or not SLUG.match(slug):
            result.error("dashboards.slug", f"dashboard slug {slug!r} must be kebab-case")
            continue
        if slug in seen:
            result.error("dashboards.duplicate", f"dashboard slug {slug!r} is declared twice")
        seen.add(slug)
        for kpi in entry.get("kpis", []) or []:
            if kpi in published_kpis:
                continue
            if isinstance(kpi, str) and kpi.split(".", 1)[0] != spoke:
                result.warn(
                    "dashboards.cross_spoke_kpi",
                    f"{kpi!r} belongs to another spoke; the widget resolves through the "
                    "hub and will show as stale when that spoke is unavailable",
                )
            else:
                result.error(
                    "dashboards.unpublished_kpi",
                    f"{kpi!r} is not in provides.kpis",
                )


def _validate_requires(requires, result: ValidationResult) -> None:
    if not isinstance(requires, list):
        result.error("requires.type", "requires must be a list")
        return
    if not requires:
        result.warn("requires.empty", "spoke declares no hub services; is that intentional?")
    for entry in requires:
        if not isinstance(entry, str):
            result.error("requires.type", f"requires entry {entry!r} must be a string")
            continue
        if entry.startswith("spoke."):
            result.error(
                "requires.spoke_dependency",
                f"{entry!r} is a hard dependency on another spoke, which would break "
                "independent disabling; move it to 'enhances'",
            )
            continue
        if entry not in HUB_SERVICES:
            result.error(
                "requires.unknown_service",
                f"{entry!r} is not a hub service; allowed values are "
                + ", ".join(sorted(HUB_SERVICES)),
            )


def _validate_enhances(enhances, spoke: str, result: ValidationResult) -> None:
    if not isinstance(enhances, list):
        result.error("enhances.type", "enhances must be a list")
        return
    for entry in enhances:
        if not isinstance(entry, str) or not entry.startswith("spoke."):
            result.error("enhances.format", f"enhances entry {entry!r} must be 'spoke.<code>'")
            continue
        code = entry.split(".", 1)[1]
        if code not in REGISTERED_SPOKES:
            result.error("enhances.unregistered", f"spoke {code!r} is not in the register")
        if code == spoke:
            result.error("enhances.self", f"spoke {spoke!r} cannot enhance itself")


def _validate_provides(provides, spoke: str, result: ValidationResult) -> None:
    if not isinstance(provides, dict):
        result.error("provides.type", "provides must be an object")
        return
    for bucket in ("objects", "events", "kpis"):
        for name in provides.get(bucket, []) or []:
            if not isinstance(name, str) or not NAMESPACED.match(name):
                result.error(
                    "provides.namespace",
                    f"{bucket} entry {name!r} must be namespaced as '<spoke>.<name>'",
                )
                continue
            if not name.startswith(f"{spoke}."):
                result.error(
                    "provides.foreign_namespace",
                    f"{name!r} is published under another spoke's namespace",
                )


def _validate_consumes(consumes, spoke: str, result: ValidationResult) -> None:
    """Cross-spoke reads must declare a missing-value policy up front."""
    if not isinstance(consumes, list):
        result.error("consumes.type", "consumes must be a list")
        return
    for entry in consumes:
        if not isinstance(entry, dict):
            result.error("consumes.type", f"consumes entry {entry!r} must be an object")
            continue
        ref = entry.get("field")
        policy = entry.get("on_missing")
        if not isinstance(ref, str) or not NAMESPACED.match(ref):
            result.error("consumes.field", f"consumes field {ref!r} must be namespaced")
            continue
        owner = ref.split(".", 1)[0]
        if owner == spoke:
            result.error(
                "consumes.own_namespace",
                f"{ref!r} belongs to this spoke; consumes is for cross-spoke reads only",
            )
        if policy not in MISSING_POLICIES:
            result.error(
                "consumes.on_missing",
                f"{ref!r} must declare on_missing as one of "
                + ", ".join(sorted(MISSING_POLICIES)),
            )
        elif policy == "fail" and not entry.get("origin_approval"):
            result.error(
                "consumes.fail_needs_origin",
                f"{ref!r} uses on_missing 'fail', which can break this spoke when "
                f"{owner!r} is disabled; it requires recorded origin approval",
            )


def validate_all(manifests: Sequence[dict]) -> Dict[str, ValidationResult]:
    """Validate a set of manifests and cross-check the spoke graph."""
    results = {m.get("spoke", f"<unnamed-{i}>"): validate_manifest(m) for i, m in enumerate(manifests)}
    published: Dict[str, str] = {}
    for manifest in manifests:
        spoke = manifest.get("spoke", "")
        for name in (manifest.get("provides", {}) or {}).get("objects", []) or []:
            if isinstance(name, str):
                if name in published and published[name] != spoke:
                    results[spoke].error(
                        "provides.collision",
                        f"{name!r} is already published by {published[name]!r}",
                    )
                published[name] = spoke
    for manifest in manifests:
        spoke = manifest.get("spoke", "")
        for entry in manifest.get("consumes", []) or []:
            if not isinstance(entry, dict):
                continue
            ref = entry.get("field")
            if not isinstance(ref, str) or "." not in ref:
                continue
            owner = ref.split(".", 1)[0]
            if owner in REGISTERED_SPOKES and owner not in {m.get("spoke") for m in manifests}:
                results[spoke].warn(
                    "consumes.absent_owner",
                    f"{ref!r} is owned by {owner!r}, which is not installed; "
                    f"the {entry.get('on_missing')!r} policy will apply",
                )
    return results
