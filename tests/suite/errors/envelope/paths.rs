//! Path-is-an-array-of-decoded-segments semantics and JSON escaping of paths.

use super::helpers::{env, f, obj, s, ten_keys_present};
use ktav::value::Scalar;
use ktav::{emit_canonical, parse, Error, ErrorKind, Value};
use serde_json::Value as Json;

// ---------------------------------------------------------------------------
// path is an array of exact decoded segments — never a joined string
// ---------------------------------------------------------------------------

#[test]
fn path_is_array_not_joined_string() {
    // Writer side: the root holds a literal "a -> b" key next to a
    // dotted "a.b" pair, so a joined-string path would be ambiguous.
    let root = obj(&[
        ("a -> b", obj(&[("x", s("1"))])),
        ("a", obj(&[("b", f("NaN"))])),
    ]);
    assert!(
        matches!(&root, Value::Object(m) if m.contains_key(&Scalar::from("a -> b"))),
        "the root must hold the literal key \"a -> b\""
    );
    let err = emit_canonical(&root).unwrap_err();
    let e = env(&err, "");
    assert_eq!(e.reason, Some("NonFiniteFloat".to_string()));
    assert_eq!(e.path, Some(vec!["a".to_string(), "b".to_string()]));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(
        json.contains("\"path\":[\"a\",\"b\"]"),
        "path must be a JSON array, got: {json}"
    );
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a", "b"]));

    // Parse side: the same ambiguity, resolved the same way.
    let src = "a -> b: 1\na.b: 1\na.b: 2\n";
    let err = parse(src).unwrap_err();
    let kind = match &err {
        Error::Structured(k @ ErrorKind::DuplicateKey { .. }) => k,
        other => panic!("expected DuplicateKey, got {other:?}"),
    };
    assert_eq!(kind.code_name(), "DuplicateKey");
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a".to_string(), "b".to_string()]));
}

#[test]
fn decoded_dot_stays_one_segment() {
    let src = "a\\.b: 1\na\\.b: 2\n";
    let err = parse(src).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            // The `key` field carries the RAW dotted-path text; the
            // envelope decodes it (see the path assertion below).
            assert_eq!(key, "a\\.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a.b".to_string()]));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(json.contains("\"path\":[\"a.b\"]"), "got: {json}");
}

// ---------------------------------------------------------------------------
// JSON string escaping
// ---------------------------------------------------------------------------

#[test]
fn json_escaping_control_quote_backslash_nonbmp() {
    // a. raw control byte → lowercase \u00xx
    let src = "a\u{1}b: v\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a\u{1}b".to_string()]));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(json.contains("\\u0001"), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a\u{1}b"]));

    // b. quote inside a quoted key → \"
    let src = "\"a\\\"b\": 1\n\"a\\\"b\": 2\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a\"b".to_string()]));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(json.contains("\\\""), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a\"b"]));

    // c. non-BMP astral char → raw UTF-8, no surrogate escaping
    let src = "\"🎉\": 1\n\"🎉\": 2\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["🎉".to_string()]));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(
        json.as_bytes().windows(4).any(|w| w == "🎉".as_bytes()),
        "raw 4-byte UTF-8 🎉 must survive, got: {json}"
    );
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["🎉"]));

    // d. line_text carrying quotes → \" escapes, serde_json round-trip
    let src = "\"a\"b: v\n";
    let err = parse(src).unwrap_err();
    assert!(matches!(
        &err,
        Error::Structured(ErrorKind::InvalidKey { .. })
    ));
    let e = env(&err, src);
    assert_eq!(e.line_text, Some("\"a\"b: v".to_string()));
    let json = e.to_json();
    ten_keys_present(&json);
    assert!(json.contains("\\\"a\\\"b: v"), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["line_text"], "\"a\"b: v");
}
