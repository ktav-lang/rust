//! Render one line of an array (item can be scalar, object, or nested array).

use crate::error::Result;
use crate::value::Value;

use super::helpers::{
    bare_item_is_pair_candidate, choose_multiline_form, cr_error, item_needs_raw_marker,
    push_indent, reject_non_finite, string_needs_multiline, MultilineForm,
};
use super::object::render_object_body;
use super::render::forced_string_text;

/// Render one line of an array (item can be scalar, object, or nested array).
///
/// `is_root_array_first` is TRUE only for index 0 of an unwrapped Array
/// root — the sole item position exposed to § 5.0.1's root-kind
/// detection, and therefore the sole position to which § 5.9.6's /
/// § 5.9.12's first-item safeguards apply.
pub(super) fn render_array_item(
    value: &Value,
    indent: usize,
    is_root_array_first: bool,
    force_strings: bool,
    out: &mut String,
) -> Result<()> {
    push_indent(out, indent);

    // `to_string_force_strings`: every leaf scalar takes the String
    // emission path on its textual form; compounds keep their structure.
    if force_strings {
        if let Some(text) = forced_string_text(value) {
            return render_string_as_item(text, indent, is_root_array_first, out);
        }
    }

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
            // § 5.9.0 NonFiniteFloat — see the matching arm in `pair.rs`.
            reject_non_finite(s)?;
            out.push_str(s);
            out.push('\n');
        }
        Value::String(s) => return render_string_as_item(s, indent, is_root_array_first, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]\n");
            } else {
                out.push_str("[\n");
                for item in items {
                    // Nested items are never root-detected (§ 5.9.6).
                    render_array_item(item, indent + 1, false, force_strings, out)?;
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
                render_object_body(obj, indent + 1, false, force_strings, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

/// Emit a String body in array-item position, at the already-pushed
/// indent. Shared by the `Value::String` arm and by every leaf scalar
/// under `force_strings`.
fn render_string_as_item(
    s: &str,
    indent: usize,
    is_root_array_first: bool,
    out: &mut String,
) -> Result<()> {
    if s.contains('\r') {
        return Err(cr_error());
    }
    if string_needs_multiline(s) {
        // Match pair.rs: prefer indented stripped form for
        // readability, fall back to verbatim when stripped can't
        // round-trip the content; the shared chooser errors when
        // neither form can hold the body (§ 5.6.1).
        let form = choose_multiline_form(s, true)?;

        if matches!(form, MultilineForm::Stripped) {
            // Stripped form (default).
            out.push_str("(\n");
            let content_indent = indent + 1;
            for line in s.split('\n') {
                if !line.is_empty() {
                    push_indent(out, content_indent);
                    out.push_str(line);
                }
                out.push('\n');
            }
            push_indent(out, indent);
            out.push_str(")\n");
        } else {
            // Verbatim fallback.
            out.push_str("((\n");
            out.push_str(s);
            out.push('\n');
            push_indent(out, indent);
            out.push_str("))\n");
        }
    } else if s.is_empty() {
        // An empty-string item would otherwise render as a bare
        // indented blank line, which the parser treats as
        // decorative and drops. Force `::` so it stays a
        // recognisable literal-string entry.
        out.push_str("::\n");
    } else {
        // § 5.9.6 / § 5.9.12: when this is the FIRST item of an
        // Array root, the bare form is additionally not used if
        // the body satisfies § 5.0.1 rule 6's phase-1
        // pair-candidate test, OR — independently of that test —
        // the body begins with U+FEFF (bare form would place it
        // at byte offset 0, where § 3.1 makes readers strip it
        // as a metadata BOM). Both exclusions sit after the
        // empty (`::`) and multi-line branches, scoped to bodies
        // whose canonical form would otherwise be bare one-line.
        if item_needs_raw_marker(s)
            || (is_root_array_first
                && (bare_item_is_pair_candidate(s) || s.starts_with('\u{FEFF}')))
        {
            out.push_str(":: ");
        }
        out.push_str(s);
        out.push('\n');
    }
    Ok(())
}
