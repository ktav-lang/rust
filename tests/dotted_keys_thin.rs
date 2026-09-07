//! Regression tests for review finding F3 — thin/event-path handling of
//! spec 0.7 § 5.3.2 dotted-key re-entry (a dotted key naming an already-
//! closed Object re-opens it and merges, emitting a separate
//! `Key`+`BeginObject`…`EndObject` block in document order) and § 6.3
//! conflict diagnostics, plus deserializer-side merging via
//! `ktav::from_str`.

use ktav::error::{ConflictKind, ErrorKind};
use ktav::thin::{parse_events, ParseEvent};
use ktav::Error;
use serde::Deserialize;

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
fn events_reopen_after_sibling_merges() {
    assert_eq!(
        collect("a.b: 1\nc: 2\na.d: 3\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("b".into()),
            Ev::Integer("1".into()),
            Ev::EndObject,
            Ev::Key("c".into()),
            Ev::Integer("2".into()),
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("d".into()),
            Ev::Integer("3".into()),
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn events_reopen_after_explicit_object_merges() {
    assert_eq!(
        collect("a: {\n    x: 1\n}\na.y: 2\n").unwrap(),
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
}

#[test]
fn events_reopen_after_empty_object_merges() {
    assert_eq!(
        collect("a: {}\na.y: 2\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::EndObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("y".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn conflict_duplicate_leaf_via_dotted_reopen() {
    // Re-opening with the same leaf must stay a DuplicateKey even though
    // the dotted path is re-entered after a sibling closed the object.
    let err = collect("a.b: 1\nc: 2\na.b: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { line, key, .. }) => {
            assert_eq!(line, 3);
            assert_eq!(key, "a.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a.b: 1\nc: 2\na.b: 2\n", "DuplicateKey");
}

#[test]
fn conflict_descend_into_scalar_after_reopen() {
    // `a.b: 1` then `a.b.c: 2` descends through the scalar `b` — must
    // be BlockedByValue even across the sibling-close/re-open seam
    // (this state was silently lost before the F3 fix).
    let err = collect("a.b: 1\nc: 2\na.b.c: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict {
            line, path, kind, ..
        }) => {
            assert_eq!(line, 3);
            assert_eq!(path, "a.b.c");
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a.b: 1\nc: 2\na.b.c: 2\n", "KeyPathConflict");
}

#[test]
fn conflict_scalar_overwrites_object_after_reopen() {
    let err = collect("a.b: 1\nc: 2\na: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { line, kind, .. }) => {
            assert_eq!(line, 3);
            assert_eq!(
                kind,
                ConflictKind::Overwrite {
                    existing: "object",
                    new_kind: "integer",
                }
            );
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a.b: 1\nc: 2\na: 2\n", "KeyPathConflict");
}

#[test]
fn conflict_inline_object_overwrites_scalar() {
    let err = collect("a: 1\na: {}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
            assert_eq!(
                kind,
                ConflictKind::Overwrite {
                    existing: "integer",
                    new_kind: "object",
                }
            );
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: 1\na: {}\n", "KeyPathConflict");
}

#[test]
fn conflict_object_onto_object_is_duplicate() {
    let err = collect("a: {\nx: 1\n}\na: {\ny: 2\n}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {\nx: 1\n}\na: {\ny: 2\n}\n", "DuplicateKey");
}

#[test]
fn conflict_scalar_then_dotted_descend() {
    code_name("a: 1\na.b: 2\n", "KeyPathConflict");
}

#[test]
fn conflict_dotted_prefix_then_scalar() {
    code_name("a.b: 1\na: 2\n", "KeyPathConflict");
}

#[test]
fn merge_object_under_dotted_prefix_extends() {
    assert!(collect("a.b: {\nx: 1\n}\na.b.c: 2\n").is_ok());
}

#[test]
fn merge_two_interleaved_reopens() {
    assert!(collect("a.b: 1\nq.r: 2\na.d: 3\nq.s: 4\n").is_ok());
}

#[test]
fn escaped_dot_does_not_collide_with_plain_path() {
    // `a\.b` is ONE segment named literally `a.b`; `a.b.c` builds
    // `a: {b: {c: 2}}`. Segment-slice paths must never join on '.'.
    assert!(collect("a\\.b: 1\na.b.c: 2\n").is_ok());
    assert!(collect("a.b.c: 1\na\\.b: 2\n").is_ok());
}

#[test]
fn from_str_reopened_block_merges_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        b: u32,
        d: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Doc {
        a: A,
        c: u32,
    }
    let doc: Doc = ktav::from_str("a.b: 1\nc: 2\na.d: 3\n").unwrap();
    // Both fields must survive — without the merge pass, serde's
    // last-win map behaviour would silently drop `b`.
    assert_eq!(doc.a, A { b: 1, d: 3 });
    assert_eq!(doc.c, 2);
}

#[test]
fn from_str_reopened_explicit_object_merges_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct X {
        x: u32,
        y: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Doc2 {
        a: X,
    }
    let doc: Doc2 = ktav::from_str("a: {\n    x: 1\n}\na.y: 2\n").unwrap();
    assert_eq!(doc.a, X { x: 1, y: 2 });
}

#[test]
fn from_str_nested_reopened_block_merges() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Inner {
        c: u32,
        d: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Outer {
        b: Inner,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Doc {
        a: Outer,
        q: u32,
    }
    let doc: Doc = ktav::from_str("a.b.c: 1\nq: 2\na.b.d: 2\n").unwrap();
    assert_eq!(doc.a.b, Inner { c: 1, d: 2 });
    assert_eq!(doc.q, 2);
}
