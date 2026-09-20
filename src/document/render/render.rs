//! Top-level entry: render a [`Value`] into a Ktav text document.

use crate::error::{Error, Result};
use crate::value::Value;

use super::array_item::render_array_item;
use super::helpers::first_item_needs_wrap;
use super::object::render_object_body;

/// Serializes `value` as a top-level Ktav document. The top-level
/// value must be an Object or an Array (spec § 5.0.1, added 0.1.1).
///
/// Top-level Arrays normally render as bare item-per-line, with no
/// surrounding `[...]` brackets — the root-kind detection in § 5.0.1
/// recovers the Array from the shape of the first line. But when the
/// first item is itself an Object or Array (empty or not), its first
/// emitted line — a lone `{` / `[` opener, or a closed inline `{}` /
/// `[]` — is read back by § 5.0.1 as establishing a *different* root,
/// silently changing the root kind on re-parse (or, for an empty
/// compound followed by more items, failing to parse at all). In that
/// case the root is wrapped in explicit `[` / `]`; see
/// `helpers::first_item_needs_wrap` for the full § 5.0.1 / § 5.9.3
/// story. The same shared check drives `emit_canonical`.
///
/// Non-representable Values are rejected up-front per § 5.9.0 with an
/// `Error::Unrepresentable` carrying a [`crate::error::ReasonCode`],
/// before any bytes are emitted — no partial output on rejection.
pub fn render(value: &Value) -> Result<String> {
    render_document(value, false)
}

/// The body of both public entry points. `force_strings` is the
/// [`to_string_force_strings`] mode: every leaf scalar is emitted as a
/// String.
///
/// The § 5.9.0 conditions are detected by the emitters as they go, not
/// by a pre-pass — see [`super::representable`] for why that keeps the
/// "no partial output" and "root before nodes" guarantees. `out` is
/// local, so a rejection discards every byte written so far;
/// `attach_path` then re-walks the Value on this cold path to name the
/// offending key.
fn render_document(value: &Value, force_strings: bool) -> Result<String> {
    // Pre-size the output buffer so renders of medium-large documents
    // don't trigger 4–6 String reallocations on the way to their final
    // size. The estimate is a lower bound (it omits indentation and
    // multi-line wrappers) — String's growth strategy will still kick
    // in for larger-than-expected outputs, but the common case skips
    // every realloc.
    // § 5.9.0's fixed precedence: the root-kind constraint is decided
    // before any node is visited. The path is empty because the offense
    // is the root itself.
    if !matches!(value, Value::Object(_) | Value::Array(_)) {
        return Err(Error::UnrepresentableAt {
            code: crate::error::ReasonCode::ScalarRoot,
            path: Vec::new(),
        });
    }
    let mut out = String::with_capacity(estimate_size(value));
    match emit_root(value, force_strings, &mut out) {
        Ok(()) => Ok(out),
        Err(e) => Err(super::representable::attach_path(value, e, force_strings)),
    }
}

/// The root-level emission, split out so the node walk can use `?`
/// freely while [`render_document`] keeps sole ownership of turning a
/// rejection into its path-carrying form.
fn emit_root(value: &Value, force_strings: bool, out: &mut String) -> Result<()> {
    match value {
        // The root Object is the only caller passing `is_root = true`
        // — its first pair's key is the only one that can land at byte
        // offset 0 (§ 5.9.10 rule (c) / § 5.9.12).
        Value::Object(o) => render_object_body(o, 0, true, force_strings, out)?,
        // § 5.9.3: an empty Array root has no items to give it shape,
        // so it must be written explicitly — otherwise `render` emits
        // nothing, and an empty document parses back as `Object({})`
        // (§ 5.0.1's default for content-free input), not `Array([])`.
        // `emit_canonical` already special-cases this; mirrored here.
        Value::Array(items) if items.is_empty() => out.push_str("[]\n"),
        Value::Array(items) => {
            // Non-empty here (the empty case is the arm above).
            if first_item_needs_wrap(&items[0]) {
                // The wrapped branch's first content line is `[` itself,
                // so no item line is ever root-detected (§ 5.9.6).
                out.push_str("[\n");
                for item in items {
                    render_array_item(item, 1, false, force_strings, out)?;
                }
                out.push_str("]\n");
            } else {
                for (index, item) in items.iter().enumerate() {
                    // § 5.9.6 / § 5.9.12: only index 0 of the unwrapped
                    // root Array is exposed to root-kind detection.
                    render_array_item(item, 0, index == 0, force_strings, out)?;
                }
            }
        }
        // Ruled out by the caller.
        _ => return Err(Error::Unrepresentable(crate::error::ReasonCode::ScalarRoot)),
    }
    Ok(())
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
///
/// Note this renders the COERCED value: Float scalars become Strings,
/// so `NonFiniteFloat` cannot fire here, while the root /
/// `EmptyKeyName` / `CRByte` / collision checks still apply.
///
/// The coercion is applied AS the document is emitted. It used to build
/// a whole second `Value` tree first — cloning every key and every
/// scalar, allocating a fresh `ObjectMap` per object, doubling peak
/// memory — and render that. Nothing in the output depended on the copy
/// existing; a flag threaded to the leaf emitters produces the same
/// bytes.
pub fn to_string_force_strings(value: &Value) -> Result<String> {
    render_document(value, true)
}

/// The textual form a leaf scalar takes under
/// [`to_string_force_strings`]. `None` for compounds, which keep their
/// structure and are walked as usual.
pub(super) fn forced_string_text(value: &Value) -> Option<&str> {
    Some(match value {
        Value::Null => "null",
        Value::Bool(true) => "true",
        Value::Bool(false) => "false",
        Value::Integer(s) | Value::Float(s) | Value::String(s) => s.as_str(),
        Value::Array(_) | Value::Object(_) => return None,
    })
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
