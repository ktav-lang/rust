//! Render one line of an array (item can be scalar, object, or nested array).

use crate::error::{Error, Result};
use crate::value::Value;

use super::helpers::{needs_raw_marker, push_indent};
use super::object::render_object_body;

/// Does `s` contain a line whose trimmed form is exactly `term`?
///
/// (Same role as in `pair.rs`: a content line trimming to the form's
/// terminator must not collide with that terminator — spec § 5.6.1.)
fn has_sole_terminator_line(s: &str, term: &str) -> bool {
    s.split('\n').any(|line| line.trim() == term)
}

pub(super) fn render_array_item(value: &Value, indent: usize, out: &mut String) -> Result<()> {
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
            out.push_str(s);
            out.push('\n');
        }
        Value::String(s) => {
            if s.contains('\n') {
                // Match pair.rs: prefer indented stripped form for readability,
                // fall back to verbatim when content has leading whitespace
                // (which stripped's dedent would clobber) or contains a
                // sole-`)` terminator line that would close the form early.
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
                         neither multi-line form can hold both (§ 5.6.1)."
                            .into(),
                    ));
                }

                if stripped_ok {
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
                if needs_raw_marker(s) {
                    out.push_str(":: ");
                }
                out.push_str(s);
                out.push('\n');
            }
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]\n");
            } else {
                out.push_str("[\n");
                for item in items {
                    render_array_item(item, indent + 1, out)?;
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
                render_object_body(obj, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}
