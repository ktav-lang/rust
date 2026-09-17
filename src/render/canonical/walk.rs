use super::num::canonical_float;
use super::shared::push_indent;
use super::strings::{emit_string_as_item, emit_string_in_pair};
use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Emit a canonical Ktav serialisation of `value` (spec § 5.9).
///
/// The top-level value must be an Object or an Array (§ 5.0.1).
/// Non-representable Values are rejected per § 5.9.0 with an
/// `Error::Unrepresentable` carrying a [`crate::error::ReasonCode`] —
/// scalar roots, empty key names, non-finite floats, `CR` bytes, and
/// the three multi-line collision cases. The check runs before any
/// bytes are emitted, so a rejection produces no partial output.
pub fn emit_canonical(value: &Value) -> Result<String> {
    super::representable::check_representable(value)?;
    let mut out = String::with_capacity(estimate_size(value));
    match value {
        Value::Object(o) => emit_object_pairs(o, 0, true, &mut out)?,
        Value::Array(items) if items.is_empty() => {
            // § 5.9.3: empty Array root → `[]\n`
            out.push_str("[]\n");
        }
        Value::Array(items) => emit_array_root(items, &mut out)?,
        _ => return Err(Error::Unrepresentable(crate::error::ReasonCode::ScalarRoot)),
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// § 5.9.3 Root-level emission
// ---------------------------------------------------------------------------

/// Emit an Object's pairs at the given indent level (root uses 0).
///
/// `is_root` is true ONLY for the document-root Object; it lets the
/// first pair's key take the § 5.9.10 rule (c) U+FEFF guard (which
/// only applies at byte offset 0, § 5.9.12). Every nested call passes
/// `false` — an interior Object's first key never lands at offset 0.
fn emit_object_pairs(
    obj: &ObjectMap,
    indent: usize,
    is_root: bool,
    out: &mut String,
) -> Result<()> {
    for (index, (k, v)) in obj.iter().enumerate() {
        emit_pair(k, v, indent, is_root && index == 0, out)?;
    }
    Ok(())
}

/// Emit a root-level Array. Items are bare at indent 0 unless the first
/// item is itself a non-empty compound (§ 5.9.3 lone-`{`/`[` wrap).
fn emit_array_root(items: &[Value], out: &mut String) -> Result<()> {
    let needs_wrap = !items.is_empty() && crate::render::helpers::first_item_needs_wrap(&items[0]);
    if needs_wrap {
        // The wrapped branch's first content line is `[` itself, so no
        // item line is ever read as the root's first line — no item is
        // exposed to root-kind detection here (§ 5.9.6 / § 5.9.3).
        out.push_str("[\n");
        for item in items {
            emit_array_item(item, 1, false, out)?;
        }
        out.push_str("]\n");
    } else {
        for (index, item) in items.iter().enumerate() {
            // § 5.9.6 / § 5.9.12: only index 0 of the unwrapped root
            // Array is exposed to root-kind detection.
            emit_array_item(item, 0, index == 0, out)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.5 Pair emission
// ---------------------------------------------------------------------------

/// Emit a single `key: value` / `key:: value` / compound pair.
///
/// `root_first_key` forwards the § 5.9.10 rule (c) guard for the
/// root Object's first-serialized key.
fn emit_pair(
    key: &str,
    value: &Value,
    indent: usize,
    root_first_key: bool,
    out: &mut String,
) -> Result<()> {
    push_indent(out, indent);
    // Spec 0.7 § 5.9.10 — bare/quoted form selection + re-escape.
    crate::render::helpers::push_escaped_key_segment(key, root_first_key, out);
    match value {
        Value::Null => {
            // § 5.9.9
            out.push_str(": null\n");
        }
        Value::Bool(b) => {
            out.push_str(": ");
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        Value::Integer(s) => {
            // § 5.9.8: canonical base-10 decimal. Always a valid integer
            // literal — no raw marker needed.
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        Value::Float(s) => {
            // § 5.9.8: canonical float form — scientific for large/small abs.
            out.push_str(": ");
            out.push_str(&canonical_float(s));
            out.push('\n');
        }
        Value::String(s) => {
            emit_string_in_pair(s, indent, out)?;
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str(": []\n");
            } else {
                out.push_str(": [\n");
                for item in items {
                    emit_array_item(item, indent + 1, false, out)?;
                }
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                out.push_str(": {}\n");
            } else {
                out.push_str(": {\n");
                emit_object_pairs(obj, indent + 1, false, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.6 Array-item emission
// ---------------------------------------------------------------------------

/// Emit one array item at the given indent level.
///
/// `is_root_array_first` is TRUE only for index 0 of an unwrapped
/// Array root — the sole item position whose body is exposed to
/// § 5.0.1's root-kind detection, and therefore the sole position to
/// which § 5.9.6's first-item safeguard applies.
fn emit_array_item(
    value: &Value,
    indent: usize,
    is_root_array_first: bool,
    out: &mut String,
) -> Result<()> {
    push_indent(out, indent);
    match value {
        Value::Null => {
            out.push_str("null\n");
        }
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        Value::Integer(s) => {
            out.push_str(s);
            out.push('\n');
        }
        Value::Float(s) => {
            out.push_str(&canonical_float(s));
            out.push('\n');
        }
        Value::String(s) => {
            emit_string_as_item(s, indent, is_root_array_first, out)?;
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]\n");
            } else {
                out.push_str("[\n");
                for item in items {
                    // Nested items are never root-detected (§ 5.9.6).
                    emit_array_item(item, indent + 1, false, out)?;
                }
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                out.push_str("{}\n");
            } else {
                out.push_str("{\n");
                emit_object_pairs(obj, indent + 1, false, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Size estimate
// ---------------------------------------------------------------------------

fn estimate_size(value: &Value) -> usize {
    match value {
        Value::Null => 5,
        Value::Bool(_) => 6,
        Value::Integer(s) | Value::Float(s) | Value::String(s) => s.len() + 8,
        Value::Array(items) => 4 + items.iter().map(estimate_size).sum::<usize>(),
        Value::Object(obj) => obj
            .iter()
            .map(|(k, v)| k.len() + 4 + estimate_size(v))
            .sum::<usize>()
            .saturating_add(4),
    }
}
