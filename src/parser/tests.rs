//! Unit tests for parser-internal helpers.

use super::classify::{classify_value_start, is_float_literal, try_parse_integer};
use super::insert::insert_value;
use super::validate::is_valid_key;
use super::value_start::ValueStart;

use crate::error::Span;
use crate::value::{ObjectMap, Value};

const S: Span = Span::EMPTY;

// --- validate ---------------------------------------------------------------

#[test]
fn valid_keys_accepted() {
    assert!(is_valid_key("port"));
    assert!(is_valid_key("a1"));
    assert!(is_valid_key("kebab-case"));
    assert!(is_valid_key("snake_case"));
    // Under 0.5.0, `#` is allowed inside keys
    assert!(is_valid_key("with#hash"));
    // Internal whitespace is allowed under 0.5.0
    assert!(is_valid_key("has space"));
    assert!(is_valid_key("first name"));
}

#[test]
fn invalid_keys_rejected() {
    assert!(!is_valid_key(""));
    assert!(!is_valid_key("with[bracket"));
    assert!(!is_valid_key("with]bracket"));
    assert!(!is_valid_key("with{brace"));
    assert!(!is_valid_key("with}brace"));
    // Spec 0.6.0 — `:` is permitted inside a DECODED key segment
    // (user expresses it via `\:`). `is_valid_key` runs on the
    // decoded form, so a literal `:` is no longer rejected here.
    assert!(is_valid_key("with:colon"));
    // Same for `.` — permitted in decoded form.
    assert!(is_valid_key("with.dot"));
    // Under 0.5.0+, `,` is still forbidden.
    assert!(!is_valid_key("with,comma"));
    assert!(!is_valid_key("with(paren"));
    assert!(!is_valid_key("with)paren"));
}

#[test]
fn paths_validated_segment_by_segment_via_insert() {
    // Path validation now lives inside `insert_value`; exercise it here
    // the same way the parser does.
    let mut t = ObjectMap::default();
    assert!(insert_value(&mut t, "a.b.c", Value::Null, 1, S).is_ok());

    let mut t = ObjectMap::default();
    assert!(insert_value(&mut t, "a", Value::Null, 1, S).is_ok());

    let mut t = ObjectMap::default();
    // empty segment inside the path
    assert!(insert_value(&mut t, "a..b", Value::Null, 1, S).is_err());

    let mut t = ObjectMap::default();
    // trailing dot -- final segment is empty
    assert!(insert_value(&mut t, "a.b.", Value::Null, 1, S).is_err());

    let mut t = ObjectMap::default();
    // lone dot -- two empty segments
    assert!(insert_value(&mut t, ".", Value::Null, 1, S).is_err());
}

// --- classify ---------------------------------------------------------------

#[test]
fn classify_scalar() {
    match classify_value_start("hello", 1, S, false).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "hello"),
        _ => panic!("expected Scalar"),
    }
}

#[test]
fn classify_keywords() {
    assert!(matches!(
        classify_value_start("null", 1, S, false).unwrap(),
        ValueStart::Null
    ));
    assert!(matches!(
        classify_value_start("true", 1, S, false).unwrap(),
        ValueStart::Bool(true)
    ));
    assert!(matches!(
        classify_value_start("false", 1, S, false).unwrap(),
        ValueStart::Bool(false)
    ));
}

#[test]
fn classify_case_sensitive_keywords() {
    // Only lowercase matches -- "True" / "NULL" are strings.
    match classify_value_start("True", 1, S, false).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "True"),
        _ => panic!("expected Scalar"),
    }
    match classify_value_start("NULL", 1, S, false).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "NULL"),
        _ => panic!("expected Scalar"),
    }
}

#[test]
fn classify_open_compounds() {
    assert!(matches!(
        classify_value_start("{", 1, S, false).unwrap(),
        ValueStart::OpenObject
    ));
    assert!(matches!(
        classify_value_start("[", 1, S, false).unwrap(),
        ValueStart::OpenArray
    ));
}

#[test]
fn classify_empty_inline_compounds() {
    assert!(matches!(
        classify_value_start("{}", 1, S, false).unwrap(),
        ValueStart::EmptyObject
    ));
    assert!(matches!(
        classify_value_start("[]", 1, S, false).unwrap(),
        ValueStart::EmptyArray
    ));
    assert!(matches!(
        classify_value_start("{ }", 1, S, false).unwrap(),
        ValueStart::EmptyObject
    ));
    assert!(matches!(
        classify_value_start("[  ]", 1, S, false).unwrap(),
        ValueStart::EmptyArray
    ));
}

#[test]
fn classify_inline_nonempty_accepted() {
    // Phase 4: inline compounds are now parsed into InlineValue
    match classify_value_start("{a: 1}", 1, S, false).unwrap() {
        ValueStart::InlineValue(v) => {
            assert!(v.as_object().is_some());
            let obj = v.as_object().unwrap();
            assert_eq!(obj.get("a"), Some(&Value::Integer("1".into())));
        }
        other => panic!(
            "expected InlineValue, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
    match classify_value_start("[1, 2]", 1, S, false).unwrap() {
        ValueStart::InlineValue(v) => {
            let arr = v.as_array().unwrap();
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer("1".into()));
            assert_eq!(arr[1], Value::Integer("2".into()));
        }
        other => panic!(
            "expected InlineValue, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

// --- insert_value -----------------------------------------------------------

#[test]
fn insert_simple_pair() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "port", Value::String("8080".into()), 1, S).unwrap();
    assert_eq!(t.get("port"), Some(&Value::String("8080".into())));
}

#[test]
fn insert_dotted_path_creates_intermediate_objects() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "a.b.c", Value::String("x".into()), 1, S).unwrap();
    let a = t.get("a").unwrap().as_object().unwrap();
    let b = a.get("b").unwrap().as_object().unwrap();
    assert_eq!(b.get("c"), Some(&Value::String("x".into())));
}

#[test]
fn insert_duplicate_rejected() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "x", Value::String("1".into()), 1, S).unwrap();
    let err = insert_value(&mut t, "x", Value::String("2".into()), 2, S);
    assert!(err.is_err());
}

#[test]
fn insert_scalar_then_nested_path_rejected() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "a", Value::String("leaf".into()), 1, S).unwrap();
    let err = insert_value(&mut t, "a.b", Value::String("x".into()), 2, S);
    assert!(err.is_err());
}

// --- key trimming (0.5.0 § 4) ----------------------------------------------

#[test]
fn insert_trims_key_segments() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, " port ", Value::String("80".into()), 1, S).unwrap();
    assert_eq!(t.get("port"), Some(&Value::String("80".into())));
}

#[test]
fn insert_trims_dotted_key_segments() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, " a . b . c ", Value::String("x".into()), 1, S).unwrap();
    let a = t.get("a").unwrap().as_object().unwrap();
    let b = a.get("b").unwrap().as_object().unwrap();
    assert_eq!(b.get("c"), Some(&Value::String("x".into())));
}

// --- integer literal parsing (0.5.0 § 3.6) ---------------------------------

#[test]
fn integer_decimal_basic() {
    assert_eq!(try_parse_integer("42"), Some(42));
    assert_eq!(try_parse_integer("0"), Some(0));
    assert_eq!(try_parse_integer("-7"), Some(-7));
    assert_eq!(try_parse_integer("+5"), Some(5));
}

#[test]
fn integer_decimal_underscores() {
    assert_eq!(try_parse_integer("1_000"), Some(1000));
    assert_eq!(try_parse_integer("1_000_000"), Some(1_000_000));
}

#[test]
fn integer_hex() {
    assert_eq!(try_parse_integer("0xFF"), Some(255));
    assert_eq!(try_parse_integer("0x1a"), Some(26));
    assert_eq!(try_parse_integer("-0x10"), Some(-16));
}

#[test]
fn integer_octal() {
    assert_eq!(try_parse_integer("0o77"), Some(63));
    assert_eq!(try_parse_integer("0o10"), Some(8));
}

#[test]
fn integer_binary() {
    assert_eq!(try_parse_integer("0b1010"), Some(10));
    assert_eq!(try_parse_integer("0b0"), Some(0));
}

#[test]
fn integer_rejects_bad_forms() {
    assert_eq!(try_parse_integer(""), None);
    assert_eq!(try_parse_integer("+"), None);
    assert_eq!(try_parse_integer("-"), None);
    assert_eq!(try_parse_integer("0x"), None);
    assert_eq!(try_parse_integer("0o"), None);
    assert_eq!(try_parse_integer("0b"), None);
    // Leading underscore
    assert_eq!(try_parse_integer("_42"), None);
    // Trailing underscore
    assert_eq!(try_parse_integer("42_"), None);
    // Double underscore
    assert_eq!(try_parse_integer("4__2"), None);
    // Underscore after prefix
    assert_eq!(try_parse_integer("0x_ff"), None);
    // Not a number
    assert_eq!(try_parse_integer("abc"), None);
    assert_eq!(try_parse_integer("hello"), None);
}

#[test]
fn integer_overflow_returns_none() {
    // i64::MAX + 1
    assert_eq!(try_parse_integer("9223372036854775808"), None);
}

#[test]
fn integer_i64_min() {
    assert_eq!(try_parse_integer("-9223372036854775808"), Some(i64::MIN));
}

#[test]
fn integer_i64_min_negative_prefixed_radixes() {
    // Magnitude 2^63 is exactly i64::MIN when negative, for every radix prefix.
    assert_eq!(try_parse_integer("-0x8000000000000000"), Some(i64::MIN));
    assert_eq!(
        try_parse_integer("-0o1000000000000000000000"),
        Some(i64::MIN)
    );
    assert_eq!(
        try_parse_integer("-0b1000000000000000000000000000000000000000000000000000000000000000"),
        Some(i64::MIN)
    );
}

#[test]
fn integer_prefixed_boundary_overflow_returns_none() {
    // Negative magnitude 2^63 + 1 overflows past i64::MIN in every radix.
    assert_eq!(try_parse_integer("-0x8000000000000001"), None);
    assert_eq!(try_parse_integer("-0o1000000000000000000001"), None);
    assert_eq!(
        try_parse_integer("-0b1000000000000000000000000000000000000000000000000000000000000001"),
        None
    );
    // POSITIVE magnitude 2^63 overflows i64::MAX — only the negative form is i64::MIN.
    assert_eq!(try_parse_integer("0x8000000000000000"), None);
    assert_eq!(try_parse_integer("0o1000000000000000000000"), None);
    assert_eq!(
        try_parse_integer("0b1000000000000000000000000000000000000000000000000000000000000000"),
        None
    );
}

#[test]
fn parse_negative_prefixed_i64_min_end_to_end() {
    // Full pipeline (§ 5.2 rule 13): -2^63 via each prefixed radix parses as
    // the integer i64::MIN, canonicalized to decimal text.
    for literal in [
        "-0x8000000000000000",
        "-0o1000000000000000000000",
        "-0b1000000000000000000000000000000000000000000000000000000000000000",
    ] {
        let doc = format!("x: {literal}");
        let v = crate::parse(&doc).unwrap();
        let obj = v.as_object().unwrap();
        assert_eq!(
            obj.get("x"),
            Some(&Value::Integer("-9223372036854775808".into())),
            "literal {literal}"
        );
    }
}

#[test]
fn parse_prefixed_overflow_falls_to_string_end_to_end() {
    // One past the boundary the integer parser returns None and the literal
    // falls through to String (§ 5.2 rule 13), not an error.
    for literal in ["-0x8000000000000001", "0x8000000000000000"] {
        let doc = format!("x: {literal}");
        let v = crate::parse(&doc).unwrap();
        let obj = v.as_object().unwrap();
        assert_eq!(obj.get("x"), Some(&Value::String(literal.into())));
    }
}

#[test]
fn parse_strict_negative_prefixed_i64_min() {
    // Strict mode rejects ALL prefixed radix literals as LossyScalar by
    // design (see `parse_strict` docs: `0x1A` is lossy) — even when the
    // value round-trips exactly to i64::MIN. Non-strict `parse` accepts
    // it (see parse_negative_prefixed_i64_min_end_to_end).
    let err = crate::parse_strict("x: -0x8000000000000000").unwrap_err();
    match err {
        crate::error::Error::Structured(crate::error::ErrorKind::LossyScalar { body, .. }) => {
            assert_eq!(body, "-0x8000000000000000");
        }
        other => panic!("expected LossyScalar, got {other:?}"),
    }
}

// --- float literal grammar (0.5.0 § 3.6) -----------------------------------

#[test]
fn float_with_decimal_point() {
    assert!(is_float_literal("3.14"));
    assert!(is_float_literal("0.0"));
    assert!(is_float_literal("-3.14"));
    assert!(is_float_literal("+3.14"));
}

#[test]
fn float_with_exponent_no_dot() {
    assert!(is_float_literal("1e10"));
    assert!(is_float_literal("1E10"));
    assert!(is_float_literal("-1e10"));
    assert!(is_float_literal("1e+10"));
    assert!(is_float_literal("1e-10"));
}

#[test]
fn float_with_dot_and_exponent() {
    assert!(is_float_literal("3.14e10"));
    assert!(is_float_literal("1.0E-3"));
}

#[test]
fn float_with_underscores() {
    assert!(is_float_literal("1_000.5"));
    assert!(is_float_literal("1.000_5"));
}

#[test]
fn float_rejects_bad_forms() {
    // Trailing dot
    assert!(!is_float_literal("1."));
    // Leading dot
    assert!(!is_float_literal(".5"));
    // Pure integer (no dot, no exponent)
    assert!(!is_float_literal("42"));
    // Empty exponent
    assert!(!is_float_literal("1e"));
    assert!(!is_float_literal("1e+"));
    // Not a number
    assert!(!is_float_literal("abc"));
}

// --- classify_value_start number inference (0.5.0 § 5.2 rules 13-14) -------

#[test]
fn classify_infers_integer() {
    match classify_value_start("42", 1, S, false).unwrap() {
        ValueStart::Integer(s) => assert_eq!(s, "42"),
        other => panic!("expected Integer, got {:?}", std::mem::discriminant(&other)),
    }
    match classify_value_start("-7", 1, S, false).unwrap() {
        ValueStart::Integer(s) => assert_eq!(s, "-7"),
        other => panic!("expected Integer, got {:?}", std::mem::discriminant(&other)),
    }
    // Hex produces canonical decimal
    match classify_value_start("0xFF", 1, S, false).unwrap() {
        ValueStart::Integer(s) => assert_eq!(s, "255"),
        other => panic!("expected Integer, got {:?}", std::mem::discriminant(&other)),
    }
}

#[test]
fn classify_infers_float() {
    match classify_value_start("3.14", 1, S, false).unwrap() {
        ValueStart::Float(_) => {}
        other => panic!("expected Float, got {:?}", std::mem::discriminant(&other)),
    }
    match classify_value_start("1e10", 1, S, false).unwrap() {
        ValueStart::Float(_) => {}
        other => panic!("expected Float, got {:?}", std::mem::discriminant(&other)),
    }
}

#[test]
fn classify_integer_overflow_falls_to_string() {
    // i64::MAX + 1 should be a String
    match classify_value_start("9223372036854775808", 1, S, false).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "9223372036854775808"),
        other => panic!(
            "expected Scalar (String), got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

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

// --- validate (spec 0.7 quoted keys, § 5.3.3) --------------------------------

use super::inline::{decode_key_segment, process_escapes};
use super::validate::{check_key, KeyValidity};
use crate::error::{Error, ErrorKind};

#[test]
fn check_key_quoted_segments() {
    use KeyValidity::{Empty, Invalid, Valid};

    // Bare still works.
    assert_eq!(check_key("port"), Valid);
    // Quoted: other quote chars and structural bytes are ordinary content.
    assert_eq!(check_key("\"a b\""), Valid);
    assert_eq!(check_key("`it's \"quoted\"`"), Valid);
    assert_eq!(check_key("\"a,b{c}d[e]:f.g\""), Valid);
    // Quoted content is never trimmed.
    assert_eq!(check_key("\" a \""), Valid);
    assert_eq!(check_key("\" \""), Valid);
    // Empty quoted content is EmptyKey (§ 6.5), for all three delimiters.
    assert_eq!(check_key("\"\""), Empty);
    assert_eq!(check_key("''"), Empty);
    assert_eq!(check_key("``"), Empty);
    // Content after the closer (§ 6.4 "nothing may follow the closer").
    assert_eq!(check_key("\"a\"b"), Invalid);
    assert_eq!(check_key("\"a\" \"b\""), Invalid);
    // Unterminated (normally diagnosed earlier as UnterminatedQuotedKey).
    assert_eq!(check_key("'\"unbalanced"), Invalid);
    // Bare forbidden control bytes / DEL (spec 0.7 § 4 <key-char>).
    assert_eq!(check_key("\u{1}a"), Invalid);
    assert_eq!(check_key("\u{7F}"), Invalid);
    // VT (0x0B) and FF (0x0C) are ALLOWED (new under 0.7).
    assert!(is_valid_key("a\u{B}b"));
    assert!(is_valid_key("a\u{C}b"));
    // Quoted forbidden control byte.
    assert_eq!(check_key("\"\u{1}\""), Invalid);
    // Quoted with escapes — structural bytes via escapes are fine.
    assert_eq!(check_key(r#""a\.b\u{41}""#), Valid);
}

#[test]
fn process_escapes_quote_escapes() {
    assert_eq!(process_escapes(r#"a"b"#, 1, S).unwrap(), "a\"b");
    assert_eq!(process_escapes(r"a\'b", 1, S).unwrap(), "a'b");
    assert_eq!(process_escapes(r"a`b", 1, S).unwrap(), "a`b");
    // Combined with the pre-existing escape set.
    assert_eq!(process_escapes(r"a\:\.b", 1, S).unwrap(), "a:.b");
}

#[test]
fn decode_key_segment_quoted() {
    assert_eq!(decode_key_segment("\"a b\"", 1, S).unwrap(), "a b");
    assert_eq!(
        decode_key_segment("`it's \"quoted\"`", 1, S).unwrap(),
        "it's \"quoted\""
    );
    // Interior is NOT trimmed (spec 0.7 § 5.3.3).
    assert_eq!(decode_key_segment("\" a \"", 1, S).unwrap(), " a ");
    assert_eq!(decode_key_segment(r#""a\:b""#, 1, S).unwrap(), "a:b");
    assert_eq!(decode_key_segment(r#""a\u0041b""#, 1, S).unwrap(), "aAb");
}

#[test]
fn parse_quoted_keys() {
    // Root-level quoted key.
    let v = crate::parse("\"a\": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Integer("1".into())));

    // A single space as the key — quoted content is never trimmed.
    let v = crate::parse("\" \": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get(" "), Some(&Value::Integer("1".into())));

    // Empty quoted content → EmptyKey (§ 6.5).
    let e = crate::parse("\"\": 1").unwrap_err();
    assert!(matches!(e, Error::Structured(ErrorKind::EmptyKey { .. })));

    // Content after the closer → InvalidKey (§ 6.4).
    let e = crate::parse("\"a\"b: 1").unwrap_err();
    assert!(matches!(e, Error::Structured(ErrorKind::InvalidKey { .. })));

    // Quotes NOT in first position are ordinary key chars (unchanged).
    let v = crate::parse("port\": 1").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("port\""),
        Some(&Value::Integer("1".into()))
    );
    let v = crate::parse("a\"b: 1").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("a\"b"),
        Some(&Value::Integer("1".into()))
    );

    // Value-side: quote escapes now decode inside inline scalar values.
    let v = crate::parse("cfg: {v: say \"hi\"}").unwrap();
    let cfg = v
        .as_object()
        .unwrap()
        .get("cfg")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("say \"hi\"".into())));
}

// --- quoted keys: scanners (spec 0.7 § 5.3.3) -------------------------------

use super::inline::find_matching_close;
use super::inline::find_unescaped_colon_inline;
use super::inline::split_top_level;
use super::inline::ColonScan;
use super::inline::InlineBody;
use super::inline::InlineBounds;
use super::inline::{key_is_single_segment, scan_unescaped_colon, split_key_path};

#[test]
fn quoted_colon_scan_finds_colon_outside_spans() {
    assert_eq!(scan_unescaped_colon("a: 1"), ColonScan::Found(1));
    // Colon inside quotes is skipped.
    assert_eq!(scan_unescaped_colon("\"a: b\": 1"), ColonScan::Found(6));
    assert_eq!(scan_unescaped_colon("'a' : 1"), ColonScan::Found(4));
    assert_eq!(scan_unescaped_colon("`a:b`: 1"), ColonScan::Found(5));
    // Dotted path with a quoted middle segment.
    assert_eq!(scan_unescaped_colon("a.\"b:c\".d: 1"), ColonScan::Found(9));
    // Whitespace after the dot is skipped before the quote test.
    assert_eq!(scan_unescaped_colon("a . \"b\": 1"), ColonScan::Found(7));
}

#[test]
fn quoted_colon_scan_unterminated() {
    assert_eq!(
        scan_unescaped_colon("\"unterm: 1"),
        ColonScan::UnterminatedQuote
    );
    assert_eq!(
        scan_unescaped_colon("a.\"unterm"),
        ColonScan::UnterminatedQuote
    );
}

#[test]
fn quoted_colon_scan_absent_and_escapes() {
    assert_eq!(scan_unescaped_colon("no colon"), ColonScan::Absent);
    // Escaped colon is not a separator.
    assert_eq!(scan_unescaped_colon("a\\:b: 1"), ColonScan::Found(4));
    // Escaped quote inside the span does not close it.
    assert_eq!(scan_unescaped_colon("\"a\\\"b\": 1"), ColonScan::Found(6));
    // Junk after the closer is still scanned normally.
    assert_eq!(scan_unescaped_colon("\"a\"b: 1"), ColonScan::Found(4));
}

#[test]
fn quoted_split_key_path_keeps_quotes_in_slices() {
    assert_eq!(split_key_path("a.\"b.c\".d"), vec!["a", "\"b.c\"", "d"]);
    assert_eq!(split_key_path("\"a\".\"b\""), vec!["\"a\"", "\"b\""]);
    // Escaped dot is not a separator (no quote bytes → fast path).
    assert_eq!(split_key_path("a\\.b"), vec!["a\\.b"]);
    // Post-dot whitespace stays in the slice — callers trim.
    assert_eq!(split_key_path("a. \"b\""), vec!["a", " \"b\""]);
    assert_eq!(split_key_path("\"a.b\""), vec!["\"a.b\""]);
}

#[test]
fn quoted_key_is_single_segment() {
    assert!(key_is_single_segment("\"a.b\""));
    assert!(!key_is_single_segment("a.\"b\".c"));
    // Unterminated span swallows the rest — one segment (defensive).
    assert!(key_is_single_segment("\"unterm"));
}

#[test]
fn quoted_find_unescaped_colon_inline() {
    // `}` inside a quoted key segment does not affect depth.
    assert_eq!(find_unescaped_colon_inline("\"a}b\": 1"), Some(5));
    // Span never closes — no colon found; caller maps to unterminated.
    assert_eq!(find_unescaped_colon_inline("\"a: 1"), None);
    // Value-side quotes are ignored.
    assert_eq!(find_unescaped_colon_inline("a: \"b}"), Some(1));
}

#[test]
fn quoted_split_top_level_object_mode() {
    // Comma inside a quoted KEY does not split.
    assert_eq!(
        split_top_level(
            "\"a}b\": 1, c: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a}b\": 1, c: 2")
        )
        .unwrap(),
        vec!["\"a}b\": 1", " c: 2"]
    );
    // Comma inside a quoted VALUE does split ("Keys only"): value
    // quotes are ordinary content, so both commas are split points.
    assert_eq!(
        split_top_level(
            "a: \"x,y\", b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: \"x,y\", b: 2")
        )
        .unwrap(),
        vec!["a: \"x", "y\"", " b: 2"]
    );
    // Unterminated quoted key segment.
    match split_top_level(
        "\"a: 1",
        1,
        S,
        InlineBody::Object,
        InlineBounds::for_input("\"a: 1"),
    ) {
        Err(crate::Error::Structured(crate::ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!(
            "expected UnterminatedInlineCompound, got: {:?}",
            other.err()
        ),
    }
}

#[test]
fn quoted_split_top_level_array_mode_ignores_quotes() {
    // Array bodies never track quotes (value positions — "Keys only"):
    // today's behaviour kept exactly, so the comma inside the quotes
    // splits.
    assert_eq!(
        split_top_level(
            "\"a,b\", c",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("\"a,b\", c")
        )
        .unwrap(),
        vec!["\"a", "b\"", " c"]
    );
}

#[test]
fn quoted_find_matching_close_object_mode() {
    // `}` inside a quoted key segment is opaque to balance counting.
    let input = "{\"a}b\": 1}";
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    // Unterminated span → no matching close.
    assert_eq!(find_matching_close("{\"a: 1}", b'{', b'}'), None);
    let input = "{a: 1, \"b}c\": 2}";
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    // Array-scope positions keep quotes as content (spec "Keys only"):
    // the `]` inside the quotes still closes the compound — no nested
    // object is involved.
    assert_eq!(find_matching_close("[\"a]b\"]", b'[', b']'), Some(3));
}

#[test]
fn quoted_find_matching_close_triple_nested() {
    // A `{` opens a fresh pair list at any nesting level, so quoted-key
    // recognition must be tracked per level (spec 0.7 § 5.3.3).
    let input = r#"{a: {b: {"c}d": 1}}}"#;
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    let input = r#"{b: {"c}d": 1}}"#;
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
}

#[test]
fn r3f1_split_top_level_trailing_ws_after_comma_no_phantom_segment() {
    // R3-F1: whitespace after a trailing comma sent `skip_segment_ws`
    // to EOF and the loop indexed out of bounds. The loop must end
    // without emitting a phantom whitespace-only segment.
    for tail in [" ", "\t", "\u{00a0}"] {
        let body = format!("\"a\": 1,{tail}");
        assert_eq!(
            split_top_level(
                &body,
                1,
                S,
                InlineBody::Object,
                InlineBounds::for_input(&body)
            )
            .unwrap(),
            vec!["\"a\": 1"]
        );
    }
    // Comma at EOF without whitespace: unchanged — the empty final
    // segment is still emitted and the caller accepts it.
    assert_eq!(
        split_top_level(
            "\"a\": 1,",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a\": 1,")
        )
        .unwrap(),
        vec!["\"a\": 1", ""]
    );
    // Quoted key and quoted VALUE before the trailing comma.
    assert_eq!(
        split_top_level(
            "\"a b\": 1, ",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a b\": 1, ")
        )
        .unwrap(),
        vec!["\"a b\": 1"]
    );
    assert_eq!(
        split_top_level(
            "a: \"x\", ",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: \"x\", ")
        )
        .unwrap(),
        vec!["a: \"x\""]
    );
}

#[test]
fn r3f1_split_top_level_dotted_key_trailing_ws_is_empty_key() {
    // R3-F1: `b.` + whitespace to EOF armed a key-segment start with
    // nothing after the dot — a structured EmptyKey (the category
    // `insert_value` raises for `a.: 1`), not a panic, not success.
    for tail in [" ", "\t"] {
        let body = format!(" \"a\": 1, b.{tail}");
        match split_top_level(
            &body,
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input(&body),
        ) {
            Err(crate::Error::Structured(crate::ErrorKind::EmptyKey { .. })) => {}
            other => panic!("expected EmptyKey, got: {:?}", other.err()),
        }
    }
    // Leading-dot form, same shape (quote bytes present: slow path).
    match split_top_level(
        " \"a\". ",
        1,
        S,
        InlineBody::Object,
        InlineBounds::for_input(" \"a\". "),
    ) {
        Err(crate::Error::Structured(crate::ErrorKind::EmptyKey { .. })) => {}
        other => panic!("expected EmptyKey, got: {:?}", other.err()),
    }
    // Dot followed by a real segment still works.
    assert_eq!(
        split_top_level(
            "a. b : 1",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a. b : 1")
        )
        .unwrap(),
        vec!["a. b : 1"]
    );
}

#[test]
fn r3f1_find_matching_close_eof_after_trailing_ws() {
    // R3-F1 sibling: untrimmed text ending in whitespace after a comma,
    // no closer — None, not an index panic.
    assert_eq!(find_matching_close("{\"a\": 1, ", b'{', b'}'), None);
    // Closer present after the whitespace: unchanged (skip stops at `}`).
    assert_eq!(find_matching_close("{\"a\": 1, }", b'{', b'}'), Some(9));
}

#[test]
fn r3f1_scan_inline_closer_eof_after_trailing_ws() {
    use super::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F1 sibling: untrimmed text ending in whitespace after a comma,
    // no closer on this text — NotFound, not an index panic.
    assert!(matches!(
        scan_inline_closer("{\"a\": 1, ", b'{', b'}', 1, S),
        InlineCloserScan::NotFound
    ));
    // Closer present after the whitespace: unchanged.
    assert!(matches!(
        scan_inline_closer("{\"a\": 1, }", b'{', b'}', 1, S),
        InlineCloserScan::Found(9)
    ));
}

#[test]
fn r3f1_find_unescaped_colon_inline_eof_after_dotted_ws() {
    // R3-F1 sibling: untrimmed text ending in whitespace after a dot —
    // None, not an index panic.
    assert_eq!(find_unescaped_colon_inline("a. "), None);
    // Dot then colon: unchanged.
    assert_eq!(find_unescaped_colon_inline("a. : 1"), Some(3));
}

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
    use super::inline::find_matching_close;
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
    use super::inline::{scan_inline_closer, InlineCloserScan};
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
    use super::inline::{split_top_level, InlineBody};
    // The nested object (with `]` in a quoted key) must be skipped
    // wholesale: the comma inside it never splits, the one after it does.
    assert_eq!(
        split_top_level(
            "{\"x]y\": 1},2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("{\"x]y\": 1},2")
        )
        .unwrap(),
        vec!["{\"x]y\": 1}", "2"]
    );
}

// --- R3-F4: mid-scalar braces have no structural meaning (spec 0.7 § 5.8.5) --

#[test]
fn r3f4_split_top_level_midvalue_balanced_brace_splits_at_inner_comma() {
    use super::inline::{split_top_level, InlineBody};
    // R3-F4: a balanced `{...}` mid-scalar must NOT be skipped wholesale;
    // the comma inside it is a real top-level separator (§ 5.8.5), and
    // the comma after the mid-scalar `}` also splits (no comma-shielding).
    assert_eq!(
        split_top_level(
            "a: x{y,z}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: x{y,z}, b: 2")
        )
        .unwrap(),
        vec!["a: x{y", "z}", " b: 2"]
    );
}

#[test]
fn r3f4_split_top_level_midvalue_brace_quote_aware_slow_path() {
    use super::inline::{split_top_level, InlineBody};
    // R3-F4: same rule on the quote-aware slow path.
    assert_eq!(
        split_top_level(
            "k: \"v\", a: x{y,z}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("k: \"v\", a: x{y,z}, b: 2")
        )
        .unwrap(),
        vec!["k: \"v\"", " a: x{y", "z}", " b: 2"]
    );
}

#[test]
fn r3f4_split_top_level_array_body_midvalue_brace_splits_at_inner_comma() {
    use super::inline::{split_top_level, InlineBody};
    // R3-F4: array bodies share the fast splitter and the rule.
    assert_eq!(
        split_top_level(
            "x{y,z}, 2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("x{y,z}, 2")
        )
        .unwrap(),
        vec!["x{y", "z}", " 2"]
    );
}

#[test]
fn r3f4_split_top_level_genuine_value_start_compounds_guard() {
    use super::inline::{split_top_level, InlineBody};
    // R3-F4 guards: a compound that IS the first code point of a value
    // keeps its structural meaning — no split inside it.
    assert_eq!(
        split_top_level(
            "{a: 1}, 2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("{a: 1}, 2")
        )
        .unwrap(),
        vec!["{a: 1}", " 2"]
    );
    assert_eq!(
        split_top_level(
            "a: {y: 1}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: {y: 1}, b: 2")
        )
        .unwrap(),
        vec!["a: {y: 1}", " b: 2"]
    );
}

#[test]
fn r3f4_scan_inline_closer_midvalue_balanced_brace_is_body_closer() {
    use super::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4: the `}` after `z` (the mid-scalar close) is the body's
    // closer — the scan must not treat the balanced span as opaque.
    assert!(matches!(
        scan_inline_closer("{a: x{y,z}, b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::Found(9)
    ));
}

#[test]
fn r3f4_scan_inline_closer_crossed_bracket_not_found() {
    use super::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4: the crossed `]` mid-scalar is not a matching `}` closer.
    assert!(matches!(
        scan_inline_closer("{a: x[y,z], b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::NotFound
    ));
}

#[test]
fn r3f4_scan_inline_closer_genuine_compounds_guard() {
    use super::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4 guards: value-start compounds still find their real closer.
    assert!(matches!(
        scan_inline_closer("{a: {y: 1}, b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::Found(16)
    ));
    assert!(matches!(
        scan_inline_closer("[{a: 1}, 2]", b'[', b']', 1, S),
        InlineCloserScan::Found(10)
    ));
}

// R8 regression: the InlineBounds memo must be a PURE memo — every
// recorded (opener, closer) pair must be exactly what the live
// dispatches compute over the same span. The recording walk once
// popped a frame at a later kind-matched closer even though a crossed
// closer inside the span had already returned the span's own depth to
// zero (the byte where the standalone scan stops with `NotFound`), so
// the memo said `Found` where the live scan said `NotFound` and split
// segmented differently (observable: different
// MalformedInlineCompound detail payloads, fuzz2-confirmed). This test
// cross-checks every recorded pair against BOTH live dispatches over
// exactly the shapes that fired, plus an exhaustive sweep of short
// structural bodies.
#[test]
fn memo_bounds_are_a_pure_memo_of_the_live_dispatches() {
    use super::inline::{
        find_matching_close, scan_inline_closer, scan_inline_closer_with_bounds, InlineCloserScan,
    };

    // Shapes that fired during the audit / differential fuzz (the four
    // MEMO_MISMATCH audit slices, embedded in a value position so the
    // gate walk actually records spans around them, and the four
    // fuzz2-diverging documents).
    let hostile = [
        "{a: {]}{, ,x{x,}",
        "{a: {[][x]}]}n\"}",
        "{a: {x   ]}, 2}",
        "{a: {]a{ :x}}",
        "{a: [a],{:[,:,}]}",
        "{a: [ a:,:[],{:[}, ]}",
        "{a: [],{[:[,}]}",
        "{a: [],[:[},a[{ {,a ]}",
        // R8-F2: the five review inputs. Pre-fix the quote-free
        // members let the fast gate walk phantom-open an Array
        // after the closed `[]` (value_start residue) and record /
        // verdict differently from their quote-bearing twins.
        "[[][text]",
        "[[]['text]",
        "[{}[text]",
        "{a: [[][text]}",
        "{a: [[][text], q: '}",
    ];

    let check = |body: &str| {
        let bytes = body.as_bytes();
        let (open, close) = if bytes[0] == b'[' {
            (b'[', b']')
        } else {
            (b'{', b'}')
        };
        let mut pairs = Vec::new();
        let verdict = scan_inline_closer_with_bounds(body, open, close, 0, S, &mut pairs);
        // Recording only happens on a Found gate; nothing to check otherwise.
        if !matches!(verdict, InlineCloserScan::Found(_)) {
            assert!(
                pairs.is_empty(),
                "bounds recorded on a non-Found gate: {body:?} {pairs:?}"
            );
            return;
        }
        for &(o, c) in &pairs {
            let ob = bytes[o];
            let (so, sc) = if ob == b'[' {
                (b'[', b']')
            } else {
                (b'{', b'}')
            };
            // BOTH consumer shapes must agree with the memo:
            // - the SUFFIX `&body[o..]` is what split's opener jump
            //   sub-scans (`scan_inline_closer(&input[i..], ...)`), and
            // - the BOUNDED VALUE `&body[o..=c]` is what the find-first
            //   dispatch hands to `known_closer` / `find_matching_close`
            //   / `scan_inline_closer`. R8-F2: checking the suffix alone
            //   let a quote-aware memo agree with a quote-aware re-scan
            //   while the actual consumer re-scanned the bounded value
            //   in quote-free fast mode. For the bounded shape the
            //   recorded closer is the last byte, so the live dispatch
            //   must find it exactly there.
            for (shape_name, span) in [("suffix", &body[o..]), ("bounded", &body[o..=c])] {
                // `c - o` is the recorded closer's index in both shapes;
                // for the bounded shape it is also the last byte, which
                // is exactly the `idx == len - 1` closed-compound read
                // the find-first dispatch gives a memo hit.
                assert!(
                    matches!(
                        scan_inline_closer(span, so, sc, 0, S),
                        InlineCloserScan::Found(f) if f == c - o
                    ),
                    "scan ({shape_name}) disagrees with the memo at {o}..{c} in {body:?}"
                );
                assert_eq!(
                    find_matching_close(span, so, sc),
                    Some(c - o),
                    "find ({shape_name}) disagrees with the memo at {o}..{c} in {body:?}"
                );
            }
        }
    };

    for b in hostile {
        check(b);
    }

    // Exhaustive sweep over short structural bodies (depth, crossed
    // closers, mid-scalar openers, raw markers, top-level commas).
    // R8-F2: the alphabet MUST carry quote bytes (all three § 5.3.3
    // delimiters) and § 3.3 whitespace — without them every body is
    // quote-free, the gate always picks the fast walk, and the whole
    // quote-aware memo surface (ScanQ recordings consumed by a
    // quote-free consumer slice) is unreachable. That blind spot is
    // why the R8-F2 family survived this test.
    let alpha: &[u8] = b"{[]}a:,.'\"` ";
    let mut buf = [0u8; 5];
    for len in 1..=5usize {
        let total = alpha.len().pow(len as u32);
        for mut idx in 0..total {
            for d in (0..len).rev() {
                buf[d] = alpha[idx % alpha.len()];
                idx /= alpha.len();
            }
            let body = std::str::from_utf8(&buf[..len]).unwrap();
            // Only bodies the gate scan accepts as inline compounds build a map.
            if body.starts_with('{') || body.starts_with('[') {
                check(body);
            }
        }
    }
}

// R8 regression pin: the fuzz2-diverging documents must keep the
// pre-R8 segmentation (ground truth probed from main @ 4477ae2). The
// memo bug changed only the `detail` payload (which segment the
// diagnostic quoted), so the pins cover the payload exactly. The
// fourth former member of this list is pinned separately below: R8-F2
// legitimately moved its whole category, not just its detail.
#[test]
fn crossed_closer_fuzz_inputs_keep_pre_r8_diagnostics() {
    let cases = [
        (
            "k: {a: [a],{:[,:,}]}",
            "inline object pair missing ':' separator in '{:['",
        ),
        (
            "k: {a: [ a:,:[],{:[}, ]}",
            "inline object pair missing ':' separator in '{:[}'",
        ),
        (
            "k: {a: [],{[:[,}]}",
            "inline object pair missing ':' separator in '{[:['",
        ),
    ];
    for (input, detail) in cases {
        let err = crate::parse(input).expect_err(input);
        match &err {
            crate::Error::Structured(crate::ErrorKind::MalformedInlineCompound {
                detail: got,
                span,
                ..
            }) => {
                assert_eq!(got, detail, "input {input:?}");
                assert_eq!(span.start as usize, 0, "input {input:?}");
                assert_eq!(span.end as usize, input.len(), "input {input:?}");
            }
            other => panic!("input {input:?}: unexpected {other:?}"),
        }
        // Same category via the strict and event entry points.
        assert!(crate::parse_strict(input).is_err(), "input {input:?}");
        assert!(
            crate::parse_events(input, |_| {}).is_err(),
            "input {input:?}"
        );
    }
}

// R8-F2 recategorization of the fourth formerly-pinned document:
// `k: {a: [],[:[},a[{ {,a ]}` reached the pair-split path only via the
// FAST gate walk, which carried `in_key = true` residue through the
// gated `[` after the top-level comma (former quirk 2) and
// phantom-opened the `[:[` Array as a nested compound, shifting the
// depth accounting so the body's own `}` seemed to close it. The
// quote-aware machine sets `in_key = false` at every opener, so the
// `}` after `[:[` matches no scope kind and the `]` before the final
// `}` is a CROSSED closer at depth 0 — § 5.2's matching-closer rule
// yields no same-line matching closer, which § 6.11 diagnoses as
// UnterminatedInlineCompound. The fast machine now agrees (R8-F2), so
// every entry point reports UnterminatedInlineCompound.
#[test]
fn r8f2_fast_in_key_residue_no_longer_recategorizes_crossed_closer() {
    let input = "k: {a: [],[:[},a[{ {,a ]}";
    let expect_unterminated = |res: Result<(), crate::Error>| {
        assert!(
            matches!(
                res,
                Err(crate::Error::Structured(
                    crate::ErrorKind::UnterminatedInlineCompound { .. }
                ))
            ),
            "input {input:?}: expected UnterminatedInlineCompound"
        );
    };
    expect_unterminated(crate::parse(input).map(|_| ()));
    expect_unterminated(crate::parse_strict(input).map(|_| ()));
    expect_unterminated(crate::parse_events(input, |_| {}).map(|_| ()));
}

// R8-F2 regression: the five review-round-8 inputs. Every one is an
// INVALID document, and the quote-aware reading gives the same
// category for all five. The value position is already consumed by the
// closed inner compound (the empty `[]` / `{}`), so the trailing
// `[text` is content after a closed value, not an unterminated one:
//
// - § 5.8.5: "The decision is made once, when the parser begins
//   reading an inline value: if the first non-whitespace code point is
//   `{` or `[`, the value is a nested compound; otherwise the value is
//   an inline scalar that runs to the next unescaped `,` / `}` / `]`"
//   — after the inner compound closes there is no open value for a
//   following `[` to open.
// - § 6.12: "Non-whitespace content after the same-line matching
//   closer of a value-position compound ... The closer makes the
//   compound closed; the trailing bytes are therefore malformed
//   content, not an unterminated compound." The enclosing compound
//   itself still closes on the same line, so the defect is
//   MalformedInlineCompound, not UnterminatedInlineCompound.
// - § 5.3.3 ("Keys only"): a quote character in a value position
//   "is ordinary content with no special meaning" — the later
//   unrelated `'` in the last input must not change any earlier
//   byte's role, so the quote-free and quote-bearing twins MUST get
//   the same verdict.
//
// Pre-R8-F2 the fast walks left `value_start` set after the empty
// Array closed (former quirk 1) and carried `in_key` residue through
// `[` openers (former quirk 2), phantom-opened a nested compound at
// the next `[`, swallowed the enclosing body's own closer, and
// reported UnterminatedInlineCompound for the quote-free twins
// (`[[][text]`, `{a: [[][text]}`) while the quote-bearing twins
// (`[[]['text]`, `{a: [[][text], q: '}`) already reported
// MalformedInlineCompound — the same bytes, two verdicts, decided by
// a later unrelated quote.
#[test]
fn r8f2_closed_empty_array_consumes_value_start_across_modes() {
    let cases = [
        "[[][text]",
        "[[]['text]",
        "[{}[text]",
        "{a: [[][text]}",
        "{a: [[][text], q: '}",
    ];
    for input in cases {
        let expect_malformed = |res: Result<(), crate::Error>| {
            assert!(
                matches!(
                    res,
                    Err(crate::Error::Structured(
                        crate::ErrorKind::MalformedInlineCompound { .. }
                    ))
                ),
                "input {input:?}: expected MalformedInlineCompound"
            );
        };
        expect_malformed(crate::parse(input).map(|_| ()));
        expect_malformed(crate::parse_strict(input).map(|_| ()));
        expect_malformed(crate::parse_events(input, |_| {}).map(|_| ()));
    }
}
