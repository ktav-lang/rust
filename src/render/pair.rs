//! Render a `key: value` (or `key:: value` / `key: { ... }` / `key: [ ... ]`)
//! line. Under spec 0.5.0, integers and floats are emitted with plain `:`
//! (no `:i` / `:f` markers).

use crate::error::{Error, Result};
use crate::value::Value;

use super::array_item::render_array_item;
use super::helpers::{needs_raw_marker, push_escaped_key_segment, push_indent};
use super::object::render_object_body;

/// Does `s` contain a line whose trimmed form is exactly `term`?
///
/// A line of the multi-line `s` whose trimmed content is exactly `)`
/// (for stripped) or `))` (for verbatim) would be read back as the
/// block terminator — so the renderer must avoid that form for such
/// content (spec § 5.6.1).
fn has_sole_terminator_line(s: &str, term: &str) -> bool {
    s.split('\n').any(|line| line.trim() == term)
}

pub(super) fn render_pair(key: &str, value: &Value, indent: usize, out: &mut String) -> Result<()> {
    push_indent(out, indent);
    // Spec 0.6.0 § 3.7 — re-escape `\`, `.`, `:` in each key segment
    // so the parser reads the same segment back. The Object value
    // stores the *decoded* leaf key as a single byte string; the
    // emitted form must escape any literal dot/colon.
    push_escaped_key_segment(key, out);
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
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        Value::String(s) => {
            if s.contains('\n') {
                // Pick the form whose terminator doesn't clash with the
                // content (spec § 5.6.1). Prefer **stripped** (`(` ... `)`)
                // because indented output is much more readable; fall back
                // to **verbatim** (`((` ... `))`) when stripped can't
                // round-trip the content losslessly.
                //
                // Stripped's parser dedents by the *minimum* leading
                // whitespace across non-blank lines, so any content that
                // has its own leading whitespace gets clobbered. We use
                // stripped only when no content line begins with
                // whitespace; otherwise verbatim (which copies bytes).
                //
                // Forms also collide with terminator lines: a sole-`)`
                // line breaks stripped, a sole-`))` line breaks verbatim.
                let has_sole_single = has_sole_terminator_line(s, ")");
                let has_sole_double = has_sole_terminator_line(s, "))");
                let has_leading_ws = s
                    .split('\n')
                    .any(|line| !line.is_empty() && line.starts_with(|c: char| c.is_whitespace()));

                let stripped_ok = !has_sole_single && !has_leading_ws;
                let verbatim_ok = !has_sole_double;

                if !stripped_ok && !verbatim_ok {
                    return Err(Error::Message(
                        "String cannot round-trip through Ktav 0.1.0 — content \
                         has both a sole-`)` line and a sole-`))` line; \
                         neither multi-line form can hold both (§ 5.6.1). \
                         Split the value across adjacent multi-line pairs."
                            .into(),
                    ));
                }

                if stripped_ok {
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
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str(": []\n");
            } else {
                out.push_str(": [\n");
                for item in items {
                    render_array_item(item, indent + 1, out)?;
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
                render_object_body(obj, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}
