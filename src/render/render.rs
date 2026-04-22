//! Top-level entry: render a [`Value`] into a Ktav text document.

use crate::error::{Error, Result};
use crate::value::Value;

use super::object::render_object_body;

/// Serializes `value` as a top-level Ktav document. The top-level value
/// must be an object.
pub fn render(value: &Value) -> Result<String> {
    let obj = match value {
        Value::Object(o) => o,
        _ => return Err(Error::Message("top-level value must be an object".into())),
    };
    let mut out = String::new();
    render_object_body(obj, 0, &mut out)?;
    Ok(out)
}
