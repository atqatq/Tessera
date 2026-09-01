# Expression DSL

*SCOR platform › hub master data › formulated field language*

> Determinism is the whole point. Replay any past date and you get the number people actually saw that day, not today's formula applied to old inputs.

## 📘 What it is

A small language for calculated master data fields. Spokes propose formulas, the hub validates and runs them.

It is deliberately not a general programming language. No loops, no assignment, no I/O, no clock, no randomness, no string building. Everything it can express is a pure function of its declared inputs.

## 🔤 Grammar

```text
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
```

Identifiers are dotted and namespaced: `srm.otif_pct`, `ctr.commercial_terms.penalty_exposure_usd`.

Comparison is non-associative. `1 < 2 < 3` is a syntax error, not a silently wrong answer.

## 🧩 Functions

| Function | Arity | Lazy | Notes |
|---|---|---|---|
| `if(cond, a, b)` | 3 | yes | Only the taken branch evaluates |
| `coalesce(a, ...)` | 1+ | yes | First non-null wins |
| `min(a, b)` | 2 | no | Units must match |
| `max(a, b)` | 2 | no | Units must match |
| `abs(a)` | 1 | no | Keeps unit and currency |
| `round(a[, places])` | 1–2 | no | Places defaults to 0 |

Anything else is `unknown_function`. There is no escape hatch, on purpose.

## 🚦 Semantics

### Nulls propagate

Arithmetic and comparison on null return null rather than raising. That is what makes the `null` missing-value policy work without special casing at every call site.

`and` and `or` follow three-valued logic and short-circuit, so `false and undefined_field` is `false` and never touches the missing field.

### Laziness where it matters

`if` does not evaluate the branch it does not take. This is what lets you guard a denominator:

```python
days_of_supply = "if(demand == 0, null, on_hand / demand)"
```

```excel
=IF(demand=0, "", on_hand / demand)
```

Without laziness the division would run and raise before the guard could help.

### Units and currency

Addition, subtraction and comparison require identical units and identical currencies. Multiplication and division require one side to be dimensionless, except that dividing like units gives a dimensionless ratio.

Composite units (kg × m) are rejected. A field that genuinely needs one is a modelling mistake, not a formula the hub should quietly accept.

**Zero is dimensionless.** `total_lines == 0` is legal against any unit, because zero has no dimension. This was found by a failing test during development: without the exception, guarding a denominator would have required a unit cast on every formula in the platform.

Money arrives already normalised. The hub converts to the USD reserve before evaluation, so a `currency_mismatch` at this layer means the normalisation step upstream is broken.

### Decimals, not floats

All arithmetic is decimal. `0.1 + 0.2` is exactly `0.3`. Binary floating point in a financial system produces reconciliation differences that nobody can explain a year later.

## ⚠️ Error codes

These are stable and part of the contract. Both implementations must produce the same code for the same input.

| Code | Meaning |
|---|---|
| `syntax` | Not valid in the grammar; carries a character position |
| `unit_mismatch` | Additive or comparison operands disagree on unit |
| `currency_mismatch` | Same, for currency |
| `unit_composition` | Product or quotient would create a composite unit |
| `type_error` | Boolean where a number was needed, or the reverse |
| `missing_input` | Referenced field absent from the environment |
| `unknown_function` | Not in the function table |
| `arity` | Wrong number of arguments |
| `division_by_zero` | Unguarded division |

## 🤖 Agent-authored formulas

Spoke agents propose formulas using this same language. There is no separate agent path and no relaxed mode.

That means an agent proposal fails validation for exactly the reasons a human's does: cycle detection, the fan-out budget, unit and currency safety, the reference permission check. An agent cannot propose a formula reading a field its spoke has no grant for, because the check happens at definition time against the proposing principal.

The determinism guarantee is what makes agent proposals reviewable at all. A human approving a formula can evaluate it against historical inputs and see exactly what it would have produced, every time.

## 🧪 Conformance

`conformance/expression-cases.json` is the specification. Both the Python reference implementation and the Rust crate run the same file.

Adding behaviour means adding a vector first, watching it fail, then implementing. A behaviour with no vector is not part of the language, however well it works.

```yaml
id: expression-dsl
type: language-reference
implementations: [python-reference, rust-scor-expr]
vectors: conformance/expression-cases.json
status: 🟡 draft
```
