//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Expression DSL for hub-managed formulated fields.
//!
//! # Contract
//!
//! This crate is held to `conformance/expression-cases.json`, the same
//! vector file the Python reference implementation runs. Do not add
//! behaviour here that no vector covers; add the vector first, watch it
//! fail, then implement.
//!
//! # Guarantees
//!
//! * Deterministic. No clock, no randomness, no I/O. Ledger replay depends
//!   on this and nothing else in the platform can restore it.
//! * Decimal arithmetic. Never binary floating point.
//! * Null propagates through arithmetic and comparison.
//! * `if`, `and` and `or` are lazy, so a guarded division never divides.
//! * Units and currencies must match for additive and comparison
//!   operations, except for a dimensionless literal zero.

#![forbid(unsafe_code)]

pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod values;

use std::collections::{BTreeSet, HashMap};

pub use values::{EvalError, Value};

use crate::parser::Node;

/// A parsed expression, ready to evaluate many times.
///
/// Parsing is done once at field-definition time; evaluation runs on every
/// dependency change, so the split matters for the throughput target.
#[derive(Debug, Clone)]
pub struct Expression {
    root: Node,
}

impl Expression {
    /// Parse source text into an evaluable expression.
    ///
    /// # Errors
    /// [`EvalError::Syntax`], carrying a character position.
    pub fn parse(source: &str) -> Result<Self, EvalError> {
        Ok(Self {
            root: parser::parse(source)?,
        })
    }

    /// Every field this expression reads. Feeds the dependency graph and
    /// the permission check, so it must be exhaustive.
    #[must_use]
    pub fn references(&self) -> BTreeSet<String> {
        parser::references(&self.root)
    }

    /// Evaluate against an environment of resolved field values.
    ///
    /// # Errors
    /// The matching [`EvalError`] variant; see the vectors for which code
    /// each situation must produce.
    pub fn eval(&self, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
        evaluator::Evaluator::new(env).eval(&self.root)
    }
}
