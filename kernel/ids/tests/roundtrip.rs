//! Proptests for identifier invariants (Part A2): every invariant that must
//! hold across all inputs gets a property, not just curated examples.

// Test code is not reachable from input; a crashing test is the signal we
// want, so the input-path lints are relaxed here (Part A4 comment rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use tessera_ids::{ModuleId, TenantId};

/// Strategy for strings that satisfy the documented id grammar.
fn valid_id_strategy() -> impl Strategy<Value = String> {
    // first char: a-z or 0-9; then 0..=63 of the full alphabet
    prop::string::string_regex("[a-z0-9][a-z0-9._-]{0,63}").expect("canned valid regex strategy")
}

/// Strategy that takes a valid id and corrupts it in a documented way.
fn corrupted_strategy() -> impl Strategy<Value = (String, Corrupt)> {
    let corrupt = prop_oneof![
        Just(Corrupt::Uppercase),
        Just(Corrupt::LeadingPunct),
        Just(Corrupt::BadChar),
        Just(Corrupt::TooLong),
        Just(Corrupt::Empty),
    ];
    (valid_id_strategy(), corrupt)
}

#[derive(Debug, Clone, Copy)]
enum Corrupt {
    Uppercase,
    LeadingPunct,
    BadChar,
    TooLong,
    Empty,
}

/// Corrupt per the variant; the result must be rejected by every id type.
fn apply(s: &str, c: Corrupt) -> String {
    match c {
        Corrupt::Uppercase => {
            // flip the first letter to uppercase; if it starts with a digit,
            // append an uppercase letter instead
            let mut chars: Vec<char> = s.chars().collect();
            if chars[0].is_ascii_lowercase() {
                chars[0] = chars[0].to_ascii_uppercase();
            } else {
                chars.push('Z');
            }
            chars.into_iter().collect()
        }
        Corrupt::LeadingPunct => format!("-{s}"),
        Corrupt::BadChar => format!("{s}é"),
        Corrupt::TooLong => {
            let mut out = s.to_string();
            while out.chars().count() <= 64 {
                out.push('a');
            }
            out
        }
        Corrupt::Empty => String::new(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Round trip: parse -> display -> parse yields the same identifier.
    #[test]
    fn round_trip_through_display(s in valid_id_strategy()) {
        let tenant = TenantId::try_from(s.as_str())?;
        let again: TenantId = tenant.to_string().parse()?;
        prop_assert_eq!(again, tenant);

        let module = ModuleId::try_from(s.as_str())?;
        let again: ModuleId = module.to_string().parse()?;
        prop_assert_eq!(again, module);
    }

    /// Every documented corruption is rejected — always, for every id kind.
    #[test]
    fn corrupted_ids_are_rejected((s, c) in corrupted_strategy()) {
        let bad = apply(&s, c);
        prop_assert!(TenantId::try_from(bad.as_str()).is_err(), "TenantId accepted `{bad}`");
        prop_assert!(ModuleId::try_from(bad.as_str()).is_err(), "ModuleId accepted `{bad}`");
    }
}
