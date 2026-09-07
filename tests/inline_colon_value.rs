//! Regression tests for review round-5 finding R5-F2 — spec 0.7 § 5.8.5.
//!
//! A literal `:` that opens a value's scalar (the value's first byte is
//! `:` and it is not the first half of a `::` raw marker) must consume
//! the value position: the next `{`/`[` is literal content, never a
//! nested-compound opener. Before the fix such a colon left `value_start`
//! armed, so `{a: :[text}` mis-scanned as an unterminated nested array
//! (`UnterminatedInlineCompound`) instead of `a = ":[text"`.
//!
//! "Controls" must pass before AND after the fix.

use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, ObjectMap, Value};

#[derive(Debug, PartialEq)]
enum Ev {
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

// --- R5-F2 primary repros (fail before the fix) --------------------------------

#[test]
fn colon_scalar_start_bracket_stays_literal() {
    // 1. fast path (no quote bytes anywhere)
    let src = "{a: :[text}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String(":[text".into()))]),
        "fast-path repro failed for {src:?}"
    );

    // 2. array scope: the item's first byte is a lone `:`
    let src = "[:[text]\n";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[Value::String(":[text".into())]),
        "array repro failed for {src:?}"
    );

    // 3. quoted key forces the quote-tracking slow path
    let src = "{\"a\": :[text}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String(":[text".into()))]),
        "slow-path repro failed for {src:?}"
    );
}

#[test]
fn thin_api_events() {
    // fast path
    let src = "{a: :[text}\n";
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str(":[text".into()),
            Ev::EndObject,
        ],
        "thin events for fast-path repro failed for {src:?}"
    );

    // slow path (quoted key)
    let src = "{\"a\": :[text}\n";
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str(":[text".into()),
            Ev::EndObject,
        ],
        "thin events for slow-path repro failed for {src:?}"
    );
}

#[test]
fn typed_api() {
    use serde_json::json;

    let src = "{a: :[text}\n";
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, json!({"a": ":[text"}), "typed fast-path repro failed");

    let src = "[:[text]\n";
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, json!([":[text"]), "typed array repro failed");
}

// --- controls (must pass before AND after the fix) -----------------------------

#[test]
fn lone_colon_value_and_scalar_tail() {
    // 4. a value that is ONLY a lone colon
    let src = "{a: :}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String(":".into()))]),
        "lone-colon value failed for {src:?}"
    );

    // 5. lone `:` followed by an ordinary non-bracket character
    //    (already worked before the fix — must keep working)
    let src = "{a: :x}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String(":x".into()))]),
        "colon scalar tail failed for {src:?}"
    );

    // 6. lone-colon value in the SECOND pair, exercising the comma re-arm
    assert_eq!(
        parse("{a: 1, b: :}\n").unwrap(),
        obj(&[
            ("a", Value::Integer("1".into())),
            ("b", Value::String(":".into())),
        ]),
        "second-pair lone colon failed"
    );
}

#[test]
fn raw_marker_controls() {
    // 7. `::` raw marker at the first byte of an array item — the scan
    //    must keep `value_start` armed across the FIRST `:` so the
    //    second one forms the marker (already worked — must keep working)
    let src = "[:: x]\n";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[Value::String("x".into())]),
        "standalone raw marker failed for {src:?}"
    );

    // 8. same-kind nesting
    let src = "[[:: x]]\n";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[arr(&[Value::String("x".into())])]),
        "nested raw marker failed for {src:?}"
    );

    // 9. raw marker in a nested array value
    let src = "{a: [:: x]}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", arr(&[Value::String("x".into())]))]),
        "raw marker in nested array failed for {src:?}"
    );
}
