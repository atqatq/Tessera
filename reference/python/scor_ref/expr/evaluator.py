# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Evaluator for the hub expression DSL.

Guarantees this module is responsible for:

* Determinism. There is no clock, no randomness, no external read. The same
  AST and the same environment always produce the same value, which is what
  makes ledger replay meaningful.
* Null propagation. NULL flows through arithmetic instead of raising, so the
  `null` missing-value policy needs no special casing.
* Unit and currency safety. Adding kilograms to pounds, or KWD to USD, is an
  error rather than a silent number.
* Laziness where it matters. `if`, `and` and `or` do not evaluate branches
  they do not need, so guarded division never divides by zero.
"""

from __future__ import annotations

from decimal import Decimal, DivisionByZero, InvalidOperation
from typing import Dict, Optional

from ..values import EvalError, Quantity, Value, check_compatible, combine_multiplicative
from .parser import Node, parse

LAZY_FUNCTIONS = {"if"}
STRICT_FUNCTIONS = {"min": (2, 2), "max": (2, 2), "abs": (1, 1), "round": (1, 2)}


class Evaluator:
    def __init__(self, env: Dict[str, Value]):
        self.env = env

    def eval(self, node: Node) -> Value:
        handler = getattr(self, f"_eval_{node.kind}", None)
        if handler is None:
            raise EvalError("internal", f"no handler for node kind {node.kind!r}")
        return handler(node)

    # -- leaves ---------------------------------------------------------

    def _eval_number(self, node: Node) -> Value:
        return Quantity(node.value)

    def _eval_bool(self, node: Node) -> Value:
        return bool(node.value)

    def _eval_null(self, node: Node) -> Value:
        return None

    def _eval_ref(self, node: Node) -> Value:
        name = str(node.value)
        if name not in self.env:
            raise EvalError("missing_input", f"field {name!r} is not in the environment")
        return self.env[name]

    # -- operators ------------------------------------------------------

    def _eval_negate(self, node: Node) -> Value:
        inner = self.eval(node.children[0])
        if inner is None:
            return None
        q = _as_quantity(inner, "unary '-'")
        return Quantity(-q.amount, q.unit, q.currency)

    def _eval_not(self, node: Node) -> Value:
        inner = self.eval(node.children[0])
        if inner is None:
            return None
        return not _as_bool(inner, "'not'")

    def _eval_binary(self, node: Node) -> Value:
        op = str(node.value)
        if op == "and":
            left = self.eval(node.children[0])
            if left is False:
                return False
            right = self.eval(node.children[1])
            if left is None or right is None:
                return None if right is not False else False
            return _as_bool(left, "'and'") and _as_bool(right, "'and'")
        if op == "or":
            left = self.eval(node.children[0])
            if left is True:
                return True
            right = self.eval(node.children[1])
            if left is None or right is None:
                return None if right is not True else True
            return _as_bool(left, "'or'") or _as_bool(right, "'or'")

        left = self.eval(node.children[0])
        right = self.eval(node.children[1])
        if left is None or right is None:
            return None
        if op in ("==", "!="):
            equal = _equals(left, right)
            return equal if op == "==" else not equal

        lq = _as_quantity(left, f"'{op}'")
        rq = _as_quantity(right, f"'{op}'")

        if op in ("<", ">", "<=", ">="):
            check_compatible(lq, rq, op)
            if op == "<":
                return lq.amount < rq.amount
            if op == ">":
                return lq.amount > rq.amount
            if op == "<=":
                return lq.amount <= rq.amount
            return lq.amount >= rq.amount

        if op in ("+", "-"):
            check_compatible(lq, rq, op)
            amount = lq.amount + rq.amount if op == "+" else lq.amount - rq.amount
            return Quantity(amount, lq.unit, lq.currency)

        unit, currency = combine_multiplicative(lq, rq, op)
        try:
            amount = lq.amount * rq.amount if op == "*" else lq.amount / rq.amount
        except (DivisionByZero, InvalidOperation):
            raise EvalError(
                "division_by_zero",
                "divided by zero; guard the denominator with if(denominator == 0, null, ...)",
            )
        return Quantity(amount, unit, currency)

    # -- calls ----------------------------------------------------------

    def _eval_call(self, node: Node) -> Value:
        name = str(node.value)
        if name == "if":
            if len(node.children) != 3:
                raise EvalError("arity", "if() takes exactly 3 arguments")
            condition = self.eval(node.children[0])
            if condition is None:
                return None
            return self.eval(node.children[1] if _as_bool(condition, "if()") else node.children[2])
        if name == "coalesce":
            if not node.children:
                raise EvalError("arity", "coalesce() takes at least 1 argument")
            for child in node.children:
                value = self.eval(child)
                if value is not None:
                    return value
            return None
        if name not in STRICT_FUNCTIONS:
            raise EvalError("unknown_function", f"{name}() is not in the expression language")

        low, high = STRICT_FUNCTIONS[name]
        if not low <= len(node.children) <= high:
            raise EvalError("arity", f"{name}() takes {low} to {high} arguments")
        args = [self.eval(child) for child in node.children]
        if any(a is None for a in args):
            return None
        return _apply_strict(name, args)


def _apply_strict(name: str, args) -> Value:
    quantities = [_as_quantity(a, f"{name}()") for a in args]
    if name in ("min", "max"):
        check_compatible(quantities[0], quantities[1], name)
        chosen = min(quantities, key=lambda q: q.amount) if name == "min" else max(
            quantities, key=lambda q: q.amount
        )
        return chosen
    if name == "abs":
        q = quantities[0]
        return Quantity(abs(q.amount), q.unit, q.currency)
    q = quantities[0]
    places = int(quantities[1].amount) if len(quantities) == 2 else 0
    return Quantity(round(q.amount, places), q.unit, q.currency)


def _as_quantity(value: Value, context: str) -> Quantity:
    if isinstance(value, Quantity):
        return value
    raise EvalError("type_error", f"{context} expected a number, got a boolean")


def _as_bool(value: Value, context: str) -> bool:
    if isinstance(value, bool):
        return value
    raise EvalError("type_error", f"{context} expected a boolean, got a number")


def _equals(left: Value, right: Value) -> bool:
    if isinstance(left, bool) or isinstance(right, bool):
        return left is right
    lq, rq = left, right
    check_compatible(lq, rq, "==")
    return lq.amount == rq.amount


def evaluate(source: str, env: Optional[Dict[str, Value]] = None) -> Value:
    """Parse and evaluate in one call. Convenience for tests and the CLI."""
    return Evaluator(env or {}).eval(parse(source))
