//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Runs the shared conformance vectors against the Rust evaluator.
//!
//! The vector file is the specification, and it is the same file the
//! Python reference implementation runs. If both suites are green, the two
//! implementations agree by construction rather than by review.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use rust_decimal::Decimal;
use scor_expr::{Expression, Value};
use serde::Deserialize;

#[derive(Deserialize)]
struct Suite {
    version: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    expression: String,
    env: HashMap<String, ValueSpec>,
    expect: ValueSpec,
}

#[derive(Deserialize, Clone)]
struct ValueSpec {
    kind: String,
    amount: Option<String>,
    unit: Option<String>,
    currency: Option<String>,
    value: Option<bool>,
    code: Option<String>,
    references: Option<Vec<String>>,
}

fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../conformance/expression-cases.json")
}

fn decode(spec: &ValueSpec) -> Value {
    match spec.kind.as_str() {
        "null" => Value::Null,
        "bool" => Value::Bool(spec.value.unwrap_or(false)),
        _ => Value::Number {
            amount: spec
                .amount
                .as_deref()
                .and_then(|a| Decimal::from_str(a).ok())
                .unwrap_or_default(),
            unit: spec.unit.clone(),
            currency: spec.currency.clone(),
        },
    }
}

/// Collect every failure before asserting, so one run reports the whole gap
/// rather than the first case that happens to break.
#[test]
fn expression_conformance() {
    let mut problems: Vec<String> = Vec::new();

    let raw = match std::fs::read_to_string(vectors_path()) {
        Ok(raw) => raw,
        Err(e) => {
            problems.push(format!(
                "conformance vectors must be readable at {:?}: {e}",
                vectors_path()
            ));
            assert!(problems.is_empty(), "{}", problems.join("\n"));
            return;
        }
    };
    let suite: Suite = match serde_json::from_str(&raw) {
        Ok(suite) => suite,
        Err(e) => {
            problems.push(format!("vector file must parse: {e}"));
            assert!(problems.is_empty(), "{}", problems.join("\n"));
            return;
        }
    };
    assert!(
        !suite.version.is_empty(),
        "vector file must declare a version"
    );

    for case in &suite.cases {
        let env: HashMap<String, Value> = case
            .env
            .iter()
            .map(|(k, v)| (k.clone(), decode(v)))
            .collect();

        if case.expect.kind == "references" {
            match Expression::parse(&case.expression) {
                Ok(expr) => {
                    let actual: Vec<String> = expr.references().into_iter().collect();
                    let mut expected = case.expect.references.clone().unwrap_or_default();
                    expected.sort();
                    if actual != expected {
                        problems.push(format!(
                            "{}: references {actual:?} != {expected:?}",
                            case.id
                        ));
                    }
                }
                Err(err) => problems.push(format!("{}: parse failed: {err}", case.id)),
            }
            continue;
        }

        let outcome = Expression::parse(&case.expression).and_then(|expr| expr.eval(&env));

        match (case.expect.kind.as_str(), outcome) {
            ("error", Err(err)) => {
                let expected = case.expect.code.as_deref().unwrap_or_default();
                if err.code() != expected {
                    problems.push(format!("{}: code {} != {expected}", case.id, err.code()));
                }
            }
            ("error", Ok(value)) => {
                problems.push(format!("{}: expected an error, got {value:?}", case.id));
            }
            (_, Ok(value)) => {
                let expected = decode(&case.expect);
                if value != expected {
                    problems.push(format!("{}: {value:?} != {expected:?}", case.id));
                }
            }
            (_, Err(err)) => problems.push(format!("{}: unexpected error {err}", case.id)),
        }
    }

    assert!(
        problems.is_empty(),
        "conformance failures:\n{}",
        problems.join("\n")
    );
}
