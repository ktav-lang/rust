//! Unit tests for parser-internal helpers.

use super::classify::{classify_value_start, validate_typed_float, validate_typed_integer};
use super::insert::insert_value;
use super::validate::is_valid_key;
use super::value_start::ValueStart;

use crate::value::{ObjectMap, Value};

// --- validate ---------------------------------------------------------------

#[test]
fn valid_keys_accepted() {
    assert!(is_valid_key("port"));
    assert!(is_valid_key("a1"));
    assert!(is_valid_key("kebab-case"));
    assert!(is_valid_key("snake_case"));
}

#[test]
fn invalid_keys_rejected() {
    assert!(!is_valid_key(""));
    assert!(!is_valid_key("has space"));
    assert!(!is_valid_key("with[bracket"));
    assert!(!is_valid_key("with]bracket"));
    assert!(!is_valid_key("with{brace"));
    assert!(!is_valid_key("with}brace"));
    assert!(!is_valid_key("with:colon"));
    assert!(!is_valid_key("with#hash"));
}

#[test]
fn paths_validated_segment_by_segment_via_insert() {
    // Path validation now lives inside `insert_value`; exercise it here
    // the same way the parser does.
    let mut t = ObjectMap::default();
    assert!(insert_value(&mut t, "a.b.c", Value::Null, 1).is_ok());

    let mut t = ObjectMap::default();
    assert!(insert_value(&mut t, "a", Value::Null, 1).is_ok());

    let mut t = ObjectMap::default();
    // empty segment inside the path
    assert!(insert_value(&mut t, "a..b", Value::Null, 1).is_err());

    let mut t = ObjectMap::default();
    // trailing dot — final segment is empty
    assert!(insert_value(&mut t, "a.b.", Value::Null, 1).is_err());

    let mut t = ObjectMap::default();
    // lone dot — two empty segments
    assert!(insert_value(&mut t, ".", Value::Null, 1).is_err());
}

// --- classify ---------------------------------------------------------------

#[test]
fn classify_scalar() {
    match classify_value_start("hello", 1).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "hello"),
        _ => panic!("expected Scalar"),
    }
}

#[test]
fn classify_keywords() {
    assert!(matches!(
        classify_value_start("null", 1).unwrap(),
        ValueStart::Null
    ));
    assert!(matches!(
        classify_value_start("true", 1).unwrap(),
        ValueStart::Bool(true)
    ));
    assert!(matches!(
        classify_value_start("false", 1).unwrap(),
        ValueStart::Bool(false)
    ));
}

#[test]
fn classify_case_sensitive_keywords() {
    // Only lowercase matches — "True" / "NULL" are strings.
    match classify_value_start("True", 1).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "True"),
        _ => panic!("expected Scalar"),
    }
    match classify_value_start("NULL", 1).unwrap() {
        ValueStart::Scalar(s) => assert_eq!(s, "NULL"),
        _ => panic!("expected Scalar"),
    }
}

#[test]
fn classify_open_compounds() {
    assert!(matches!(
        classify_value_start("{", 1).unwrap(),
        ValueStart::OpenObject
    ));
    assert!(matches!(
        classify_value_start("[", 1).unwrap(),
        ValueStart::OpenArray
    ));
}

#[test]
fn classify_empty_inline_compounds() {
    assert!(matches!(
        classify_value_start("{}", 1).unwrap(),
        ValueStart::EmptyObject
    ));
    assert!(matches!(
        classify_value_start("[]", 1).unwrap(),
        ValueStart::EmptyArray
    ));
    assert!(matches!(
        classify_value_start("{ }", 1).unwrap(),
        ValueStart::EmptyObject
    ));
    assert!(matches!(
        classify_value_start("[  ]", 1).unwrap(),
        ValueStart::EmptyArray
    ));
}

#[test]
fn classify_inline_nonempty_rejected() {
    assert!(classify_value_start("{a: 1}", 1).is_err());
    assert!(classify_value_start("[1, 2]", 1).is_err());
}

// --- insert_value -----------------------------------------------------------

#[test]
fn insert_simple_pair() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "port", Value::String("8080".into()), 1).unwrap();
    assert_eq!(t.get("port"), Some(&Value::String("8080".into())));
}

#[test]
fn insert_dotted_path_creates_intermediate_objects() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "a.b.c", Value::String("x".into()), 1).unwrap();
    let a = t.get("a").unwrap().as_object().unwrap();
    let b = a.get("b").unwrap().as_object().unwrap();
    assert_eq!(b.get("c"), Some(&Value::String("x".into())));
}

#[test]
fn insert_duplicate_rejected() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "x", Value::String("1".into()), 1).unwrap();
    let err = insert_value(&mut t, "x", Value::String("2".into()), 2);
    assert!(err.is_err());
}

#[test]
fn insert_scalar_then_nested_path_rejected() {
    let mut t = ObjectMap::default();
    insert_value(&mut t, "a", Value::String("leaf".into()), 1).unwrap();
    let err = insert_value(&mut t, "a.b", Value::String("x".into()), 2);
    assert!(err.is_err());
}

// --- typed scalar validation ------------------------------------------------

#[test]
fn typed_integer_accepts_plain_digits() {
    assert_eq!(validate_typed_integer(" 42 ", 1).unwrap(), "42");
    assert_eq!(validate_typed_integer("0", 1).unwrap(), "0");
    assert_eq!(
        validate_typed_integer("9999999999999999999999", 1).unwrap(),
        "9999999999999999999999"
    );
}

#[test]
fn typed_integer_strips_leading_plus() {
    assert_eq!(validate_typed_integer(" +5", 1).unwrap(), "5");
    assert_eq!(validate_typed_integer(" +0", 1).unwrap(), "0");
}

#[test]
fn typed_integer_preserves_leading_minus() {
    assert_eq!(validate_typed_integer(" -42", 1).unwrap(), "-42");
    assert_eq!(validate_typed_integer(" -0", 1).unwrap(), "-0");
}

#[test]
fn typed_integer_rejects_non_digit() {
    assert!(validate_typed_integer(" abc", 1).is_err());
    assert!(validate_typed_integer(" 1.5", 1).is_err());
    assert!(validate_typed_integer(" ", 1).is_err());
    assert!(validate_typed_integer("", 1).is_err());
    assert!(validate_typed_integer(" +", 1).is_err());
    assert!(validate_typed_integer(" -", 1).is_err());
}

#[test]
fn typed_integer_rejects_compound_opener() {
    assert!(validate_typed_integer(" {", 1).is_err());
    assert!(validate_typed_integer(" [", 1).is_err());
    assert!(validate_typed_integer(" (", 1).is_err());
}

#[test]
fn typed_integer_error_mentions_category() {
    let err = validate_typed_integer(" abc", 5).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("InvalidTypedScalar"), "got: {}", msg);
    assert!(msg.contains("Line 5"), "got: {}", msg);
}

#[test]
fn typed_float_accepts_simple() {
    assert_eq!(validate_typed_float(" 3.14", 1).unwrap(), "3.14");
    assert_eq!(validate_typed_float(" 0.0", 1).unwrap(), "0.0");
}

#[test]
fn typed_float_strips_leading_plus_on_mantissa() {
    assert_eq!(validate_typed_float(" +3.14", 1).unwrap(), "3.14");
}

#[test]
fn typed_float_preserves_minus_and_exponent_signs() {
    assert_eq!(validate_typed_float(" -3.14", 1).unwrap(), "-3.14");
    assert_eq!(validate_typed_float(" 3.14e+10", 1).unwrap(), "3.14e+10");
    assert_eq!(validate_typed_float(" 3.14E-10", 1).unwrap(), "3.14E-10");
    assert_eq!(validate_typed_float(" 3.14e10", 1).unwrap(), "3.14e10");
}

#[test]
fn typed_float_requires_decimal_point() {
    assert!(validate_typed_float(" 42", 1).is_err());
    assert!(validate_typed_float(" 1.", 1).is_err());
    assert!(validate_typed_float(" .5", 1).is_err());
}

#[test]
fn typed_float_rejects_invalid_exponent() {
    assert!(validate_typed_float(" 1.5e", 1).is_err());
    assert!(validate_typed_float(" 1.5e+", 1).is_err());
    assert!(validate_typed_float(" 1.5ee5", 1).is_err());
}

#[test]
fn typed_float_rejects_empty_and_compound() {
    assert!(validate_typed_float("", 1).is_err());
    assert!(validate_typed_float(" ", 1).is_err());
    assert!(validate_typed_float(" {", 1).is_err());
    assert!(validate_typed_float(" [", 1).is_err());
    assert!(validate_typed_float(" (", 1).is_err());
}
