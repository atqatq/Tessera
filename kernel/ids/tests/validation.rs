//! Tests for identifier validation.
//!
//! Written before the implementation exists (TDD, Part A1): the first run
//! must fail to compile because the API does not exist yet — that is the
//! red state. The rules under test are the contract:
//!
//! - 1..=64 characters
//! - lowercase `a-z`, digits `0-9`, and `.`, `-`, `_` only
//! - first character must be a lowercase letter or a digit
//! - a `TenantId` must not be assignable to a `ModuleId` (distinct types)

// Test code is not reachable from input; a crashing test is the signal we
// want, so the input-path lints are relaxed here (Part A4 comment rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::convert::TryFrom;

use tessera_ids::{EpochMs, ModuleId, RoleId, SubjectId, TenantId};

const VALID: &[&str] = &[
    "a",
    "0",
    "acme",
    "acme-corp",
    "inv",
    "kernel.access",
    "site_04",
    "a.b.c-d_e",
    "9lives",
];

const INVALID: &[&str] = &[
    "",          // empty
    "Acme",      // uppercase
    "acme Corp", // space
    "-lead",     // first char not a letter/digit
    ".dot",      // first char not a letter/digit
    "_under",    // first char not a letter/digit
    "tënant",    // non-ASCII
    "a\tb",      // control character
    "a/b",       // path character
    "café",      // non-ASCII
];

#[test]
fn accepts_documented_valid_shapes() {
    for s in VALID {
        assert!(
            TenantId::try_from(*s).is_ok(),
            "expected `{s}` to be a valid TenantId"
        );
        assert!(
            ModuleId::try_from(*s).is_ok(),
            "expected `{s}` to be a valid ModuleId"
        );
        assert!(
            SubjectId::try_from(*s).is_ok(),
            "expected `{s}` to be a valid SubjectId"
        );
        assert!(
            RoleId::try_from(*s).is_ok(),
            "expected `{s}` to be a valid RoleId"
        );
    }
}

#[test]
fn rejects_documented_invalid_shapes() {
    for s in INVALID {
        assert!(
            TenantId::try_from(*s).is_err(),
            "expected `{s}` to be rejected as TenantId"
        );
        assert!(
            ModuleId::try_from(*s).is_err(),
            "expected `{s}` to be rejected as ModuleId"
        );
    }
}

#[test]
fn rejects_ids_over_64_characters() {
    let ok_64 = "a".repeat(64);
    let bad_65 = "a".repeat(65);
    assert!(TenantId::try_from(ok_64.as_str()).is_ok());
    assert!(TenantId::try_from(bad_65.as_str()).is_err());
}

#[test]
fn display_round_trips_through_parse() {
    let tenant =
        TenantId::try_from("acme-corp").unwrap_or_else(|e| panic!("valid id rejected: {e}"));
    assert_eq!(tenant.as_str(), "acme-corp");
    assert_eq!(tenant.to_string(), "acme-corp");
    let reparsed: TenantId = tenant
        .to_string()
        .parse()
        .unwrap_or_else(|e| panic!("round trip failed: {e}"));
    assert_eq!(reparsed, tenant);
}

#[test]
fn distinct_kinds_are_distinct_types() {
    // A ModuleId and a TenantId with the same text are still different
    // types: this is what makes cross-assignment unrepresentable. Runtime
    // proof is limited to independence of parsing; the compile-time proof
    // lives in the library doctest (`compile_fail`).
    let tenant = TenantId::try_from("inv").unwrap_or_else(|e| panic!("valid id rejected: {e}"));
    let module = ModuleId::try_from("inv").unwrap_or_else(|e| panic!("valid id rejected: {e}"));
    assert_eq!(tenant.as_str(), module.as_str());
}

#[test]
fn epoch_ms_is_ordered_u64() {
    let earlier = EpochMs::new(1_700_000_000_000);
    let later = EpochMs::new(1_700_000_000_001);
    assert!(earlier < later);
    assert_eq!(earlier.as_u64(), 1_700_000_000_000);
}

#[test]
fn parse_from_std_string_path() {
    let owned = String::from("acme");
    let tenant = TenantId::try_from(owned).unwrap_or_else(|e| panic!("valid id rejected: {e}"));
    assert_eq!(tenant.as_str(), "acme");
    let parsed: TenantId = "acme"
        .parse()
        .unwrap_or_else(|e| panic!("valid id rejected: {e}"));
    assert_eq!(parsed, tenant);
}
