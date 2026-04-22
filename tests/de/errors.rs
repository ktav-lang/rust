//! Parser and deserializer error paths.

use ktav::{from_file, from_str, Error};
use serde::Deserialize;

use super::common::fixture;

#[test]
fn unclosed_object() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: {\n  y: 1\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("Unclosed")),
        "got: {:?}",
        err
    );
}

#[test]
fn unclosed_array() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: Vec<String>,
    }
    let err = from_str::<Cfg>("x: [\n  a\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("Unclosed")),
        "got: {:?}",
        err
    );
}

#[test]
fn mismatched_bracket() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: {\n]\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("does not match")),
        "got: {:?}",
        err
    );
}

#[test]
fn stray_close_bracket() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("}\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("without matching")),
        "got: {:?}",
        err
    );
}

#[test]
fn duplicate_key() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_str::<Cfg>("port: 80\nport: 443\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("duplicate key")),
        "got: {:?}",
        err
    );
}

#[test]
fn empty_key() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>(": value\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("Empty key")),
        "got: {:?}",
        err
    );
}

#[test]
fn key_with_special_chars() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("[foo]: bar\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("Invalid key")),
        "got: {:?}",
        err
    );
}

#[test]
fn line_without_colon_in_object() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("just-some-text\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("no ':'")),
        "got: {:?}",
        err
    );
}

#[test]
fn inline_nonempty_object() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: { a: 1 }\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("inline non-empty object")),
        "got: {:?}",
        err
    );
}

#[test]
fn inline_nonempty_array() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: Vec<String>,
    }
    let err = from_str::<Cfg>("x: [a b c]\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("inline non-empty array")),
        "got: {:?}",
        err
    );
}

#[test]
fn bracket_value_without_double_colon() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: [a-z]+\n").unwrap_err();
    assert!(
        matches!(err, Error::Syntax(ref m) if m.contains("inline")),
        "got: {:?}",
        err
    );
}

#[test]
fn empty_input_yields_missing_required_field() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_str::<Cfg>("").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("port"), "got: {}", msg);
}

#[test]
fn from_file_missing_path_is_io_error() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_file::<Cfg, _>(fixture("does_not_exist.conf")).unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}
