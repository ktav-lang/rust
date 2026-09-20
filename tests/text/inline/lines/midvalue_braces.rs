//! Regression tests for review finding R3-F4 — spec 0.7 § 5.8.5:
//! a `{`/`[` byte that is not the first non-whitespace code point of
//! an inline value is a literal character with NO structural meaning,
//! even when the braces happen to balance. Balanced mid-scalar spans
//! must not be skipped wholesale (swallowing the comma inside).

use std::collections::BTreeMap;

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, Error, Value};

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

fn collect(src: &str) -> Result<Vec<Ev>, Error> {
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
    })?;
    Ok(out)
}

fn check_kind(src: &str, kind: &str) {
    let err = parse(src).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::MalformedInlineCompound { .. })
            if kind == "MalformedInlineCompound" => {}
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })
            if kind == "UnterminatedInlineCompound" => {}
        other => panic!("expected {kind} for {src:?}, got {other:?}"),
    }
}

// --- negative: balanced { mid-scalar (group A) ------------------------------

#[test]
fn midvalue_balanced_brace_rejected() {
    // `x{y,z}` inside an object body: the balanced `{...}` span must not
    // be skipped; the `}` mid-scalar leaves trailing content after the
    // body's real closer is consumed early.
    check_kind("{a: x{y,z}, b: 2}", "MalformedInlineCompound");
    check_kind("{a: x{y,z}, b: 2}\n", "MalformedInlineCompound");
    check_kind("cfg: {a: x{y,z}, b: 2}", "MalformedInlineCompound");
    check_kind("{\"k\": v, a: x{y,z}, b: 2}", "MalformedInlineCompound");
}

// --- negative: crossed ] after mid-scalar [ (group B) -----------------------

#[test]
fn midvalue_crossed_bracket_rejected() {
    // `x[y,z]` mid-scalar: the crossed `]` is not a matching `}` closer.
    check_kind("{a: x[y,z], b: 2}", "UnterminatedInlineCompound");
    check_kind("{a: x[y,z], b: 2}\n", "UnterminatedInlineCompound");
    check_kind("cfg: {a: x[y,z], b: 2}", "UnterminatedInlineCompound");
    check_kind("{\"k\": v, a: x[y,z], b: 2}", "UnterminatedInlineCompound");
}

// --- negative: array bodies share the fast splitter (group C) ---------------

#[test]
fn array_body_midvalue_rejected() {
    // Same rule inside array bodies.
    check_kind("[{a: x{y,z}, b: 2}]", "UnterminatedInlineCompound");
    check_kind("[a, x{y,z}, 2]", "UnterminatedInlineCompound");
    check_kind("[x{y,z}, 2]", "UnterminatedInlineCompound");
}

// --- negative: thin API parity (group D) ------------------------------------

#[test]
fn thin_api_parity() {
    let a = collect("{a: x{y,z}, b: 2}").unwrap_err();
    let b = parse("{a: x{y,z}, b: 2}").unwrap_err();
    let (ak, bk) = match (&a, &b) {
        (Error::Structured(t), Error::Structured(o)) => (t.code_name(), o.code_name()),
        _ => panic!("expected structured errors"),
    };
    assert_eq!(ak, bk);
    assert_eq!(ak, "MalformedInlineCompound");

    let a = collect("{a: x[y,z], b: 2}").unwrap_err();
    let b = parse("{a: x[y,z], b: 2}").unwrap_err();
    let (ak, bk) = match (&a, &b) {
        (Error::Structured(t), Error::Structured(o)) => (t.code_name(), o.code_name()),
        _ => panic!("expected structured errors"),
    };
    assert_eq!(ak, bk);
    assert_eq!(ak, "UnterminatedInlineCompound");

    assert!(from_str::<BTreeMap<String, i32>>("{a: x{y,z}, b: 2}").is_err());
}

// --- positive: spec § 5.8.5 example (group E) -------------------------------

#[test]
fn spec_example_unbalanced_midvalue_literal() {
    let v = parse("{a: hello{world, b: x}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("x".into())));

    let v = parse("{a: hello{world, b: x}\n").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("x".into())));

    let v = parse("cfg: {a: hello{world, b: x}").unwrap();
    let obj = v
        .as_object()
        .unwrap()
        .get("cfg")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("x".into())));

    let v = parse("{\"k\": v, a: hello{world, b: x}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("k"), Some(&Value::String("v".into())));
    assert_eq!(obj.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("x".into())));

    let v = parse("[{a: hello{world, b: x}]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let obj = arr[0].as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("x".into())));
}

// --- positive: escaped mid-scalar brackets (group F) ------------------------

#[test]
fn escaped_midvalue_brackets_literal() {
    let v = parse("{a: x\\{y\\}, b: 2}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("x{y}".into())));
    assert_eq!(obj.get("b"), Some(&Value::Integer("2".into())));

    let v = parse("[{a: x\\{y\\}, b: 2}]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let obj = arr[0].as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::String("x{y}".into())));
    assert_eq!(obj.get("b"), Some(&Value::Integer("2".into())));
}

// --- positive: genuine value-start compounds still nest (group G) -----------

#[test]
fn value_start_compound_still_nests() {
    let v = parse("{a: {y: 1}, b: 2}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("y"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("b"), Some(&Value::Integer("2".into())));

    let v = parse("[{a: {y: 1}, b: 2}]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    let obj = arr[0].as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("y"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("b"), Some(&Value::Integer("2".into())));
}

// --- positive: thin + typed (group H) ---------------------------------------

#[test]
fn thin_positive_events() {
    assert_eq!(
        collect("{a: hello{world, b: x}").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("hello{world".into()),
            Ev::Key("b".into()),
            Ev::Str("x".into()),
            Ev::EndObject
        ]
    );
}

#[test]
fn typed_positive() {
    let m: BTreeMap<String, String> = from_str("{a: hello{world, b: x}").unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m["a"], "hello{world");
    assert_eq!(m["b"], "x");
}
