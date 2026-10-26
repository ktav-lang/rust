//! Минимальное раскрытие inline-обёрток при потере читаемости.

use super::{
    detect_eol, encode_inline, line_indent, parse_index, plan_value, Kind, Node, PlanCtx, Slot,
    SlotKind,
};
use crate::edit::EditError;
use crate::error::Span;
use crate::parser::inline::{
    find_unescaped_colon_inline, has_quote_bytes, parse_inline_array, process_escapes,
    split_top_level, InlineBody, InlineBounds, MAX_INLINE_DEPTH,
};
use crate::value::Value;
use crate::whitespace::is_ktav_whitespace;

pub(super) fn adjust<S: AsRef<str>>(
    source: &str,
    candidate: String,
    root: &Node,
    path: &[S],
    replacement: &Value,
    splice: (usize, usize, usize),
) -> Result<String, EditError> {
    let (from, to, inserted) = splice;
    let Some((start, end)) = inline_owner(source, root, path, from)? else {
        return Ok(candidate);
    };
    if from < start {
        return Ok(candidate);
    }
    let new_end = end
        .checked_sub(to - from)
        .and_then(|n| n.checked_add(inserted))
        .ok_or_else(|| unsupported("invalid layout range"))?;
    let body = candidate
        .get(start..new_end)
        .ok_or_else(|| unsupported("invalid layout boundaries"))?;
    if !is_compound(body) {
        return Ok(candidate);
    }
    let old_width = physical_line(source, from).chars().count();
    let limit = old_width.max(100);
    let force = requires_block(replacement, 0)?;
    if !force && physical_line(&candidate, from).chars().count() <= limit {
        return Ok(candidate);
    }
    let indent = line_indent(source, start);
    let eol = detect_eol(source, end);
    let Some(expanded) = expand(
        body,
        &indent,
        eol,
        limit,
        (from - start, from - start + inserted),
        force,
        0,
    )?
    else {
        return Ok(candidate);
    };
    let mut result = String::with_capacity(candidate.len());
    result.push_str(&candidate[..start]);
    result.push_str(&expanded);
    result.push_str(&candidate[new_end..]);
    Ok(result)
}

fn unsupported(reason: &str) -> EditError {
    EditError::Unsupported {
        reason: reason.into(),
    }
}

pub(super) fn physical_line(text: &str, pos: usize) -> &str {
    let start = text[..pos].rfind(['\n', '\r']).map_or(0, |n| n + 1);
    let end = text[pos..]
        .find(['\n', '\r'])
        .map_or(text.len(), |n| pos + n);
    &text[start..end]
}

fn is_compound(body: &str) -> bool {
    (body.starts_with('{') && body.ends_with('}')) || (body.starts_with('[') && body.ends_with(']'))
}

fn slot_body(slot: Slot) -> (usize, usize) {
    match slot {
        Slot::Pair { body, .. } | Slot::Item { body, .. } => (body.0 as usize, body.1 as usize),
        Slot::Block { start, end, .. } => (start as usize, end as usize),
    }
}

fn inline_owner<S: AsRef<str>>(
    source: &str,
    root: &Node,
    path: &[S],
    pos: usize,
) -> Result<Option<(usize, usize)>, EditError> {
    let mut node = root;
    if root.inline {
        let line = physical_line(source, pos);
        let body = line
            .trim_matches(is_ktav_whitespace)
            .trim_start_matches('\u{FEFF}');
        let start = body.as_ptr() as usize - source.as_ptr() as usize;
        return Ok(Some((start, start + body.len())));
    }
    for segment in path {
        node = match &node.kind {
            Kind::Object(_) => node.child(segment.as_ref()),
            Kind::Array(_) => node.item(parse_index(segment.as_ref())?),
            Kind::Scalar => None,
        }
        .ok_or(EditError::NotFound)?;
        if let Some(slot) = node.slot() {
            let (start, end) = slot_body(slot);
            if let Some(body) = source.get(start..end) {
                if matches!(node.kind, Kind::Object(_) | Kind::Array(_))
                    && is_compound(body)
                    && !body.contains(['\n', '\r'])
                {
                    return Ok(Some((start, end)));
                }
            }
        }
    }
    if node.inline {
        let line = physical_line(source, pos)
            .trim_matches(is_ktav_whitespace)
            .trim_start_matches('\u{FEFF}');
        let body = if is_compound(line) {
            line
        } else if let Some(colon) = find_unescaped_colon_inline(line) {
            line[colon + 1..].trim_start_matches(is_ktav_whitespace)
        } else {
            return Ok(None);
        };
        if is_compound(body) {
            let start = body.as_ptr() as usize - source.as_ptr() as usize;
            return Ok(Some((start, start + body.len())));
        }
    }
    Ok(None)
}

fn requires_block(value: &Value, depth: usize) -> Result<bool, EditError> {
    if depth > MAX_INLINE_DEPTH {
        return Err(unsupported(
            "replacement nesting exceeds inline depth limit (128)",
        ));
    }
    match value {
        Value::String(s) => {
            if s.contains(['\n', '\r']) {
                return Ok(true);
            }
            let encoded = encode_inline(s, false);
            let mut escapes = 0usize;
            let mut bytes = encoded.bytes();
            while let Some(byte) = bytes.next() {
                if byte == b'\\' {
                    escapes += 1;
                    bytes.next();
                }
            }
            let plain = s.chars().count();
            let overhead = encoded.chars().count().saturating_sub(plain);
            Ok(escapes >= 4 && overhead >= plain / 4 + usize::from(plain % 4 != 0))
        }
        Value::Object(object) => {
            for (_, child) in object.iter() {
                if requires_block(child, depth + 1)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Array(array) => {
            for child in array {
                if requires_block(child, depth + 1)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Null | Value::Bool(_) | Value::Integer(_) | Value::Float(_) => Ok(false),
    }
}

fn expand(
    body: &str,
    indent: &str,
    eol: &str,
    limit: usize,
    changed: (usize, usize),
    force: bool,
    depth: usize,
) -> Result<Option<String>, EditError> {
    if depth > MAX_INLINE_DEPTH {
        return Err(unsupported("layout nesting exceeds inline depth limit"));
    }
    let object = body.starts_with('{');
    let inner = &body[1..body.len() - 1];
    let parts = split_top_level(
        inner,
        1,
        Span { start: 0, end: 0 },
        if object {
            InlineBody::Object
        } else {
            InlineBody::Array
        },
        InlineBounds::for_input(body),
        has_quote_bytes(body.as_bytes()),
    )
    .map_err(|_| unsupported("cannot split inline layout"))?;
    let child_indent = format!("{indent}    ");
    let mut out = String::with_capacity(body.len());
    out.push(if object { '{' } else { '[' });
    for part in parts {
        let part = part.trim_matches(is_ktav_whitespace);
        if part.is_empty() {
            continue;
        }
        let offset = part.as_ptr() as usize - body.as_ptr() as usize;
        let touched = offset < changed.1 && offset + part.len() > changed.0;
        let (prefix, value, raw) = if object {
            let colon = find_unescaped_colon_inline(part)
                .ok_or_else(|| unsupported("inline pair has no separator"))?;
            let after = &part[colon + 1..];
            let raw = after.starts_with(':');
            let value = if raw { &after[1..] } else { after };
            let value = value.trim_start_matches(is_ktav_whitespace);
            (&part[..part.len() - value.len()], value, raw)
        } else {
            ("", part, false)
        };
        out.push_str(eol);
        out.push_str(&child_indent);
        if !raw
            && is_compound(value)
            && touched
            && (force || child_indent.chars().count() + part.chars().count() > limit)
        {
            let value_offset = value.as_ptr() as usize - body.as_ptr() as usize;
            let local = (
                changed.0.saturating_sub(value_offset),
                changed.1.saturating_sub(value_offset),
            );
            out.push_str(prefix);
            let Some(child) = expand(value, &child_indent, eol, limit, local, force, depth + 1)?
            else {
                return Ok(None);
            };
            out.push_str(&child);
        } else {
            let Some(child) = standalone(part, prefix, value, raw, object, &child_indent, eol)?
            else {
                return Ok(None);
            };
            out.push_str(&child);
        }
    }
    out.push_str(eol);
    out.push_str(indent);
    out.push(if object { '}' } else { ']' });
    Ok(Some(out))
}

/// Порог force-раскрытия для вставки (глубина > лимита — typed Err).
pub(super) fn requires_block_for(value: &Value) -> Result<bool, EditError> {
    requires_block(value, 0)
}
fn standalone(
    part: &str,
    prefix: &str,
    body: &str,
    raw: bool,
    object: bool,
    indent: &str,
    eol: &str,
) -> Result<Option<String>, EditError> {
    // При переносе inline escapes перестают декодироваться: проверяем семантику.
    let inline = if object {
        format!("{{{part}}}")
    } else {
        format!("[{part}]")
    };
    let expected = crate::parse(&inline).map_err(|_| unsupported("invalid inline layout"))?;
    let line = if object {
        part.to_string()
    } else {
        format!("[\n{part}\n]")
    };
    if crate::parse(&line).ok().as_ref() == Some(&expected) {
        return Ok(Some(part.to_string()));
    }
    let value = if raw {
        Value::String(
            process_escapes(body, 1, Span { start: 0, end: 0 })
                .map_err(|_| unsupported("invalid inline string"))?
                .into_owned()
                .into(),
        )
    } else {
        let wrapped = format!("[{body}]");
        let parsed = parse_inline_array(
            &wrapped,
            1,
            Span { start: 0, end: 0 },
            false,
            InlineBounds::for_input(&wrapped),
        )
        .map_err(|_| unsupported("cannot decode moved inline value"))?;
        match parsed {
            Value::Array(mut values) if values.len() == 1 => values.remove(0),
            _ => return Err(unsupported("moved inline value is not a single value")),
        }
    };
    if let Value::String(s) = &value {
        if s.contains('\r')
            || s.lines()
                .any(|line| line.trim_matches(|c: char| c == ' ' || c == '\t') == "))")
        {
            return Ok(None);
        }
    }
    let pc = PlanCtx {
        slot: if object {
            SlotKind::Pair
        } else {
            SlotKind::Item
        },
        role_item_block: false,
        existing_raw: raw,
        root_first: false,
        indent,
        eol,
        inline_compound: true,
        block_compound: false,
    };
    let (body, desired_raw) = plan_value(&value, &pc)?;
    let mut out = String::new();
    if object {
        let colon = find_unescaped_colon_inline(part)
            .ok_or_else(|| unsupported("moved pair has no separator"))?;
        out.push_str(&part[..colon + 1]);
        if desired_raw {
            out.push(':');
        }
        let ws = if raw {
            &prefix[colon + 2..]
        } else {
            &prefix[colon + 1..]
        };
        if ws.is_empty() {
            out.push(' ');
        } else {
            out.push_str(ws);
        }
    } else if desired_raw {
        out.push_str(":: ");
    }
    out.push_str(body.as_str());
    Ok(Some(out))
}
