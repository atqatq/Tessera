//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Evaluator for the hub expression DSL.
//!
//! Guarantees this module is responsible for:
//!
//! * Determinism. There is no clock, no randomness, no external read. The
//!   same AST and the same environment always produce the same value, which
//!   is what makes ledger replay meaningful.
//! * Null propagation. [`Value::Null`] flows through arithmetic instead of
//!   raising, so the `null` missing-value policy needs no special casing.
//! * Unit and currency safety. Adding kilograms to pounds, or KWD to USD,
//!   is an error rather than a silent number.
//! * Laziness where it matters. `if`, `and` and `or` do not evaluate
//!   branches they do not need, so guarded division never divides by zero.

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use super::parser::{BinOp, Node};
use super::values::{check_compatible, combine_multiplicative, EvalError, Value};

/// Strict (non-lazy) built-ins and their inclusive arity bounds. Mirrors
/// the Python reference's `STRICT_FUNCTIONS`.
const STRICT_FUNCTIONS: [(&str, (usize, usize)); 4] = [
    ("min", (2, 2)),
    ("max", (2, 2)),
    ("abs", (1, 1)),
    ("round", (1, 2)),
];

pub(crate) struct Evaluator<'e> {
    env: &'e HashMap<String, Value>,
}

impl<'e> Evaluator<'e> {
    pub(crate) fn new(env: &'e HashMap<String, Value>) -> Self {
        Self { env }
    }

    pub(crate) fn eval(&self, node: &Node) -> Result<Value, EvalError> {
        match node {
            Node::Number(amount) => Ok(Value::Number {
                amount: *amount,
                unit: None,
                currency: None,
            }),
            Node::Bool(b) => Ok(Value::Bool(*b)),
            Node::Null => Ok(Value::Null),
            Node::Ref(name) => self.env.get(name).cloned().ok_or_else(|| {
                EvalError::MissingInput(format!("field {name:?} is not in the environment"))
            }),
            Node::Negate(inner) => self.eval_negate(inner),
            Node::Not(inner) => self.eval_not(inner),
            Node::Binary { op, left, right } => self.eval_binary(*op, left, right),
            Node::Call { name, args } => self.eval_call(name, args),
        }
    }

    fn eval_negate(&self, inner: &Node) -> Result<Value, EvalError> {
        let value = self.eval(inner)?;
        match value {
            Value::Null => Ok(Value::Null),
            Value::Number {
                amount,
                unit,
                currency,
            } => Ok(Value::Number {
                amount: -amount,
                unit,
                currency,
            }),
            Value::Bool(_) => Err(EvalError::TypeError(
                "unary '-' expected a number, got a boolean".to_string(),
            )),
        }
    }

    fn eval_not(&self, inner: &Node) -> Result<Value, EvalError> {
        let value = self.eval(inner)?;
        match value {
            Value::Null => Ok(Value::Null),
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Number { .. } => Err(EvalError::TypeError(
                "'not' expected a boolean, got a number".to_string(),
            )),
        }
    }

    fn eval_binary(&self, op: BinOp, left: &Node, right: &Node) -> Result<Value, EvalError> {
        match op {
            BinOp::And => self.eval_and(left, right, false),
            BinOp::Or => self.eval_and(left, right, true),
            BinOp::Eq | BinOp::Ne => self.eval_eq_ne(op, left, right),
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => self.eval_cmp(op, left, right),
            BinOp::Add | BinOp::Sub => self.eval_add_sub(op, left, right),
            BinOp::Mul | BinOp::Div => self.eval_mul_div(op, left, right),
        }
    }

    /// Lazy `and` / `or`. `is_or` selects the dual semantics so the two
    /// share one short-circuit skeleton.
    fn eval_and(&self, left: &Node, right: &Node, is_or: bool) -> Result<Value, EvalError> {
        let short = if is_or {
            Value::Bool(true)
        } else {
            Value::Bool(false)
        };
        let l = self.eval(left)?;
        if l == short {
            return Ok(short);
        }
        let r = self.eval(right)?;
        if l == Value::Null || r == Value::Null {
            // The other operand decides only when it is the identity value:
            // `null or false` is null, `null and false` is false.
            return Ok(if r == short { short } else { Value::Null });
        }
        let context = if is_or { "'or'" } else { "'and'" };
        let (lv, rv) = (as_bool(&l, context)?, as_bool(&r, context)?);
        Ok(Value::Bool(if is_or { lv || rv } else { lv && rv }))
    }

    fn eval_eq_ne(&self, op: BinOp, left: &Node, right: &Node) -> Result<Value, EvalError> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;
        if left == Value::Null || right == Value::Null {
            return Ok(Value::Null);
        }
        let equal = equals(&left, &right)?;
        Ok(Value::Bool(if op == BinOp::Eq { equal } else { !equal }))
    }

    fn eval_cmp(&self, op: BinOp, left: &Node, right: &Node) -> Result<Value, EvalError> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;
        if left == Value::Null || right == Value::Null {
            return Ok(Value::Null);
        }
        require_quantity(&left, op.as_str())?;
        require_quantity(&right, op.as_str())?;
        check_compatible(&left, &right, op.as_str())?;
        let (la, ra) = (amount_of(&left), amount_of(&right));
        let result = match op {
            BinOp::Lt => la < ra,
            BinOp::Gt => la > ra,
            BinOp::Le => la <= ra,
            _ => la >= ra,
        };
        Ok(Value::Bool(result))
    }

    fn eval_add_sub(&self, op: BinOp, left: &Node, right: &Node) -> Result<Value, EvalError> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;
        if left == Value::Null || right == Value::Null {
            return Ok(Value::Null);
        }
        require_quantity(&left, op.as_str())?;
        require_quantity(&right, op.as_str())?;
        check_compatible(&left, &right, op.as_str())?;
        let (unit, currency) = units_of(&left);
        let amount = if op == BinOp::Add {
            amount_of(&left) + amount_of(&right)
        } else {
            amount_of(&left) - amount_of(&right)
        };
        Ok(Value::Number {
            amount,
            unit,
            currency,
        })
    }

    fn eval_mul_div(&self, op: BinOp, left: &Node, right: &Node) -> Result<Value, EvalError> {
        let left = self.eval(left)?;
        let right = self.eval(right)?;
        if left == Value::Null || right == Value::Null {
            return Ok(Value::Null);
        }
        require_quantity(&left, op.as_str())?;
        require_quantity(&right, op.as_str())?;
        let (unit, currency) = combine_multiplicative(&left, &right, op.as_str())?;
        let (la, ra) = (amount_of(&left), amount_of(&right));
        if op == BinOp::Div && ra.is_zero() {
            return Err(EvalError::DivisionByZero(
                "divided by zero; guard the denominator with if(denominator == 0, null, ...)"
                    .to_string(),
            ));
        }
        let amount = if op == BinOp::Mul { la * ra } else { la / ra };
        Ok(Value::Number {
            amount,
            unit,
            currency,
        })
    }

    fn eval_call(&self, name: &str, args: &[Node]) -> Result<Value, EvalError> {
        if name == "if" {
            let [cond, then, otherwise] = args else {
                return Err(EvalError::Arity(
                    "if() takes exactly 3 arguments".to_string(),
                ));
            };
            let condition = self.eval(cond)?;
            if condition == Value::Null {
                return Ok(Value::Null);
            }
            return if as_bool(&condition, "if()")? {
                self.eval(then)
            } else {
                self.eval(otherwise)
            };
        }
        if name == "coalesce" {
            if args.is_empty() {
                return Err(EvalError::Arity(
                    "coalesce() takes at least 1 argument".to_string(),
                ));
            }
            for arg in args {
                let value = self.eval(arg)?;
                if value != Value::Null {
                    return Ok(value);
                }
            }
            return Ok(Value::Null);
        }

        let Some((low, high)) = STRICT_FUNCTIONS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, bounds)| *bounds)
        else {
            return Err(EvalError::UnknownFunction(format!(
                "{name}() is not in the expression language"
            )));
        };
        if args.len() < low || args.len() > high {
            return Err(EvalError::Arity(format!(
                "{name}() takes {low} to {high} arguments"
            )));
        }
        let mut values: Vec<Value> = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval(arg)?);
        }
        if values.iter().any(|v| v == &Value::Null) {
            return Ok(Value::Null);
        }
        apply_strict(name, &values)
    }
}

/// All strict built-ins have been arity-checked against
/// `STRICT_FUNCTIONS` before this runs, so `args.first()` / `args.get(1)`
/// cover every index the match arms use.
fn apply_strict(name: &str, args: &[Value]) -> Result<Value, EvalError> {
    for arg in args {
        require_quantity(arg, &format!("{name}()"))?;
    }
    match name {
        "min" | "max" => {
            let (Some(a), Some(b)) = (args.first(), args.get(1)) else {
                return Err(EvalError::Arity(format!("{name}() takes 2 arguments")));
            };
            check_compatible(a, b, name)?;
            let (aa, ab) = (amount_of(a), amount_of(b));
            let a_wins = if name == "min" { aa <= ab } else { aa >= ab };
            Ok(if a_wins { a.clone() } else { b.clone() })
        }
        "abs" => {
            let Some(q) = args.first() else {
                return Err(EvalError::Arity("abs() takes 1 argument".to_string()));
            };
            let (unit, currency) = units_of(q);
            Ok(Value::Number {
                amount: abs_of(amount_of(q)),
                unit,
                currency,
            })
        }
        // "round" — the only remaining member of STRICT_FUNCTIONS.
        _ => {
            let Some(q) = args.first() else {
                return Err(EvalError::Arity(
                    "round() takes 1 to 2 arguments".to_string(),
                ));
            };
            let places = match args.get(1) {
                Some(p) => round_places(amount_of(p))?,
                None => 0,
            };
            let (unit, currency) = units_of(q);
            Ok(Value::Number {
                amount: round_half_even(amount_of(q), places)?,
                unit,
                currency,
            })
        }
    }
}

fn abs_of(amount: Decimal) -> Decimal {
    if amount.is_sign_negative() {
        -amount
    } else {
        amount
    }
}

fn units_of(value: &Value) -> (Option<String>, Option<String>) {
    match value {
        Value::Number { unit, currency, .. } => (unit.clone(), currency.clone()),
        _ => (None, None),
    }
}

fn amount_of(value: &Value) -> Decimal {
    match value {
        Value::Number { amount, .. } => *amount,
        _ => Decimal::ZERO,
    }
}

/// Python's `int(Decimal)` truncates toward zero; reject values outside
/// the rounding-precision domain instead of silently clamping.
fn round_places(amount: Decimal) -> Result<i32, EvalError> {
    amount.trunc().to_i32().ok_or_else(|| {
        EvalError::TypeError(format!(
            "round() places {} is outside the supported range",
            amount.trunc()
        ))
    })
}

/// Round half to even, matching Python's `Decimal.__round__`.
fn round_half_even(amount: Decimal, places: i32) -> Result<Decimal, EvalError> {
    if places >= 0 {
        // rust_decimal carries at most 28 fractional digits; rounding to
        // more than that is already the identity for any representable
        // value, so clamping is numerically equivalent.
        let dp = places.min(28) as u32;
        return Ok(amount.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven));
    }
    // Negative places round to tens / hundreds / ... Python expresses the
    // result with a negative scale; rescaling through an exact power of
    // ten is numerically equal.
    let factor = ten_pow((-places).min(28) as u32);
    let scaled = amount / factor;
    let rounded = scaled.round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
    Ok(rounded * factor)
}

fn ten_pow(exp: u32) -> Decimal {
    let mut result = Decimal::ONE;
    for _ in 0..exp {
        result *= Decimal::TEN;
    }
    result
}

fn require_quantity(value: &Value, context: &str) -> Result<(), EvalError> {
    if matches!(value, Value::Number { .. }) {
        Ok(())
    } else {
        Err(EvalError::TypeError(format!(
            "{context} expected a number, got a boolean"
        )))
    }
}

fn as_bool(value: &Value, context: &str) -> Result<bool, EvalError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(EvalError::TypeError(format!(
            "{context} expected a boolean, got a number"
        ))),
    }
}

fn equals(left: &Value, right: &Value) -> Result<bool, EvalError> {
    match (left, right) {
        (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
        (Value::Bool(_), _) | (_, Value::Bool(_)) => Ok(false),
        // Null cannot reach here in practice: the binary evaluator
        // propagates it before comparison. Totality is kept for safety.
        (Value::Null, Value::Null) => Ok(true),
        (Value::Null, _) | (_, Value::Null) => Ok(false),
        _ => {
            check_compatible(left, right, "==")?;
            Ok(amount_of(left) == amount_of(right))
        }
    }
}
