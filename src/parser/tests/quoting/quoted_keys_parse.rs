//! Quoted-key tests at the parse level (spec 0.7 § 5.3.3).

use crate::parser::inline::{has_quote_bytes, InlineBounds};
use crate::parser::tests::S;
use crate::value::Value;
// --- quoted keys: parse-level (spec 0.7 § 5.3.3) ----------------------------

#[test]
fn quoted_key_with_spaces() {
    let v = crate::parse("\"a b\": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a b"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_key_backtick_with_inner_quotes() {
    let v = crate::parse("`it's \"quoted\"`: 1").unwrap();
    let obj = v.as_object().unwrap();
    let key = obj.keys().next().unwrap().clone();
    assert_eq!(key, "it's \"quoted\"");
    assert_eq!(key.len(), 13);
}

#[test]
fn quoted_key_interior_not_trimmed() {
    let v = crate::parse("\" a \": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get(" a "), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_segment_in_dotted_path() {
    let v = crate::parse("a.\"b.c\".d: 1").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    let mid = a.get("b.c").unwrap().as_object().unwrap();
    assert_eq!(mid.get("d"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_adjacent_segments_decode() {
    let v = crate::parse("\"a\".\"b\": 1").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("b"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_key_comma_inside_inline_object() {
    let v = crate::parse("k: {\"a,b\": 1, c: 2}").unwrap();
    let k = v
        .as_object()
        .unwrap()
        .get("k")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(k.len(), 2);
    assert_eq!(k.get("a,b"), Some(&Value::Integer("1".into())));
    assert_eq!(k.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn quoted_key_brace_inside_inline_object() {
    let v = crate::parse("k: {\"a}b\": 1, c: 2}").unwrap();
    let k = v
        .as_object()
        .unwrap()
        .get("k")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(k.len(), 2);
    assert_eq!(k.get("a}b"), Some(&Value::Integer("1".into())));
    assert_eq!(k.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn quoted_key_brace_inside_root_inline_object() {
    let v = crate::parse("{\"a}b\": 1, c: 2}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a}b"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn quoted_key_unterminated_inline_root_is_unterminated_compound() {
    let err = crate::parse("{\"a: 1}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got: {}", other),
    }
}

#[test]
fn quoted_key_unterminated_after_established_object() {
    let err = crate::parse("y: 1\n'unterminated: 1").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::UnterminatedQuotedKey { .. }) => {}
        other => panic!("expected UnterminatedQuotedKey, got: {}", other),
    }
}

#[test]
fn quoted_key_unterminated_in_root_pair_falls_to_array() {
    // Unterminated quote swallows the colon, so the root line is
    // array-item shape (§ 5.0.1 rule 7) and the line is stored verbatim.
    let v = crate::parse("'tis the season: fa").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], Value::String("'tis the season: fa".into()));
}

#[test]
fn quoted_root_object_key() {
    let v = crate::parse("\"port\": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("port"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_key_unterminated_in_established_object_line() {
    let err = crate::parse("cfg:\n  a: 1\n  \"unterminated: 1").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::UnterminatedQuotedKey { .. }) => {}
        other => panic!("expected UnterminatedQuotedKey, got: {}", other),
    }
}

// R3-F2: quote-awareness must apply to nested Objects inside an
// Array-outer scan, not only when the OUTERMOST container is an Object.
#[test]
fn r3f2_find_matching_close_array_nested_quoted_keys() {
    use crate::parser::inline::find_matching_close;
    // `]` inside a quoted key of a nested Object must be opaque.
    assert_eq!(
        find_matching_close("[{\"x]y\": 1},2]", b'[', b']'),
        Some(13)
    );
    assert_eq!(
        find_matching_close("[{\"x}y\": 1},2]", b'[', b']'),
        Some(13)
    );
    // Unterminated quoted key swallows the rest — no matching close.
    assert_eq!(find_matching_close("[{\"x]y\": 1", b'[', b']'), None);
    // Doubly nested arrays with a quoted-key object at the bottom.
    // Input is 14 bytes; the matching closer is the last byte at index 13.
    assert_eq!(
        find_matching_close("[[{\"x]y\": 1}]]", b'[', b']'),
        Some(13)
    );
}

#[test]
fn r3f2_scan_inline_closer_array_nested_quoted_keys() {
    use crate::parser::inline::{scan_inline_closer, InlineCloserScan};
    assert!(matches!(
        scan_inline_closer("[{\"x]y\": 1},2]", b'[', b']', 1, S),
        InlineCloserScan::Found(13)
    ));
    assert!(matches!(
        scan_inline_closer("[{\"x}y\": 1},2]", b'[', b']', 1, S),
        InlineCloserScan::Found(13)
    ));
    assert!(matches!(
        scan_inline_closer("[[{\"x]y\": 1}]]", b'[', b']', 1, S),
        InlineCloserScan::Found(13)
    ));
    // Non-regression: object-outer body already tracks quotes (10 bytes; closer at index 9).
    assert!(matches!(
        scan_inline_closer("{\"x}y\": 1}", b'{', b'}', 1, S),
        InlineCloserScan::Found(9)
    ));
}

#[test]
fn r3f2_split_top_level_array_body_quoted_key_object() {
    use crate::parser::inline::{split_top_level, InlineBody};
    // The nested object (with `]` in a quoted key) must be skipped
    // wholesale: the comma inside it never splits, the one after it does.
    assert_eq!(
        split_top_level(
            "{\"x]y\": 1},2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("{\"x]y\": 1},2"),
            has_quote_bytes("{\"x]y\": 1},2".as_bytes())
        )
        .unwrap(),
        vec!["{\"x]y\": 1}", "2"]
    );
}
