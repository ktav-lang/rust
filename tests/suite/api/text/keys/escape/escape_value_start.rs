//! Regression tests for review finding R4-F4 — spec 0.7 § 3.7 / § 5.8.5:
//! an escape processed at an inline value-start position (e.g. a leading
//! `\{` or `\[`) starts a scalar value and must CONSUME the value-start
//! position, so a later unescaped `[`/`{` in the same value is ordinary
//! literal data, not a fresh nested opener.

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

// --- 1. primary repro: escaped `{` starts a scalar (split_top_level_fast) ---

#[test]
fn escaped_brace_consumes_value_start() {
    let src = r"{a: \{[text}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("{[text".into()))]),
        "escaped-brace repro failed for {src:?}"
    );
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("{[text".into()),
            Ev::EndObject
        ]
    );
}

// --- 2. mirror case: escaped `[` ---------------------------------------------

#[test]
fn escaped_bracket_consumes_value_start() {
    let src = r"{a: \[{text}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("[{text".into()))]),
        "escaped-bracket repro failed for {src:?}"
    );
}

// --- 3. unicode escape: full \uXXXX consumed, later `[` is literal -----------

#[test]
fn unicode_escape_consumes_value_start() {
    let src = r"{a: \u0041[text}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("A[text".into()))]),
        "unicode-escape repro failed for {src:?}"
    );
}

// --- 4. nested compound: escape inside the closer scan (fast path) -----------

#[test]
fn escaped_brace_in_nested_compound() {
    let src = r"{x: {a: \{[text}}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("x", obj(&[("a", Value::String("{[text".into()))]))]),
        "nested escaped-brace repro failed for {src:?}"
    );
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("{[text".into()),
            Ev::EndObject,
            Ev::EndObject
        ]
    );
}

#[test]
fn unicode_escape_in_nested_compound() {
    let src = r"{x: {a: \u0041[text}}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("x", obj(&[("a", Value::String("A[text".into()))]))]),
        "nested unicode-escape repro failed for {src:?}"
    );
}

// --- 5. quote branch of the splitter (split_top_level slow path) -------------

// NB: the originally planned `{a: \{[text}, k: "v"}` is NOT pinned: the
// unescaped `}` inside the scalar is treated as the body's own closer
// (§ 5.2 rule 8), making the trailing `, k: ...` a MalformedInlineCompound.
// This variant keeps a quote byte in the body (forcing the slow, quote-
// tracking splitter) without a closer byte inside the escaped scalar.
#[test]
fn escaped_brace_with_quote_in_body() {
    let src = r#"{a: \{[text, "k": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", Value::String("{[text".into())),
            ("k", Value::Integer("2".into())),
        ]),
        "slow-path splitter repro failed for {src:?}"
    );
}

// --- 6. quote branch of the closer scan (scan_inline_closer slow path) -------

// Same shaping note as above: the nested variant must keep the quote byte
// (forcing the quote-tracking closer scan) but no `}` inside the scalar.
#[test]
fn escaped_brace_in_nested_compound_with_quote() {
    let src = r#"{x: {a: \{[text, "k": 2}}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[(
            "x",
            obj(&[
                ("a", Value::String("{[text".into())),
                ("k", Value::Integer("2".into())),
            ])
        )]),
        "slow-path closer-scan repro failed for {src:?}"
    );
}

// --- 7. controls: genuine value-start compounds are unaffected ---------------

#[test]
fn genuine_value_start_compounds_unaffected() {
    assert_eq!(
        parse("{a: {b: 1}}\n").unwrap(),
        obj(&[("a", obj(&[("b", Value::Integer("1".into()))]))])
    );
    assert_eq!(
        parse("{a: [1]}\n").unwrap(),
        obj(&[("a", arr(&[Value::Integer("1".into())]))])
    );
    assert_eq!(
        parse("[1, {a: 2}]\n").unwrap(),
        arr(&[
            Value::Integer("1".into()),
            obj(&[("a", Value::Integer("2".into()))])
        ])
    );
}

// --- 8. controls: escaped / dotted KEY bytes behave exactly as before --------

#[test]
fn key_escapes_and_dotted_keys_unaffected() {
    // Established by tests/key_escaping.rs (top-level): `\.` decodes to a
    // literal dot and is NOT a path separator.
    let v = parse("a\\.b: 1\n").unwrap();
    let Value::Object(map) = v else {
        panic!("expected Object root for a\\.b");
    };
    assert_eq!(map.len(), 1);
    assert_eq!(map.keys().next().unwrap().as_str(), "a.b");

    // Established by tests/key_escaping.rs: `\:` decodes to a literal colon.
    let v = parse("a\\:b: 1\n").unwrap();
    let Value::Object(map) = v else {
        panic!("expected Object root for a\\:b");
    };
    assert_eq!(map.keys().next().unwrap().as_str(), "a:b");

    // Dotted key inside an inline object nests (established by
    // `parse_inline_dotted_keys` in src/parser/tests.rs).
    assert_eq!(
        parse("{a.b: 1}\n").unwrap(),
        obj(&[("a", obj(&[("b", Value::Integer("1".into()))]))])
    );
}

// --- 9. typed API -------------------------------------------------------------

#[test]
fn typed_api() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Doc {
        a: String,
    }
    let d: Doc = from_str(
        r"{a: \{[text}
",
    )
    .unwrap();
    assert_eq!(d.a, "{[text");
}

// --- 10. error-kind pin: mid-scalar opener without escape still errors -------

#[test]
fn pins_unchanged() {
    // Without the leading escape, the `[` IS a value-start opener and the
    // unclosed compound is an error — the escape must not weaken this.
    let err = parse("{a: {[text}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
}
