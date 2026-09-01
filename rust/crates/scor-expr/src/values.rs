//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Value model for the expression DSL.
//!
//! Every number carries an optional unit and an optional currency. The hub
//! normalises money to the USD reserve before evaluation, so a currency
//! mismatch here is a bug upstream, not a conversion opportunity.

use rust_decimal::Decimal;

/// A value produced by the evaluator.
///
/// This is the same model as the Python reference: a [`Value::Number`] is a
/// decimal amount with optional unit and currency, [`Value::Bool`] is a
/// plain boolean, and [`Value::Null`] is a first-class value that
/// propagates through arithmetic rather than raising.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A number with an optional unit and optional currency.
    Number {
        /// Decimal amount.
        amount: Decimal,
        /// Unit of measure, if dimensioned.
        unit: Option<String>,
        /// ISO 4217 code, if monetary.
        currency: Option<String>,
    },
    /// A boolean.
    Bool(bool),
    /// The absence of a value.
    Null,
}

impl Value {
    /// Dimensionless zero is compatible with any unit or currency, so
    /// `total_lines == 0` is legal in every unit.
    #[must_use]
    pub fn is_bare_zero(&self) -> bool {
        matches!(
            self,
            Value::Number {
                amount,
                unit: None,
                currency: None
            } if amount.is_zero()
        )
    }

    /// Human-readable description used in error messages.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Value::Number {
                amount,
                unit,
                currency,
            } => {
                let mut parts = vec![amount.to_string()];
                if let Some(u) = unit {
                    parts.push(u.clone());
                }
                if let Some(c) = currency {
                    parts.push(c.clone());
                }
                parts.join(" ")
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

/// Every error the evaluator can produce.
///
/// The machine code returned by [`EvalError::code`] must match the `code`
/// field in the conformance vectors exactly.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EvalError {
    /// The source text is not valid in the grammar.
    #[error("syntax: {0}")]
    Syntax(String),
    /// Additive or comparison operands disagree on unit.
    #[error("unit_mismatch: {0}")]
    UnitMismatch(String),
    /// Additive or comparison operands disagree on currency.
    #[error("currency_mismatch: {0}")]
    CurrencyMismatch(String),
    /// A product or quotient would create a composite unit.
    #[error("unit_composition: {0}")]
    UnitComposition(String),
    /// An operand had the wrong type.
    #[error("type_error: {0}")]
    TypeError(String),
    /// A referenced field is absent from the environment.
    #[error("missing_input: {0}")]
    MissingInput(String),
    /// The function is not in the language.
    #[error("unknown_function: {0}")]
    UnknownFunction(String),
    /// Wrong number of arguments.
    #[error("arity: {0}")]
    Arity(String),
    /// Division by zero without a guard.
    #[error("division_by_zero: {0}")]
    DivisionByZero(String),
}

impl EvalError {
    /// Stable machine code, matched against the conformance vectors.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Syntax(_) => "syntax",
            Self::UnitMismatch(_) => "unit_mismatch",
            Self::CurrencyMismatch(_) => "currency_mismatch",
            Self::UnitComposition(_) => "unit_composition",
            Self::TypeError(_) => "type_error",
            Self::MissingInput(_) => "missing_input",
            Self::UnknownFunction(_) => "unknown_function",
            Self::Arity(_) => "arity",
            Self::DivisionByZero(_) => "division_by_zero",
        }
    }
}

/// Additive and comparison operations require identical units and
/// currencies.
///
/// The one exception is a dimensionless literal zero, which is compatible
/// with any unit or currency. Guarding a denominator with `x == 0` is the
/// single most common expression in the platform and must not need a cast.
///
/// # Errors
/// [`EvalError::UnitMismatch`] or [`EvalError::CurrencyMismatch`].
pub fn check_compatible(left: &Value, right: &Value, op: &str) -> Result<(), EvalError> {
    if left.is_bare_zero() || right.is_bare_zero() {
        return Ok(());
    }
    let (lu, lc, ru, rc) = match (left, right) {
        (
            Value::Number {
                unit: lu,
                currency: lc,
                ..
            },
            Value::Number {
                unit: ru,
                currency: rc,
                ..
            },
        ) => (lu, lc, ru, rc),
        _ => return Ok(()),
    };
    if lu != ru {
        return Err(EvalError::UnitMismatch(format!(
            "cannot apply '{op}' to {} and {}",
            lu.as_deref().unwrap_or("dimensionless"),
            ru.as_deref().unwrap_or("dimensionless"),
        )));
    }
    if lc != rc {
        return Err(EvalError::CurrencyMismatch(format!(
            "cannot apply '{op}' to {} and {}; normalise to the reserve first",
            lc.as_deref().unwrap_or("none"),
            rc.as_deref().unwrap_or("none"),
        )));
    }
    Ok(())
}

/// Unit and currency for a product or quotient.
///
/// One side must be dimensionless. Composite units are deliberately not
/// supported: a field that needs kg*m is a modelling mistake, not a formula
/// the hub should silently accept.
///
/// # Errors
/// [`EvalError::UnitComposition`] when neither side is dimensionless and
/// the units do not cancel.
pub fn combine_multiplicative(
    left: &Value,
    right: &Value,
    op: &str,
) -> Result<(Option<String>, Option<String>), EvalError> {
    let (lu, lc, ru, rc) = match (left, right) {
        (
            Value::Number {
                unit: lu,
                currency: lc,
                ..
            },
            Value::Number {
                unit: ru,
                currency: rc,
                ..
            },
        ) => (lu, lc, ru, rc),
        _ => return Ok((None, None)),
    };
    if lu.is_none() && lc.is_none() {
        return Ok((ru.clone(), rc.clone()));
    }
    if ru.is_none() && rc.is_none() {
        return Ok((lu.clone(), lc.clone()));
    }
    if op == "/" && lu == ru && lc == rc {
        return Ok((None, None));
    }
    Err(EvalError::UnitComposition(format!(
        "'{op}' between {} and {} would create a composite unit, which is not supported",
        left.describe(),
        right.describe(),
    )))
}
