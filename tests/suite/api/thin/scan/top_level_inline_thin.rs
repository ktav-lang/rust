//! Regression tests for review finding F2 — explicit top-level roots
//! through the thin/event path (spec § 5.0.1): whole-document inline
//! compounds must not be double-wrapped, lone-`{`/`[`-opened roots must
//! consume at their matching close, and error categories must match the
//! owned parser (`ktav::parse`).

use std::collections::HashMap;

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::Error;

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
fn events_inline_array_root() {
    assert_eq!(
        collect("[1, 2]\n").unwrap(),
        vec![
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::Integer("2".into()),
            Ev::EndArray
        ]
    );
}

#[test]
fn events_inline_object_root() {
    assert_eq!(
        collect("{a: 1}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Integer("1".into()),
            Ev::EndObject
        ]
    );
}

#[test]
fn events_inline_empty_roots() {
    assert_eq!(collect("{}").unwrap(), vec![Ev::BeginObject, Ev::EndObject]);
    assert_eq!(collect("[]").unwrap(), vec![Ev::BeginArray, Ev::EndArray]);
}

#[test]
fn events_explicit_multiline_object_root() {
    assert_eq!(
        collect("{\n    name: alice\n    port: 8080\n}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("name".into()),
            Ev::Str("alice".into()),
            Ev::Key("port".into()),
            Ev::Integer("8080".into()),
            Ev::EndObject
        ]
    );
}

#[test]
fn events_explicit_multiline_array_root() {
    assert_eq!(
        collect("[\n    foo\n    bar\n]\n").unwrap(),
        vec![
            Ev::BeginArray,
            Ev::Str("foo".into()),
            Ev::Str("bar".into()),
            Ev::EndArray
        ]
    );
}

#[test]
fn orphan_line_after_inline_root() {
    let err = collect("{a: 1}\nother: x\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::OrphanLineAfterTopLevelInline { line: 2, .. }) => {}
        other => panic!("expected OrphanLineAfterTopLevelInline, got {other:?}"),
    }
    code_name("{a: 1}\nother: x\n", "OrphanLineAfterTopLevelInline");
}

#[test]
fn orphan_line_after_explicit_root_close() {
    code_name("{\n    a: 1\n}\nb: 2\n", "OrphanLineAfterTopLevelInline");
    // A comment after a consumed root is still fine.
    assert!(collect("{a: 1}\n## trailing comment\n").is_ok());
}

#[test]
fn from_str_vec_of_i64_inline_array_root() {
    let v: Vec<i64> = ktav::from_str("[1, 2]\n").unwrap();
    assert_eq!(v, vec![1, 2]);
}

#[test]
fn from_str_serde_json_inline_array_root() {
    let v: serde_json::Value = ktav::from_str("[1, 2]\n").unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn from_str_serde_json_inline_object_root() {
    let v: serde_json::Value = ktav::from_str("{a: 1}\n").unwrap();
    assert_eq!(v, serde_json::json!({"a": 1}));
}

#[test]
fn from_str_map_inline_object_root() {
    let m: HashMap<String, i64> = ktav::from_str("{a: 1}\n").unwrap();
    assert_eq!(m.len(), 1);
    assert_eq!(m["a"], 1);
}

#[test]
fn empty_and_comments_only_default_to_empty_object_root() {
    assert_eq!(collect("").unwrap(), vec![Ev::BeginObject, Ev::EndObject]);
    assert_eq!(
        collect("## only a comment\n").unwrap(),
        vec![Ev::BeginObject, Ev::EndObject]
    );
}

#[test]
fn bare_closer_first_line_is_unbalanced() {
    code_name("}", "UnbalancedBracket");
}

#[test]
fn explicit_root_unclosed_at_eof_closes_like_implicit() {
    assert_eq!(
        collect("{\n    a: 1\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Integer("1".into()),
            Ev::EndObject
        ]
    );
    assert!(ktav::parse("{\n    a: 1\n").is_ok());
    let m: HashMap<String, i64> = ktav::from_str("{\n    a: 1\n").unwrap();
    assert_eq!(m["a"], 1);
}

#[test]
fn explicit_root_mismatched_close_is_unbalanced() {
    code_name("[\n    a: 1\n}\n", "UnbalancedBracket");
}

#[test]
fn from_str_serde_json_explicit_multiline_object_root() {
    let v: serde_json::Value = ktav::from_str("{\n    name: alice\n    port: 8080\n}\n").unwrap();
    assert_eq!(v, serde_json::json!({"name": "alice", "port": 8080}));
}
