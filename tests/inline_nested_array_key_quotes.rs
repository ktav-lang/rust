//! Regression tests for review round-6 finding R6-F1 — spec 0.7 § 5.3.3 /
//! § 5.8.3 / § 5.8.4.
//!
//! R6-F1 (comma-arm key context in `find_matching_close`'s object slow
//! path): at a `,` the scanner re-armed key context unconditionally, so a
//! comma belonging to a nested ARRAY scope put the scanner in key context.
//! A quote opening the next array item was then misread as a quoted-key
//! span; the REAL key's opening quote closed that bogus span, and the `}`
//! inside the real key hit depth 0 — a valid document was rejected as
//! MalformedInlineCompound (§ 6.12). § 5.3.3 grants quoting to KEY segments
//! only, so a `"` in an Array item is ordinary content; § 5.8.3/§ 5.8.4
//! allow these items and the nested Object.
//!
//! Metamorphic cases fail until the fix lands; controls and error pins
//! must pass before AND after the fix.

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

// --- Group A: R6-F1 primary repros (fail until the fix) ------------------------

#[test]
fn object_in_array_quoted_key_after_nested_array() {
    let src = r#"[{a: [1, "x], "b}c": 2}]
"#;
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            (
                "a",
                arr(&[Value::Integer("1".into()), Value::String("\"x".into())])
            ),
            ("b}c", Value::Integer("2".into())),
        ])]),
        "array-wrapped object repro failed for {src:?}"
    );
}

#[test]
fn object_as_pair_value_quoted_key_after_nested_array() {
    let src = r#"{outer: {a: [1, "x], "b}c": 2}}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[(
            "outer",
            obj(&[
                (
                    "a",
                    arr(&[Value::Integer("1".into()), Value::String("\"x".into())])
                ),
                ("b}c", Value::Integer("2".into())),
            ]),
        )]),
        "object-as-pair-value repro failed for {src:?}"
    );
}

#[test]
fn single_quote_delimiter_variant() {
    let src = r"[{a: [1, 'x], 'b}c': 2}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            (
                "a",
                arr(&[Value::Integer("1".into()), Value::String("'x".into())])
            ),
            ("b}c", Value::Integer("2".into())),
        ])]),
        "single-quote variant failed for {src:?}"
    );
}

#[test]
fn backtick_delimiter_variant() {
    let src = r"[{a: [1, `x], `b}c`: 2}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            (
                "a",
                arr(&[Value::Integer("1".into()), Value::String("`x".into())])
            ),
            ("b}c", Value::Integer("2".into())),
        ])]),
        "backtick variant failed for {src:?}"
    );
}

#[test]
fn multi_pair_two_nested_arrays_mixed_delimiters() {
    let src = r#"[{a: [1, "x], "b}c": 2, e: [3, 'y], 'f}g': 4}]
"#;
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            (
                "a",
                arr(&[Value::Integer("1".into()), Value::String("\"x".into())])
            ),
            ("b}c", Value::Integer("2".into())),
            (
                "e",
                arr(&[Value::Integer("3".into()), Value::String("'y".into())])
            ),
            ("f}g", Value::Integer("4".into())),
        ])]),
        "multi-pair mixed-delimiter repro failed for {src:?}"
    );
}

// --- Group B: control — the SAME object unwrapped parses (pre-fix too) ---------

#[test]
fn control_unwrapped_object_parses() {
    let src = r#"{a: [1, "x], "b}c": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            (
                "a",
                arr(&[Value::Integer("1".into()), Value::String("\"x".into())])
            ),
            ("b}c", Value::Integer("2".into())),
        ]),
        "control (unwrapped object) failed for {src:?}"
    );
}

// --- Group C: serde-level concrete JSON oracle ---------------------------------

#[test]
fn serde_json_oracle() {
    let v: serde_json::Value = from_str(
        r#"[{a: [1, "x], "b}c": 2}]
"#,
    )
    .unwrap();
    assert_eq!(
        v,
        serde_json::json!([{"a": [1, "\"x"], "b}c": 2}]),
        "serde JSON for array-wrapped object diverged from spec oracle"
    );

    let v: serde_json::Value = from_str(
        r#"{outer: {a: [1, "x], "b}c": 2}}
"#,
    )
    .unwrap();
    assert_eq!(
        v,
        serde_json::json!({"outer": {"a": [1, "\"x"], "b}c": 2}}),
        "serde JSON for object-as-pair-value diverged from spec oracle"
    );
}

// --- Group D: thin API — pinned event streams ----------------------------------

#[test]
fn thin_events_control_unwrapped() {
    assert_eq!(
        collect(r#"{a: [1, "x], "b}c": 2}"#),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::Str("\"x".into()),
            Ev::EndArray,
            Ev::Key("b}c".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
        ],
        "control (unwrapped) event stream diverged"
    );
}

#[test]
fn thin_events_array_wrapped_object() {
    assert_eq!(
        collect(r#"[{a: [1, "x], "b}c": 2}]"#),
        vec![
            Ev::BeginArray,
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::Str("\"x".into()),
            Ev::EndArray,
            Ev::Key("b}c".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndArray,
        ],
        "array-wrapped event stream diverged"
    );
}

// --- Group E: controls / error pins (must pass before AND after the fix) -------

#[test]
fn control_quoted_key_with_brace_without_nested_array() {
    // Quoted key containing `}` directly after a comma, no nested array:
    // the pre-fix re-arm happened to produce the right answer here — stays Ok.
    assert_eq!(
        parse(r#"{a: 1, "b}c": 2}"#).unwrap(),
        obj(&[
            ("a", Value::Integer("1".into())),
            ("b}c", Value::Integer("2".into())),
        ]),
        "control (quoted key with brace, flat object) failed"
    );
}

#[test]
fn error_pins_unterminated_and_trailing() {
    // § 6.11: the inner object never closes.
    let err = parse(r#"[{"a": [1, "x], "b}c": 2"#).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
    // § 6.12: content after the inner object's closer.
    let err = parse(r#"[{a: [1, "x], "b}c": 2}x]"#).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::MalformedInlineCompound { .. }) => {}
        other => panic!("expected MalformedInlineCompound, got {other:?}"),
    }
}

#[test]
fn error_pin_unescaped_closer_in_nested_array_item_quotes() {
    // R3-F3 model (see inline_value_quotes.rs): value-position quotes are
    // content, so an unescaped `}` inside a nested array item is structural.
    // The splitter (scan_inline_closer) rejects this both before and after
    // the R6-F1 fix — pinning that the fix does not relax it.
    let err = parse(r#"[{a: [1, "}"], b: 2}]"#).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
}
