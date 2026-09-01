# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Value model for the expression DSL.

Every number carries an optional unit and an optional currency. The hub
normalises money to the USD reserve before evaluation, so a currency
mismatch here is a bug upstream, not a conversion opportunity.
"""

from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Optional, Union


class EvalError(Exception):
    """Raised when an expression cannot be evaluated. Never swallowed."""

    def __init__(self, code: str, message: str):
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


@dataclass(frozen=True)
class Quantity:
    """A number with an optional unit and optional currency."""

    amount: Decimal
    unit: Optional[str] = None
    currency: Optional[str] = None

    def is_dimensionless(self) -> bool:
        return self.unit is None and self.currency is None

    def describe(self) -> str:
        parts = [str(self.amount)]
        if self.unit:
            parts.append(self.unit)
        if self.currency:
            parts.append(self.currency)
        return " ".join(parts)


#: NULL is a first-class value. It propagates through arithmetic rather than
#: raising, which is what the `null` missing-value policy relies on.
NULL = None

Value = Union[Quantity, bool, None]


def quantity(amount, unit: Optional[str] = None, currency: Optional[str] = None) -> Quantity:
    """Build a Quantity from anything Decimal accepts."""
    if isinstance(amount, Quantity):
        return amount
    if isinstance(amount, Decimal):
        return Quantity(amount, unit, currency)
    return Quantity(Decimal(str(amount)), unit, currency)


def _is_bare_zero(q: Quantity) -> bool:
    """Zero has no dimension, so `total_lines == 0` is legal in every unit."""
    return q.amount == 0 and q.is_dimensionless()


def check_compatible(left: Quantity, right: Quantity, op: str) -> None:
    """Additive and comparison operations require identical units and currencies.

    The one exception is a dimensionless literal zero, which is compatible
    with any unit or currency. Guarding a denominator with `x == 0` is the
    single most common expression in the platform and must not need a cast.
    """
    if _is_bare_zero(left) or _is_bare_zero(right):
        return
    if left.unit != right.unit:
        raise EvalError(
            "unit_mismatch",
            f"cannot apply '{op}' to {left.unit or 'dimensionless'} "
            f"and {right.unit or 'dimensionless'}",
        )
    if left.currency != right.currency:
        raise EvalError(
            "currency_mismatch",
            f"cannot apply '{op}' to {left.currency or 'none'} "
            f"and {right.currency or 'none'}; normalise to the reserve first",
        )


def combine_multiplicative(left: Quantity, right: Quantity, op: str) -> tuple:
    """Return (unit, currency) for a product or quotient.

    One side must be dimensionless. Composite units are deliberately not
    supported: a field that needs kg*m is a modelling mistake, not a
    formula the hub should silently accept.
    """
    if left.is_dimensionless():
        return (right.unit, right.currency)
    if right.is_dimensionless():
        return (left.unit, left.currency)
    if op == "/" and left.unit == right.unit and left.currency == right.currency:
        return (None, None)
    raise EvalError(
        "unit_composition",
        f"'{op}' between {left.describe()} and {right.describe()} would create "
        "a composite unit, which is not supported",
    )
