//! Top-level entry: render a [`Value`] into a Ktav text document.

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

use super::array_item::render_array_item;
use super::helpers::first_item_needs_wrap;
use super::object::render_object_body;

/// Serializes `value` as a top-level Ktav document. The top-level
/// value must be an Object or an Array (spec § 5.0.1, added 0.1.1).
///
/// Top-level Arrays normally render as bare item-per-line, with no
/// surrounding `[...]` brackets — the root-kind detection in § 5.0.1
/// recovers the Array from the shape of the first line. But when the
/// first item is itself a non-empty Object or Array, rendering it
/// bare would start the document with a lone `{` / `[`, which § 5.0.1
/// reads back as "the root itself is that compound" rather than "the
/// first item of a root Array" — silently changing the root kind on
/// re-parse. § 5.9.3 wraps the root in `[` / `]` in exactly this case;
/// `emit_canonical` already does, and this now matches it.
pub fn render(value: &Value) -> Result<String> {
    // Pre-size the output buffer so renders of medium-large documents
    // don't trigger 4–6 String reallocations on the way to their final
    // size. The estimate is a lower bound (it omits indentation and
    // multi-line wrappers) — String's growth strategy will still kick
    // in for larger-than-expected outputs, but the common case skips
    // every realloc.
    let mut out = String::with_capacity(estimate_size(value));
    match value {
        Value::Object(o) => render_object_body(o, 0, &mut out)?,
        // § 5.9.3: an empty Array root has no items to give it shape,
        // so it must be written explicitly — otherwise `render` emits
        // nothing, and an empty document parses back as `Object({})`
        // (§ 5.0.1's default for content-free input), not `Array([])`.
        // `emit_canonical` already special-cases this; mirrored here.
        Value::Array(items) if items.is_empty() => out.push_str("[]\n"),
        Value::Array(items) => {
            // Non-empty here (the empty case is the arm above).
            if first_item_needs_wrap(&items[0]) {
                out.push_str("[\n");
                for item in items {
                    render_array_item(item, 1, &mut out)?;
                }
                out.push_str("]\n");
            } else {
                for item in items {
                    render_array_item(item, 0, &mut out)?;
                }
            }
        }
        _ => {
            return Err(Error::Message(
                "top-level value must be an Object or an Array".into(),
            ))
        }
    }
    Ok(out)
}

/// Render `value` with **every scalar coerced to a String**: typed
/// integers, typed floats, booleans, and null are flattened to their
/// textual form and emitted via the raw-marker `::` so the output
/// round-trips back through the parser as the same string scalars.
///
/// Useful for dumping configuration in a "everything is a string"
/// shape — e.g. for environments or downstream consumers that don't
/// understand the `:i` / `:f` typed markers, or for diffs where you
/// want the textual form to be the canonical source of truth.
///
/// Compounds (Object / Array) preserve their structure; only leaf
/// scalars are coerced.
pub fn to_string_force_strings(value: &Value) -> Result<String> {
    let coerced = coerce_scalars_to_strings(value);
    render(&coerced)
}

fn coerce_scalars_to_strings(value: &Value) -> Value {
    match value {
        Value::Null => Value::String("null".into()),
        Value::Bool(true) => Value::String("true".into()),
        Value::Bool(false) => Value::String("false".into()),
        Value::Integer(s) => Value::String(s.clone()),
        Value::Float(s) => Value::String(s.clone()),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(items) => Value::Array(items.iter().map(coerce_scalars_to_strings).collect()),
        Value::Object(obj) => {
            let mut out = ObjectMap::with_capacity_and_hasher(obj.len(), Default::default());
            for (k, v) in obj {
                out.insert(k.clone(), coerce_scalars_to_strings(v));
            }
            Value::Object(out)
        }
    }
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
