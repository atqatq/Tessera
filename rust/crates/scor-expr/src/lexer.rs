//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Tokeniser for the hub expression DSL.
//!
//! The grammar is deliberately small. No loops, no assignment, no I/O, no
//! string concatenation. If a token is not listed here it is not in the
//! language, and the parser will say so with a position.
//!
//! Numbers and identifiers are ASCII-only. This is not cosmetic: the Python
//! reference used to accept Unicode digits via `str.isdigit()` and then
//! crash inside `Decimal()` on some of them. Both implementations now
//! enforce ASCII, and the conformance vectors pin the rule down for both.

use std::fmt;

use super::values::EvalError;

/// Keywords reserved by the grammar.
pub const KEYWORDS: [&str; 6] = ["and", "or", "not", "true", "false", "null"];

/// Two-character operators.
pub const TWO_CHAR_OPS: [&str; 4] = ["<=", ">=", "==", "!="];

/// One-character operators.
pub const ONE_CHAR_OPS: [char; 9] = ['+', '-', '*', '/', '<', '>', '(', ')', ','];

/// The kind of a [`Token`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A decimal literal.
    Number,
    /// A field reference or function name.
    Ident,
    /// A reserved keyword (`and`, `or`, `not`, `true`, `false`, `null`).
    Keyword,
    /// An operator or punctuation.
    Op,
    /// End of input.
    End,
}

/// A lexical token with its source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Token kind.
    pub kind: TokenKind,
    /// Raw source text (empty for `End`).
    pub text: String,
    /// Byte offset of the token start in the source.
    pub pos: usize,
}

impl Token {
    /// The shared end-of-input token shape.
    #[must_use]
    pub fn end(pos: usize) -> Self {
        Self {
            kind: TokenKind::End,
            text: String::new(),
            pos,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Tokenise `source` into a token list terminated by [`TokenKind::End`].
///
/// # Errors
/// [`EvalError::Syntax`] on any character outside the grammar, or an
/// identifier that ends with `.`.
#[allow(
    clippy::indexing_slicing,
    reason = "the scan loop guards every access with i<n, the only lookahead uses chars.get(i+1), and text_of receives ranges the loop produced, all within bounds"
)]
pub fn tokenize(source: &str) -> Result<Vec<Token>, EvalError> {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    fn text_of(chars: &[char], from: usize, to: usize) -> String {
        chars[from..to].iter().collect()
    }

    while i < n {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        let starts_number = ch.is_ascii_digit()
            || (ch == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit));
        if starts_number {
            let start = i;
            let mut seen_dot = false;
            while i < n && (chars[i].is_ascii_digit() || (chars[i] == '.' && !seen_dot)) {
                if chars[i] == '.' {
                    seen_dot = true;
                }
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: text_of(&chars, start, i),
                pos: start,
            });
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
            let text = text_of(&chars, start, i);
            if text.ends_with('.') {
                return Err(EvalError::Syntax(format!(
                    "identifier ends with '.' at position {start}"
                )));
            }
            let kind = if KEYWORDS.contains(&text.as_str()) {
                TokenKind::Keyword
            } else {
                TokenKind::Ident
            };
            tokens.push(Token {
                kind,
                text,
                pos: start,
            });
            continue;
        }
        if i + 1 < n {
            let two = text_of(&chars, i, i + 2);
            if TWO_CHAR_OPS.contains(&two.as_str()) {
                tokens.push(Token {
                    kind: TokenKind::Op,
                    text: two,
                    pos: i,
                });
                i += 2;
                continue;
            }
        }
        if ONE_CHAR_OPS.contains(&ch) {
            tokens.push(Token {
                kind: TokenKind::Op,
                text: ch.to_string(),
                pos: i,
            });
            i += 1;
            continue;
        }
        return Err(EvalError::Syntax(format!(
            "unexpected character {ch:?} at position {i}"
        )));
    }
    tokens.push(Token::end(n));
    Ok(tokens)
}
