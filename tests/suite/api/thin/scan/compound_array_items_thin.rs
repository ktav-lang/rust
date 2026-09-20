//! Regression tests for review finding R2-F1 — bare compound array
//! items (anonymous Object/Array elements) must survive the § 5.3.2
//! reopen-merge pass in `src/thin/merge.rs`. Pre-fix, an anonymous
//! compound's Begin pushed no merge stack frame while its closer popped
//! one, corrupting the merged stream: `ktav::from_str` panicked with
//! "stream is balanced; root frame always open" or failed with
//! "expected Key or EndObject in map, got Some(EndArray)" on VALID
//! documents.

use serde::Deserialize;

#[test]
fn from_str_array_in_array_with_reopen() {
    let v: serde_json::Value = ktav::from_str("a.x: 1\nitems: [[2]]\na.y: 3\n").unwrap();
    assert_eq!(
        v,
        serde_json::json!({"a": {"x": 1, "y": 3}, "items": [[2]]})
    );
}

#[test]
fn from_str_object_in_array_with_reopen() {
    let v: serde_json::Value = ktav::from_str("a.x: 1\nitems: [{b: 2}]\na.y: 3\n").unwrap();
    assert_eq!(
        v,
        serde_json::json!({"a": {"x": 1, "y": 3}, "items": [{"b": 2}]})
    );
}

#[test]
fn from_str_reopen_inside_root_array() {
    let v: serde_json::Value =
        ktav::from_str("[\n  {\n    a.x: 1\n    s: 2\n    a.y: 3\n  }\n]\n").unwrap();
    assert_eq!(v, serde_json::json!([{"a": {"x": 1, "y": 3}, "s": 2}]));
}

#[test]
fn from_str_compound_item_in_other_branch_than_reopen() {
    let v: serde_json::Value = ktav::from_str("a.x: 1\ns: 0\na.y: 3\nitems: [[2]]\n").unwrap();
    assert_eq!(
        v,
        serde_json::json!({"a": {"x": 1, "y": 3}, "s": 0, "items": [[2]]})
    );
}

#[test]
fn from_str_compound_items_no_reopen_fast_path() {
    let v: serde_json::Value = ktav::from_str("items: [[1], {b: 2}]\nz: 3\n").unwrap();
    assert_eq!(v, serde_json::json!({"items": [[1], {"b": 2}], "z": 3}));
}

#[derive(Debug, Deserialize, PartialEq)]
struct Doc {
    a: A,
    items: Vec<Vec<i64>>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct A {
    x: i64,
    y: i64,
}

#[test]
fn typed_deserialize_array_in_array_with_reopen() {
    let doc: Doc = ktav::from_str("a.x: 1\nitems: [[2]]\na.y: 3\n").unwrap();
    assert_eq!(
        doc,
        Doc {
            a: A { x: 1, y: 3 },
            items: vec![vec![2]],
        }
    );
}

#[test]
fn owned_parse_parity() {
    for doc in [
        "a.x: 1\nitems: [[2]]\na.y: 3\n",
        "a.x: 1\nitems: [{b: 2}]\na.y: 3\n",
        "[\n  {\n    a.x: 1\n    s: 2\n    a.y: 3\n  }\n]\n",
        "a.x: 1\ns: 0\na.y: 3\nitems: [[2]]\n",
    ] {
        assert!(ktav::parse(doc).is_ok(), "doc should be valid: {doc:?}");
    }
}
