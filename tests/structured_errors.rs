//! Walks every invalid fixture in `spec/versions/0.1/tests/invalid/` and
//! asserts the parser returns `Error::Structured(kind)` (never the legacy
//! `Error::Syntax(_)` variant) and that the kind matches the expected
//! category for the fixture's filename.
//!
//! The mapping fixture-name → expected `ErrorKind` is hard-coded here
//! because the spec's `.json` oracle does not yet carry an `error`
//! category field. Once spec §7 (Error Categories) lands, this table
//! moves into the JSON.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::{CompoundKind, ConflictKind, Error, ErrorKind, Span};

const SPEC_VERSION: &str = "0.1";

fn resolve_spec_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates: Vec<PathBuf> = [
        std::env::var("KTAV_SPEC_DIR").ok().map(PathBuf::from),
        Some(manifest.join("spec")),
        Some(manifest.join("../spec")),
    ]
    .into_iter()
    .flatten()
    .collect();
    candidates.into_iter().find(|p| p.join("versions").is_dir())
}

#[derive(Debug)]
#[allow(dead_code)]
enum Expected {
    MissingSeparatorSpace,
    InvalidTypedScalar,
    DuplicateKey,
    KeyPathConflict(Option<ConflictKind>),
    EmptyKey,
    InvalidKey,
    UnclosedCompound(Option<CompoundKind>),
    UnbalancedBracket,
    InlineNonEmptyCompound,
    MissingSeparator,
    Other,
}

/// Map a fixture's stem (filename without extension) to the expected
/// structured-error category. Best-effort: `None` means we accept any
/// `Error::Structured(_)` (some fixtures don't fall cleanly into one
/// of the seven canonical categories — e.g. bracket-mismatch lines).
fn expected_for(stem: &str) -> Option<Expected> {
    use Expected::*;
    Some(match stem {
        "missing_space_after_colon"
        | "missing_space_after_raw_marker"
        | "missing_space_after_raw_marker_in_array"
        | "missing_space_after_int_marker"
        | "missing_space_after_float_marker" => MissingSeparatorSpace,

        "typed_float_empty_body"
        | "typed_float_without_decimal"
        | "typed_integer_empty_body"
        | "typed_integer_only_sign"
        | "typed_integer_with_decimal"
        | "typed_integer_with_letter"
        | "typed_opens_multiline"
        | "typed_opens_object" => InvalidTypedScalar,

        "duplicate_dotted_key" | "duplicate_in_nested" | "duplicate_top_level" => DuplicateKey,

        "dotted_over_array" | "dotted_over_scalar" | "scalar_over_dotted" => KeyPathConflict(None),

        "empty_key_leading_colon" | "empty_key_nested" => EmptyKey,

        "key_with_brace" | "key_with_bracket" | "key_with_space" => InvalidKey,

        "eof_inside_array" => UnclosedCompound(Some(CompoundKind::Array)),
        "eof_inside_object" => UnclosedCompound(Some(CompoundKind::Object)),
        "eof_inside_multiline_stripped" => UnclosedCompound(Some(CompoundKind::MultilineStripped)),
        "eof_inside_multiline_verbatim" => UnclosedCompound(Some(CompoundKind::MultilineVerbatim)),

        // Bracket mismatches → UnbalancedBracket.
        "array_closed_with_brace"
        | "object_closed_with_bracket"
        | "extra_close_brace"
        | "extra_close_bracket" => UnbalancedBracket,

        // Inline non-empty `key: { … }` / `[ … ]` → InlineNonEmptyCompound.
        "inline_nonempty_array" | "inline_nonempty_object" => InlineNonEmptyCompound,

        // Orphan / bare-word lines that lack a `:` separator.
        "orphan_line_bare_word" | "orphan_line_inside_object" => MissingSeparator,

        _ => return None,
    })
}

fn matches_expected(kind: &ErrorKind, exp: &Expected) -> bool {
    match (kind, exp) {
        (ErrorKind::MissingSeparatorSpace { .. }, Expected::MissingSeparatorSpace) => true,
        (ErrorKind::InvalidTypedScalar { .. }, Expected::InvalidTypedScalar) => true,
        (ErrorKind::DuplicateKey { .. }, Expected::DuplicateKey) => true,
        (ErrorKind::KeyPathConflict { kind: ck, .. }, Expected::KeyPathConflict(want)) => {
            match want {
                None => true,
                Some(w) => std::mem::discriminant(ck) == std::mem::discriminant(w),
            }
        }
        (ErrorKind::EmptyKey { .. }, Expected::EmptyKey) => true,
        (ErrorKind::InvalidKey { .. }, Expected::InvalidKey) => true,
        (ErrorKind::UnclosedCompound { kind: ck, .. }, Expected::UnclosedCompound(want)) => {
            match want {
                None => true,
                Some(w) => std::mem::discriminant(ck) == std::mem::discriminant(w),
            }
        }
        (ErrorKind::UnbalancedBracket { .. }, Expected::UnbalancedBracket) => true,
        (ErrorKind::InlineNonEmptyCompound { .. }, Expected::InlineNonEmptyCompound) => true,
        (ErrorKind::MissingSeparator { .. }, Expected::MissingSeparator) => true,
        (ErrorKind::Other { .. }, Expected::Other) => true,
        _ => false,
    }
}

#[test]
fn invalid_fixtures_return_structured_errors() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping structured_errors: spec dir not found");
        return;
    };
    let invalid_dir = spec_root
        .join("versions")
        .join(SPEC_VERSION)
        .join("tests")
        .join("invalid");

    let mut files: Vec<PathBuf> = fs::read_dir(&invalid_dir)
        .map(|it| {
            it.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("ktav"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();

    let mut failures: Vec<String> = Vec::new();
    let mut unmapped: Vec<String> = Vec::new();
    let mut total = 0usize;
    let mut mapped = 0usize;

    for path in &files {
        total += 1;
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let text = fs::read_to_string(path).unwrap_or_default();

        let err = match ktav::parse(&text) {
            Ok(_) => {
                failures.push(format!("{stem}: parsed OK; expected error"));
                continue;
            }
            Err(e) => e,
        };

        let kind = match &err {
            Error::Structured(k) => k,
            Error::Syntax(m) => {
                failures.push(format!("{stem}: legacy Error::Syntax(_): {m}"));
                continue;
            }
            other => {
                failures.push(format!("{stem}: unexpected error variant: {other:?}"));
                continue;
            }
        };

        match expected_for(stem) {
            Some(exp) => {
                mapped += 1;
                if !matches_expected(kind, &exp) {
                    failures.push(format!(
                        "{stem}: kind mismatch — expected {exp:?}, got {kind:?}"
                    ));
                }
            }
            None => {
                unmapped.push(stem.to_string());
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{}/{} fixture(s) failed structured-error check:\n{}",
            failures.len(),
            total,
            failures.join("\n")
        );
    }
    eprintln!(
        "structured_errors: {mapped}/{total} fixtures mapped, {} unmapped",
        unmapped.len()
    );
    if !unmapped.is_empty() {
        eprintln!("unmapped fixtures (still asserted Structured(_)): {unmapped:?}");
    }
}

// ---------------------------------------------------------------------------
// Trigger tests for the three categories promoted out of `Other` in 0.1.6.
// Each asserts (line, span byte-range) explicitly.
// ---------------------------------------------------------------------------

#[test]
fn unbalanced_bracket_stray_close() {
    let src = "}\n";
    let err = ktav::parse(src).unwrap_err();
    let kind = match err {
        Error::Structured(k) => k,
        other => panic!("expected Structured, got {other:?}"),
    };
    match kind {
        ErrorKind::UnbalancedBracket {
            line,
            span,
            found,
            expected: CompoundKind::Object,
        } => {
            assert_eq!(line, 1);
            assert_eq!(span, Span::new(0, 1));
            assert_eq!(found, '}');
        }
        other => panic!("expected UnbalancedBracket, got {other:?}"),
    }
}

#[test]
fn unbalanced_bracket_mismatched_closer() {
    let src = "obj: {\n]\n";
    let err = ktav::parse(src).unwrap_err();
    match err {
        Error::Structured(ErrorKind::UnbalancedBracket {
            line, span, found, ..
        }) => {
            assert_eq!(line, 2);
            assert_eq!(found, ']');
            // Line 2 starts at byte 7; the trimmed `]` is byte 7..8.
            assert_eq!(span, Span::new(7, 8));
        }
        other => panic!("expected UnbalancedBracket, got {other:?}"),
    }
}

#[test]
fn inline_nonempty_compound_object_trigger() {
    let src = "server: { host: 127.0.0.1 }\n";
    let err = ktav::parse(src).unwrap_err();
    match err {
        Error::Structured(ErrorKind::InlineNonEmptyCompound { line, span, body }) => {
            assert_eq!(line, 1);
            assert_eq!(body, "object");
            assert_eq!(span, Span::new(0, 27));
        }
        other => panic!("expected InlineNonEmptyCompound, got {other:?}"),
    }
}

#[test]
fn inline_nonempty_compound_array_trigger() {
    let src = "items: [ a b c ]\n";
    let err = ktav::parse(src).unwrap_err();
    match err {
        Error::Structured(ErrorKind::InlineNonEmptyCompound { line, span, body }) => {
            assert_eq!(line, 1);
            assert_eq!(body, "array");
            assert_eq!(span, Span::new(0, 16));
        }
        other => panic!("expected InlineNonEmptyCompound, got {other:?}"),
    }
}

#[test]
fn missing_separator_trigger() {
    let src = "just-some-text\n";
    let err = ktav::parse(src).unwrap_err();
    match err {
        Error::Structured(ErrorKind::MissingSeparator { line, span }) => {
            assert_eq!(line, 1);
            assert_eq!(span, Span::new(0, 14));
        }
        other => panic!("expected MissingSeparator, got {other:?}"),
    }
}
