//! Tests for the parser's helper fns: `validate`, `classify`, `insert_value`, key trimming, integer/float literal grammar, and `classify_value_start` inference.

use super::S;
use crate::parser::classify::{classify_value_start, is_float_literal, try_parse_integer};
use crate::parser::insert::insert_value;
use crate::parser::validate::is_valid_key;
use crate::parser::value_start::ValueStart;
use crate::value::{ObjectMap, Value};
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
