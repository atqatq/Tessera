//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Pratt parser for the hub expression DSL.
//!
//! Grammar (precedence low to high):
//!
//! ```text
//! expr     := or_expr
//! or_expr  := and_expr ('or' and_expr)*
//! and_expr := not_expr ('and' not_expr)*
//! not_expr := 'not' not_expr | cmp
//! cmp      := sum (('<'|'>'|'<='|'>='|'=='|'!=') sum)?
//! sum      := product (('+'|'-') product)*
//! product  := unary (('*'|'/') unary)*
//! unary    := '-' unary | primary
//! primary  := number | ident | call | 'true' | 'false' | 'null' | '(' expr ')'
//! call     := ident '(' (expr (',' expr)*)? ')'
//! ```
//!
//! Comparison is non-associative on purpose: `a < b < c` is a bug in every
//! language that allows it, so it is a syntax error here.

use std::collections::BTreeSet;
use std::str::FromStr;

use rust_decimal::Decimal;

use super::lexer::{tokenize, Token, TokenKind};
use super::values::EvalError;

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Logical or (lazy).
    Or,
    /// Logical and (lazy).
    And,
    /// Less than.
    Lt,
    /// Greater than.
    Gt,
    /// Less than or equal.
    Le,
    /// Greater than or equal.
    Ge,
    /// Equality.
    Eq,
    /// Inequality.
    Ne,
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
}

impl BinOp {
    fn from_text(text: &str) -> Option<Self> {
        match text {
            "or" => Some(Self::Or),
            "and" => Some(Self::And),
            "<" => Some(Self::Lt),
            ">" => Some(Self::Gt),
            "<=" => Some(Self::Le),
            ">=" => Some(Self::Ge),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Ne),
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            _ => None,
        }
    }

    /// The source spelling, used in error messages.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Or => "or",
            Self::And => "and",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
        }
    }
}

/// An AST node. Parsing happens once at field-definition time; evaluation
/// runs on every dependency change, so the tree is cheap to clone and
/// holds no allocations beyond names and literals.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A decimal literal.
    Number(Decimal),
    /// `true` / `false`.
    Bool(bool),
    /// The absence of a value.
    Null,
    /// A field reference, feeding the dependency graph.
    Ref(String),
    /// Unary `-`.
    Negate(Box<Node>),
    /// Unary `not`.
    Not(Box<Node>),
    /// A binary operation.
    Binary {
        /// The operator.
        op: BinOp,
        /// Left operand.
        left: Box<Node>,
        /// Right operand.
        right: Box<Node>,
    },
    /// A function call.
    Call {
        /// Function name.
        name: String,
        /// Arguments.
        args: Vec<Node>,
    },
}

/// Operator sets per precedence level, low to high. Mirrors the Python
/// reference's `BINARY_LEVELS`.
const BINARY_LEVELS: [&[&str]; 5] = [
    &["or"],
    &["and"],
    &["<", ">", "<=", ">=", "==", "!="],
    &["+", "-"],
    &["*", "/"],
];

struct Parser {
    tokens: Vec<Token>,
    end: Token,
    i: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let end = tokens.last().cloned().unwrap_or_else(|| Token::end(0));
        Self { tokens, end, i: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.i).unwrap_or(&self.end)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.i < self.tokens.len() {
            self.i += 1;
        }
        tok
    }

    fn expect(&mut self, text: &str) -> Result<Token, EvalError> {
        let tok = self.peek();
        if tok.text != text {
            return Err(EvalError::Syntax(format!(
                "expected {text:?} at position {}, found {:?}",
                tok.pos,
                if tok.text.is_empty() {
                    "end"
                } else {
                    &tok.text
                }
            )));
        }
        Ok(self.advance())
    }

    fn parse(mut self) -> Result<Node, EvalError> {
        let node = self.parse_level(0)?;
        if self.peek().kind != TokenKind::End {
            let tok = self.peek();
            return Err(EvalError::Syntax(format!(
                "unexpected {:?} at position {}",
                tok.text, tok.pos
            )));
        }
        Ok(node)
    }

    fn parse_level(&mut self, level: usize) -> Result<Node, EvalError> {
        let Some(ops) = BINARY_LEVELS.get(level) else {
            return self.parse_unary();
        };
        let mut left = self.parse_level(level + 1)?;
        let mut matched = false;
        loop {
            let tok = self.peek();
            let is_op = (tok.kind == TokenKind::Op || tok.kind == TokenKind::Keyword)
                && ops.contains(&tok.text.as_str());
            if !is_op {
                break;
            }
            if ops.iter().any(|o| o.contains('<') || o.contains('>')) && matched {
                let tok = self.peek();
                return Err(EvalError::Syntax(format!(
                    "chained comparison at position {}; wrap it in parentheses",
                    tok.pos
                )));
            }
            let op_tok = self.advance();
            let op = BinOp::from_text(&op_tok.text).ok_or_else(|| {
                EvalError::Syntax(format!("unknown operator at position {}", op_tok.pos))
            })?;
            let right = self.parse_level(level + 1)?;
            left = Node::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
            matched = true;
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Node, EvalError> {
        let tok = self.peek().clone();
        if tok.text == "-" && tok.kind == TokenKind::Op {
            self.advance();
            let inner = self.parse_unary()?;
            return Ok(Node::Negate(Box::new(inner)));
        }
        if tok.text == "not" && tok.kind == TokenKind::Keyword {
            self.advance();
            let inner = self.parse_unary()?;
            return Ok(Node::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Node, EvalError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number => {
                let amount = Decimal::from_str(&tok.text).map_err(|e| {
                    EvalError::Syntax(format!("invalid number at position {}: {e}", tok.pos))
                })?;
                Ok(Node::Number(amount))
            }
            TokenKind::Keyword => match tok.text.as_str() {
                "true" => Ok(Node::Bool(true)),
                "false" => Ok(Node::Bool(false)),
                "null" => Ok(Node::Null),
                other => Err(EvalError::Syntax(format!(
                    "keyword {other:?} cannot start an expression"
                ))),
            },
            TokenKind::Ident => {
                if self.peek().text == "(" {
                    self.advance();
                    let mut args: Vec<Node> = Vec::new();
                    if self.peek().text != ")" {
                        args.push(self.parse_level(0)?);
                        while self.peek().text == "," {
                            self.advance();
                            args.push(self.parse_level(0)?);
                        }
                    }
                    self.expect(")")?;
                    return Ok(Node::Call {
                        name: tok.text,
                        args,
                    });
                }
                Ok(Node::Ref(tok.text))
            }
            _ => {
                if tok.text == "(" {
                    let inner = self.parse_level(0)?;
                    self.expect(")")?;
                    return Ok(inner);
                }
                Err(EvalError::Syntax(format!(
                    "unexpected {:?} at position {}",
                    if tok.text.is_empty() {
                        "end of input"
                    } else {
                        tok.text.as_str()
                    },
                    tok.pos
                )))
            }
        }
    }
}

/// Parse source text into an AST.
///
/// # Errors
/// [`EvalError::Syntax`] on empty input or any grammar violation.
pub fn parse(source: &str) -> Result<Node, EvalError> {
    if source.trim().is_empty() {
        return Err(EvalError::Syntax("expression is empty".to_string()));
    }
    Parser::new(tokenize(source)?).parse()
}

/// Every field this expression reads. This is what feeds the dependency
/// graph and the permission check, so it must be exhaustive.
#[must_use]
pub fn references(node: &Node) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut stack: Vec<&Node> = vec![node];
    while let Some(current) = stack.pop() {
        match current {
            Node::Ref(name) => {
                found.insert(name.clone());
            }
            Node::Negate(inner) | Node::Not(inner) => stack.push(inner),
            Node::Binary { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            Node::Call { args, .. } => stack.extend(args.iter()),
            Node::Number(_) | Node::Bool(_) | Node::Null => {}
        }
    }
    found
}
