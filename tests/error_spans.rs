//! Per-variant byte-range assertions for the `span` field on every
//! `ErrorKind` variant, plus `Span::slice` / `Span::line_col` semantics.
//!
//! These are the contract tests for the `0.1.6` Span plumbing: every
//! variant's `span` must point at the offending substring inside the
//! original input.

use ktav::{Error, ErrorKind, Span};

fn span_of(text: &str) -> (ErrorKind, String) {
    let err = ktav::parse(text).expect_err("expected parse failure");
    match err {
        Error::Structured(k) => {
            let slice = k
                .span()
                .slice(text)
                .map(|s| s.to_string())
                .unwrap_or_default();
            (k, slice)
        }
        other => panic!("expected Structured, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// One fixture per variant — span byte range checked.
// ---------------------------------------------------------------------------

#[test]
fn span_missing_separator_space() {
    // `key:value` — body slice "value". Anchor with a known-Object
    // pair so first-line `key:value` doesn't get parsed as a top-
    // level Array bare-scalar (spec § 5.0.1).
    let text = "anchor: ok\nkey:value\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::MissingSeparatorSpace { .. }));
    assert_eq!(slice, "value");
}

#[test]
fn span_missing_separator_space_for_old_typed_marker() {
    // Under 0.5.0, `port:i abc` needs anchoring with a pair to force
    // Object root. The `i` is glued to `:` → MissingSeparatorSpace.
    let text = "anchor: ok\nport:i abc\n";
    let (k, _slice) = span_of(text);
    assert!(matches!(k, ErrorKind::MissingSeparatorSpace { .. }));
}

#[test]
fn span_duplicate_key() {
    let text = "port: 80\nport: 443\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::DuplicateKey { .. }));
    // Span covers the duplicate `port` key on line 2.
    assert_eq!(slice, "port");
}

#[test]
fn span_key_path_conflict() {
    let text = "db: 1\ndb.x: 2\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::KeyPathConflict { .. }));
    // Span covers the `db.x` key on line 2.
    assert_eq!(slice, "db.x");
}

#[test]
fn span_empty_key() {
    // Anchor with a known-Object pair (spec § 5.0.1).
    let text = "anchor: ok\n: value\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::EmptyKey { .. }));
    // Span points at the leading `:` byte (relative slice still ":").
    assert_eq!(slice, ":");
}

#[test]
fn span_invalid_key() {
    // Under 0.5.0, `a.` has an empty trailing segment.
    // The parser may report EmptyKey or InvalidKey.
    let text = "a.: 1\n";
    let (k, _slice) = span_of(text);
    assert!(
        matches!(k, ErrorKind::InvalidKey { .. } | ErrorKind::EmptyKey { .. }),
        "got: {k:?}"
    );
}

#[test]
fn span_unclosed_compound_object() {
    let text = "obj: {\n  a: 1\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::UnclosedCompound { .. }));
    // Span starts at the `{` opener and runs to EOF.
    assert!(slice.starts_with('{'), "got: {slice:?}");
}

#[test]
fn span_unbalanced_bracket_stray() {
    let text = "}\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::UnbalancedBracket { .. }));
    assert_eq!(slice, "}");
}

#[test]
fn span_unbalanced_bracket_mismatch() {
    let text = "obj: {\n]\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::UnbalancedBracket { .. }));
    assert_eq!(slice, "]");
}

#[test]
fn inline_nonempty_compound_now_parses_as_inline_object() {
    // Under 0.5.0, `{ host: 127.0.0.1 }` is a valid inline object.
    let text = "server: { host: 127.0.0.1 }\n";
    let v = ktav::parse(text).unwrap();
    let obj = v.as_object().unwrap();
    let server = obj.get("server").unwrap().as_object().unwrap();
    assert_eq!(
        server.get("host"),
        Some(&ktav::Value::String("127.0.0.1".into()))
    );
}

#[test]
fn span_missing_separator() {
    // Anchor with a known-Object pair (spec § 5.0.1).
    let text = "anchor: ok\njust-some-text\n";
    let (k, slice) = span_of(text);
    assert!(matches!(k, ErrorKind::MissingSeparator { .. }));
    assert_eq!(slice, "just-some-text");
}

// ---------------------------------------------------------------------------
// `Span::slice` and `Span::line_col` semantics.
// ---------------------------------------------------------------------------

#[test]
fn span_slice_returns_substring() {
    let s = Span::new(2, 5);
    assert_eq!(s.slice("abcdef"), Some("cde"));
}

#[test]
fn span_slice_out_of_bounds_returns_none() {
    let s = Span::new(10, 20);
    assert_eq!(s.slice("abcdef"), None);
}

#[test]
fn span_slice_non_char_boundary_returns_none() {
    // 'ы' is 2 bytes in UTF-8. Slicing inside it yields None.
    let text = "ыz";
    let s = Span::new(1, 2);
    assert_eq!(s.slice(text), None);
}

#[test]
fn span_line_col_first_line() {
    let text = "abc\ndef\n";
    let s = Span::new(0, 1);
    assert_eq!(s.line_col(text), (1, 0));
    let s = Span::new(2, 3);
    assert_eq!(s.line_col(text), (1, 2));
}

#[test]
fn span_line_col_second_line() {
    let text = "abc\ndef\n";
    // 'd' is at byte 4.
    let s = Span::new(4, 5);
    assert_eq!(s.line_col(text), (2, 0));
    // 'f' is at byte 6.
    let s = Span::new(6, 7);
    assert_eq!(s.line_col(text), (2, 2));
}

#[test]
fn span_line_col_with_multibyte_utf8_returns_byte_column() {
    // 'й' is a 2-byte UTF-8 char (0xD0 0xB9). Column is 0-based BYTES,
    // not Unicode chars.
    let text = "йx\nz";
    // Byte 2 = 'x' (after the 2-byte 'й'); column should be 2.
    let s = Span::new(2, 3);
    assert_eq!(s.line_col(text), (1, 2));
    // Byte 3 = '\n', still on line 1, column 3.
    let s = Span::new(3, 4);
    assert_eq!(s.line_col(text), (1, 3));
    // Byte 4 = 'z' on line 2 column 0.
    let s = Span::new(4, 5);
    assert_eq!(s.line_col(text), (2, 0));
}

#[test]
fn span_line_col_with_emoji_returns_byte_column() {
    // 🦀 is 4 bytes in UTF-8.
    let text = "🦀X";
    let s = Span::new(4, 5);
    let (line, col) = s.line_col(text);
    assert_eq!((line, col), (1, 4));
}
