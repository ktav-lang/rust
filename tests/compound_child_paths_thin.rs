//! Regression tests for review finding R2-F2 — a compound value's
//! INTERNAL key paths (inline `a: {x: 1}` and explicit multi-line
//! `a: {` … `}`) must be registered into the enclosing frame's
//! persistent table (spec 0.7 § 5.3.2 / § 6.3), so a later dotted key
//! (`a.x: 2`) collides exactly like the owned parser. Also covers the
//! `insert_dotted` parity fix: an occupied final segment of a DOTTED
//! key is always `DuplicateKey`, regardless of shape.

use ktav::error::{ConflictKind, ErrorKind};
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

/// Thin-vs-owned category parity: both parsers must agree on the error
/// category (`code_name`) for `src`.
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
fn inline_object_child_collision() {
    let err = collect("a: {x: 1}\na.x: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 2);
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {x: 1}\na.x: 2\n", "DuplicateKey");
}

#[test]
fn inline_object_descend_through_leaf() {
    let err = collect("a: {x: 1}\na.x.y: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict {
            line, path, kind, ..
        }) => {
            assert_eq!(line, 2);
            assert_eq!(path, "a.x.y");
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {x: 1}\na.x.y: 2\n", "KeyPathConflict");
}

#[test]
fn explicit_object_child_collision() {
    let err = collect("a: {\n    x: 1\n}\na.x: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 4);
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {\n    x: 1\n}\na.x: 2\n", "DuplicateKey");
}

#[test]
fn inline_child_collision_after_sibling() {
    let err = collect("a: {x: 1}\ns: 0\na.x: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 3);
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {x: 1}\ns: 0\na.x: 2\n", "DuplicateKey");
}

#[test]
fn explicit_child_collision_after_sibling() {
    let err = collect("a: {\n    x: 1\n}\ns: 0\na.x: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 5);
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {\n    x: 1\n}\ns: 0\na.x: 2\n", "DuplicateKey");
}

#[test]
fn inline_nested_object_collisions() {
    let err = collect("a: {x: {y: 1}}\na.x.y: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 2);
            assert_eq!(key, "a.x.y");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {x: {y: 1}}\na.x.y: 2\n", "DuplicateKey");

    let err = collect("a: {x: {y: 1}}\na.x.y.z: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {x: {y: 1}}\na.x.y.z: 2\n", "KeyPathConflict");
}

#[test]
fn explicit_nested_frame_collisions() {
    let doc = "a: {\n    x: {\n        y: 1\n    }\n}\n";
    let err = collect(&format!("{doc}a.x.y: 2\n")).unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a.x.y");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name(&format!("{doc}a.x.y: 2\n"), "DuplicateKey");

    let err = collect(&format!("{doc}a.x.y.z: 2\n")).unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name(&format!("{doc}a.x.y.z: 2\n"), "KeyPathConflict");

    let err = collect(&format!("{doc}a.x: 2\n")).unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name(&format!("{doc}a.x: 2\n"), "DuplicateKey");
}

#[test]
fn dotted_occupied_object_is_duplicate() {
    // insert_dotted parity: an occupied final segment of a DOTTED key
    // is DuplicateKey even when shapes differ (leaf onto Object, then
    // Object onto leaf).
    let err = collect("a.b: {}\nc: 2\na.b: 3\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 3);
            assert_eq!(key, "a.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a.b: {}\nc: 2\na.b: 3\n", "DuplicateKey");

    let err = collect("a.b: 1\nc: 2\na.b: {}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 3);
            assert_eq!(key, "a.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a.b: 1\nc: 2\na.b: {}\n", "DuplicateKey");
}

#[test]
fn array_leaf_blocks_descent_with_inline_child() {
    // Paths stop at array boundaries — the inline object inside the
    // array registers nothing outside it, and `a` itself is a leaf
    // ("array"), so descending is BlockedByValue.
    let err = collect("a: [{b: 1}]\na.b: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: [{b: 1}]\na.b: 2\n", "KeyPathConflict");

    assert!(collect("a: [{b: 1}]\nc: 2\n").is_ok());
}

#[test]
fn from_str_rejects_inline_child_collision() {
    let res: Result<serde_json::Value, _> = ktav::from_str("a: {x: 1}\na.x: 2\n");
    match res {
        Err(Error::Structured(ErrorKind::DuplicateKey { key, .. })) => {
            assert_eq!(key, "a.x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
}

#[test]
fn valid_compound_reopen_stream_unchanged() {
    // Valid merges must stay Ok with byte-identical event streams.
    assert_eq!(
        collect("a: {x: 1}\na.y: 2\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::Integer("1".into()),
            Ev::EndObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("y".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
    assert_eq!(
        collect("a.b: {\nx: 1\n}\na.b.c: 2\n").unwrap(),
        vec![
            // The grouped dotted prefix `a.b` stays open as one synthetic
            // across the explicit compound — no reopen seam is emitted.
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("b".into()),
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::Integer("1".into()),
            Ev::EndObject,
            Ev::Key("b".into()),
            Ev::BeginObject,
            Ev::Key("c".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn valid_new_sibling_after_inline_object() {
    assert!(collect("a: {x: 1}\na.y: 2\n").is_ok());
    assert!(collect("a: {}\na.y: 2\n").is_ok());
}
