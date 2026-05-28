//! Tests for `ktav::to_string_force_strings` — coerces every scalar
//! (Integer / Float / Bool / Null) to a String and emits the result
//! using the standard render. The output round-trips back through
//! the parser as the same set of String scalars (no typed markers,
//! no keywords).

use ktav::{to_string_force_strings, Value};

fn coerce_then_reparse(src: &str) -> Value {
    let parsed = ktav::parse(src).unwrap();
    let dumped = to_string_force_strings(&parsed).unwrap();
    ktav::parse(&dumped).unwrap()
}

#[test]
fn integer_becomes_string_after_round_trip() {
    let v = coerce_then_reparse("port: 8080\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    let port = map.get("port").unwrap();
    assert!(matches!(port, Value::String(s) if s == "8080"));
}

#[test]
fn float_becomes_string_after_round_trip() {
    let v = coerce_then_reparse("ratio: 0.5\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    let ratio = map.get("ratio").unwrap();
    assert!(matches!(ratio, Value::String(s) if s == "0.5"));
}

#[test]
fn bool_becomes_string_after_round_trip() {
    let v = coerce_then_reparse("flag: true\nother: false\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    assert!(matches!(map.get("flag"), Some(Value::String(s)) if s == "true"));
    assert!(matches!(map.get("other"), Some(Value::String(s)) if s == "false"));
}

#[test]
fn null_becomes_string_after_round_trip() {
    let v = coerce_then_reparse("missing: null\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    assert!(matches!(map.get("missing"), Some(Value::String(s)) if s == "null"));
}

#[test]
fn string_passes_through_unchanged() {
    let parsed = ktav::parse("name: alice\n").unwrap();
    let out = to_string_force_strings(&parsed).unwrap();
    // Plain string with no special chars renders as `key: value` —
    // the canonical short form. Round-trip is preserved.
    assert_eq!(out, "name: alice\n");
}

#[test]
fn nested_object_preserved() {
    let v = coerce_then_reparse("server: {\n    host: localhost\n    port: 8080\n}\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    let server = match map.get("server").unwrap() {
        Value::Object(m) => m,
        _ => panic!("expected nested Object"),
    };
    assert!(matches!(server.get("host"), Some(Value::String(s)) if s == "localhost"));
    assert!(matches!(server.get("port"), Some(Value::String(s)) if s == "8080"));
}

#[test]
fn nested_array_coerces_items() {
    let v = coerce_then_reparse("items: [\n    1\n    2\n    3.14\n]\n");
    let map = match v {
        Value::Object(m) => m,
        _ => panic!("expected Object"),
    };
    let items = match map.get("items").unwrap() {
        Value::Array(a) => a,
        _ => panic!("expected nested Array"),
    };
    assert_eq!(items.len(), 3);
    assert!(items.iter().all(|i| matches!(i, Value::String(_))));
}

#[test]
fn top_level_array_force_strings() {
    let v = coerce_then_reparse("1\n2.5\ntrue\nplain\n");
    let items = match v {
        Value::Array(a) => a,
        _ => panic!("expected top-level Array"),
    };
    assert_eq!(items.len(), 4);
    assert!(items.iter().all(|i| matches!(i, Value::String(_))));
}

#[test]
fn idempotent_after_one_pass() {
    let original = ktav::parse("port: 8080\nflag: true\nname: alice\n").unwrap();
    let once = to_string_force_strings(&original).unwrap();
    let twice = to_string_force_strings(&ktav::parse(&once).unwrap()).unwrap();
    // Once-coerced text reparses to all-String; coercing again
    // produces the same text.
    assert_eq!(once, twice);
}
