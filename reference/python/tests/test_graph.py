# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

import pytest

from scor_ref.graph import CycleError, DependencyGraph, build_from_definitions


def graph_of(**edges) -> DependencyGraph:
    g = DependencyGraph()
    for field, inputs in edges.items():
        g.add_field(field.replace("__", "."), [i.replace("__", ".") for i in inputs])
    return g


class TestCycleDetection:
    def test_acyclic_graph_passes(self):
        g = graph_of(c=["b"], b=["a"], a=[])
        g.detect_cycle()

    def test_direct_self_reference_is_a_cycle(self):
        g = graph_of(a=["a"])
        with pytest.raises(CycleError) as exc:
            g.detect_cycle()
        assert exc.value.path == ["a", "a"]

    def test_two_field_cycle_is_detected(self):
        g = graph_of(a=["b"], b=["a"])
        with pytest.raises(CycleError):
            g.detect_cycle()

    def test_cycle_error_reports_the_actual_path(self):
        g = graph_of(a=["b"], b=["c"], c=["a"])
        with pytest.raises(CycleError) as exc:
            g.detect_cycle()
        assert exc.value.path[0] == exc.value.path[-1]
        assert set(exc.value.path) == {"a", "b", "c"}

    def test_cycle_through_another_spoke_is_detected(self):
        g = graph_of(
            srm__health=["ctr__exposure"],
            ctr__exposure=["src__spend"],
            src__spend=["srm__health"],
        )
        with pytest.raises(CycleError) as exc:
            g.detect_cycle()
        assert "srm.health" in exc.value.path

    def test_diamond_dependency_is_not_a_cycle(self):
        g = graph_of(d=["b", "c"], b=["a"], c=["a"], a=[])
        g.detect_cycle()


class TestTopologicalOrder:
    def test_inputs_come_before_dependents(self):
        g = graph_of(c=["b"], b=["a"], a=[])
        order = g.topological_order()
        assert order.index("a") < order.index("b") < order.index("c")

    def test_order_covers_every_field(self):
        g = graph_of(d=["b", "c"], b=["a"], c=["a"], a=[])
        assert sorted(g.topological_order()) == ["a", "b", "c", "d"]

    def test_order_is_stable_across_runs(self):
        g = graph_of(d=["b", "c"], b=["a"], c=["a"], a=[])
        assert len({tuple(g.topological_order()) for _ in range(20)}) == 1

    def test_cyclic_graph_cannot_be_ordered(self):
        g = graph_of(a=["b"], b=["a"])
        with pytest.raises(CycleError):
            g.topological_order()


class TestFanOut:
    def test_leaf_field_has_no_dependents(self):
        g = graph_of(b=["a"], a=[])
        assert g.fan_out("b", {}, budget=10).dependents == []

    def test_transitive_dependents_are_counted(self):
        g = graph_of(c=["b"], b=["a"], a=[])
        assert g.fan_out("a", {}, budget=10).dependents == ["b", "c"]

    def test_estimate_sums_rows_per_dependent(self):
        g = graph_of(c=["b"], b=["a"], a=[])
        result = g.fan_out("a", {"b": 1_000, "c": 250_000}, budget=1_000_000)
        assert result.estimated_rows == 251_000
        assert result.within_budget

    def test_breaching_the_budget_is_flagged(self):
        g = graph_of(c=["b"], b=["a"], a=[])
        result = g.fan_out("a", {"b": 1_000, "c": 5_000_000}, budget=1_000_000)
        assert not result.within_budget

    def test_a_widely_referenced_field_blows_the_budget(self):
        """The scenario the budget exists to catch."""
        g = DependencyGraph()
        g.add_field("inv.unit_cost_usd", [])
        rows = {}
        for i in range(200):
            name = f"tenant_{i}.extended_value_usd"
            g.add_field(name, ["inv.unit_cost_usd"])
            rows[name] = 50_000
        result = g.fan_out("inv.unit_cost_usd", rows, budget=1_000_000)
        assert result.estimated_rows == 10_000_000
        assert not result.within_budget


class TestBuildFromDefinitions:
    def test_definitions_become_edges(self):
        g = build_from_definitions(
            [
                {"field": "srm.health", "inputs": ["srm.otif_pct", "ctr.exposure"]},
                {"field": "srm.otif_pct", "inputs": []},
                {"field": "ctr.exposure", "inputs": []},
            ]
        )
        assert g.inputs_of("srm.health") == {"srm.otif_pct", "ctr.exposure"}
        assert g.dependents_of("ctr.exposure") == {"srm.health"}

    def test_referenced_but_undeclared_input_still_appears(self):
        g = build_from_definitions([{"field": "a", "inputs": ["b"]}])
        assert "b" in g.fields
