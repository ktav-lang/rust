//! Resolves issue rust#13's third open question: does `format_str`
//! equal `emit_canonical` for a comment-free document?
//!
//! Answer, pinned here: yes, *provided the document also has no blank
//! lines* — a comment-free document can still contain grouping blank
//! lines, which `format_str` preserves (per the resolution of open
//! question 1, see `format_str`'s doc comment) but `emit_canonical`
//! always drops (blanks are no more part of the `Value` model than
//! comments are, § 3.5). So the exact equivalence needs "no comments
//! AND no blank lines", not just "no comments" — this file tests both
//! the equivalence (trivia-free corpus fixtures) and the documented
//! divergence (a blank-line-only fixture).

use std::fs;
use std::path::{Path, PathBuf};

const SPEC_VERSION: &str = "0.8";

fn resolve_spec_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(env) = std::env::var("KTAV_SPEC_DIR") {
        candidates.push(PathBuf::from(env));
    }
    candidates.push(manifest.join("spec"));
    candidates.push(manifest.join("../spec"));
    candidates.into_iter().find(|p| p.join("versions").join(SPEC_VERSION).is_dir())
}

fn tests_root(spec_root: &Path) -> PathBuf {
    spec_root
        .join("versions")
        .join(SPEC_VERSION)
        .join("tests")
        .join("valid")
}

fn collect_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ktav_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !stem.ends_with(".canonical") {
                out.push(path);
            }
        }
    }
}

/// True iff no physical line is blank and none starts (after trimming
/// ASCII/§3.3 whitespace-ish leading spaces/tabs) with `##`. Good
/// enough for the corpus: fixtures use plain ASCII indentation.
fn is_trivia_free(text: &str) -> bool {
    text.lines().all(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with("##")
    })
}

#[test]
fn format_matches_emit_canonical_for_trivia_free_documents() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("format_canonical_equivalence: no spec checkout found, skipping");
        return;
    };

    let mut files = Vec::new();
    collect_ktav_files(&tests_root(&spec_root), &mut files);
    assert!(!files.is_empty(), "expected valid/**/*.ktav fixtures");

    let mut checked = 0usize;
    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        if !is_trivia_free(&text) {
            continue;
        }
        let Ok(value) = ktav::parse(&text) else {
            continue;
        };
        let Ok(canonical) = ktav::emit_canonical(&value) else {
            continue;
        };
        let formatted = ktav::format_str(&text).unwrap_or_else(|e| {
            panic!(
                "format_str failed on trivia-free fixture {}: {e}",
                path.display()
            )
        });
        assert_eq!(
            formatted,
            canonical,
            "format_str != emit_canonical for trivia-free fixture {}",
            path.display()
        );
        checked += 1;
    }
    assert!(
        checked > 50,
        "expected many trivia-free fixtures, only checked {checked}"
    );
}

#[test]
fn format_matches_emit_canonical_for_hand_written_trivia_free_docs() {
    let cases = [
        "host: localhost\nport: 8080\n",
        "db: {\n    host: primary\n    port: 5432\n}\n",
        "items: [\n    a\n    b\n    c\n]\n",
        "a.b.c: 1\n",
        "{}\n",
        "[]\n",
    ];
    for src in cases {
        let value = ktav::parse(src).unwrap();
        let canonical = ktav::emit_canonical(&value).unwrap();
        let formatted = ktav::format_str(src).unwrap();
        assert_eq!(formatted, canonical, "mismatch for input {src:?}");
    }
}

/// Every `*.canonical.ktav` fixture is already in § 5.9 canonical form
/// and trivia-free by construction, so it must be a fixed point of
/// `format_str`: formatting it changes nothing at all.
#[test]
fn format_is_a_fixed_point_on_canonical_fixtures() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("format_canonical_equivalence: no spec checkout found, skipping");
        return;
    };

    fn collect_canonical_files(root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_canonical_files(&path, out);
            } else if path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.ends_with(".canonical"))
            {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    collect_canonical_files(&tests_root(&spec_root), &mut files);
    assert!(!files.is_empty(), "expected *.canonical.ktav fixtures");

    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        let formatted = ktav::format_str(&text).unwrap_or_else(|e| {
            panic!(
                "format_str failed on canonical fixture {}: {e}",
                path.display()
            )
        });
        assert_eq!(
            formatted,
            text,
            "format_str changed an already-canonical fixture {}",
            path.display()
        );
    }
}

/// The documented divergence: a comment-free document with a grouping
/// blank line does NOT equal `emit_canonical` (which drops the blank),
/// because `format_str` preserves it.
#[test]
fn format_diverges_from_canonical_when_blank_lines_present() {
    let src = "host: localhost\n\nport: 8080\n";
    let value = ktav::parse(src).unwrap();
    let canonical = ktav::emit_canonical(&value).unwrap();
    let formatted = ktav::format_str(src).unwrap();

    assert_eq!(canonical, "host: localhost\nport: 8080\n");
    assert_eq!(formatted, "host: localhost\n\nport: 8080\n");
    assert_ne!(
        formatted, canonical,
        "blank-line grouping should survive format_str but not emit_canonical"
    );
}
