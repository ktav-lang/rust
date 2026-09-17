//! Shared helpers for the `error_envelope` integration tests.

use ktav::value::Scalar;
use ktav::{Error, ErrorEnvelope, ObjectMap, Value};
use serde_json::Value as Json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

pub(crate) fn f(v: &str) -> Value {
    Value::Float(v.parse().unwrap_or_else(|_| panic!("float scalar: {v:?}")))
}

pub(crate) fn n(v: i64) -> Value {
    Value::Integer(v.to_string().parse().unwrap_or_else(|_| unreachable!()))
}

pub(crate) fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert(Scalar::from(*k), v.clone());
    }
    Value::Object(m)
}

pub(crate) fn env(err: &Error, src: &str) -> ErrorEnvelope {
    ErrorEnvelope::from_error(err, src)
}

/// Cross-check: the envelope JSON must have EXACTLY the ten
/// contractual fields, in order — absent info is an explicit null,
/// never an omitted key. `message` comes last so the nine original
/// fields keep the positions they shipped with.
pub(crate) fn ten_keys_present(json: &str) {
    let v: Json = serde_json::from_str(json).unwrap();
    let object = v.as_object().expect("envelope JSON is an object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
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
            "message",
        ]
    );
}
