//! Regression tests for review finding A3 — spec 0.7 § 3.7 / § 5.2 rule 15:
//! escape decoding must borrow when there are no escapes, and an OWNED
//! decode (≥ 1 recognised escape) must force the inline value to be a
//! `String` — never re-classified as keyword/numeric — in BOTH the owned
//! parser (`ktav::parse`) and the thin event parser (`ktav::thin`).

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{parse, Error, ObjectMap, Value};

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

fn root_key(src: &str) -> String {
    let Value::Object(map) = parse(src).unwrap() else {
        panic!("{src:?}: expected Object root");
    };
    assert_eq!(map.len(), 1, "{src:?}: expected exactly one root key");
    map.keys().next().unwrap().to_string()
}

// --- 1. owned decode forces String, never Bool (§ 5.2 rule 15) ---------------

#[test]
fn owned_decode_forces_string_not_bool() {
    let v = parse(
        r"{a: \u0074rue}
",
    )
    .unwrap();
    assert_eq!(v, obj(&[("a", Value::String("true".into()))]));
    let Value::Object(map) = v else {
        panic!("expected Object")
    };
    let a = map.get("a").unwrap();
    assert!(a.as_bool().is_none(), "decoded `true` must NOT be Bool");
}

#[test]
fn owned_decode_forces_string_not_null() {
    let v = parse(
        r"{a: \u006eull}
",
    )
    .unwrap();
    assert_eq!(v, obj(&[("a", Value::String("null".into()))]));
    let Value::Object(map) = v else {
        panic!("expected Object")
    };
    let a = map.get("a").unwrap();
    assert!(!a.is_null(), "decoded `null` must NOT be Null");
}

#[test]
fn plain_keywords_still_classify() {
    assert_eq!(
        parse("{a: true}\n").unwrap(),
        obj(&[("a", Value::Bool(true))])
    );
    assert_eq!(parse("{a: null}\n").unwrap(), obj(&[("a", Value::Null)]));
}

// --- 2. thin parser: same rule-15 force --------------------------------------

#[test]
fn thin_owned_decode_yields_str_not_bool() {
    assert_eq!(
        collect("{a: \\u0074rue}\n"),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("true".into()),
            Ev::EndObject
        ]
    );
}

// --- 3. quoted-no-escape interiors are preserved byte-exact ------------------

// Pins the borrowed quoted-interior fast path (owned parser and thin
// parser) against accidental trimming: quoted content is never trimmed
// (§ 5.3.3).
#[test]
fn quoted_interior_whitespace_preserved_both_parsers() {
    assert_eq!(root_key("\" a b \": 1\n"), " a b ");
    assert_eq!(
        collect("\" a b \": 1\n"),
        vec![
            Ev::BeginObject,
            Ev::Key(" a b ".into()),
            Ev::Integer("1".into()),
            Ev::EndObject
        ]
    );
}

// --- 4. escaped separators stay literal AND are not separators ---------------

// Bare `a\.b` / `a\:b` are covered by tests/key_escaping*.rs; here the
// QUOTED variants through the inline-object path and the escaped-colon
// quoted form.
#[test]
fn quoted_escaped_dot_is_literal_not_separator() {
    assert_eq!(
        parse(
            r"{q\.x: 1}
"
        )
        .unwrap(),
        obj(&[("q.x", Value::Integer("1".into()))])
    );
    assert_eq!(
        root_key(
            r"q\.x: 1
"
        ),
        "q.x"
    );
}

#[test]
fn thin_quoted_escaped_dot_is_literal_not_separator() {
    assert_eq!(
        collect(
            r"{q\.x: 1}
"
        ),
        vec![
            Ev::BeginObject,
            Ev::Key("q.x".into()),
            Ev::Integer("1".into()),
            Ev::EndObject
        ]
    );
}

#[test]
fn quoted_escaped_colon_decodes_in_interior() {
    assert_eq!(
        root_key(
            r#""a\:b": 1
"#
        ),
        "a:b"
    );
    assert_eq!(
        collect(
            r#""a\:b": 1
"#
        ),
        vec![
            Ev::BeginObject,
            Ev::Key("a:b".into()),
            Ev::Integer("1".into()),
            Ev::EndObject
        ]
    );
}

// --- 5. error behavior unchanged ---------------------------------------------

#[test]
fn bad_escape_sequence_error_unchanged() {
    let err = parse(
        r"{a: \x}
",
    )
    .unwrap_err();
    match &err {
        Error::Structured(ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got {other:?}"),
    }
    let err = parse_events(
        r"{a: \x}
",
        |_| {},
    )
    .unwrap_err();
    match err {
        Error::Structured(ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got {other:?}"),
    }
}

// --- 6. decoded numeric-looking text never re-classifies ---------------------

#[test]
fn decoded_numeric_looks_like_integer_but_is_string() {
    // `\u0031` is '1': the decoded body looks numeric but the owned
    // decode must force String, never Integer.
    let v = parse(
        r"{a: \u0031}
",
    )
    .unwrap();
    assert_eq!(v, obj(&[("a", Value::String("1".into()))]));
    let Value::Object(map) = v else {
        panic!("expected Object")
    };
    assert!(matches!(map.get("a"), Some(Value::String(_))));
    assert_eq!(
        collect(
            r"{a: \u0031}
"
        ),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("1".into()),
            Ev::EndObject
        ]
    );
}
