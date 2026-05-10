//! Tests for the `Error::line()` and `Error::span()` accessors plus
//! `ErrorKind::line()` / `ErrorKind::span()`.

use std::io;

use ktav::{Error, Span};

fn parse_err(text: &str) -> Error {
    ktav::parse(text).expect_err("expected parse failure")
}

// ---------------------------------------------------------------------------
// Non-Structured error variants — `line()` and `span()` return None.
// ---------------------------------------------------------------------------

#[test]
fn io_error_has_no_line_or_span() {
    let e = Error::Io(io::Error::new(io::ErrorKind::Other, "x"));
    assert_eq!(e.line(), None);
    assert_eq!(e.span(), None);
}

#[test]
fn syntax_error_has_no_line_or_span() {
    let e = Error::Syntax("free-form".into());
    assert_eq!(e.line(), None);
    assert_eq!(e.span(), None);
}

#[test]
fn message_error_has_no_line_or_span() {
    let e = Error::Message("custom".into());
    assert_eq!(e.line(), None);
    assert_eq!(e.span(), None);
}

// ---------------------------------------------------------------------------
// Structured variants — `line()` and `span()` return Some(...) (except
// `UnclosedCompound` whose line is detected at EOF).
// ---------------------------------------------------------------------------

#[test]
fn missing_separator_space_has_line_and_span() {
    // Anchor with a known-Object pair so the test exercises pair-line
    // grammar (spec § 5.3 + § 6.10), not top-level Array root
    // (§ 5.0.1) where colon-bearing scalars are valid items.
    let e = parse_err("anchor: ok\nport:8080\n");
    assert_eq!(e.line(), Some(2));
    let s = e.span().expect("span");
    assert!(s.end > s.start);
}

#[test]
fn invalid_typed_scalar_has_line_and_span() {
    let e = parse_err("port:i abc\n");
    assert_eq!(e.line(), Some(1));
    assert!(e.span().is_some());
}

#[test]
fn duplicate_key_has_line_and_span() {
    let e = parse_err("port: 80\nport: 443\n");
    assert_eq!(e.line(), Some(2));
    assert!(e.span().is_some());
}

#[test]
fn key_path_conflict_has_line_and_span() {
    let e = parse_err("db: 1\ndb.x: 2\n");
    assert_eq!(e.line(), Some(2));
    assert!(e.span().is_some());
}

#[test]
fn empty_key_has_line_and_span() {
    // Anchor with a known-Object pair so empty-key error exercises
    // pair-line grammar instead of top-level Array root (§ 5.0.1
    // would treat `: value` as a bare-scalar Array item).
    let e = parse_err("anchor: ok\n: value\n");
    assert_eq!(e.line(), Some(2));
    assert!(e.span().is_some());
}

#[test]
fn invalid_key_has_line_and_span() {
    let e = parse_err("a.: 1\n");
    assert_eq!(e.line(), Some(1));
    assert!(e.span().is_some());
}

#[test]
fn unclosed_compound_has_no_line_but_has_span() {
    // EOF-detected: no line is meaningful, but the span covers
    // opener..EOF.
    let e = parse_err("obj: {\n  a: 1\n");
    assert_eq!(e.line(), None);
    let s = e.span().expect("span");
    assert!(s.end >= s.start);
}

#[test]
fn unbalanced_bracket_has_line_and_span() {
    let e = parse_err("}\n");
    assert_eq!(e.line(), Some(1));
    assert_eq!(
        e.span(),
        Some(Span::new(0, 1)),
        "stray `}}` at top of input"
    );
}

#[test]
fn inline_nonempty_compound_has_line_and_span() {
    let e = parse_err("server: { a: 1 }\n");
    assert_eq!(e.line(), Some(1));
    assert!(e.span().is_some());
}

#[test]
fn missing_separator_has_line_and_span() {
    // Anchor with a known-Object pair so a colon-less line errors
    // as MissingSeparator inside an Object instead of being parsed
    // as a top-level Array bare-scalar (§ 5.0.1).
    let e = parse_err("anchor: ok\njust-some-text\n");
    assert_eq!(e.line(), Some(2));
    let s = e.span().expect("span");
    assert!(s.end > s.start);
}

// ---------------------------------------------------------------------------
// `ErrorKind::line()` / `ErrorKind::span()` direct checks.
// ---------------------------------------------------------------------------

#[test]
fn error_kind_line_is_none_for_unclosed_compound() {
    let e = parse_err("obj: {\n");
    if let Error::Structured(k) = &e {
        assert_eq!(k.line(), None);
        // Span is always populated, even if EMPTY.
        let _ = k.span();
    } else {
        panic!("expected structured: {e:?}");
    }
}
