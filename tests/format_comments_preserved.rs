//! Comment preservation, pinned by count, text, and attachment point
//! (issue rust#13's "Definition of done"), over targeted fixtures
//! covering every attachment shape: a plain pair, a nested block, an
//! array item, a dotted-key leaf, an inline-compound line, an empty
//! compound holding only a comment, end-of-file trivia, and comments
//! around a top-level inline document.
//!
//! Also folds in the fixed corpus fixtures under
//! `valid/{comments,empty,top_level_array,top_level_inline}` that
//! exercise the same shapes end to end.

use std::fs;
use std::path::{Path, PathBuf};

/// Number of `##`-comment lines in `text` (by the same rule the parser
/// uses: first non-whitespace chars are `##`).
fn comment_count(text: &str) -> usize {
    text.lines().filter(|l| l.trim().starts_with("##")).count()
}

fn comment_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| l.starts_with("##"))
        .collect()
}

/// Asserts: every comment line in `src` reappears verbatim (same text,
/// same count) in `format_str(src)`, and re-parsing the formatted
/// output preserves the document's `Value`.
fn assert_comments_preserved(src: &str) -> String {
    let formatted =
        ktav::format_str(src).unwrap_or_else(|e| panic!("format_str failed on {src:?}: {e}"));

    assert_eq!(
        comment_count(src),
        comment_count(&formatted),
        "comment count changed\nsrc:\n{src}\nformatted:\n{formatted}"
    );
    assert_eq!(
        comment_lines(src),
        comment_lines(&formatted),
        "comment text changed\nsrc:\n{src}\nformatted:\n{formatted}"
    );

    let original_value = ktav::parse(src).unwrap();
    let reparsed = ktav::parse(&formatted)
        .unwrap_or_else(|e| panic!("re-parse of formatted output failed: {e}\n{formatted}"));
    assert_eq!(
        original_value, reparsed,
        "structure changed by formatting\n{src}"
    );

    formatted
}

#[test]
fn comment_before_top_level_pair() {
    let out = assert_comments_preserved("## why this port\nport: 8080\n");
    let comment_idx = out.find("## why this port").unwrap();
    let key_idx = out.find("port: 8080").unwrap();
    assert!(
        comment_idx < key_idx,
        "comment must precede the key it attaches to"
    );
}

#[test]
fn comment_inline_object_expands_and_keeps_comment_first() {
    let out = assert_comments_preserved("## why this port\ndb: {host: primary, port: 5432}\n");
    assert_eq!(
        out,
        "## why this port\ndb: {\n    host: primary\n    port: 5432\n}\n"
    );
}

#[test]
fn comment_before_nested_key_inside_block() {
    let out = assert_comments_preserved(
        "db: {\n    ## about host\n    host: primary\n    port: 5432\n}\n",
    );
    let comment_idx = out.find("## about host").unwrap();
    let host_idx = out.find("host: primary").unwrap();
    let port_idx = out.find("port: 5432").unwrap();
    assert!(comment_idx < host_idx && host_idx < port_idx);
}

#[test]
fn comment_before_array_item() {
    let out = assert_comments_preserved("items: [\n    ## first\n    a\n    ## second\n    b\n]\n");
    assert_eq!(
        out,
        "items: [\n    ## first\n    a\n    ## second\n    b\n]\n"
    );
}

#[test]
fn comment_before_dotted_key_attaches_to_leaf_segment() {
    // Documented simplification (see `format_str`'s doc comment): the
    // comment attaches to the deepest synthesized segment, not the
    // outermost one.
    let out = assert_comments_preserved("## note\na.b.c: 1\n");
    assert_eq!(
        out,
        "a: {\n    b: {\n        ## note\n        c: 1\n    }\n}\n"
    );
}

#[test]
fn comment_alone_inside_otherwise_empty_object() {
    let out = assert_comments_preserved("db: {\n    ## just a note\n}\n");
    assert_eq!(out, "db: {\n    ## just a note\n}\n");
}

#[test]
fn comment_alone_inside_otherwise_empty_array() {
    let out = assert_comments_preserved("items: [\n    ## nothing here yet\n]\n");
    assert_eq!(out, "items: [\n    ## nothing here yet\n]\n");
}

#[test]
fn comment_after_top_level_inline_document() {
    let out = assert_comments_preserved("{a: 1, b: 2}\n## trailing note\n");
    assert_eq!(out, "a: 1\nb: 2\n## trailing note\n");
}

#[test]
fn comment_before_top_level_inline_document() {
    let out = assert_comments_preserved("## header\n{a: 1}\n");
    assert_eq!(out, "## header\na: 1\n");
}

#[test]
fn eof_trailing_comment_on_implicit_root() {
    let out = assert_comments_preserved("port: 8080\n## trailing note\n");
    assert_eq!(out, "port: 8080\n## trailing note\n");
}

#[test]
fn comments_only_document_round_trips() {
    let out = assert_comments_preserved("## just a comment\n## and another\n");
    assert_eq!(out, "## just a comment\n## and another\n");
}

#[test]
fn multiple_comments_and_multi_blank_runs_collapse_but_keep_all_comments() {
    let src = "## a\n\n\n\n## b\nkey: value\n\n\n\n## trailing\n";
    let out = assert_comments_preserved(src);
    // Every comment survives, and 3+ blank runs collapse to one.
    assert_eq!(out, "## a\n\n## b\nkey: value\n\n## trailing\n");
}

#[test]
fn no_comments_inside_multiline_string_block_are_not_comments() {
    // `##` inside a multi-line string body is ordinary content, never a
    // comment (spec § 3.4 only classifies lines outside such blocks).
    let src = "note: (\n    ## not a comment\n    still text\n)\n";
    let value = ktav::parse(src).unwrap();
    assert_eq!(
        value.as_object().unwrap().get("note").unwrap().as_str(),
        Some("## not a comment\nstill text")
    );
    // The `##` line survives as string BODY content (inside the
    // multi-line block), not as a hoisted trivia comment: re-parsing
    // the formatted output must reproduce the exact same String value,
    // `##` and all.
    let out = ktav::format_str(src).unwrap();
    let reparsed = ktav::parse(&out).unwrap();
    assert_eq!(value, reparsed);
}

// ---------------------------------------------------------------------------
// Corpus fixtures
// ---------------------------------------------------------------------------

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

#[test]
fn corpus_commented_fixtures_preserve_comments() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("format_comments_preserved: no spec checkout found, skipping");
        return;
    };
    let tests_dir = spec_root.join("versions").join(SPEC_VERSION).join("tests");

    let candidates = [
        tests_dir.join("valid/comments/indented.ktav"),
        tests_dir.join("valid/comments/own_line.ktav"),
        tests_dir.join("valid/comments/single_hash_is_literal.ktav"),
        tests_dir.join("valid/empty/comments_only.ktav"),
        tests_dir.join("valid/top_level_array/with_comments_and_blanks.ktav"),
        tests_dir.join("valid/top_level_inline/with_comments.ktav"),
    ];

    // Every listed fixture must be present. A `continue`-on-missing
    // loop guarded only by "checked > 0" would let five of the six
    // silently disappear — a rename in the corpus would quietly shrink
    // this test's coverage instead of failing it.
    let mut missing: Vec<&PathBuf> = Vec::new();
    for path in &candidates {
        match fs::read_to_string(path) {
            Ok(text) => {
                assert_comments_preserved(&text);
            }
            Err(_) => missing.push(path),
        }
    }
    assert!(
        missing.is_empty(),
        "comment fixtures named here are absent from the corpus \
         (renamed or removed? update this list deliberately): {missing:#?}"
    );
}
