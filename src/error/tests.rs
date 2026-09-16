//! Unit tests for the `Error` type.

use std::io;

use super::Error;

#[test]
fn display_formats_syntax_error() {
    let e = Error::Syntax("boom".into());
    assert!(format!("{}", e).contains("Syntax error: boom"));
}

#[test]
fn display_formats_message_error() {
    let e = Error::Message("bad value".into());
    assert_eq!(format!("{}", e), "bad value");
}

#[test]
fn display_formats_io_error() {
    let e = Error::Io(io::Error::new(io::ErrorKind::NotFound, "nope"));
    assert!(format!("{}", e).contains("I/O error"));
}

#[test]
fn from_io_error_wraps() {
    let io_err = io::Error::new(io::ErrorKind::Other, "x");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn ser_error_custom_produces_message() {
    use serde::ser::Error as _;
    let e = Error::custom("map keys must be strings");
    assert!(matches!(e, Error::Message(_)));
}

#[test]
fn de_error_custom_produces_message() {
    use serde::de::Error as _;
    let e = Error::custom("missing field");
    assert!(matches!(e, Error::Message(_)));
}

#[test]
fn envelope_message_body_round_trip() {
    use super::ErrorEnvelope;
    let text = "input JSON: expected value at line 1 column 1";
    let e = Error::Message(text.to_string());
    let env = ErrorEnvelope::from_error(&e, "");
    let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
    assert_eq!(v["error"], "Message");
    assert_eq!(v["body"], text);
    assert_eq!(v["reason"], serde_json::Value::Null);
    assert_eq!(v["line"], serde_json::Value::Null);
    assert_eq!(v["line_text"], serde_json::Value::Null);
    assert_eq!(v["span"], serde_json::Value::Null);
    assert_eq!(v["path"], serde_json::Value::Null);
    assert_eq!(v["canonical"], serde_json::Value::Null);
    assert_eq!(v["spec_section"], serde_json::Value::Null);
}

#[test]
fn envelope_syntax_body_round_trip() {
    use super::ErrorEnvelope;
    let text = "legacy parse failure detail";
    let e = Error::Syntax(text.to_string());
    let env = ErrorEnvelope::from_error(&e, "");
    let v: serde_json::Value = serde_json::from_str(&env.to_json()).unwrap();
    assert_eq!(v["error"], "Syntax");
    assert_eq!(v["body"], text);
    assert_eq!(v["span"], serde_json::Value::Null);
    assert_eq!(v["spec_section"], serde_json::Value::Null);
}
