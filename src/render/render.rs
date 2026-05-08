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
    // Pre-size the output buffer so renders of medium-large documents
    // don't trigger 4–6 String reallocations on the way to their final
    // size. The estimate is a lower bound (it omits indentation and
    // multi-line wrappers) — String's growth strategy will still kick
    // in for larger-than-expected outputs, but the common case skips
    // every realloc.
    let mut out = String::with_capacity(estimate_size(value));
    render_object_body(obj, 0, &mut out)?;
    Ok(out)
}

/// Rough byte-size estimate for a rendered `Value`, used to pre-size
/// the output buffer. Counts `key: value\n` overhead per scalar pair
/// (~4 bytes for `: \n`), array brackets, and recursive children. Does
/// not account for indentation or `:: `/`:i `/`:f `/multi-line wrappers,
/// so under-estimates by ~10–20 % — that's fine; growth covers the gap.
fn estimate_size(value: &Value) -> usize {
    match value {
        Value::Null => 5,    // `null\n`
        Value::Bool(_) => 6, // `false\n`
        Value::Integer(s) | Value::Float(s) | Value::String(s) => s.len() + 4,
        Value::Array(items) => 4 + items.iter().map(estimate_size).sum::<usize>(),
        Value::Object(obj) => obj
            .iter()
            .map(|(k, v)| k.len() + 4 + estimate_size(v))
            .sum::<usize>()
            .saturating_add(4),
    }
}
