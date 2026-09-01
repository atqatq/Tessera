# Identifier grammar

One grammar for every string identifier in the kernel — tenants,
modules, subjects, roles, intent references — implemented once
(`tessera-ids::validate`) and reused everywhere.

## The rules

1. **Length:** 1 to 64 characters.
2. **Alphabet:** lowercase `a-z`, digits `0-9`, and `.`, `-`, `_`.
3. **First character:** a lowercase letter or a digit (no leading
   `.`, `-`, `_`).

The grammar is deliberately boring: safe in URLs, file paths, CSV
columns, and shell completion, and it cannot be confused with a glob —
`*` and `?` are not in the alphabet.

## The types

| Type | Identifies |
|---|---|
| `TenantId` | the isolation boundary for every store |
| `ModuleId` | a module or kernel service (`inv`, `kernel.access`) |
| `SubjectId` | any permission-checked principal: human, service, agent |
| `RoleId` | a role from the tenancy registry (carries L3 rules) |
| `EpochMs` | milliseconds since the Unix epoch (a `u64` newtype) |

These are distinct types with no conversions between them: a
`ModuleId` cannot be assigned to a `TenantId`, and that is a compile
error, not a runtime hope (the library carries the `compile_fail`
doctest that proves it).

## Errors

`InvalidId` reports the most specific reason, scanning left to right:
`Empty`, `TooLong(n)`, `InvalidChar(c)`, `InvalidFirstChar(c)`. It is
`#[non_exhaustive]`; match with a wildcard arm.
