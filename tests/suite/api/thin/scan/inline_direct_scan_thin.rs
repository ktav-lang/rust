//! Regression tests for the A2 step-2 direct inline-compound scanner
//! (`thin/inline_emit.rs`): keyed inline compounds are scanned straight
//! into a reusable staging `Vec<Event>` before dotted-key
//! reconciliation and path registration, preserving error precedence;
//! child paths then register via `register_inline_child_paths` and the
//! staged events flush after the synthetic prefix events.

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
fn inline_event_stream_shape() {
    assert_eq!(
        collect("a: {x: 1, y: [2, 3], z:: w}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::Integer("1".into()),
            Ev::Key("y".into()),
            Ev::BeginArray,
            Ev::Integer("2".into()),
            Ev::Integer("3".into()),
            Ev::EndArray,
            Ev::Key("z".into()),
            Ev::Str("w".into()),
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn inline_dotted_keys_merge_into_one_subtree() {
    assert_eq!(
        collect("a: {b.c: 1, b.d: 2}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("b".into()),
            Ev::BeginObject,
            Ev::Key("c".into()),
            Ev::Integer("1".into()),
            Ev::Key("d".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn inline_children_visible_to_dotted_reentry_duplicate() {
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
fn inline_children_block_deeper_dotted_descent() {
    let err = collect("a: {x: 1}\na.x.y: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { path, kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
            assert_eq!(path, "a.x.y");
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {x: 1}\na.x.y: 2\n", "KeyPathConflict");
}

#[test]
fn inline_registered_object_insert_dotted_parity() {
    let err = collect("a: {b.c: 1}\na.b: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {b.c: 1}\na.b: 2\n", "DuplicateKey");
}

#[test]
fn array_leaf_blocks_descent() {
    // Arrays are leaves: inline array items register no interior paths,
    // so descending into `l` from outside is BlockedByValue.
    let err = collect("l: [{x: 1}]\nl.x: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { path, kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
            assert_eq!(path, "l.x");
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("l: [{x: 1}]\nl.x: 2\n", "KeyPathConflict");
}

#[test]
fn inline_internal_duplicate_beats_outer_conflict() {
    // The outer key `a` conflicts (Object-onto-Leaf) AND the inline body
    // has an internal duplicate — the INLINE-INTERNAL duplicate wins
    // because the scan completes before registration.
    let err = collect("a: 1\na: {x: 1, x: 2}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: 1\na: {x: 1, x: 2}\n", "DuplicateKey");
}

#[test]
fn inline_internal_duplicate_beats_reconcile_empty_key() {
    // Inline-internal precedence over the dotted-key `a..b` EmptyKey
    // that reconciliation would raise.
    let err = collect("a..b: {x: 1, x: 2}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "x");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a..b: {x: 1, x: 2}\n", "DuplicateKey");
}

#[test]
fn reconcile_empty_key_after_valid_inline() {
    let err = collect("a..b: {x: 1}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::EmptyKey { .. }) => {}
        other => panic!("expected EmptyKey, got {other:?}"),
    }
    code_name("a..b: {x: 1}\n", "EmptyKey");
}

#[test]
fn inline_internal_outcome_table() {
    // Descending through leaf `b` inside the inline object.
    let err = collect("a: {b: 1, b.c: 2}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, path, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
            assert_eq!(path, "b.c");
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {b: 1, b.c: 2}\n", "KeyPathConflict");

    let err = collect("a: {b.c: 1, b: 2}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, path, .. }) => {
            assert_eq!(
                kind,
                ConflictKind::Overwrite {
                    existing: "object",
                    new_kind: "integer",
                }
            );
            assert_eq!(path, "b");
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {b.c: 1, b: 2}\n", "KeyPathConflict");

    let err = collect("a: {b: {c: 1}, b: 2}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, path, .. }) => {
            assert_eq!(
                kind,
                ConflictKind::Overwrite {
                    existing: "object",
                    new_kind: "integer",
                }
            );
            assert_eq!(path, "b");
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
    code_name("a: {b: {c: 1}, b: 2}\n", "KeyPathConflict");

    // Object onto object is the DuplicateKey arm.
    let err = collect("a: {b: {c: 1}, b: {d: 2}}\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    code_name("a: {b: {c: 1}, b: {d: 2}}\n", "DuplicateKey");

    let evs = collect("a: {b: {c: 1}, b.d: 2}\n").unwrap();
    assert!(evs.windows(6).any(|w| w
        == [
            Ev::Key("b".into()),
            Ev::BeginObject,
            Ev::Key("c".into()),
            Ev::Integer("1".into()),
            Ev::Key("d".into()),
            Ev::Integer("2".into()),
        ]));
}

#[test]
fn inline_root_stream_and_orphan() {
    assert_eq!(
        collect("{x: 1}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("x".into()),
            Ev::Integer("1".into()),
            Ev::EndObject,
        ]
    );
    let err = collect("{x: 1}\ny: 2\n").unwrap_err();
    match err {
        Error::Structured(ErrorKind::OrphanLineAfterTopLevelInline { .. }) => {}
        other => panic!("expected OrphanLineAfterTopLevelInline, got {other:?}"),
    }
    code_name("{x: 1}\ny: 2\n", "OrphanLineAfterTopLevelInline");
}

#[test]
fn deep_inline_matches_owned_error_exactly() {
    let src = format!("k: {}1{}", "{a: ".repeat(129), "}".repeat(129));
    let thin = collect(&src).unwrap_err();
    let owned = ktav::parse(&src).unwrap_err();
    let (t, o) = match (&thin, &owned) {
        (Error::Structured(t), Error::Structured(o)) => (t, o),
        _ => panic!("expected structured errors"),
    };
    assert_eq!(format!("{t:?}"), format!("{o:?}"));
    match t {
        ErrorKind::MalformedInlineCompound { detail, .. } => {
            assert_eq!(detail, "nesting depth exceeds limit (128)");
        }
        other => panic!("expected MalformedInlineCompound, got {other:?}"),
    }
}

#[test]
fn escaped_and_canonical_scalars_borrow_or_alloc_identically() {
    assert_eq!(
        collect("a: {s: \\{x\\}, i: 0x1F, f: 1.50, n: null}\n").unwrap(),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginObject,
            Ev::Key("s".into()),
            Ev::Str("{x}".into()),
            Ev::Key("i".into()),
            Ev::Integer("31".into()),
            Ev::Key("f".into()),
            Ev::Float("1.5".into()),
            Ev::Key("n".into()),
            Ev::Null,
            Ev::EndObject,
            Ev::EndObject,
        ]
    );
}

#[test]
fn reopened_inline_key_merges_in_from_str() {
    assert!(collect("a: {x: 1}\na.y: 2\n").is_ok());
    let v: serde_json::Value = ktav::from_str("a: {x: 1}\na.y: 2\n").unwrap();
    assert_eq!(v, serde_json::json!({"a": {"x": 1, "y": 2}}));
}
