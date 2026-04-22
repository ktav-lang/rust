//! Unit tests for `Value` accessors.

use super::{ObjectMap, Value};

#[test]
fn is_null_only_for_null_variant() {
    assert!(Value::Null.is_null());
    assert!(!Value::Bool(true).is_null());
    assert!(!Value::String("x".into()).is_null());
    assert!(!Value::Integer("1".into()).is_null());
    assert!(!Value::Float("1.0".into()).is_null());
    assert!(!Value::Array(vec![]).is_null());
    assert!(!Value::Object(ObjectMap::default()).is_null());
}

#[test]
fn as_bool_only_for_bool_variant() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
    assert_eq!(Value::Null.as_bool(), None);
    assert_eq!(Value::String("true".into()).as_bool(), None);
    assert_eq!(Value::Integer("1".into()).as_bool(), None);
    assert_eq!(Value::Float("1.0".into()).as_bool(), None);
}

#[test]
fn as_str_only_for_string_variant() {
    assert_eq!(Value::String("x".into()).as_str(), Some("x"));
    assert_eq!(Value::Null.as_str(), None);
    assert_eq!(Value::Bool(true).as_str(), None);
    // Integer/Float do NOT promote to as_str — they live in their own
    // accessors.
    assert_eq!(Value::Integer("1".into()).as_str(), None);
    assert_eq!(Value::Float("1.0".into()).as_str(), None);
}

#[test]
fn as_integer_only_for_integer_variant() {
    assert_eq!(Value::Integer("42".into()).as_integer(), Some("42"));
    assert_eq!(Value::Integer("-1".into()).as_integer(), Some("-1"));
    assert_eq!(
        Value::Integer("99999999999999999999".into()).as_integer(),
        Some("99999999999999999999")
    );
    assert_eq!(Value::String("42".into()).as_integer(), None);
    assert_eq!(Value::Float("1.0".into()).as_integer(), None);
    assert_eq!(Value::Null.as_integer(), None);
}

#[test]
fn as_float_only_for_float_variant() {
    assert_eq!(Value::Float("0.5".into()).as_float(), Some("0.5"));
    assert_eq!(
        Value::Float("-3.14e-10".into()).as_float(),
        Some("-3.14e-10")
    );
    assert_eq!(Value::String("0.5".into()).as_float(), None);
    assert_eq!(Value::Integer("1".into()).as_float(), None);
    assert_eq!(Value::Null.as_float(), None);
}

#[test]
fn as_array_only_for_array_variant() {
    let empty: Vec<Value> = vec![];
    assert_eq!(Value::Array(empty.clone()).as_array(), Some(&empty));
    assert!(Value::String("x".into()).as_array().is_none());
    assert!(Value::Integer("1".into()).as_array().is_none());
    assert!(Value::Float("1.0".into()).as_array().is_none());
}

#[test]
fn as_object_only_for_object_variant() {
    let obj = ObjectMap::default();
    assert!(Value::Object(obj.clone()).as_object().is_some());
    assert!(Value::Array(vec![]).as_object().is_none());
    assert!(Value::Integer("1".into()).as_object().is_none());
    assert!(Value::Float("1.0".into()).as_object().is_none());
}

#[test]
fn integer_and_float_equality_is_variant_scoped() {
    // Same content in different variants is NOT equal — variant is load-
    // bearing.
    assert_ne!(Value::Integer("42".into()), Value::String("42".into()),);
    assert_ne!(Value::Integer("42".into()), Value::Float("42".into()),);
    assert_ne!(Value::Float("1.0".into()), Value::String("1.0".into()),);
}

#[test]
fn integer_and_float_clone_roundtrips() {
    let i = Value::Integer("12345".into());
    let i2 = i.clone();
    assert_eq!(i, i2);
    let f = Value::Float("1.5e10".into());
    let f2 = f.clone();
    assert_eq!(f, f2);
}
