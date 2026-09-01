# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Pratt parser for the hub expression DSL.

Grammar (precedence low to high):

    expr     := or_expr
    or_expr  := and_expr ('or' and_expr)*
    and_expr := not_expr ('and' not_expr)*
    not_expr := 'not' not_expr | cmp
    cmp      := sum (('<'|'>'|'<='|'>='|'=='|'!=') sum)?
    sum      := product (('+'|'-') product)*
    product  := unary (('*'|'/') unary)*
    unary    := '-' unary | primary
    primary  := number | ident | call | 'true' | 'false' | 'null' | '(' expr ')'
    call     := ident '(' (expr (',' expr)*)? ')'

Comparison is non-associative on purpose: `a < b < c` is a bug in every
language that allows it, so it is a syntax error here.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import List, Set

from ..values import EvalError
from .lexer import Token, tokenize


@dataclass(frozen=True)
class Node:
    kind: str
    value: object = None
    children: tuple = field(default=())


BINARY_LEVELS = [
    {"or"},
    {"and"},
    {"<", ">", "<=", ">=", "==", "!="},
    {"+", "-"},
    {"*", "/"},
]


class Parser:
    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.i = 0

    def peek(self) -> Token:
        return self.tokens[self.i]

    def advance(self) -> Token:
        tok = self.tokens[self.i]
        self.i += 1
        return tok

    def expect(self, text: str) -> Token:
        tok = self.peek()
        if tok.text != text:
            raise EvalError(
                "syntax", f"expected {text!r} at position {tok.pos}, found {tok.text or 'end'!r}"
            )
        return self.advance()

    def parse(self) -> Node:
        node = self.parse_level(0)
        if self.peek().kind != "end":
            tok = self.peek()
            raise EvalError("syntax", f"unexpected {tok.text!r} at position {tok.pos}")
        return node

    def parse_level(self, level: int) -> Node:
        if level >= len(BINARY_LEVELS):
            return self.parse_unary()
        ops = BINARY_LEVELS[level]
        left = self.parse_level(level + 1)
        is_comparison = "<" in ops
        matched = False
        while self.peek().text in ops and self.peek().kind in ("op", "keyword"):
            if is_comparison and matched:
                tok = self.peek()
                raise EvalError(
                    "syntax",
                    f"chained comparison at position {tok.pos}; wrap it in parentheses",
                )
            op = self.advance().text
            right = self.parse_level(level + 1)
            left = Node("binary", op, (left, right))
            matched = True
        return left

    def parse_unary(self) -> Node:
        tok = self.peek()
        if tok.text == "-" and tok.kind == "op":
            self.advance()
            return Node("negate", None, (self.parse_unary(),))
        if tok.text == "not" and tok.kind == "keyword":
            self.advance()
            return Node("not", None, (self.parse_unary(),))
        return self.parse_primary()

    def parse_primary(self) -> Node:
        tok = self.advance()
        if tok.kind == "number":
            return Node("number", Decimal(tok.text))
        if tok.kind == "keyword":
            if tok.text == "true":
                return Node("bool", True)
            if tok.text == "false":
                return Node("bool", False)
            if tok.text == "null":
                return Node("null")
            raise EvalError("syntax", f"keyword {tok.text!r} cannot start an expression")
        if tok.kind == "ident":
            if self.peek().text == "(":
                self.advance()
                args = []
                if self.peek().text != ")":
                    args.append(self.parse_level(0))
                    while self.peek().text == ",":
                        self.advance()
                        args.append(self.parse_level(0))
                self.expect(")")
                return Node("call", tok.text, tuple(args))
            return Node("ref", tok.text)
        if tok.text == "(":
            inner = self.parse_level(0)
            self.expect(")")
            return inner
        raise EvalError("syntax", f"unexpected {tok.text or 'end of input'!r} at position {tok.pos}")


def parse(source: str) -> Node:
    """Parse source text into an AST."""
    if not source or not source.strip():
        raise EvalError("syntax", "expression is empty")
    return Parser(tokenize(source)).parse()


def references(node: Node) -> Set[str]:
    """Every field this expression reads. This is what feeds the dependency graph."""
    found: Set[str] = set()
    stack = [node]
    while stack:
        current = stack.pop()
        if current.kind == "ref":
            found.add(str(current.value))
        stack.extend(current.children)
    return found
