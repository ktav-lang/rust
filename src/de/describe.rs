//! Human-readable label for a `Value` variant — used in error messages.

use crate::value::Value;

pub(super) fn describe(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a bool",
        Value::Integer(_) => "a typed integer",
        Value::Float(_) => "a typed float",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}
