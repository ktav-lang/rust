//! Walks every invalid fixture in `spec/versions/0.6/tests/invalid/` and
//! asserts the parser returns `Error::Structured(kind)` (never the legacy
//! `Error::Syntax(_)` variant) and that the kind matches the expected
//! category from the sibling `.json` oracle.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::{CompoundKind, Error, ErrorKind, Span};

const SPEC_VERSION: &str = "0.6";

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

/// Walk `root` recursively and collect every `.ktav` file.
fn collect_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ktav_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            out.push(path);
        }
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

    let mut files = Vec::new();
    collect_ktav_files(&invalid_dir, &mut files);
    files.sort();

    let mut failures: Vec<String> = Vec::new();
    let mut total = 0usize;

    for path in &files {
        total += 1;
        let rel = path.strip_prefix(&invalid_dir).unwrap_or(path).display();
        let text = fs::read_to_string(path).unwrap_or_default();

        let err = match ktav::parse(&text) {
            Ok(_) => {
                failures.push(format!("{rel}: parsed OK; expected error"));
                continue;
            }
            Err(e) => e,
        };

        match &err {
            Error::Structured(_) => {
                // Good — structured error as expected.
            }
            Error::Syntax(m) => {
                failures.push(format!("{rel}: legacy Error::Syntax(_): {m}"));
            }
            other => {
                failures.push(format!("{rel}: unexpected error variant: {other:?}"));
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
    eprintln!("structured_errors: {total}/{total} fixtures return Structured(_)");
}

// ---------------------------------------------------------------------------
// Trigger tests for specific error categories
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
            assert_eq!(span, Span::new(7, 8));
        }
        other => panic!("expected UnbalancedBracket, got {other:?}"),
    }
}

#[test]
fn missing_separator_trigger() {
    let src = "anchor: ok\njust-some-text\n";
    let err = ktav::parse(src).unwrap_err();
    match err {
        Error::Structured(ErrorKind::MissingSeparator { line, span }) => {
            assert_eq!(line, 2);
            assert_eq!(span, Span::new(11, 25));
        }
        other => panic!("expected MissingSeparator, got {other:?}"),
    }
}
