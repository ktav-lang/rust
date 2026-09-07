//! Regression tests for review round-5 finding R5-F1 — spec 0.7 § 5.3.3.
//!
//! R5-F1 (comma-arm key context in the fast, no-quote-tracking path of
//! `scan_inline_closer`): at a `,` inside a NESTED scope, key context must
//! be re-derived from the CURRENT scope, not from the outermost opener.
//! Previously the fast path used the whole-scan `object` flag, so an
//! Object nested inside an Array (or an Array inside an Object) put the
//! comma arm in the wrong context, making unquoted-key documents fail
//! while their quoted-key equivalents (which take the slow path) parse —
//! violating § 5.3.3's quoted/unquoted key equivalence.
//!
//! Metamorphic cases fail until the fix lands; negative controls must
//! pass before AND after the fix.

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, Error, ObjectMap, Value};

#[derive(Debug, PartialEq)]
enum Ev {
    Null,
    Bool(bool),
    Integer(String),
    Float(String),
    Str(String),
    Key(String),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

fn collect(src: &str) -> Vec<Ev> {
    let mut out = Vec::new();
    parse_events(src, |e| {
        out.push(match e {
            ParseEvent::Null => Ev::Null,
            ParseEvent::Bool(b) => Ev::Bool(b),
            ParseEvent::Integer(s) => Ev::Integer(s.to_string()),
            ParseEvent::Float(s) => Ev::Float(s.to_string()),
            ParseEvent::Str(s) => Ev::Str(s.to_string()),
            ParseEvent::Key(s) => Ev::Key(s.to_string()),
            ParseEvent::BeginObject => Ev::BeginObject,
            ParseEvent::EndObject => Ev::EndObject,
            ParseEvent::BeginArray => Ev::BeginArray,
            ParseEvent::EndArray => Ev::EndArray,
            _ => unreachable!(),
        });
    })
    .unwrap();
    out
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

fn arr(items: &[Value]) -> Value {
    Value::Array(items.to_vec())
}

// --- Group A: R5-F1 primary repros ---------------------------------------------

#[test]
fn object_in_array_second_pair_with_inline_array_value() {
    let src = r"[{a: 1, b: [2]}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            ("a", Value::Integer("1".into())),
            ("b", arr(&[Value::Integer("2".into())])),
        ]),]),
        "object-in-array comma repro failed for {src:?}"
    );
}

#[test]
fn array_in_object_item_after_comma_keeps_value_context() {
    let src = r"{a: [1, x: [text]}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[(
            "a",
            arr(&[Value::Integer("1".into()), Value::String("x: [text".into()),])
        ),]),
        "array-in-object comma repro failed for {src:?}"
    );
}

// --- Group B: § 5.3.3 quoting invariance (metamorphic) -------------------------

#[test]
fn quoting_first_key_inside_array_is_invariant() {
    let unquoted = parse("[{a: 1, b: [2]}]\n");
    let quoted = parse(
        r#"[{"a": 1, b: [2]}]
"#,
    );
    let v1 = unquoted.unwrap_or_else(|e| panic!("unquoted variant errored: {e:?}"));
    let v2 = quoted.unwrap_or_else(|e| panic!("quoted variant errored: {e:?}"));
    assert_eq!(v1, v2, "quoting the first key changed the parsed value");
}

#[test]
fn quoting_key_before_array_value_is_invariant() {
    let unquoted = parse("{a: [1, x: [text]}\n");
    let quoted = parse("{\"a\": [1, x: [text]}\n");
    let v1 = unquoted.unwrap_or_else(|e| panic!("unquoted variant errored: {e:?}"));
    let v2 = quoted.unwrap_or_else(|e| panic!("quoted variant errored: {e:?}"));
    assert_eq!(v1, v2, "quoting the first key changed the parsed value");
}

// --- Group C: serde-level invariance -------------------------------------------

#[test]
fn serde_quoting_invariance() {
    let v1: serde_json::Value = from_str("[{a: 1, b: [2]}]\n").unwrap();
    let v2: serde_json::Value = from_str("[{\"a\": 1, b: [2]}]\n").unwrap();
    assert_eq!(v1, v2, "serde values differ for case 1 quoted/unquoted");

    let v3: serde_json::Value = from_str("{a: [1, x: [text]}\n").unwrap();
    let v4: serde_json::Value = from_str("{\"a\": [1, x: [text]}\n").unwrap();
    assert_eq!(v3, v4, "serde values differ for case 2 quoted/unquoted");
}

// --- Group D: thin-API event streams -------------------------------------------

#[test]
fn thin_events_quoting_invariance() {
    let e1 = collect("[{a: 1, b: [2]}]\n");
    let e2 = collect("[{\"a\": 1, b: [2]}]\n");
    assert_eq!(e1, e2, "event streams differ for quoted/unquoted case 1");
    assert_eq!(
        e1,
        vec![
            Ev::BeginArray,
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Integer("1".into()),
            Ev::Key("b".into()),
            Ev::BeginArray,
            Ev::Integer("2".into()),
            Ev::EndArray,
            Ev::EndObject,
            Ev::EndArray,
        ],
        "pinned event stream for case 1 failed"
    );
}

// --- Group E: negative controls (must pass before AND after the fix) -----------

#[test]
fn negative_controls_unterminated() {
    // Outer array's `]` missing (`{a: 1, b: [2]}]` would be a complete doc).
    let err = parse("[{a: 1, b: [2]}").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("control 1: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // No trailing close for the outer object.
    let err = parse("{a: [1, x: [text]").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("control 2: expected UnterminatedInlineCompound, got {other:?}"),
    }
}
