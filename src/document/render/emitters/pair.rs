//! Render a `key: value` (or `key:: value` / `key: { ... }` / `key: [ ... ]`)
//! line. Under spec 0.5.0, integers and floats are emitted with plain `:`
//! (no `:i` / `:f` markers).

use crate::error::Result;
use crate::value::Value;

use super::array_item::render_array_item;
use super::helpers::{
    choose_multiline_form, cr_error, empty_key_error, needs_raw_marker, push_escaped_key_segment,
    push_indent, reject_non_finite, string_needs_multiline, MultilineForm,
};
use super::object::render_object_body;
use super::render::forced_string_text;

pub(super) fn render_pair(
    key: &str,
    value: &Value,
    indent: usize,
    root_first_key: bool,
    force_strings: bool,
    out: &mut String,
) -> Result<()> {
    // § 5.9.0 EmptyKeyName, raised before the key's first byte is
    // pushed so the check keeps its place in document order.
    if key.is_empty() {
        return Err(empty_key_error());
    }
    push_indent(out, indent);
    // Spec 0.7 § 5.9.10 — bare/quoted form selection + re-escape per
    // segment. The Object value stores the *decoded* leaf key as a
    // single byte string; the emitted form must escape (or quote
    // around) any literal structural byte. `root_first_key` is set
    // only for the document-root Object's first-serialized key (the
    // § 5.9.10 rule (c) U+FEFF guard, § 5.9.12).
    push_escaped_key_segment(key, root_first_key, out);

    // `to_string_force_strings`: every leaf scalar takes the String
    // emission path below, on its textual form. Compounds fall through
    // and keep their structure.
    if force_strings {
        if let Some(text) = forced_string_text(value) {
            return render_string_in_pair(text, indent, out);
        }
    }

    match value {
        Value::Null => {
            out.push_str(": null\n");
        }
        Value::Bool(b) => {
            out.push_str(": ");
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        Value::Integer(s) => {
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        Value::Float(s) => {
            // § 5.9.0 NonFiniteFloat. The pretty writer emits the stored
            // payload verbatim, so unlike the canonical writer it has no
            // canonicalisation step to learn the value from.
            reject_non_finite(s)?;
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        Value::String(s) => return render_string_in_pair(s, indent, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str(": []\n");
            } else {
                out.push_str(": [\n");
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
                out.push_str(": {}\n");
            } else {
                out.push_str(": {\n");
                render_object_body(obj, indent + 1, false, force_strings, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

/// Emit a String body in pair position, starting at the `:` that follows
/// the already-emitted key. Shared by the `Value::String` arm and by
/// every leaf scalar under `force_strings`.
fn render_string_in_pair(s: &str, indent: usize, out: &mut String) -> Result<()> {
    if s.contains('\r') {
        return Err(cr_error());
    }
    if string_needs_multiline(s) {
        // Pick the form whose terminator doesn't clash with the
        // content (spec § 5.6.1). Prefer **stripped** (`(` ... `)`)
        // because indented output is much more readable; fall back
        // to **verbatim** (`((` ... `))`) when stripped can't
        // round-trip the content losslessly. The shared chooser
        // errors when neither form can hold the body.
        let form = choose_multiline_form(s, true)?;

        if matches!(form, MultilineForm::Stripped) {
            // Stripped form (default). Each content line gets a
            // `content_indent` prefix; the dedent on parse strips
            // it back off, so the round-trip is byte-exact (blank
            // lines inside `s` remain blank: spec § 5.6 replaces
            // them with the empty string).
            out.push_str(": (\n");
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
            // Verbatim form (fallback). Exactly one `\n` is pushed
            // after `s`: if `s` already ends with `\n`, the result
            // is `...\n\n` before `))`, i.e. a blank content line
            // that preserves the trailing newline through the
            // verbatim-join round-trip.
            out.push_str(": ((\n");
            out.push_str(s);
            out.push('\n');
            push_indent(out, indent);
            out.push_str("))\n");
        }
    } else {
        if needs_raw_marker(s) {
            out.push_str(":: ");
        } else {
            out.push_str(": ");
        }
        out.push_str(s);
        out.push('\n');
    }
    Ok(())
}
