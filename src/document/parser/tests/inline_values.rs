//! Tests for inline compound parsing, escape-forces-String behaviour, the float domain floor, top-level inline forms, and unicode escapes.

use crate::value::Value;
// --- inline compound parsing (Phase 4) ------------------------------------

#[test]
fn parse_inline_object_single_pair() {
    let v = crate::parse("a: {name: alice}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("name"), Some(&Value::String("alice".into())));
}

#[test]
fn parse_inline_object_multiple_pairs() {
    let v = crate::parse("server: {host: localhost, port: 8080, tls: true}").unwrap();
    let obj = v.as_object().unwrap();
    let server = obj.get("server").unwrap().as_object().unwrap();
    assert_eq!(server.get("host"), Some(&Value::String("localhost".into())));
    assert_eq!(server.get("port"), Some(&Value::Integer("8080".into())));
    assert_eq!(server.get("tls"), Some(&Value::Bool(true)));
}

#[test]
fn parse_inline_array_integers() {
    let v = crate::parse("a: [1, 2, 3]").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_array().unwrap();
    assert_eq!(a[0], Value::Integer("1".into()));
    assert_eq!(a[1], Value::Integer("2".into()));
    assert_eq!(a[2], Value::Integer("3".into()));
}

#[test]
fn parse_inline_nested_objects() {
    let v = crate::parse("cfg: {outer: {middle: {inner: deep}}}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    let outer = cfg.get("outer").unwrap().as_object().unwrap();
    let middle = outer.get("middle").unwrap().as_object().unwrap();
    assert_eq!(middle.get("inner"), Some(&Value::String("deep".into())));
}

#[test]
fn parse_inline_midvalue_brace_is_literal() {
    let v = crate::parse("cfg: {a: hello{world, b: x}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("hello{world".into())));
    assert_eq!(cfg.get("b"), Some(&Value::String("x".into())));
}

/// Regression tests for the § 5.2 rules 6–9 closer scan
/// (`scan_inline_closer`): nested compounds at value positions,
/// `::`-prefixed scalar items in arrays, and quote-path (slow-path)
/// bodies must all find their matching closer instead of tripping over
/// literal braces.
#[test]
fn parse_inline_closer_scan_value_positions_and_raw_items() {
    // Array whose first item is an object (value position after `[`).
    let v = crate::parse("x: [{b: 1}]").unwrap();
    let arr = v.as_object().unwrap().get("x").unwrap().as_array().unwrap();
    assert_eq!(
        arr[0].as_object().unwrap().get("b"),
        Some(&Value::Integer("1".into()))
    );

    // Array whose first item is a nested array.
    let v = crate::parse("x: [[1]]").unwrap();
    let arr = v.as_object().unwrap().get("x").unwrap().as_array().unwrap();
    let inner = arr[0].as_array().unwrap();
    assert_eq!(inner[0], Value::Integer("1".into()));

    // Nested at two levels inside object values.
    let v = crate::parse("{a: [{b: 1, c: 2}]}").unwrap();
    let a = v.as_object().unwrap().get("a").unwrap().as_array().unwrap();
    assert_eq!(
        a[0].as_object().unwrap().get("c"),
        Some(&Value::Integer("2".into()))
    );

    // A `::`-prefixed item is a plain <inline-scalar> (§ 4: an item
    // position derives only <inline-value>; the raw `::` branch exists
    // only at inline-PAIR positions). The scalar `:: {abc` terminates at
    // the FIRST unescaped closer (R5-F3) — that `}` mismatches the `[`
    // opener, so the array has no matching closer.
    match crate::parse("x: [:: {abc}]") {
        Err(crate::Error::Structured(crate::error::ErrorKind::UnterminatedInlineCompound {
            ..
        })) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
    match crate::parse("[:: {abc}]") {
        Err(crate::Error::Structured(crate::error::ErrorKind::UnterminatedInlineCompound {
            ..
        })) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
    // Escaping the closers keeps the braces in the scalar (§ 3.7); the
    // `::` prefix stays literal content, so the scalar is `:: {abc}` →
    // String (§ 5.2), and the unescaped `]` closes the array.
    let v = crate::parse("x: [:: \\{abc\\}]").unwrap();
    let arr = v.as_object().unwrap().get("x").unwrap().as_array().unwrap();
    assert_eq!(arr[0], Value::String(":: {abc}".into()));

    // Slow path (quote bytes present): array-of-object value after a
    // quoted key/value pair.
    let v = crate::parse("{k: \"v\", arr: [{b: 1}]}").unwrap();
    let obj = v.as_object().unwrap();
    let arr = obj.get("arr").unwrap().as_array().unwrap();
    assert_eq!(
        arr[0].as_object().unwrap().get("b"),
        Some(&Value::Integer("1".into()))
    );
    // § 5.3.3 opacity is keys-only, so the `]` inside the value quotes
    // is structural; it returns depth to zero without matching the
    // body's `}`, so the body has no matching closer (§ 5.2 rule 9)
    // and the document is rejected.
    assert!(crate::parse("{a: \"x] y\", b: 1}").is_err());

    // Root-position array with an object first item.
    let v = crate::parse("[{b: 1}]").unwrap();
    assert_eq!(
        v.as_array().unwrap()[0].as_object().unwrap().get("b"),
        Some(&Value::Integer("1".into()))
    );
}

#[test]
fn parse_inline_escape_comma() {
    let v = crate::parse("a: {greeting: hello\\, world}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(
        a.get("greeting"),
        Some(&Value::String("hello, world".into()))
    );
}

#[test]
fn parse_inline_empty_value() {
    let v = crate::parse("a: {x:, y: 1}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("x"), Some(&Value::String("".into())));
    assert_eq!(a.get("y"), Some(&Value::Integer("1".into())));
}

#[test]
fn parse_inline_trailing_comma() {
    let v = crate::parse("a: {name: alice, age: 30,}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("name"), Some(&Value::String("alice".into())));
    assert_eq!(a.get("age"), Some(&Value::Integer("30".into())));
}

#[test]
fn parse_inline_no_whitespace() {
    let v = crate::parse("a: {x:1,y:2,z:3}").unwrap();
    let obj = v.as_object().unwrap();
    let a = obj.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("x"), Some(&Value::Integer("1".into())));
    assert_eq!(a.get("y"), Some(&Value::Integer("2".into())));
    assert_eq!(a.get("z"), Some(&Value::Integer("3".into())));
}

#[test]
fn parse_inline_dotted_keys() {
    let v = crate::parse("cfg: {a.b: 1, a.c: 2}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    let a = cfg.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("b"), Some(&Value::Integer("1".into())));
    assert_eq!(a.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn parse_inline_escape_newline() {
    let v = crate::parse("multiline: {body: line1\\nline2\\nline3}").unwrap();
    let obj = v.as_object().unwrap();
    let ml = obj.get("multiline").unwrap().as_object().unwrap();
    assert_eq!(
        ml.get("body"),
        Some(&Value::String("line1\nline2\nline3".into()))
    );
}

// --- 0.7 § 3.7 / § 5.2: a recognised escape forces String ----------------

#[test]
fn parse_inline_escape_forces_string_not_float() {
    let v = crate::parse("cfg: {v: 1\\.0}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("1.0".into())));
}

#[test]
fn parse_inline_escape_forces_string_not_float_exponent() {
    let v = crate::parse("cfg: {v: 1\\.e2}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("1.e2".into())));
}

#[test]
fn parse_inline_escape_forces_string_in_array_item() {
    let v = crate::parse("cfg: [1\\.0]").unwrap();
    let obj = v.as_object().unwrap();
    let arr = obj.get("cfg").unwrap().as_array().unwrap();
    assert_eq!(arr[0], Value::String("1.0".into()));
}

#[test]
fn parse_inline_unescaped_float_still_classifies_float() {
    let v = crate::parse("cfg: {v: 1.5}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::Float("1.5".into())));
}

#[test]
fn parse_inline_keywords_still_classify() {
    let v = crate::parse("cfg: {t: true, f: false, n: null}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("t"), Some(&Value::Bool(true)));
    assert_eq!(cfg.get("f"), Some(&Value::Bool(false)));
    assert_eq!(cfg.get("n"), Some(&Value::Null));
}

// --- 0.7 § 5.2 rule 14 / § 5.9.8: float domain floor (overflow→String,
// underflow→±0.0 Float) and zero canonicalisation -------------------------

#[test]
fn parse_float_positive_overflow_to_string() {
    let v = crate::parse("v: 1e9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::String("1e9999".into())));
}

#[test]
fn parse_float_negative_overflow_to_string() {
    let v = crate::parse("v: -1e9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::String("-1e9999".into())));
}

#[test]
fn parse_float_underflow_to_positive_zero() {
    let v = crate::parse("v: 1e-9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::Float("0.0".into())));
}

#[test]
fn parse_float_negative_underflow_to_negative_zero() {
    let v = crate::parse("v: -1e-9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::Float("-0.0".into())));
}

#[test]
fn parse_float_finite_literals_still_float() {
    let v = crate::parse("a: 3.14\nb: 1e6\nc: 1e-2").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Float("3.14".into())));
    assert_eq!(obj.get("b"), Some(&Value::Float("1000000.0".into())));
    assert_eq!(obj.get("c"), Some(&Value::Float("0.01".into())));
}

#[test]
fn parse_strict_float_overflow_to_string_same_as_lax() {
    // Domain exclusion is not a lossy-form mismatch, so strict does not error.
    let v = crate::parse_strict("v: 1e9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::String("1e9999".into())));
    let v = crate::parse_strict("v: -1e9999").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("v"), Some(&Value::String("-1e9999".into())));
}

#[test]
fn parse_strict_float_underflow_is_lossy_written_form() {
    // Underflow is an ordinary finite Float, so strict's canonical-form
    // check applies — distinct from overflow's domain exclusion.
    match crate::parse_strict("v: 1e-9999") {
        Err(crate::Error::Structured(crate::ErrorKind::LossyScalar { .. })) => {}
        other => panic!("expected LossyScalar, got {:?}", other),
    }
}

#[test]
fn parse_inline_float_positive_overflow_to_string() {
    let v = crate::parse("v: {x: 1e9999}").unwrap();
    let obj = v.as_object().unwrap();
    let inner = obj.get("v").unwrap().as_object().unwrap();
    assert_eq!(inner.get("x"), Some(&Value::String("1e9999".into())));
}

#[test]
fn parse_inline_float_negative_overflow_to_string() {
    let v = crate::parse("v: {x: -1e9999}").unwrap();
    let obj = v.as_object().unwrap();
    let inner = obj.get("v").unwrap().as_object().unwrap();
    assert_eq!(inner.get("x"), Some(&Value::String("-1e9999".into())));
}

#[test]
fn parse_inline_float_underflow_to_positive_zero() {
    let v = crate::parse("v: {x: 1e-9999}").unwrap();
    let obj = v.as_object().unwrap();
    let inner = obj.get("v").unwrap().as_object().unwrap();
    assert_eq!(inner.get("x"), Some(&Value::Float("0.0".into())));
}

#[test]
fn parse_inline_float_negative_underflow_to_negative_zero() {
    let v = crate::parse("v: {x: -1e-9999}").unwrap();
    let obj = v.as_object().unwrap();
    let inner = obj.get("v").unwrap().as_object().unwrap();
    assert_eq!(inner.get("x"), Some(&Value::Float("-0.0".into())));
}

#[test]
fn parse_strict_inline_float_overflow_to_string_same_as_lax() {
    let v = crate::parse_strict("v: {x: 1e9999}").unwrap();
    let obj = v.as_object().unwrap();
    let inner = obj.get("v").unwrap().as_object().unwrap();
    assert_eq!(inner.get("x"), Some(&Value::String("1e9999".into())));
}

#[test]
fn parse_float_zero_canonical_roundtrip() {
    let v = crate::parse("z: 1e-9999\nnz: -1e-9999\n").unwrap();
    let out = crate::emit_canonical(&v).unwrap();
    assert_eq!(out, "z: 0.0\nnz: -0.0\n");
    assert_eq!(crate::parse(&out).unwrap(), v);
}

#[test]
fn parse_float_overflow_string_canonical_roundtrip() {
    let v = crate::parse("v: 1e9999\nw: -1e9999\n").unwrap();
    let out = crate::emit_canonical(&v).unwrap();
    assert_eq!(out, "v:: 1e9999\nw:: -1e9999\n");
    assert_eq!(crate::parse(&out).unwrap(), v);
}

#[test]
fn parse_inline_nested_arrays() {
    let v = crate::parse("matrix: [[1, 2], [3, 4], [5, 6]]").unwrap();
    let obj = v.as_object().unwrap();
    let matrix = obj.get("matrix").unwrap().as_array().unwrap();
    assert_eq!(matrix.len(), 3);
    let first = matrix[0].as_array().unwrap();
    assert_eq!(first[0], Value::Integer("1".into()));
    assert_eq!(first[1], Value::Integer("2".into()));
}

#[test]
fn parse_inline_mixed_nested() {
    let v = crate::parse("users: [{name: alice, age: 30}, {name: bob, age: 25}]").unwrap();
    let obj = v.as_object().unwrap();
    let users = obj.get("users").unwrap().as_array().unwrap();
    assert_eq!(users.len(), 2);
    let alice = users[0].as_object().unwrap();
    assert_eq!(alice.get("name"), Some(&Value::String("alice".into())));
    assert_eq!(alice.get("age"), Some(&Value::Integer("30".into())));
}

// --- top-level inline (section 5.0.1 rules 2-5) ---------------------------

#[test]
fn parse_top_level_inline_object() {
    let v = crate::parse("{a: 1, b: hello}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("b"), Some(&Value::String("hello".into())));
}

#[test]
fn parse_top_level_inline_array() {
    let v = crate::parse("[1, 2, 3]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], Value::Integer("1".into()));
}

#[test]
fn parse_top_level_explicit_object() {
    let v = crate::parse("{\na: 1\nb: 2\n}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("b"), Some(&Value::Integer("2".into())));
}

#[test]
fn parse_top_level_explicit_array() {
    let v = crate::parse("[\nfoo\nbar\n]").unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], Value::String("foo".into()));
    assert_eq!(arr[1], Value::String("bar".into()));
}

#[test]
fn parse_orphan_after_top_level_inline() {
    let err = crate::parse("{a: 1}\norphan: line").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::OrphanLineAfterTopLevelInline { .. }) => {}
        other => panic!("expected OrphanLineAfterTopLevelInline, got: {}", other),
    }
}

// --- inline error cases ---------------------------------------------------

#[test]
fn parse_inline_unterminated_object() {
    let err = crate::parse("cfg: {a: 1, b: 2").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got: {}", other),
    }
}

#[test]
fn parse_inline_double_comma() {
    let err = crate::parse("arr: [1,, 2]").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::MalformedInlineCompound { .. }) => {}
        other => panic!("expected MalformedInlineCompound, got: {}", other),
    }
}

#[test]
fn parse_inline_bad_escape() {
    let err = crate::parse("cfg: {a: foo\\t}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_inline_backslash_at_eol() {
    let err = crate::parse("cfg: {a: foo\\").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

// --- 0.7 § 3.7 / § 3.7.1 / § 6.13: \uXXXX unicode escapes -----------------

#[test]
fn parse_unicode_escape_basic_inline() {
    let v = crate::parse("cfg: {a: \\u0041}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("A".into())));
}

#[test]
fn parse_unicode_escape_not_greedy_inline() {
    let v = crate::parse("cfg: {a: \\u00411}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("A1".into())));
}

#[test]
fn parse_unicode_escape_not_greedy_in_key() {
    let v = crate::parse("k\\u00411: v").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("kA1"), Some(&Value::String("v".into())));
}

#[test]
fn parse_unicode_escape_hex_case_insensitive() {
    let v = crate::parse("cfg: {a: \\u00e9, b: \\u00E9, c: \\uAbCd}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("\u{e9}".into())));
    assert_eq!(cfg.get("b"), Some(&Value::String("\u{e9}".into())));
    assert_eq!(cfg.get("c"), Some(&Value::String("\u{ABCD}".into())));
}

#[test]
fn parse_unicode_escape_too_few_digits() {
    let err = crate::parse("cfg: {a: \\u12}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_too_few_digits_at_value_end() {
    let err = crate::parse("cfg: [\\u12]").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_non_hex_before_fourth_digit() {
    let err = crate::parse("cfg: {a: \\u12g4}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_surrogate_pair() {
    let v = crate::parse("cfg: {a: \\uD83D\\uDE00}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("\u{1F600}".into())));
}

#[test]
fn parse_unicode_escape_lone_high_surrogate() {
    let err = crate::parse("cfg: {a: \\uD800}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_lone_high_surrogate_before_literal() {
    let err = crate::parse("cfg: {a: \\uD800x}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_high_surrogate_then_non_low_escape() {
    let err = crate::parse("cfg: {a: \\uD800\\u0041}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_high_surrogate_then_high_surrogate() {
    let err = crate::parse("cfg: {a: \\uD800\\uD801}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_lone_low_surrogate() {
    let err = crate::parse("cfg: {a: \\uDC00}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_boundaries_around_surrogate_range() {
    let v = crate::parse("cfg: {a: \\uD7FF, b: \\uE000}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("\u{D7FF}".into())));
    assert_eq!(cfg.get("b"), Some(&Value::String("\u{E000}".into())));
}

#[test]
fn parse_unicode_escape_in_key() {
    let v = crate::parse("\\u0041b: v").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("Ab"), Some(&Value::String("v".into())));
}

#[test]
fn parse_unicode_escape_decoded_dot_is_not_structural() {
    let v = crate::parse("a\\u002Eb: v").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a.b"), Some(&Value::String("v".into())));
    assert!(obj.get("a").is_none());
}

#[test]
fn parse_unicode_escape_decoded_colon_is_not_structural() {
    let v = crate::parse("\\u003A: v").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get(":"), Some(&Value::String("v".into())));
}

#[test]
fn parse_unicode_escape_malformed_in_key_still_errors() {
    let err = crate::parse("a\\u12.b: v").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_not_processed_in_plain_body() {
    let v = crate::parse("note: \\u0041").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("note"), Some(&Value::String("\\u0041".into())));
}

#[test]
fn parse_unicode_escape_not_processed_in_multiline_string() {
    let v = crate::parse("note: (\n\\u0041\n)").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("note"), Some(&Value::String("\\u0041".into())));
}

#[test]
fn parse_uppercase_u_is_not_unicode_escape() {
    let err = crate::parse("cfg: {a: \\U0041}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_forces_string_not_integer() {
    let v = crate::parse("cfg: {v: \\u0030}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("0".into())));
}

#[test]
fn parse_named_escapes_still_work_regression() {
    let v = crate::parse(
        "cfg: {a: \\\\, b: \\,, c: \\}, d: \\], e: \\{, f: \\[, g: \\n, h: \\r, i: \\., j: \\:}",
    )
    .unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("a"), Some(&Value::String("\\".into())));
    assert_eq!(cfg.get("b"), Some(&Value::String(",".into())));
    assert_eq!(cfg.get("c"), Some(&Value::String("}".into())));
    assert_eq!(cfg.get("d"), Some(&Value::String("]".into())));
    assert_eq!(cfg.get("e"), Some(&Value::String("{".into())));
    assert_eq!(cfg.get("f"), Some(&Value::String("[".into())));
    // § 4 / § 3.7 (spec 0.7): trimming is a source-matching concern and
    // happens BEFORE decoding, so `\n`/`\r` decode to real edge
    // whitespace that survives in the String value.
    assert_eq!(cfg.get("g"), Some(&Value::String("\n".into())));
    assert_eq!(cfg.get("h"), Some(&Value::String("\r".into())));
    assert_eq!(cfg.get("i"), Some(&Value::String(".".into())));
    assert_eq!(cfg.get("j"), Some(&Value::String(":".into())));
}

#[test]
fn parse_unicode_escape_high_then_malformed_low_errors() {
    let err = crate::parse("cfg: {a: \\uD800\\uZZ}").unwrap_err();
    match err {
        crate::Error::Structured(crate::ErrorKind::BadEscapeSequence { .. }) => {}
        other => panic!("expected BadEscapeSequence, got: {}", other),
    }
}

#[test]
fn parse_unicode_escape_preserves_decoded_edge_whitespace() {
    // Renamed behaviour per spec 0.7 § 4 / § 3.7: decoded edge whitespace
    // (here U+0009 tabs) is PRESERVED — only source-level edge whitespace
    // is trimmed.
    let v = crate::parse("cfg: {v: \\u0009A\\u0009}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("\tA\t".into())));
}

#[test]
fn parse_unicode_escape_interior_whitespace_preserved() {
    // § 5.2: trimming removes edge whitespace only; a decoded newline
    // in the interior survives.
    let v = crate::parse("cfg: {v: A\\nB}").unwrap();
    let obj = v.as_object().unwrap();
    let cfg = obj.get("cfg").unwrap().as_object().unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("A\nB".into())));
}
