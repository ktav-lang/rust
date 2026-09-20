//! Regression tests for the slimmed thin-parser path representation:
//! the parse-wide arena now stores one `PathShape` per `NodeId`
//! (identity = `(parent NodeId, decoded segment)` pair, enforced by the
//! shared FxHashMap index — never a joined `.`-string), so a deep chain
//! of explicit objects is O(depth) node metadata, a flat object is
//! O(keys) probes, and dotted re-entry after an explicit object close
//! still merges via the shared index.

use ktav::error::{ConflictKind, ErrorKind};
use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, Error};

/// Owned mirror of `ParseEvent` (events borrow from the input, so a
/// HRTB callback cannot push them into an outer Vec — same idiom as
/// `tests/dotted_keys_thin.rs`).
#[derive(Debug, PartialEq, Eq)]
enum Ev {
    Integer(String),
    Key(String),
    BeginObject,
    EndObject,
}

fn events(src: &str) -> Vec<Ev> {
    let mut out = Vec::new();
    parse_events(src, |e| {
        out.push(match e {
            ParseEvent::Integer(s) => Ev::Integer(s.to_string()),
            ParseEvent::Key(s) => Ev::Key(s.to_string()),
            ParseEvent::BeginObject => Ev::BeginObject,
            ParseEvent::EndObject => Ev::EndObject,
            other => unreachable!("unexpected event {other:?}"),
        });
    })
    .expect("parse should succeed");
    out
}

// ---------------------------------------------------------------------------
// Deep chains
// ---------------------------------------------------------------------------

#[test]
fn deep_chain_d8_events_and_value() {
    let src = "a: {\na: {\na: {\na: {\na: {\na: {\na: {\na: {\nx: 1\n}\n}\n}\n}\n}\n}\n}\n}\n";
    let evs = events(src);
    let mut expected: Vec<Ev> = vec![Ev::BeginObject];
    for _ in 0..8 {
        expected.push(Ev::Key("a".into()));
        expected.push(Ev::BeginObject);
    }
    expected.push(Ev::Key("x".into()));
    expected.push(Ev::Integer("1".into()));
    for _ in 0..8 {
        expected.push(Ev::EndObject);
    }
    expected.push(Ev::EndObject);
    assert_eq!(evs.len(), expected.len());
    for (i, (got, want)) in evs.iter().zip(expected.iter()).enumerate() {
        assert_eq!(got, want, "event {i} mismatch");
    }
    let v: serde_json::Value = from_str(src).expect("from_str should succeed");
    assert_eq!(
        v,
        serde_json::json!({"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"x":1}}}}}}}}})
    );
}

#[test]
fn deep_chain_d32_value() {
    let mut src = String::new();
    for _ in 0..32 {
        src.push_str("a: {\n");
    }
    src.push_str("x: 1\n");
    for _ in 0..32 {
        src.push_str("}\n");
    }
    // Expected 32-level nesting, built iteratively.
    let mut expected = serde_json::json!({"x": 1});
    for _ in 0..32 {
        expected = serde_json::json!({ "a": expected });
    }
    let v: serde_json::Value = from_str(&src).expect("from_str should succeed");
    assert_eq!(v, expected);

    // Event stream: depth must return to 0 and `x` appear exactly once.
    let mut depth: i32 = 0;
    let mut max_key_x: usize = 0;
    parse_events(&src, |e| match e {
        ParseEvent::BeginObject => depth += 1,
        ParseEvent::EndObject => depth -= 1,
        ParseEvent::Key("x") => max_key_x += 1,
        _ => {}
    })
    .expect("parse_events should succeed");
    assert_eq!(depth, 0, "event stream depth must return to 0");
    assert_eq!(max_key_x, 1, "exactly one Key(\"x\")");
}

// ---------------------------------------------------------------------------
// Flat wide object
// ---------------------------------------------------------------------------

#[test]
fn flat_wide_k64_events_and_value() {
    let mut src = String::new();
    for i in 0..64 {
        src.push_str(&format!("k{i}: {i}\n"));
    }
    let evs = events(&src);
    // One BeginObject + 64 * (Key + Integer) + one EndObject.
    assert_eq!(evs.len(), 1 + 64 * 2 + 1);
    assert_eq!(evs[0], Ev::BeginObject);
    assert_eq!(evs[1], Ev::Key("k0".into()));
    assert_eq!(evs[2], Ev::Integer("0".into()));
    assert_eq!(evs[evs.len() - 3], Ev::Key("k63".into()));
    assert_eq!(evs[evs.len() - 2], Ev::Integer("63".into()));
    assert_eq!(evs[evs.len() - 1], Ev::EndObject);

    let mut expected = serde_json::Map::new();
    for i in 0..64 {
        expected.insert(format!("k{i}"), serde_json::json!(i));
    }
    let v: serde_json::Value = from_str(&src).expect("from_str should succeed");
    assert_eq!(v, serde_json::Value::Object(expected));
}

// ---------------------------------------------------------------------------
// Fold visibility: dotted re-entry after an explicit object close
// ---------------------------------------------------------------------------

#[test]
fn dotted_reentry_after_explicit_object_merges() {
    // The closed child's subtree must still be visible to the parent
    // frame's dotted descent via the shared parse-wide index.
    let src = "a: { x: 1 }\na.y: 2\n";
    let v: serde_json::Value = from_str(src).expect("from_str should succeed");
    assert_eq!(v, serde_json::json!({"a":{"x":1,"y":2}}));
}

#[test]
fn explicit_object_after_dotted_key_is_duplicate() {
    let src = "a.x: 1\na: { y: 2 }\n";
    let err = parse_events(src, |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    let err = ktav::parse(src).unwrap_err();
    assert!(matches!(
        err,
        Error::Structured(ErrorKind::DuplicateKey { .. })
    ));
}

// ---------------------------------------------------------------------------
// Diagnostic matrix
// ---------------------------------------------------------------------------

#[test]
fn diag_duplicate_leaf() {
    let err = parse_events("a: 1\na: 2\n", |_| {}).unwrap_err();
    assert!(matches!(
        err,
        Error::Structured(ErrorKind::DuplicateKey { .. })
    ));
}

#[test]
fn diag_leaf_onto_object() {
    let err = parse_events("a: 1\na: {x: 2}\n", |_| {}).unwrap_err();
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
}

#[test]
fn diag_object_onto_leaf() {
    let err = parse_events("a: {x: 1}\na: 2\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
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
}

#[test]
fn diag_dotted_final_occupied() {
    let err = parse_events("a.b: 1\na.b: 2\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, "a.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
}

#[test]
fn diag_dotted_through_leaf() {
    let err = parse_events("a: 1\na.b: 2\n", |_| {}).unwrap_err();
    match err {
        Error::Structured(ErrorKind::KeyPathConflict { kind, .. }) => {
            assert_eq!(kind, ConflictKind::BlockedByValue);
        }
        other => panic!("expected KeyPathConflict, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Identity is never a joined `.`-string
// ---------------------------------------------------------------------------

#[test]
fn literal_dots_are_not_separators() {
    // `a\.b` is a valid § 3.7 escape spelling (same ten escapes in
    // tests/key_escaping_thin.rs), but use quoted keys for extra
    // clarity on the coexistence case.
    let src = "\"a.b\": 1\na.b: 2\n";
    let v: serde_json::Value = from_str(src).expect("from_str should succeed");
    assert_eq!(v, serde_json::json!({"a.b": 1, "a": {"b": 2}}));

    // Quoted `"a.b"` and escaped `a\.b` both decode to the single
    // segment `a.b` under the same parent — same node, hence a
    // DuplicateKey.
    let src_dup = "\"a.b\": 1\na\\.b: 2\n";
    let err = parse_events(src_dup, |_| {}).unwrap_err();
    match err {
        // NOTE: the DuplicateKey payload carries the RAW key spelling
        // from the source line (`a\.b`), not the decoded segment.
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            assert_eq!(key, r"a\.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
}
