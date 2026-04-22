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
