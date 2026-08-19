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
