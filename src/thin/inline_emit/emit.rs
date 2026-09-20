use std::borrow::Cow;

use bumpalo::Bump;

use crate::error::{Error, Span};
use crate::parser::classify::{has_redundant_leading_zero, is_float_literal, try_parse_integer};
use crate::parser::inline::parse_float_value;

use super::super::event::Event;
use super::node::Node;

/// Same plain-decimal fast path as `parser::classify`'s (identical
/// code): a canonical ASCII decimal integer borrows the source slice,
/// skipping itoa + arena allocation.
#[inline]
pub(crate) fn fast_plain_decimal_i64(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0];
    if first == b'0' {
        return if bytes.len() == 1 { Some(0) } else { None };
    }
    if !(b'1'..=b'9').contains(&first) {
        return None;
    }
    let mut acc: i64 = (first - b'0') as i64;
    for &b in &bytes[1..] {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(d as i64)?;
    }
    Some(acc)
}

/// Port of `classify_inline_scalar` (owned, `strict = false`): keywords,
/// canonical integer/float forms, String fallback. Canonical strings that
/// reproduce the source borrow it; only reformatting allocates.
pub(super) fn classify_inline_scalar<'a>(
    body: &'a str,
    _line_num: usize,
    _span: Span,
    bump: &'a Bump,
) -> Result<Node<'a>, Error> {
    if body.is_empty() {
        return Ok(Node::Leaf(Event::Str("")));
    }

    match body {
        "null" => return Ok(Node::Leaf(Event::Null)),
        "true" => return Ok(Node::Leaf(Event::Bool(true))),
        "false" => return Ok(Node::Leaf(Event::Bool(false))),
        _ => {}
    }

    // § 5.2 rules 13-14 exception: a redundant leading zero is never a
    // number, so the thin API reports the same String the owned API does.
    if has_redundant_leading_zero(body) {
        return Ok(Node::Leaf(Event::Str(body)));
    }

    if fast_plain_decimal_i64(body).is_some() {
        return Ok(Node::Leaf(Event::Integer(body)));
    }
    if let Some(val) = try_parse_integer(body) {
        let mut buf = itoa::Buffer::new();
        let canonical = buf.format(val);
        return Ok(Node::Leaf(Event::Integer(bump.alloc_str(canonical))));
    }

    if is_float_literal(body) {
        if let Some(val) = parse_float_value(body) {
            let mut buf = ryu::Buffer::new();
            let canonical = buf.format(val);
            if canonical == body {
                return Ok(Node::Leaf(Event::Float(body)));
            }
            return Ok(Node::Leaf(Event::Float(bump.alloc_str(canonical))));
        }
    }

    Ok(Node::Leaf(Event::Str(body)))
}

#[inline]
pub(super) fn cow_to_bump<'a>(cow: Cow<'a, str>, bump: &'a Bump) -> &'a str {
    match cow {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => bump.alloc_str(&s),
    }
}

/// Emit a finished node into the flat scratch buffer in the exact
/// order the former `value_to_events` walked the owned `Value` tree:
/// insertion order per object, nested Begin/End brackets per compound.
/// `transfers` counts moves/copies as in `scan_inline_array_into`.
pub(super) fn emit_node<'a>(
    node: &Node<'a>,
    bump: &'a Bump,
    buf: &mut Vec<Event<'a>>,
    transfers: &mut u64,
) {
    match node {
        Node::Leaf(ev) => {
            buf.push(*ev);
            *transfers += 1;
        }
        Node::Array(events) => {
            *transfers += events.len() as u64;
            buf.extend_from_slice(events);
        }
        Node::Object(map) => {
            buf.push(Event::BeginObject);
            *transfers += 1;
            for (k, v) in map {
                let key: &'a str = match k {
                    Cow::Borrowed(s) => s,
                    Cow::Owned(s) => bump.alloc_str(s),
                };
                buf.push(Event::Key(key));
                *transfers += 1;
                emit_node(v, bump, buf, transfers);
            }
            buf.push(Event::EndObject);
            *transfers += 1;
        }
    }
}
