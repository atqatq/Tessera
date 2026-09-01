#!/usr/bin/env python3
"""Generates the committed conformance vectors from the Python reference.

The vectors are data in version control and are the contract between the
Python reference and the Rust implementation. Both sides consume the same
files byte-identically; regenerating them and getting a diff means a
behavioural change on purpose, which belongs in the commit message.

Stdlib only. Deterministic: no timestamps, no randomness, stable ordering.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tessera_ref import access as acc  # noqa: E402
from tessera_ref import ledger as led  # noqa: E402

VECTORS = Path(__file__).resolve().parents[1] / "vectors"


# ---------------------------------------------------------------- access

def rule(role, module, action, column, effect):
    return {"role": role, "module": module, "action": action,
            "column": column, "effect": effect}


def grant(owner, granted_to, columns, expires_at=None):
    return {"owner": owner, "granted_to": granted_to,
            "columns": columns, "expires_at": expires_at}


def user(role="planner", subject="u-1"):
    return {"kind": "user", "subject": subject, "role": role}


def agent(tier=acc.ADVISE, subject="ag-inv-1"):
    return {"kind": "agent", "subject": subject, "tier": tier}


def origin(subject="root"):
    return {"kind": "origin", "subject": subject}


def req(target, action, columns, actor, home="inv", intent=None):
    return {"target": target, "action": action, "columns": columns,
            "actor": actor, "home": home, "origin_intent": intent}


def base_env():
    return {
        "now": 1_000,
        "module_enabled": True,
        "known_columns": ["sku", "qty", "price"],
        "rules": [rule("planner", "inv", acc.RULE_READ, "*", acc.ALLOW)],
        "grants": [],
        "agent_allowlist": [],
        "origin_approvals": [],
    }


def case(name, env, request):
    code, layer = acc.evaluate(env, request)
    return {"name": name, "env": env, "request": request,
            "expected": {"code": code, "layer": layer}}


def access_cases():
    cases = []

    # ---- all fourteen codes, one clean case each -------------------
    env = base_env()
    cases.append(case(
        "origin_with_recorded_intent_is_allowed", env,
        req("inv", acc.READ, ["sku"], origin(), intent="int-1")))

    cases.append(case(
        "planner_read_is_allowed_by_star_rule", env,
        req("inv", acc.READ, ["sku"], user())))

    cases.append(case(
        "agent_read_is_allowed_by_tier", env,
        req("inv", acc.READ, ["sku"], agent(acc.OBSERVE))))

    peer_env = base_env()
    peer_env["rules"].append(rule("planner", "ord", acc.RULE_READ, "*", acc.ALLOW))
    peer_env["grants"].append(grant("ord", "ful", ["sku", "qty"]))
    r = req("ord", acc.READ, ["sku"], user(), home="ful")
    cases.append(case("covered_peer_read_is_allowed_by_grant", peer_env, r))

    disabled = {**base_env(), "module_enabled": False}
    cases.append(case(
        "a_disabled_module_gates_everyone_including_origin", disabled,
        req("inv", acc.READ, [], origin(), intent="int-1")))

    cases.append(case(
        "origin_without_recorded_intent_is_refused", env,
        req("inv", acc.READ, ["sku"], origin())))

    cases.append(case(
        "observe_agent_may_not_propose", env,
        req("inv", acc.PROPOSE, ["qty"], agent(acc.OBSERVE))))

    cases.append(case(
        "act_agent_write_needs_an_allowlist_entry", env,
        req("inv", acc.WRITE, ["qty"], agent(acc.ACT))))

    allowlisted = {**base_env(),
                   "agent_allowlist": [["inv", acc.WRITE]]}
    cases.append(case(
        "act_agent_write_needs_origin_approval_even_when_allowlisted",
        allowlisted,
        req("inv", acc.WRITE, ["qty"], agent(acc.ACT))))

    no_grant_env = base_env()
    cases.append(case(
        "peer_read_requires_a_grant", no_grant_env,
        req("ord", acc.READ, ["sku"], user(), home="ful")))

    expired_env = base_env()
    expired_env["rules"].append(rule("planner", "ord", acc.RULE_READ, "*", acc.ALLOW))
    expired_env["grants"].append(grant("ord", "ful", ["sku"], expires_at=1_000))
    cases.append(case(
        "an_expired_grant_is_denied_at_the_expiry_instant", expired_env,
        req("ord", acc.READ, ["sku"], user(), home="ful")))

    deny_env = base_env()
    deny_env["rules"].append(rule("planner", "inv", acc.RULE_READ, "price", acc.DENY))
    cases.append(case(
        "explicit_deny_beats_allow_on_the_same_column", deny_env,
        req("inv", acc.READ, ["price"], user())))

    cases.append(case(
        "unknown_column_is_denied", env,
        req("inv", acc.READ, ["not_a_column"], user())))

    uncovered = {**base_env(), "rules": []}
    cases.append(case(
        "uncovered_column_falls_through_to_default_deny", uncovered,
        req("inv", acc.READ, ["qty"], user())))

    # ---- behavioural boundaries ------------------------------------
    cases.append(case(
        "origin_bypasses_unknown_columns_and_rules", env,
        req("inv", acc.READ, ["not_a_column"], origin(), intent="int-1")))

    write_rule_env = base_env()
    write_rule_env["rules"].append(rule("planner", "inv", acc.RULE_WRITE, "qty", acc.ALLOW))
    cases.append(case(
        "propose_is_judged_by_the_write_rules", write_rule_env,
        req("inv", acc.PROPOSE, ["qty"], user())))
    cases.append(case(
        "write_uses_the_same_rules_as_propose", write_rule_env,
        req("inv", acc.WRITE, ["qty"], user())))

    cases.append(case(
        "a_module_level_read_needs_the_star_rule", env,
        req("inv", acc.READ, [], user())))
    cases.append(case(
        "a_module_level_read_without_star_rule_is_denied", uncovered,
        req("inv", acc.READ, [], user())))

    cases.append(case(
        "glob_exact_rules_do_not_match_longer_names",
        {**base_env(), "rules": [rule("planner", "inv", acc.RULE_READ, "qty", acc.ALLOW)]},
        req("inv", acc.READ, ["qty_reserved"], user())))

    grant_env = base_env()
    grant_env["rules"].append(rule("planner", "ord", acc.RULE_READ, "*", acc.ALLOW))
    grant_env["grants"].append(grant("ord", "ful", ["qty"]))
    cases.append(case(
        "a_grant_must_cover_every_requested_column", grant_env,
        req("ord", acc.READ, ["sku", "qty"], user(), home="ful")))
    cases.append(case(
        "peer_writes_do_not_exist_as_an_operation", grant_env,
        req("ord", acc.WRITE, ["qty"], user(), home="ful")))
    cases.append(case(
        "agent_peer_read_is_allowed_by_grant",
        {**grant_env, "grants": [grant("ord", "inv", ["sku"])]},
        req("ord", acc.READ, ["sku"], agent(acc.OBSERVE))))
    cases.append(case(
        "agent_peer_read_without_grant_is_denied", grant_env,
        req("ord", acc.READ, ["sku"], agent(acc.OBSERVE))))

    approved = {**base_env(),
                "agent_allowlist": [["inv", acc.WRITE]],
                "origin_approvals": [["ag-inv-1", "inv", acc.WRITE]]}
    cases.append(case(
        "allowlisted_and_approved_act_agent_may_write", approved,
        req("inv", acc.WRITE, ["qty"], agent(acc.ACT))))

    auditor_env = base_env()
    auditor_env["rules"].append(rule("auditor", "inv", acc.RULE_READ, "price", acc.ALLOW))
    cases.append(case(
        "kernel_level_user_without_home_is_judged_by_rules_alone",
        auditor_env,
        req("inv", acc.READ, ["price"], user(role="auditor", subject="u-audit"), home=None)))

    one_tick = {**expired_env, "now": 999}
    cases.append(case(
        "a_grant_one_tick_before_expiry_still_allows", one_tick,
        req("ord", acc.READ, ["sku"], user(), home="ful")))

    return cases


# ---------------------------------------------------------------- ledger

def ledger_entry(height, valid_ms, system_ms, payload: bytes):
    return {"height": height, "valid_ms": valid_ms,
            "system_ms": system_ms, "payload_hex": payload.hex()}


def tamper(records, kind, index, byte):
    """Apply a single-byte tamper to the committed records."""
    out = [dict(r) for r in records]
    if kind == "payload":
        # payload tamper is applied to the *entry*, not the record hash
        return out, ("payload", index, byte)
    if kind == "hash":
        out[index]["hash_hex"] = _flip(out[index]["hash_hex"], byte)
        return out, None
    if kind == "prev":
        out[index]["prev_hex"] = _flip(out[index]["prev_hex"], byte)
        return out, None
    raise ValueError(kind)


def _flip(hexstr, byte):
    raw = bytearray(bytes.fromhex(hexstr))
    raw[byte % len(raw)] ^= 0x01
    return raw.hex()


def ledger_cases():
    t0, s0 = 1_700_000_000_000, 1_700_000_000_050
    cases = []

    def build(name, tenant, entries, tamper_spec=None):
        records = led.build_chain(tenant, entries)
        stored = [dict(r) for r in records]
        stored_entries = [dict(e) for e in entries]
        first_broken = None
        if tamper_spec:
            kind, index, byte = tamper_spec
            if kind == "payload":
                raw = bytearray(bytes.fromhex(stored_entries[index]["payload_hex"]))
                if raw:
                    raw[byte % len(raw)] ^= 0x01
                    stored_entries[index]["payload_hex"] = raw.hex()
                else:
                    stored_entries[index]["payload_hex"] = "00"
            elif kind == "hash":
                stored[index]["hash_hex"] = _flip(stored[index]["hash_hex"], byte)
            elif kind == "prev":
                stored[index]["prev_hex"] = _flip(stored[index]["prev_hex"], byte)
            first_broken = led.verify(stored, tenant, stored_entries)
        else:
            first_broken = led.verify(stored, tenant, stored_entries)
        cases.append({
            "name": name,
            "tenant": tenant,
            "entries": entries,
            "tamper": ({"kind": tamper_spec[0], "record": tamper_spec[1],
                        "byte": tamper_spec[2]} if tamper_spec else None),
            "expected": {
                "records": stored,
                "first_broken_height": first_broken,
            },
        })

    build("genesis_single_entry", "acme",
          [ledger_entry(0, t0, s0, b"order#1")])
    build("chain_of_three", "acme",
          [ledger_entry(i, t0 + i, s0 + i, f"event-{i}".encode())
           for i in range(3)])
    build("payload_tamper_detected_at_exact_height", "acme",
          [ledger_entry(i, t0 + i, s0 + i, f"event-{i}".encode())
           for i in range(3)],
          ("payload", 1, 0))
    build("hash_tamper_detected_at_exact_height", "acme",
          [ledger_entry(i, t0 + i, s0 + i, f"event-{i}".encode())
           for i in range(3)],
          ("hash", 0, 7))
    build("prev_tamper_detected_at_exact_height", "acme",
          [ledger_entry(i, t0 + i, s0 + i, f"event-{i}".encode())
           for i in range(3)],
          ("prev", 2, 3))
    build("empty_payload_entry", "acme",
          [ledger_entry(0, t0, s0, b""), ledger_entry(1, t0 + 1, s0 + 1, b"x")])
    build("cross_tenant_same_events_different_hashes", "otherco",
          [ledger_entry(i, t0 + i, s0 + i, f"event-{i}".encode())
           for i in range(3)])
    return cases


# ---------------------------------------------------------------- main

def main() -> int:
    VECTORS.mkdir(parents=True, exist_ok=True)
    access_doc = {
        "domain": "tessera-access/1",
        "codes": acc.CODES,
        "cases": access_cases(),
    }
    ledger_doc = {
        "domain": "tessera-ledger/1",
        "algorithm": ("SHA-256(\"tessera-ledger/1\" || tenant || 0x00 || prev"
                      " || u64be(height) || u64be(valid_ms) || u64be(system_ms)"
                      " || u32be(payload_len) || payload)"),
        "cases": ledger_cases(),
    }
    (VECTORS / "access.vectors.json").write_text(
        json.dumps(access_doc, indent=2) + "\n", encoding="utf-8")
    (VECTORS / "ledger.vectors.json").write_text(
        json.dumps(ledger_doc, indent=2) + "\n", encoding="utf-8")
    n_access = len(access_doc["cases"])
    n_ledger = len(ledger_doc["cases"])
    print(f"wrote {n_access} access cases, {n_ledger} ledger cases")
    covered = {c["expected"]["code"] for c in access_doc["cases"]}
    missing = [c for c in acc.CODES if c not in covered]
    if missing:
        print(f"ERROR: codes not covered by vectors: {missing}", file=sys.stderr)
        return 1
    print("all 14 decision codes covered")
    return 0


if __name__ == "__main__":
    sys.exit(main())
