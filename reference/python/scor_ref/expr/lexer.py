# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Tokeniser for the hub expression DSL.

The grammar is deliberately small. No loops, no assignment, no I/O, no
string concatenation. If a token is not listed here it is not in the
language, and the parser will say so with a position.

Numbers and identifiers are ASCII-only. This is not cosmetic: Unicode
digits pass str.isdigit() but Decimal() rejects some of them, which used
to turn a malformed expression into an uncaught InvalidOperation instead
of a syntax error. The Rust implementation enforces the same rule, and
the conformance vectors pin it down for both.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import List

from ..values import EvalError

KEYWORDS = {"and", "or", "not", "true", "false", "null"}

TWO_CHAR_OPS = {"<=", ">=", "==", "!="}
ONE_CHAR_OPS = set("+-*/<>(),")


@dataclass(frozen=True)
class Token:
    kind: str  # number | ident | keyword | op | end
    text: str
    pos: int


def tokenize(source: str) -> List[Token]:
    tokens: List[Token] = []
    i = 0
    n = len(source)
    while i < n:
        ch = source[i]
        if ch.isspace():
            i += 1
            continue
        if ch in "0123456789" or (ch == "." and i + 1 < n and source[i + 1] in "0123456789"):
            start = i
            seen_dot = False
            while i < n and (source[i] in "0123456789" or (source[i] == "." and not seen_dot)):
                if source[i] == ".":
                    seen_dot = True
                i += 1
            tokens.append(Token("number", source[start:i], start))
            continue
        if ch.isascii() and ch.isalpha() or ch == "_":
            start = i
            while i < n and ((source[i].isascii() and source[i].isalnum()) or source[i] in "_."):
                i += 1
            text = source[start:i]
            if text.endswith("."):
                raise EvalError("syntax", f"identifier ends with '.' at position {start}")
            kind = "keyword" if text in KEYWORDS else "ident"
            tokens.append(Token(kind, text, start))
            continue
        if source[i : i + 2] in TWO_CHAR_OPS:
            tokens.append(Token("op", source[i : i + 2], i))
            i += 2
            continue
        if ch in ONE_CHAR_OPS:
            tokens.append(Token("op", ch, i))
            i += 1
            continue
        raise EvalError("syntax", f"unexpected character {ch!r} at position {i}")
    tokens.append(Token("end", "", n))
    return tokens
