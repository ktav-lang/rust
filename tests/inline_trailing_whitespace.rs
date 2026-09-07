//! Regression tests for review finding R3-F1 (round-3 review): trailing
//! whitespace after an inline object's trailing comma (or after a
//! dotted-key `.`) sent `skip_segment_ws` to EOF and the scanner
//! indexed out of bounds — a panic on `{"a": 1, }`. All three public
//! APIs (`ktav::parse`, `ktav::from_str`, `ktav::thin::parse_events`)
//! must accept valid trailing-comma forms and raise a structured
//! `EmptyKey` for the genuinely invalid `b.` + whitespace form.

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, Error, Value};

fn expect_key_a_1(src: &str) {
    // Owned/tree parser: exactly one key `a` with integer 1.
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root, got {value:?}");
    };
    assert_eq!(obj.len(), 1, "{src:?}: expected exactly one key");
    assert_eq!(
        obj.get("a"),
        Some(&Value::Integer("1".into())),
        "{src:?}: value mismatch"
    ); // Serde path: serde_json as oracle.
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, serde_json::json!({"a": 1}), "{src:?}: serde mismatch");
    // Thin/event parser: Key("a"), Integer("1").
    let mut keys = Vec::new();
    let mut ints = Vec::new();
    parse_events(src, |e| match e {
        ParseEvent::Key(k) => keys.push(k.to_string()),
        ParseEvent::Integer(i) => ints.push(i.to_string()),
        _ => {}
    })
    .unwrap();
    assert_eq!(keys, vec!["a".to_string()], "{src:?}: thin keys mismatch");
    assert_eq!(ints, vec!["1".to_string()], "{src:?}: thin ints mismatch");
}

#[test]
fn valid_trailing_comma_root_level() {
    expect_key_a_1("{\"a\": 1, }\n");
    expect_key_a_1("{\"a\": 1,\t}\n");
    expect_key_a_1("{\"a\": 1, \u{00a0}}\n");
}

#[test]
fn valid_trailing_comma_nested() {
    let src = "outer: {\"a\": 1, }\n";
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root");
    };
    let Value::Object(inner) = obj.get("outer").unwrap() else {
        panic!("{src:?}: expected nested Object under `outer`");
    };
    assert_eq!(
        inner.get("a"),
        Some(&Value::Integer("1".into())),
        "{src:?}: nested value mismatch"
    );
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, serde_json::json!({"outer": {"a": 1}}));
    let mut keys = Vec::new();
    parse_events(src, |e| {
        if let ParseEvent::Key(k) = e {
            keys.push(k.to_string());
        }
    })
    .unwrap();
    assert_eq!(keys, vec!["outer".to_string(), "a".to_string()]);
    let src = "outer: {\"a\": 1,\t}\n";
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root");
    };
    let Value::Object(inner) = obj.get("outer").unwrap() else {
        panic!("{src:?}: expected nested Object under `outer`");
    };
    assert_eq!(inner.get("a"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_key_and_value_before_trailing_comma() {
    // Quoted KEY before the trailing comma.
    let src = "{ \"a b\": 1, }\n";
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root");
    };
    assert_eq!(obj.get("a b"), Some(&Value::Integer("1".into())));
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, serde_json::json!({"a b": 1}));
    let mut keys = Vec::new();
    parse_events(src, |e| {
        if let ParseEvent::Key(k) = e {
            keys.push(k.to_string());
        }
    })
    .unwrap();
    assert_eq!(keys, vec!["a b".to_string()]);

    // Quoted VALUE before the trailing comma.
    let src = "{ a: \"x\", }\n";
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root");
    };
    assert_eq!(obj.get("a"), Some(&Value::String("\"x\"".into())));
    // NOTE: ktav::Value::String (and the serde mapping) keep the quote
    // delimiters of a quoted scalar body — pinned as-is.
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, serde_json::json!({"a": "\"x\""}));
    let mut strs = Vec::new();
    parse_events(src, |e| {
        if let ParseEvent::Str(s) = e {
            strs.push(s.to_string());
        }
    })
    .unwrap();
    assert_eq!(strs, vec!["\"x\"".to_string()]);
}

#[test]
fn no_space_trailing_comma_still_fine() {
    expect_key_a_1("{\"a\": 1,}\n");
    // Comma at line end with no closer on that line: per § 5.2 rule 9
    // the compound continues on the next line — with no next line this
    // stays invalid (unchanged behaviour).
    match parse("{\"a\": 1,\n") {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!("expected UnterminatedInlineCompound, got {:?}", other.err()),
    }
}

#[test]
fn dotted_key_trailing_ws_is_empty_key() {
    // `{ "a": 1, b. }` — a dotted key whose final segment is empty:
    // structured EmptyKey, never a panic, never success.
    let src = "{ \"a\": 1, b. }\n";
    match parse(src) {
        Err(Error::Structured(ErrorKind::EmptyKey { .. })) => {}
        other => panic!("{src:?}: expected EmptyKey, got {:?}", other.err()),
    }
    let err = from_str::<serde_json::Value>(src).unwrap_err();
    assert!(
        matches!(err, Error::Structured(ErrorKind::EmptyKey { .. })),
        "{src:?}: from_str expected EmptyKey, got {err:?}"
    );
    let thin = parse_events(src, |_| {}).unwrap_err();
    assert!(
        matches!(thin, Error::Structured(ErrorKind::EmptyKey { .. })),
        "{src:?}: thin expected EmptyKey, got {thin:?}"
    );

    // Nested invalid form errors via the owned parser too.
    let src = "outer: { \"a\": 1, b. }\n";
    match parse(src) {
        Err(Error::Structured(ErrorKind::EmptyKey { .. })) => {}
        other => panic!("{src:?}: expected EmptyKey, got {:?}", other.err()),
    }
}
