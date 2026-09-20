//! Regression tests for review finding R3-F3 — spec § 5.3.3 quote
//! opacity is keys-only; in inline scalar value positions `"` is an
//! ordinary byte, with no special meaning.

use std::collections::HashMap;

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{Error, Value};

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

fn code_name(src: &str, via_thin: &str) {
    let thin = collect(src).unwrap_err();
    let owned = ktav::parse(src).unwrap_err();
    let (tk, ok) = match (&thin, &owned) {
        (Error::Structured(t), Error::Structured(o)) => (t.code_name(), o.code_name()),
        _ => panic!("expected structured errors"),
    };
    assert_eq!(tk, ok, "thin vs owned category mismatch for {src:?}");
    assert_eq!(tk, via_thin);
}

#[test]
fn unescaped_closer_inside_value_quotes_is_rejected() {
    let check = |src: &str, kind: &str| {
        let err = ktav::parse(src).unwrap_err();
        match &err {
            Error::Structured(ErrorKind::MalformedInlineCompound { .. })
                if kind == "MalformedInlineCompound" => {}
            Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })
                if kind == "UnterminatedInlineCompound" => {}
            other => panic!("expected {kind} for {src:?}, got {other:?}"),
        }
    };
    // The exact repro from the review finding.
    check("{x: \"a}b\"}", "MalformedInlineCompound");
    check("{x: \"a}b\"}\n", "MalformedInlineCompound");
    check("{a: \"x] y\", b: 1}", "UnterminatedInlineCompound");
    check("x: [\"a]b\"]", "MalformedInlineCompound");
    check("x: [\"a}b\"]", "UnterminatedInlineCompound");
}

#[test]
fn escaped_brace_inside_value_string_stays_literal() {
    let v = ktav::parse("{x: \"a\\}b\"}").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("x"),
        Some(&Value::String("\"a}b\"".into()))
    );

    let v = ktav::parse("x: [\"a\\]b\"]").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("x"),
        Some(&Value::Array(vec![Value::String("\"a]b\"".into())]))
    );

    let v = ktav::parse("{k: \"v\", x: [\"a\\]b\"]}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("k"), Some(&Value::String("\"v\"".into())));
    assert_eq!(
        obj.get("x"),
        Some(&Value::Array(vec![Value::String("\"a]b\"".into())]))
    );
}

#[test]
fn thin_api_matches_owned() {
    // Unescaped `}` inside value quotes: rejected, same category as owned.
    code_name("{x: \"a}b\"}", "MalformedInlineCompound");

    // Escaped `\}` stays literal in the thin scalar.
    assert_eq!(
        collect("{x: \"a\\}b\"}").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::Str("\"a}b\"".into()),
            Ev::EndObject
        ]
    );
}

#[test]
fn from_str_value_quotes() {
    assert!(ktav::from_str::<HashMap<String, String>>("{x: \"a}b\"}").is_err());
    let m: HashMap<String, String> = ktav::from_str("{x: \"a\\}b\"}").unwrap();
    assert_eq!(m["x"], "\"a}b\"");
}
