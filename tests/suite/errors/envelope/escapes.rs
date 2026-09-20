//! Escaping behavior pinned by questions Q1 and Q3.

use super::helpers::{obj, s};
use ktav::{emit_canonical, parse, parse_strict, Error, ErrorKind};

// ---------------------------------------------------------------------------
// Q1 — parens have an escape path
// ---------------------------------------------------------------------------

#[test]
fn q1_parens_have_an_escape_path() {
    let doc = parse("a\\u0028b: v\n").unwrap();
    assert_eq!(
        doc,
        obj(&[("a(b", s("v"))]),
        "\\u0028 must decode to ( in a key"
    );
    let doc = parse("a\\u0029b: v\n").unwrap();
    assert_eq!(doc, obj(&[("a)b", s("v"))]));

    let err = parse("a\\(b: v\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::BadEscapeSequence { sequence, .. }) => {
            assert_eq!(sequence, "\\(");
        }
        other => panic!("expected BadEscapeSequence, got {other:?}"),
    }

    let text = emit_canonical(&obj(&[("a(b", s("v"))])).unwrap();
    assert!(
        text.starts_with("\"a(b\": v"),
        "canonical must quote the paren key, got: {text:?}"
    );
    assert_eq!(parse(&text).unwrap(), obj(&[("a(b", s("v"))]));

    let doc = parse("a\\u0020: v\n").unwrap();
    assert_eq!(
        doc,
        obj(&[("a ", s("v"))]),
        "the escaped space must survive key trimming (§ 6.13 escape path)"
    );

    let doc = parse("\\uD83D\\uDE00: v\n").unwrap();
    assert_eq!(doc, obj(&[("😀", s("v"))]));

    assert!(parse_strict("a\\u0028b: v\n").is_ok());
}
// ---------------------------------------------------------------------------
// Q3 — named escapes win over \uXXXX in the canonical writer
// ---------------------------------------------------------------------------

#[test]
fn q3_named_escape_wins_over_unicode_escape() {
    // a. literal backslash → \\ (named form), never \u005C
    let text = emit_canonical(&obj(&[("path\\to", s("v"))])).unwrap();
    assert!(text.contains("path\\\\to"), "got: {text:?}");
    assert!(!text.contains("u005C"));

    // b. leading LF → \n (named form), never \u000A
    let text = emit_canonical(&obj(&[("\nlf", s("v"))])).unwrap();
    assert!(text.starts_with("\\nlf: v"), "got: {text:?}");
    assert!(!text.contains("u000A"));
    // g. and it round-trips
    assert_eq!(parse(&text).unwrap(), obj(&[("\nlf", s("v"))]));

    // c. DEL → \u007F with uppercase hex
    let text = emit_canonical(&obj(&[("a\u{7F}b", s("v"))])).unwrap();
    assert!(text.contains("\\u007F"), "got: {text:?}");
    assert!(!text.contains("\\u007f"));

    // d. NUL → \u0000
    let text = emit_canonical(&obj(&[("a\u{0}b", s("v"))])).unwrap();
    assert!(text.contains("\\u0000"), "got: {text:?}");

    // e. a literal dot forces quoting but the dot stays raw —
    // § 5.9.10 prefers quoting over the \. escape
    let text = emit_canonical(&obj(&[("a.b", s("v"))])).unwrap();
    assert!(text.contains("\"a.b\""), "got: {text:?}");
    assert!(!text.contains("\\."));

    // f. an interior tab stays raw inside the emitted key
    let text = emit_canonical(&obj(&[("a\tb", s("v"))])).unwrap();
    assert!(text.contains("a\tb"), "got: {text:?}");
}
