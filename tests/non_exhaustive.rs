//! Documents the `#[non_exhaustive]` contract on `Error`, `ErrorKind`,
//! `ConflictKind`, and `CompoundKind`. A `match` over any of these
//! enums MUST include a wildcard arm — the test below relies on the
//! wildcard arm being reachable for some constructed value.

use ktav::{CompoundKind, ConflictKind, Error, ErrorKind, Span};

#[test]
fn error_match_requires_wildcard() {
    let e = Error::Message("x".to_string());
    let described = match &e {
        Error::Io(_) => "io",
        Error::Structured(_) => "structured",
        Error::Syntax(_) => "syntax",
        Error::Message(_) => "message",
        // Required wildcard — without `#[non_exhaustive]` this would be
        // an unreachable-pattern warning.
        _ => "other",
    };
    assert_eq!(described, "message");
}

#[test]
fn error_kind_match_requires_wildcard() {
    let k = ErrorKind::EmptyKey {
        line: 1,
        span: Span::EMPTY,
    };
    let described = match &k {
        ErrorKind::MissingSeparatorSpace { .. } => "mss",
        ErrorKind::InvalidTypedScalar { .. } => "its",
        ErrorKind::DuplicateKey { .. } => "dup",
        ErrorKind::KeyPathConflict { .. } => "kpc",
        ErrorKind::EmptyKey { .. } => "ek",
        ErrorKind::InvalidKey { .. } => "ik",
        ErrorKind::UnclosedCompound { .. } => "uc",
        ErrorKind::UnbalancedBracket { .. } => "ub",
        ErrorKind::InlineNonEmptyCompound { .. } => "inc",
        ErrorKind::MissingSeparator { .. } => "ms",
        ErrorKind::Other { .. } => "other",
        // Required wildcard — `#[non_exhaustive]` keeps this reachable.
        _ => "future",
    };
    assert_eq!(described, "ek");
}

#[test]
fn conflict_kind_match_requires_wildcard() {
    let c = ConflictKind::BlockedByValue;
    let described = match c {
        ConflictKind::Overwrite { .. } => "ow",
        ConflictKind::BlockedByValue => "bv",
        ConflictKind::SyntheticReopen => "sr",
        _ => "future",
    };
    assert_eq!(described, "bv");
}

#[test]
fn compound_kind_match_requires_wildcard() {
    let c = CompoundKind::Object;
    let described = match c {
        CompoundKind::Object => "o",
        CompoundKind::Array => "a",
        CompoundKind::MultilineStripped => "ms",
        CompoundKind::MultilineVerbatim => "mv",
        _ => "future",
    };
    assert_eq!(described, "o");
}
