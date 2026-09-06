//! Ktav spec 0.7 § 5.3.3 (quoted key segments) for the zero-copy event
//! parser (`ktav::thin::parse_events`), mirroring the owned-parser
//! matrix in `tests/quoted_keys.rs`. The thin parser has its own
//! independent key lexer, so quoted-segment behaviour must hold in both.

use ktav::thin::{parse_events, ParseEvent};
use ktav::{Error, ErrorKind};

/// Collect the `Key` events of a successful parse (panics on error).
fn keys_of(src: &str) -> Vec<String> {
    let mut keys = Vec::new();
    parse_events(src, |ev| {
        if let ParseEvent::Key(k) = ev {
            keys.push(k.to_string());
        }
    })
    .unwrap();
    keys
}

#[test]
fn thin_quoted_keys_parse() {
    let src = "\"a\": 1\n'b c': 2\n`d e`: 3\n";
    assert_eq!(keys_of(src), ["a", "b c", "d e"]);
}

#[test]
fn thin_unterminated_quoted_key_in_object() {
    let src = "x: 1\n'unterminated: 1\n";
    let err = parse_events(src, |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::UnterminatedQuotedKey { .. }) => {}
        other => panic!("expected UnterminatedQuotedKey, got {other:?}"),
    }
}

#[test]
fn thin_unterminated_quoted_key_inline_value() {
    let err = parse_events("y: {\"a: 1}\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        Error::Structured(kind @ (ErrorKind::MalformedInlineCompound { .. }
        | ErrorKind::UnterminatedQuotedKey { .. })) => {
            panic!("expected UnterminatedInlineCompound, got {kind:?}")
        }
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
}

#[test]
fn thin_empty_quoted_key() {
    let err = parse_events("\"\": 1\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::EmptyKey { .. }) => {}
        other => panic!("expected EmptyKey, got {other:?}"),
    }
}

#[test]
fn thin_quoted_segment_in_dotted_path() {
    let src = "a.\"b.c\".d: 1\n";
    assert_eq!(keys_of(src), ["a", "b.c", "d"]);
}

#[test]
fn thin_first_line_quote_colon_is_array() {
    let src = "'tis the season: fa\n";
    parse_events(src, |_| {}).unwrap();
}

#[test]
fn thin_content_after_closer() {
    let err = parse_events("\"a\"b: 1\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::InvalidKey { .. }) => {}
        other => panic!("expected InvalidKey, got {other:?}"),
    }
}

#[test]
fn thin_quoted_key_delimiters_stripped_in_events() {
    let src = "\"a\": 1\n\"b.c\": 2\n";
    assert_eq!(keys_of(src), ["a", "b.c"]);
}

#[test]
fn thin_quoted_key_with_escape_in_events() {
    let src = "\"a\\nb\": 1\n";
    assert_eq!(keys_of(src), ["a\nb"]);
}

#[test]
fn thin_value_quotes_ordinary() {
    let mut strings = Vec::new();
    parse_events("a: \"b\"\n", |ev| {
        if let ParseEvent::Str(s) = ev {
            strings.push(s.to_string());
        }
    })
    .unwrap();
    assert_eq!(strings, ["\"b\""]);
}
