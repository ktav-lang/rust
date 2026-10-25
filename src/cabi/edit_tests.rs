use super::{dumps, edit_value, edit_value_segments};
use serde_json::{json, Value};

fn error(payload: String, code: &str) -> Value {
    let value: Value = serde_json::from_str(&payload).unwrap();
    let keys: Vec<_> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "error",
            "reason",
            "line",
            "line_text",
            "span",
            "path",
            "body",
            "canonical",
            "spec_section",
            "message"
        ]
    );
    assert_eq!(value["error"], code);
    assert!(value["message"].is_string());
    value
}

#[test]
fn scalar_wire_replacements_keep_types_and_unrelated_bytes() {
    for (wire, expected) in [
        (r#"{"$i":"9007199254740993"}"#, "9007199254740993"),
        (r#"{"$f":"1.0"}"#, "1.0"),
        ("true", "true"),
        ("null", "null"),
        (r#""42""#, ":42"),
    ] {
        let actual = edit_value(b"## top\na: old\nb: keep\n", b"a", wire.as_bytes()).unwrap();
        let expected = if expected == ":42" {
            "## top\na:: 42\nb: keep\n".to_string()
        } else {
            format!("## top\na: {expected}\nb: keep\n")
        };
        assert_eq!(actual, expected.as_bytes());
    }
    assert!(
        dumps(b"true").is_err(),
        "ограничение document root у dumps не меняется"
    );
}

#[test]
fn textual_and_literal_segment_paths_are_distinct() {
    let source = b"\"a.b\": {x: 1}\na: {b: {x: 2}}\n";
    let replacement = br#"{"$i":"3"}"#;
    let expected = b"\"a.b\": {x: 3}\na: {b: {x: 2}}\n";
    assert_eq!(
        edit_value(source, br"a\.b.x", replacement).unwrap(),
        expected
    );
    assert_eq!(
        edit_value_segments(source, br#"["a.b","x"]"#, replacement).unwrap(),
        expected
    );
    assert_eq!(
        edit_value(source, b"a.b.x", replacement).unwrap(),
        b"\"a.b\": {x: 1}\na: {b: {x: 3}}\n"
    );
}

#[test]
fn edit_operations_share_readability_policy_and_noop() {
    let source = b"a: [old, keep]\n";
    let expected = b"a: [\n    ((\none\ntwo\n    ))\n    keep\n]\n";
    assert_eq!(
        edit_value(source, b"a.0", br#""one\ntwo""#).unwrap(),
        expected
    );
    assert_eq!(
        edit_value_segments(source, br#"["a","0"]"#, br#""one\ntwo""#).unwrap(),
        expected
    );
    assert_eq!(
        edit_value(b"n: +1\r\n", b"n", br#"{"$i":"1"}"#).unwrap(),
        b"n: +1\r\n"
    );
}

#[test]
fn edit_error_codes_and_path_offsets_do_not_invent_source_positions() {
    let invalid = error(
        edit_value("я: 1\n".as_bytes(), "я..x".as_bytes(), b"true").unwrap_err(),
        "EditInvalidPath",
    );
    assert_eq!(invalid["body"], "3");
    let missing = error(
        edit_value(b"a: 1\n", b"missing", b"null").unwrap_err(),
        "EditNotFound",
    );
    assert!(missing["body"].is_null());
    let unsupported = error(
        edit_value(b"a: {\n## keep\nx: 1\n}\n", b"a", b"null").unwrap_err(),
        "EditUnsupported",
    );
    assert!(unsupported["body"].as_str().unwrap().contains("comment"));
    for value in [invalid, missing, unsupported] {
        for key in [
            "reason",
            "line",
            "line_text",
            "span",
            "path",
            "canonical",
            "spec_section",
        ] {
            assert!(value[key].is_null(), "{key}: {value}");
        }
    }
}

#[test]
fn transport_errors_are_message_envelopes() {
    for path in [b"not json".as_slice(), b"[0]", b"[null]", b"{}"] {
        error(
            edit_value_segments(b"a: 1\n", path, b"true").unwrap_err(),
            "Message",
        );
    }
    error(
        edit_value(b"a: 1\n", &[0xff], b"true").unwrap_err(),
        "Message",
    );
    for wire in [b"not json".as_slice(), br#"{"$i":"bad"}"#] {
        error(edit_value(b"a: 1\n", b"a", wire).unwrap_err(), "Message");
    }
    error(
        edit_value_segments(b"a: 1\n", b"[]", b"true").unwrap_err(),
        "EditNotFound",
    );
}

#[test]
fn source_diagnostics_precede_invalid_transport() {
    let utf8 = error(
        edit_value(b"bad: {\r\nb: \xff", b"", b"not json").unwrap_err(),
        "InvalidUtf8",
    );
    assert_eq!(utf8["line"], 2);
    assert_eq!(utf8["span"], json!({"start":11,"end":12}));
    assert!(utf8["line_text"].is_null());
    let eof = error(
        edit_value_segments(b"a: {\r\n b: [", b"bad", b"bad").unwrap_err(),
        "UnclosedCompound",
    );
    assert_eq!(eof["line"], 2);
    assert_eq!(eof["span"], json!({"start":10,"end":11}));
    assert_eq!(eof["line_text"], " b: [");
}
