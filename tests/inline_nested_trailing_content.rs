//! Regression tests for the wrong-error-category bug in
//! `parse_inline_value_raw` — spec § 6.11 / § 6.12.
//!
//! When a nested inline compound's body contains a matching closer that is
//! NOT the last byte (trailing content follows it), the parser must report
//! `MalformedInlineCompound` — not `UnterminatedInlineCompound`, which is
//! reserved for the "no closer at all" case (§ 6.11).

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{parse, Error, ObjectMap, Value};

fn parse_err(text: &str) -> Error {
    parse(text).expect_err("expected parse failure")
}

fn assert_malformed(text: &str) {
    let err = parse_err(text);
    match &err {
        Error::Structured(ErrorKind::MalformedInlineCompound { .. }) => {}
        other => panic!("parse({text:?}): expected MalformedInlineCompound, got {other:?}"),
    }
}

fn assert_malformed_thin(text: &str) {
    let err =
        parse_events(text, |_ev: ParseEvent<'_>| ()).expect_err("expected thin parse failure");
    match &err {
        Error::Structured(ErrorKind::MalformedInlineCompound { .. }) => {}
        other => {
            panic!("thin::parse_events({text:?}): expected MalformedInlineCompound, got {other:?}")
        }
    }
}

fn assert_malformed_from_str(text: &str) {
    let err = ktav::from_str::<Doc>(text).expect_err("expected from_str failure");
    match &err {
        Error::Structured(ErrorKind::MalformedInlineCompound { .. }) => {}
        other => {
            panic!("ktav::from_str({text:?}): expected MalformedInlineCompound, got {other:?}")
        }
    }
}

fn expect_ok(text: &str, expected: Value) {
    match parse(text) {
        Ok(v) => assert_eq!(v, expected, "parse({text:?}) wrong value"),
        other => panic!("parse({text:?}): expected Ok, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Trailing content after a matching closer ⇒ MalformedInlineCompound (§ 6.12)
// ---------------------------------------------------------------------------

#[test]
fn owned_object_wrapper_array_value_trailing_content() {
    assert_malformed("{a: [1] junk}\n");
}

#[test]
fn owned_object_wrapper_object_value_trailing_content() {
    assert_malformed("{a: {b: 1} junk}\n");
}

#[test]
fn owned_array_item_wrapper_array_trailing_content() {
    assert_malformed("[[1] junk]\n");
}

#[test]
fn owned_array_item_wrapper_object_trailing_content() {
    assert_malformed("[{a:1} junk]\n");
}

#[test]
fn thin_object_wrapper_array_value_trailing_content() {
    assert_malformed_thin("{a: [1] junk}\n");
}

#[test]
fn thin_object_wrapper_object_value_trailing_content() {
    assert_malformed_thin("{a: {b: 1} junk}\n");
}

#[test]
fn thin_array_item_wrapper_array_trailing_content() {
    assert_malformed_thin("[[1] junk]\n");
}

#[test]
fn thin_array_item_wrapper_object_trailing_content() {
    assert_malformed_thin("[{a:1} junk]\n");
}

#[test]
fn from_str_object_wrapper_array_value_trailing_content() {
    assert_malformed_from_str("{a: [1] junk}\n");
}

#[test]
fn from_str_wellformed_control() {
    match ktav::from_str::<Doc>("{a: [1]}\n") {
        Ok(doc) => assert_eq!(doc, Doc { a: vec![1] }, "from_str wrong value"),
        other => panic!("ktav::from_str(\"{{a: [1]}}\\n\"): expected Ok, got {other:?}"),
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Doc {
    a: Vec<i64>,
}

// ---------------------------------------------------------------------------
// Controls — behavior that must not change.
// ---------------------------------------------------------------------------

#[test]
fn control_root_level_trailing_content_is_malformed() {
    // Root-level path already reports the correct category.
    assert_malformed("a: [1] junk\n");
}

#[test]
fn control_wellformed_object_wrapper_array_value() {
    let mut inner = ObjectMap::default();
    inner.insert("a".into(), Value::Array(vec![Value::Integer("1".into())]));
    expect_ok("{a: [1]}\n", Value::Object(inner));
}

#[test]
fn control_wellformed_object_wrapper_object_value() {
    let mut inner_obj = ObjectMap::default();
    inner_obj.insert("b".into(), Value::Integer("1".into()));
    let mut outer = ObjectMap::default();
    outer.insert("a".into(), Value::Object(inner_obj));
    expect_ok("{a: {b: 1}}\n", Value::Object(outer));
}
