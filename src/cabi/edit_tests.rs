use super::{
    dumps, edit_value, edit_value_segments, insert_value, insert_value_segments, remove_value,
    remove_value_segments, rename_key, rename_key_segments,
};
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

// --- task114: insert/remove/rename C ABI wrappers -----------------------------

/// Код ошибки из envelope без переписывания каждого негативного кейса.
trait ErrorCode {
    fn error_code(self) -> String;
}

impl ErrorCode for Result<Vec<u8>, String> {
    fn error_code(self) -> String {
        let payload = self.expect_err("ожидалась ошибка, получен успех");
        let value: Value = serde_json::from_str(&payload).unwrap();
        value["error"].as_str().unwrap().to_string()
    }
}

#[test]
fn insert_wrappers_append_object_key_with_wire_replacement() {
    // Точность выше 2^53 сохраняется внутри i64; переполнение отклоняется.
    assert_eq!(
        insert_value(b"a: 1\n", b"b", br#"{"$i":"9007199254740993"}"#).unwrap(),
        b"a: 1\nb: 9007199254740993\n"
    );
    assert_eq!(
        insert_value(b"a: 1\n", b"b", br#"{"$i":"9223372036854775808"}"#).error_code(),
        "EditUnsupported"
    );
    assert_eq!(
        insert_value_segments(b"a: 1\n", br#"["b"]"#, br#"{"$f":"-0.0"}"#).unwrap(),
        b"a: 1\nb: -0.0\n"
    );
    // Inline-родитель: сегмент перед closer, sibling нетронут.
    assert_eq!(
        insert_value(b"a: {x: 1}\n", b"a.y", br#""two""#).unwrap(),
        b"a: {x: 1, y: two}\n"
    );
}

#[test]
fn insert_wrapper_key_encodings_and_collision() {
    // Literal-ключ с точкой: textual path экранирует, emitter даёт quoted.
    assert_eq!(
        insert_value(b"a: 1\n", b"a\\.b", br#"{"$i":"2"}"#).unwrap(),
        b"a: 1\n\"a.b\": 2\n",
    );
    // Unicode-ключ через segments.
    assert_eq!(
        insert_value_segments(b"k: 1\n", "[\"ключ\"]".as_bytes(), br#"{"$i":"2"}"#).unwrap(),
        "k: 1\nключ: 2\n".as_bytes()
    );
    // Existing key — EditAlreadyExists, никогда не overwrite.
    let collision = error(
        insert_value(b"a: 1\n", b"a", b"null").unwrap_err(),
        "EditAlreadyExists",
    );
    assert!(collision["body"].is_null());
    assert!(collision["span"].is_null());
}

#[test]
fn insert_wrapper_errors_are_typed() {
    assert_eq!(
        insert_value(b"a: 1\n", b"b.c", b"null").error_code(),
        "EditNotFound"
    );
    assert_eq!(
        insert_value_segments(b"a: 1\n", b"[]", b"null").error_code(),
        "EditNotFound"
    );
    let invalid = error(
        insert_value(b"a: 1\n", b"a..b", b"null").unwrap_err(),
        "EditInvalidPath",
    );
    assert_eq!(invalid["body"], "2");
}

#[test]
fn remove_wrappers_delete_leaf_and_array_elements() {
    assert_eq!(remove_value(b"a: 1\nb: 2\n", b"a").unwrap(), b"b: 2\n");
    assert_eq!(
        remove_value_segments(b"a: [1, 2, 3]\n", br#"["a","1"]"#).unwrap(),
        b"a: [1, 3]\n"
    );
    assert_eq!(
        remove_value(b"a: 1\n", b"missing").error_code(),
        "EditNotFound"
    );
    assert_eq!(
        remove_value_segments(b"a: 1\n", b"[]").error_code(),
        "EditNotFound"
    );
}

#[test]
fn rename_wrappers_rename_in_place_and_noop_is_byte_identical() {
    assert_eq!(
        rename_key(b"a: 1\nb: 2\n", b"b", b"x").unwrap(),
        b"a: 1\nx: 2\n"
    );
    assert_eq!(
        rename_key_segments(b"a: {x: 1, y: 2}\n", br#"["a","x"]"#, b"w").unwrap(),
        b"a: {w: 1, y: 2}\n"
    );
    // No-op same key: byte-identical, включая raw-написание и CRLF.
    assert_eq!(
        rename_key(b"k:: 042\r\n", b"k", b"k").unwrap(),
        b"k:: 042\r\n"
    );
    // Коллизия с существующим соседом.
    assert_eq!(
        rename_key(b"a: 1\nb: 2\n", b"a", b"b").error_code(),
        "EditAlreadyExists"
    );
    // Unicode-ключ.
    assert_eq!(
        rename_key(b"a: 1\n", b"a", "ключ".as_bytes()).unwrap(),
        "ключ: 1\n".as_bytes()
    );
}

#[test]
fn literal_backslash_key_textual_and_segments() {
    // Ключ x\y: в source пишется x\\y (bare-escape);
    // textual path экранирует каждый backslash, segments-JSON — один.
    assert_eq!(
        rename_key(b"x\\\\y: 1\n", b"x\\\\y", b"m").unwrap(),
        b"m: 1\n"
    );
    assert_eq!(
        insert_value(b"k: 1\n", b"x\\\\y", br#"{"$i":"2"}"#).unwrap(),
        b"k: 1\nx\\\\y: 2\n",
    );
    assert_eq!(
        rename_key_segments(b"x\\\\y: 1\n", br#"["x\\y"]"#, b"m").unwrap(),
        b"m: 1\n"
    );
}

#[test]
fn segments_transport_empty_bytes_vs_json_array() {
    // Пустые transport-байты — Message (segments не JSON);
    // декодированный JSON [] — честный EditNotFound. Все три семейства.
    let empty: &[u8] = &[];
    let arr: &[u8] = br#"[]"#;
    assert_eq!(
        insert_value_segments(b"a: 1\n", empty, b"null").error_code(),
        "Message"
    );
    assert_eq!(
        insert_value_segments(b"a: 1\n", arr, b"null").error_code(),
        "EditNotFound"
    );
    assert_eq!(
        remove_value_segments(b"a: 1\n", empty).error_code(),
        "Message"
    );
    assert_eq!(
        remove_value_segments(b"a: 1\n", arr).error_code(),
        "EditNotFound"
    );
    assert_eq!(
        rename_key_segments(b"a: 1\n", empty, b"x").error_code(),
        "Message"
    );
    assert_eq!(
        rename_key_segments(b"a: 1\n", arr, b"x").error_code(),
        "EditNotFound"
    );
}
