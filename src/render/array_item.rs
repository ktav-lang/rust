//! Render one line of an array (item can be scalar, object, or nested array).

use crate::error::Result;
use crate::value::Value;

use super::helpers::{item_needs_raw_marker, push_indent};
use super::object::render_object_body;

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
            if s.contains('\r') {
                return Err(crate::render::helpers::cr_error());
            }
            if crate::render::helpers::string_needs_multiline(s) {
                // Match pair.rs: prefer indented stripped form for
                // readability, fall back to verbatim when stripped can't
                // round-trip the content; the shared chooser errors when
                // neither form can hold the body (§ 5.6.1).
                let form = crate::render::helpers::choose_multiline_form(s, true)?;

                if matches!(form, crate::render::helpers::MultilineForm::Stripped) {
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
                if item_needs_raw_marker(s) {
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
