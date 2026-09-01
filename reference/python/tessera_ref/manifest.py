"""Manifest validator for the frozen module-manifest schema v1.

A convenience checker mirroring the constraints of
``schemas/module-manifest/v1/module-manifest.schema.json`` — the JSON
Schema is the normative artefact; this stdlib-only mirror lets the
committed conformance vectors be replayed anywhere. When the kernel's
plugin host lands, it replays the same files (vectors are the
contract).
"""

from __future__ import annotations

import json
import re
from pathlib import Path

SCHEMA_V1 = Path(__file__).resolve().parents[2] / "schemas" / "module-manifest" / "v1" / "module-manifest.schema.json"

ID_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{0,63}$")
SEMVER_RE = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?"
    r"(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
)

# The frozen requires enum — kernel.notary is deliberately absent:
# notarisation is a pluggable adapter, not a service (E3).
REQUIRES_ENUM = [
    "kernel.access", "kernel.ledger", "kernel.master_data",
    "kernel.events", "kernel.plugin_host", "kernel.tenancy",
    "kernel.ingest", "kernel.ids", "kernel.origin", "kernel.ai_core",
    "kernel.agents", "kernel.grid",
]
TIERS = {"observe", "advise", "act"}
GRANULARITY = {"exact", "banded", "aggregate"}


class ManifestInvalid(ValueError):
    """The manifest is refused; the message names the violated rule."""


def _reject(cond: bool, message: str) -> None:
    if cond:
        raise ManifestInvalid(message)


def validate(manifest: dict) -> None:
    """Refuses anything the frozen v1 schema refuses."""
    _reject(not isinstance(manifest, dict), "manifest must be an object")
    _reject(manifest.get("schema_version") != 1,
            "unknown schema_version: forward compatibility is a refusal, not a guess")

    top_keys = {"schema_version", "module", "requires", "permissions", "agent",
                "kpis", "telemetry", "egress_candidates", "compatibility"}
    unknown = set(manifest) - top_keys
    _reject(bool(unknown), f"unknown fields: {sorted(unknown)}")
    missing = top_keys - set(manifest)
    _reject(bool(missing), f"missing fields: {sorted(missing)}")

    module = manifest["module"]
    _reject(not isinstance(module, dict), "module must be an object")
    mod_keys = {"id", "name", "version", "description"}
    unknown = set(module) - mod_keys
    _reject(bool(unknown), f"module: unknown fields: {sorted(unknown)}")
    missing = mod_keys - set(module)
    _reject(bool(missing), f"module: missing fields: {sorted(missing)}")

    mid = module["id"]
    _reject(not isinstance(mid, str) or not ID_RE.match(mid),
            f"module.id `{mid}` violates the identifier grammar")
    _reject(mid.startswith("kernel."), "module ids never carry the kernel. prefix")
    _reject(not SEMVER_RE.match(str(module["version"])),
            f"module.version `{module['version']}` is not SemVer 2.0.0")

    requires = manifest["requires"]
    _reject(not isinstance(requires, list), "requires must be an array")
    for r in requires:
        _reject(r not in REQUIRES_ENUM,
                f"requires `{r}` is not a kernel service (modules are not services; "
                "kernel.notary is an adapter, not a service)")

    permissions = manifest["permissions"]
    _reject(not isinstance(permissions, dict), "permissions must be an object")
    for pr in permissions.get("peer_reads", []):
        _reject(not isinstance(pr, dict) or "module" not in pr or "columns" not in pr,
                "permissions.peer_reads entries need module + columns")
        _reject(not ID_RE.match(pr["module"]), f"peer_reads module `{pr['module']}` invalid")
        _reject(not isinstance(pr["columns"], list), "peer_reads.columns must be an array")

    agent = manifest["agent"]
    _reject(agent.get("tier") not in TIERS,
            f"agent.tier `{agent.get('tier')}` is not observe|advise|act")
    _reject(not isinstance(agent.get("craft"), str) or not agent["craft"],
            "agent.craft is required")

    for kpi in manifest["kpis"]:
        _reject(not isinstance(kpi, dict) or not {"id", "label", "unit"} <= set(kpi),
                "kpis entries need id, label, unit")
        _reject(not ID_RE.match(kpi["id"]), f"kpi id `{kpi['id']}` violates the grammar")

    for t in manifest["telemetry"]:
        _reject(not isinstance(t, dict) or not {"kind", "description"} <= set(t),
                "telemetry entries need kind, description")

    for e in manifest["egress_candidates"]:
        _reject(not isinstance(e, dict) or "column" not in e, "egress entries need column")
        _reject(e.get("granularity") not in GRANULARITY,
                f"granularity `{e.get('granularity')}` is not exact|banded|aggregate")

    compat = manifest["compatibility"]
    _reject(compat.get("manifest_schema") != 1, "compatibility.manifest_schema must be 1 for v1")
    _reject(not isinstance(compat.get("kernel_api"), str) or not compat["kernel_api"],
            "compatibility.kernel_api is required")


def validate_file(path: Path) -> None:
    validate(json.loads(Path(path).read_text(encoding="utf-8")))
