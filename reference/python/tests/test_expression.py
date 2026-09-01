# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

from decimal import Decimal

import pytest

from scor_ref.expr import evaluate, parse, references
from scor_ref.values import EvalError, Quantity, quantity


def n(value):
    return Quantity(Decimal(str(value)))


class TestParsing:
    def test_precedence_follows_arithmetic(self):
        assert evaluate("2 + 3 * 4") == n(14)

    def test_parentheses_override_precedence(self):
        assert evaluate("(2 + 3) * 4") == n(20)

    def test_unary_minus_binds_tighter_than_addition(self):
        assert evaluate("-2 + 5") == n(3)

    def test_empty_expression_is_rejected(self):
        with pytest.raises(EvalError, match="empty"):
            parse("   ")

    def test_chained_comparison_is_a_syntax_error(self):
        with pytest.raises(EvalError, match="chained comparison"):
            parse("1 < 2 < 3")

    def test_unknown_character_reports_position(self):
        with pytest.raises(EvalError, match="position 2"):
            parse("1 $ 2")

    def test_unbalanced_parenthesis_is_rejected(self):
        with pytest.raises(EvalError):
            parse("(1 + 2")

    def test_references_are_collected_for_the_dependency_graph(self):
        ast = parse("if(src.spend_usd > 0, srm.otif_pct, ctr.baseline)")
        assert references(ast) == {"src.spend_usd", "srm.otif_pct", "ctr.baseline"}

    def test_function_names_are_not_references(self):
        assert references(parse("min(a, b)")) == {"a", "b"}


class TestNullPropagation:
    def test_arithmetic_with_null_yields_null(self):
        assert evaluate("a + 1", {"a": None}) is None

    def test_comparison_with_null_yields_null(self):
        assert evaluate("a > 1", {"a": None}) is None

    def test_coalesce_picks_first_non_null(self):
        assert evaluate("coalesce(a, b, 5)", {"a": None, "b": None}) == n(5)

    def test_and_short_circuits_on_false(self):
        assert evaluate("false and missing_field") is False

    def test_or_short_circuits_on_true(self):
        assert evaluate("true or missing_field") is True

    def test_null_and_true_is_null(self):
        assert evaluate("a and true", {"a": None}) is None


class TestLaziness:
    def test_if_does_not_evaluate_the_untaken_branch(self):
        expression = "if(demand == 0, null, on_hand / demand)"
        assert evaluate(expression, {"demand": n(0), "on_hand": n(50)}) is None

    def test_unguarded_division_by_zero_is_an_error(self):
        with pytest.raises(EvalError, match="division_by_zero"):
            evaluate("on_hand / demand", {"demand": n(0), "on_hand": n(50)})


class TestUnitsAndCurrency:
    def test_adding_matching_units_keeps_the_unit(self):
        result = evaluate("a + b", {"a": quantity(2, "kg"), "b": quantity(3, "kg")})
        assert result == quantity(5, "kg")

    def test_adding_mismatched_units_is_rejected(self):
        with pytest.raises(EvalError, match="unit_mismatch"):
            evaluate("a + b", {"a": quantity(2, "kg"), "b": quantity(3, "lb")})

    def test_adding_mismatched_currencies_is_rejected(self):
        with pytest.raises(EvalError, match="currency_mismatch"):
            evaluate(
                "a + b",
                {"a": quantity(2, None, "USD"), "b": quantity(3, None, "KWD")},
            )

    def test_scaling_by_a_dimensionless_number_keeps_the_unit(self):
        result = evaluate("a * 3", {"a": quantity(2, "kg")})
        assert result == quantity(6, "kg")

    def test_dividing_like_units_produces_a_ratio(self):
        result = evaluate("a / b", {"a": quantity(6, "lines"), "b": quantity(3, "lines")})
        assert result == n(2)

    def test_multiplying_two_united_values_is_rejected(self):
        with pytest.raises(EvalError, match="unit_composition"):
            evaluate("a * b", {"a": quantity(2, "kg"), "b": quantity(3, "m")})


class TestTypeSafety:
    def test_adding_a_boolean_is_rejected(self):
        with pytest.raises(EvalError, match="type_error"):
            evaluate("a + 1", {"a": True})

    def test_not_on_a_number_is_rejected(self):
        with pytest.raises(EvalError, match="type_error"):
            evaluate("not a", {"a": n(1)})

    def test_missing_input_names_the_field(self):
        with pytest.raises(EvalError, match="srm.otif_pct"):
            evaluate("srm.otif_pct + 1", {})

    def test_unknown_function_is_rejected(self):
        with pytest.raises(EvalError, match="unknown_function"):
            evaluate("today()")

    def test_wrong_arity_is_rejected(self):
        with pytest.raises(EvalError, match="arity"):
            evaluate("min(1)")


class TestDeterminism:
    def test_same_input_gives_same_output_every_time(self):
        env = {"a": n(7), "b": n(3)}
        results = {evaluate("round(a / b, 4)", env) for _ in range(50)}
        assert len(results) == 1

    def test_decimal_arithmetic_is_not_binary_float(self):
        assert evaluate("0.1 + 0.2") == n("0.3")


class TestSpecFormulas:
    """The formulas published in the architecture spec must evaluate."""

    def test_reorder_point(self):
        env = {
            "average_daily_demand": quantity(120, "units_per_day"),
            "lead_time_days": quantity(14, "days"),
            "safety_stock": quantity(400, "units"),
        }
        expression = "(average_daily_demand * lead_time_days) + safety_stock"
        with pytest.raises(EvalError, match="unit_composition"):
            evaluate(expression, env)

    def test_reorder_point_with_dimensionless_lead_time(self):
        env = {
            "average_daily_demand": quantity(120, "units"),
            "lead_time_days": quantity(14),
            "safety_stock": quantity(400, "units"),
        }
        expression = "(average_daily_demand * lead_time_days) + safety_stock"
        assert evaluate(expression, env) == quantity(2080, "units")

    def test_supplier_otif(self):
        env = {
            "on_time_in_full_lines": quantity(920, "lines"),
            "total_delivered_lines": quantity(1000, "lines"),
        }
        expression = (
            "if(total_delivered_lines == 0, null, "
            "(on_time_in_full_lines / total_delivered_lines) * 100)"
        )
        assert evaluate(expression, env) == n(92)

    def test_realised_savings(self):
        env = {
            "baseline_price_usd": quantity(10, None, "USD"),
            "paid_price_usd": quantity(9, None, "USD"),
            "purchased_qty": quantity(500),
        }
        expression = "(baseline_price_usd - paid_price_usd) * purchased_qty"
        assert evaluate(expression, env) == quantity(500, None, "USD")

    def test_supplier_health_index_from_the_spec(self):
        env = {
            "otif_pct": n(92),
            "quality_ppm": n(2000),
            "price_realisation_pct": n(80),
        }
        expression = (
            "(otif_pct * 0.4) "
            "+ ((1 - min(quality_ppm / 10000, 1)) * 100 * 0.3) "
            "+ (price_realisation_pct * 0.3)"
        )
        assert evaluate(expression, env) == n("84.8")


class TestZeroIsDimensionless:
    """Regression: guarding a denominator must not require a unit cast."""

    def test_united_value_compares_against_bare_zero(self):
        assert evaluate("a == 0", {"a": quantity(1000, "lines")}) is False

    def test_united_zero_compares_against_bare_zero(self):
        assert evaluate("a == 0", {"a": quantity(0, "lines")}) is True

    def test_money_compares_against_bare_zero(self):
        assert evaluate("a > 0", {"a": quantity(5, None, "USD")}) is True

    def test_adding_bare_zero_keeps_the_unit(self):
        assert evaluate("a + 0", {"a": quantity(5, "kg")}) == quantity(5, "kg")

    def test_non_zero_mismatch_is_still_rejected(self):
        with pytest.raises(EvalError, match="unit_mismatch"):
            evaluate("a > 1", {"a": quantity(5, "kg")})
