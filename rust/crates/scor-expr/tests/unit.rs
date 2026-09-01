//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Unit tests for behaviours the conformance vectors pin only indirectly:
//! lazy null semantics, rounding, and the strict built-ins' arity table.
//! These mirror key cases from the Python reference's `test_expression.py`.

use std::collections::HashMap;
use std::str::FromStr;

use rust_decimal::Decimal;
use scor_expr::{EvalError, Expression, Value};

fn num(amount: &str) -> Value {
    Value::Number {
        amount: Decimal::from_str(amount).unwrap_or_default(),
        unit: None,
        currency: None,
    }
}

fn num_unit(amount: &str, unit: &str) -> Value {
    Value::Number {
        amount: Decimal::from_str(amount).unwrap_or_default(),
        unit: Some(unit.to_string()),
        currency: None,
    }
}

fn eval(source: &str, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
    Expression::parse(source).and_then(|e| e.eval(env))
}

fn env_of(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect()
}

#[test]
fn null_and_false_is_false() {
    let env = env_of(&[]);
    assert_eq!(eval("null and false", &env), Ok(Value::Bool(false)));
}

#[test]
fn null_or_true_is_true() {
    let env = env_of(&[]);
    assert_eq!(eval("null or true", &env), Ok(Value::Bool(true)));
}

#[test]
fn null_and_true_is_null() {
    let env = env_of(&[]);
    assert_eq!(eval("null and true", &env), Ok(Value::Null));
}

#[test]
fn null_or_false_is_null() {
    let env = env_of(&[]);
    assert_eq!(eval("null or false", &env), Ok(Value::Null));
}

#[test]
fn lazy_branch_does_not_raise() {
    let env = env_of(&[]);
    // The untaken branch would be a missing-input error.
    assert_eq!(eval("true or missing + 1", &env), Ok(Value::Bool(true)));
    assert_eq!(eval("false and missing + 1", &env), Ok(Value::Bool(false)));
}

#[test]
fn round_is_half_even() {
    let env = env_of(&[]);
    assert_eq!(eval("round(0.5)", &env), Ok(num("0")));
    assert_eq!(eval("round(1.5)", &env), Ok(num("2")));
    assert_eq!(eval("round(2.5)", &env), Ok(num("2")));
    assert_eq!(eval("round(2.675, 2)", &env), Ok(num("2.68")));
}

#[test]
fn round_negative_places() {
    let env = env_of(&[]);
    assert_eq!(eval("round(1234, -2)", &env), Ok(num("1200")));
}

#[test]
fn arithmetic_is_exact_decimal() {
    let env = env_of(&[]);
    assert_eq!(eval("0.1 + 0.2 == 0.3", &env), Ok(Value::Bool(true)));
}

#[test]
fn equality_ignores_scale() {
    let env = env_of(&[]);
    assert_eq!(eval("920 / 1000 * 100 == 92", &env), Ok(Value::Bool(true)));
}

#[test]
fn negation_keeps_null() {
    let env = env_of(&[("a", Value::Null)]);
    assert_eq!(eval("-a", &env), Ok(Value::Null));
    assert_eq!(eval("-5", &env), Ok(num("-5")));
}

#[test]
fn parenthesised_comparison_is_fine() {
    let env = env_of(&[]);
    assert_eq!(eval("(1 < 2) == true", &env), Ok(Value::Bool(true)));
}

#[test]
fn strict_arity_is_enforced() {
    let env = env_of(&[]);
    assert_eq!(eval("min(1)", &env).map_err(|e| e.code()), Err("arity"));
    assert_eq!(
        eval("if(true, 1)", &env).map_err(|e| e.code()),
        Err("arity")
    );
    assert_eq!(eval("coalesce()", &env).map_err(|e| e.code()), Err("arity"));
}

#[test]
fn min_max_keep_units() {
    let env = env_of(&[("a", num_unit("2", "kg")), ("b", num_unit("3", "kg"))]);
    assert_eq!(eval("min(a, b)", &env), Ok(num_unit("2", "kg")));
    assert_eq!(eval("max(a, b)", &env), Ok(num_unit("3", "kg")));
}

#[test]
fn absolute_value_keeps_sign_negative_units() {
    let env = env_of(&[("a", num("-7"))]);
    assert_eq!(eval("abs(a)", &env), Ok(num("7")));
}

#[test]
fn chained_comparison_inside_parens_is_allowed() {
    let env = env_of(&[]);
    // One comparison per parenthesised group is a different expression
    // from a chained comparison.
    assert_eq!(eval("(1 < 2) == (2 < 3)", &env), Ok(Value::Bool(true)));
}

#[test]
fn division_requires_a_guard() {
    let env = env_of(&[("d", num("0")), ("n", num("10"))]);
    assert_eq!(
        eval("n / d", &env).map_err(|e| e.code()),
        Err("division_by_zero")
    );
}

#[test]
fn empty_expression_is_rejected() {
    let env = env_of(&[]);
    assert_eq!(eval("   ", &env).map_err(|e| e.code()), Err("syntax"));
}
