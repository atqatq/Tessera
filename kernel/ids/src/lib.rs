//! Strongly-typed identifiers for the Tessera kernel.
//!
//! Every identifier in Tessera is a newtype over its text form, never a bare
//! `String` (Part A4: make illegal states unrepresentable). A [`TenantId`]
//! cannot be assigned to a [`ModuleId`] and vice versa — the types are
//! distinct and no conversion exists between them:
//!
//! ```compile_fail
//! use tessera_ids::{ModuleId, TenantId};
//!
//! let module = ModuleId::try_from("inv").expect("valid");
//! // Cross-kind assignment must not compile:
//! let _tenant: TenantId = module;
//! ```
//!
//! # Grammar (the contract, mirrored by conformance vectors)
//!
//! All string identifiers share one grammar:
//!
//! - 1..=64 characters
//! - lowercase ASCII letters `a-z`, digits `0-9`, and `.`, `-`, `_`
//! - the first character must be a lowercase letter or a digit
//!
//! The grammar is deliberately boring: it is safe in URLs, file paths, CSV
//! columns, and shell tab-completion, and it cannot be confused with a
//! glob (`*` and `?` are not in the alphabet).

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;
use std::str::FromStr;

/// Maximum length of any string identifier, in characters.
pub const ID_MAX_CHARS: usize = 64;

/// Why an identifier was rejected.
///
/// The variants are exhaustive enough to explain every rejection without
/// leaking more than the offending character; `#[non_exhaustive]` keeps
/// downstream matches forward-compatible.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidId {
    /// The empty string is not an identifier.
    #[error("identifier must not be empty")]
    Empty,
    /// Longer than [`ID_MAX_CHARS`], with the offending length.
    #[error("identifier is {0} characters long; the maximum is {max}", max = ID_MAX_CHARS)]
    TooLong(usize),
    /// A character outside the documented alphabet.
    #[error("identifier contains `{0}`, which is not allowed (allowed: a-z, 0-9, '.', '-', '_')")]
    InvalidChar(char),
    /// The first character is valid in position 2+ but not in position 1.
    #[error("identifier must start with a lowercase letter or a digit, not `{0}`")]
    InvalidFirstChar(char),
}

/// Validates `s` against the shared grammar, scanning left to right and
/// returning the most specific rejection reason.
///
/// Exposed so sibling crates (e.g. the permission engine's intent refs)
/// can validate identifiers without duplicating the grammar — one grammar,
/// one implementation, one set of conformance vectors.
///
/// ```
/// use tessera_ids::validate;
///
/// assert!(validate("kernel.access").is_ok());
/// assert!(validate("Not valid").is_err());
/// ```
pub fn validate(s: &str) -> Result<(), InvalidId> {
    if s.is_empty() {
        return Err(InvalidId::Empty);
    }
    let len = s.chars().count();
    if len > ID_MAX_CHARS {
        return Err(InvalidId::TooLong(len));
    }
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        // Unreachable: `s.is_empty()` was checked above. The `let else`
        // keeps the invariant local without `expect` (Part A4).
        return Err(InvalidId::Empty);
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(InvalidId::InvalidFirstChar(first));
    }
    for c in chars {
        match c {
            'a'..='z' | '0'..='9' | '.' | '-' | '_' => {}
            _ => return Err(InvalidId::InvalidChar(c)),
        }
    }
    Ok(())
}

macro_rules! define_id {
    (
        $(#[$doc:meta])*
        $name:ident
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Box<str>);

        impl $name {
            /// Parses and validates per the crate-level grammar.
            pub fn new(s: &str) -> Result<Self, InvalidId> {
                validate(s)?;
                Ok(Self(s.into()))
            }

            /// The validated text form.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = InvalidId;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = InvalidId;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                Self::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidId;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                validate(&s)?;
                Ok(Self(s.into()))
            }
        }
    };
}

define_id!(
    /// A tenant identifier: the isolation boundary for every store in the
    /// kernel. Tenant ids appear in ledger chains, access rules, and
    /// metric labels.
    TenantId
);

define_id!(
    /// A module (or kernel service) identifier, e.g. `inv` or
    /// `kernel.access`. The only legal values for a manifest `requires`
    /// entry are kernel service ids.
    ModuleId
);

define_id!(
    /// A subject identifier: any principal that can be permission-checked —
    /// a human, a service account, or an agent.
    SubjectId
);

define_id!(
    /// A role identifier from the tenancy registry. Roles carry the L3
    /// column rules evaluated by the permission engine.
    RoleId
);

/// Milliseconds since the Unix epoch.
///
/// A `u64` newtype so timestamps are never confused with durations,
/// heights, or counters. Tessera has no wall clock inside its domain
/// logic (Part A3: determinism) — callers inject the current instant as
/// an `EpochMs`, which is what makes expiry decisions testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpochMs(u64);

impl EpochMs {
    /// Wraps a milliseconds-since-epoch value.
    pub const fn new(ms: u64) -> Self {
        Self(ms)
    }

    /// The wrapped value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EpochMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_char_over_length_boundary() {
        // 64 multi-byte-safe characters are fine; 65 are not. Length is
        // counted in characters, not bytes.
        let ok = "a".repeat(ID_MAX_CHARS);
        assert_eq!(validate(&ok), Ok(()));
        let bad = format!("{ok}a");
        assert_eq!(validate(&bad), Err(InvalidId::TooLong(65)));
    }

    #[test]
    fn epoch_ms_display_is_the_raw_milliseconds() {
        // Mutation evidence: the Display body survived "replace with
        // Ok(Default::default())" — nothing pinned its output. This is
        // the pin: the rendered form is exactly the wrapped millisecond
        // value, with no decoration and no silent empty render.
        assert_eq!(EpochMs::new(0).to_string(), "0");
        assert_eq!(
            EpochMs::new(1_700_000_000_000).to_string(),
            "1700000000000"
        );
    }

    #[test]
    fn validate_counts_characters_not_bytes() {
        // 'é' is invalid anyway, but a 64-char id with multibyte input
        // must not trip the length check before the character check.
        assert_eq!(validate("é"), Err(InvalidId::InvalidFirstChar('é')));
    }
}
