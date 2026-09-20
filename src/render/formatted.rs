//! Comment-preserving writer that backs [`crate::format_str`].
//!
//! Mirrors [`super::canonical`]'s recursive descent exactly — same
//! indentation, same key-quoting / string-escaping / number-
//! canonicalisation rules, same root-array wrap safeguard — reusing its
//! string- and key-emission helpers unchanged. The only addition is
//! flushing each [`crate::parser::fmt_parser::TriviaLine`] run
//! immediately before the construct it is attached to.

use crate::error::{Error, ReasonCode, Result};
use crate::parser::fmt_parser::{to_plain_value, FmtDoc, PArray, PObject, PValue, TriviaLine};

use super::canonical::{canonical_float_checked, emit_string_as_item, emit_string_in_pair};
use super::helpers::{empty_key_error, push_escaped_key_segment, push_indent};

/// Format a trivia-carrying document to Ktav text (§ 5.9 canonical
/// structure, with comments and blank-line grouping preserved).
///
/// Like the canonical writer, this detects the § 5.9.0 conditions as it
/// emits rather than in a pre-pass. The pre-pass was more expensive here
/// than anywhere else: it called `to_plain_value` to materialise an
/// entire throw-away `Value` tree from the trivia-carrying one — a fresh
/// `ObjectMap` per object, every key and scalar cloned — purely so the
/// shared predicate had something of the right type to walk. Trivia
/// cannot change representability, so nothing about that copy affected
/// the answer.
pub(crate) fn emit_formatted(doc: &FmtDoc) -> Result<String> {
    let mut out = String::new();
    emit_trivia(&doc.leading, 0, &mut out);
    let emitted = match &doc.root {
        PValue::Object(o) => emit_object_pairs(o, 0, true, &mut out),
        PValue::Array(a) if a.items.is_empty() && a.trailing.is_empty() => {
            out.push_str("[]\n");
            Ok(())
        }
        PValue::Array(a) => emit_array_root(a, &mut out),
        // § 5.0.1 root-kind detection cannot produce a scalar root, so
        // this is unreachable in practice — reported rather than
        // panicked, because a writer rejection is the honest outcome if
        // the invariant ever breaks.
        _ => {
            return Err(Error::UnrepresentableAt {
                code: ReasonCode::ScalarRoot,
                path: Vec::new(),
            })
        }
    };
    match emitted {
        Ok(()) => Ok(out),
        // Cold: `to_plain_value` is paid only once emission has already
        // failed, to name the offending key.
        Err(e) => Err(super::representable::attach_path(
            &to_plain_value(&doc.root),
            e,
            false,
        )),
    }
}

fn emit_trivia(trivia: &[TriviaLine], indent: usize, out: &mut String) {
    for line in trivia {
        match line {
            TriviaLine::Blank => out.push('\n'),
            TriviaLine::Comment(text) => {
                push_indent(out, indent);
                out.push_str(text);
                out.push('\n');
            }
        }
    }
}

fn emit_object_pairs(obj: &PObject, indent: usize, is_root: bool, out: &mut String) -> Result<()> {
    for (index, (k, (trivia, v))) in obj.pairs.iter().enumerate() {
        emit_trivia(trivia, indent, out);
        emit_pair(k, v, indent, is_root && index == 0, out)?;
    }
    emit_trivia(&obj.trailing, indent, out);
    Ok(())
}

fn emit_array_root(arr: &PArray, out: &mut String) -> Result<()> {
    // Wrap when the first item is itself a compound (§ 5.9.3 lone-`{`/`[`
    // safeguard — mirrors `helpers::first_item_needs_wrap`), or when the
    // root array is structurally empty but still carries trivia (e.g. a
    // comment inside an otherwise-empty `[ ]`): the compact `[]` form has
    // nowhere to put it.
    let first_is_compound =
        !arr.items.is_empty() && matches!(&arr.items[0].1, PValue::Object(_) | PValue::Array(_));
    let needs_wrap = first_is_compound || (arr.items.is_empty() && !arr.trailing.is_empty());

    if needs_wrap {
        out.push_str("[\n");
        emit_array_items(arr, 1, out)?;
        out.push_str("]\n");
    } else {
        for (index, (trivia, item)) in arr.items.iter().enumerate() {
            emit_trivia(trivia, 0, out);
            emit_array_item(item, 0, index == 0, out)?;
        }
        emit_trivia(&arr.trailing, 0, out);
    }
    Ok(())
}

/// Emit an Array's items (never root-detected) at `indent`, followed by
/// its trailing trivia. Shared by a pair's Array value, a nested array
/// item, and a wrapped root array.
fn emit_array_items(arr: &PArray, indent: usize, out: &mut String) -> Result<()> {
    for (trivia, item) in &arr.items {
        emit_trivia(trivia, indent, out);
        emit_array_item(item, indent, false, out)?;
    }
    emit_trivia(&arr.trailing, indent, out);
    Ok(())
}

fn emit_pair(
    key: &str,
    value: &PValue,
    indent: usize,
    root_first_key: bool,
    out: &mut String,
) -> Result<()> {
    // § 5.9.0 EmptyKeyName, before the key's first byte is pushed.
    if key.is_empty() {
        return Err(empty_key_error());
    }
    push_indent(out, indent);
    push_escaped_key_segment(key, root_first_key, out);
    match value {
        PValue::Null => out.push_str(": null\n"),
        PValue::Bool(b) => {
            out.push_str(": ");
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        PValue::Integer(s) => {
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        PValue::Float(s) => {
            out.push_str(": ");
            out.push_str(&canonical_float_checked(s)?);
            out.push('\n');
        }
        PValue::String(s) => emit_string_in_pair(s, indent, out)?,
        PValue::Array(a) => {
            if a.items.is_empty() && a.trailing.is_empty() {
                out.push_str(": []\n");
            } else {
                out.push_str(": [\n");
                emit_array_items(a, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        PValue::Object(o) => {
            if o.pairs.is_empty() && o.trailing.is_empty() {
                out.push_str(": {}\n");
            } else {
                out.push_str(": {\n");
                emit_object_pairs(o, indent + 1, false, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

fn emit_array_item(
    value: &PValue,
    indent: usize,
    is_root_array_first: bool,
    out: &mut String,
) -> Result<()> {
    push_indent(out, indent);
    match value {
        PValue::Null => out.push_str("null\n"),
        PValue::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        PValue::Integer(s) => {
            out.push_str(s);
            out.push('\n');
        }
        PValue::Float(s) => {
            out.push_str(&canonical_float_checked(s)?);
            out.push('\n');
        }
        PValue::String(s) => emit_string_as_item(s, indent, is_root_array_first, out)?,
        PValue::Array(a) => {
            if a.items.is_empty() && a.trailing.is_empty() {
                out.push_str("[]\n");
            } else {
                out.push_str("[\n");
                emit_array_items(a, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        PValue::Object(o) => {
            if o.pairs.is_empty() && o.trailing.is_empty() {
                out.push_str("{}\n");
            } else {
                out.push_str("{\n");
                emit_object_pairs(o, indent + 1, false, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}
