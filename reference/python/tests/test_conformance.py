# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Runs the shared conformance vectors against the reference implementation.

The Rust crates run the same JSON files. If this file and the Rust
equivalent both pass, the two implementations agree by construction.
"""

from __future__ import annotations

import json
from decimal import Decimal
from pathlib import Path

import pytest

from scor_ref.expr import evaluate, parse, references
from scor_ref.values import EvalError, Quantity

CONFORMANCE = Path(__file__).resolve().parents[3] / "conformance"


def load(name: str) -> dict:
    return json.loads((CONFORMANCE / name).read_text())


def decode_value(spec: dict):
    kind = spec["kind"]
    if kind == "null":
        return None
    if kind == "bool":
        return spec["value"]
    return Quantity(Decimal(spec["amount"]), spec.get("unit"), spec.get("currency"))


EXPRESSION_CASES = load("expression-cases.json")["cases"]


@pytest.mark.parametrize("case", EXPRESSION_CASES, ids=lambda c: c["id"])
def test_expression_conformance(case):
    env = {name: decode_value(spec) for name, spec in case["env"].items()}
    expected = case["expect"]

    if expected["kind"] == "error":
        with pytest.raises(EvalError) as exc:
            evaluate(case["expression"], env)
        assert exc.value.code == expected["code"], (
            f"{case['id']}: expected error code {expected['code']!r}, got {exc.value.code!r}"
        )
        return

    if expected["kind"] == "references":
        assert sorted(references(parse(case["expression"]))) == sorted(expected["references"])
        return

    assert evaluate(case["expression"], env) == decode_value(expected)


def test_every_case_has_a_unique_id():
    ids = [case["id"] for case in EXPRESSION_CASES]
    assert len(ids) == len(set(ids))


def test_vector_file_declares_a_version():
    assert load("expression-cases.json")["version"]
