//! Direct inline-compound scanner for the thin event path.
//!
//! Scans a closed inline compound (`{ … }` / `[ … ]`) into a sequence
//! of [`Event`]s through a borrowed intermediate `Node` tree — no
//! owned `Value`, no string copies: unmodified scalars and keys are
//! borrowed from the source line, and only genuine decoding or
//! normalization allocates in the arena (escape-decoded strings,
//! canonical integer/float forms via itoa/ryu).
//!
//! Transient state per compound: one flat `Vec<Event>` scratch that
//! the scan appends to — array items and nested arrays land there
//! directly in source order, so an array never materializes a
//! per-item node list — plus one insertion-ordered key table
//! (`InlineMap`, an `IndexMap`) per object scope, which
//! duplicate/conflict detection and dotted-key merge require. An
//! array that is directly an object member's value is staged as one
//! flat bracketed event block (`Node::Array(Vec<Event>)`) until its
//! object's insertion order is final. `scan_inline_events` copies
//! the finished scratch into its sink only after the whole compound
//! validates, so a failing compound pushes no events.
//!
//! Dotted keys inside the compound are expanded with the SAME shared
//! § 6.3 outcome tables the owned parser uses (`parser::insert`), so
//! duplicate/conflict diagnostics are byte-identical. Emission order
//! matches the owned path's `Value`-tree walk exactly: insertion
//! order per object, with dotted keys merged into shared sub-objects.

use std::borrow::Cow;

use bumpalo::Bump;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::error::{Error, ErrorKind, Span};
use crate::parser::classify::{is_float_literal, try_parse_integer};
use crate::parser::inline::{
    find_matching_close, find_unescaped_colon_inline, malformed_closer_not_at_end,
    parse_float_value, process_escapes, scan_inline_closer, split_top_level, InlineBody,
    InlineCloserScan, MAX_INLINE_DEPTH,
};
use crate::parser::insert::{insert_value, InsertShape, InsertTable, OccupiedShape};

use super::event::{Event, EventSink};

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

/// A lightweight inline value staged before emission. Scalars are
/// ready-made leaf events; an array is a flat, complete event block
/// (BeginArray .. EndArray) — arrays impose no ordering or duplicate
/// rules of their own, so source order is emission order and no
/// per-item nodes are staged; an object keeps the insertion-ordered
/// key table that duplicate/conflict detection and dotted-key merge
/// require.
pub(crate) enum Node<'a> {
    /// A scalar leaf (Null / Bool / Integer / Float / Str).
    Leaf(Event<'a>),
    /// Flat event block of an array value, brackets included.
    Array(Vec<Event<'a>>),
    Object(InlineMap<'a>),
}

/// Insertion-ordered table backing one inline object scope. Keys are
/// borrowed from the source when unescaped; escape-decoded keys stay
/// in a `Cow::Owned` until emission copies them into the arena.
pub(crate) type InlineMap<'a> = IndexMap<Cow<'a, str>, Node<'a>, FxBuildHasher>;

impl InsertShape for Node<'_> {
    fn is_object(&self) -> bool {
        matches!(self, Node::Object(_))
    }
    fn kind_label(&self) -> &'static str {
        match self {
            Node::Leaf(Event::Null) => "null",
            Node::Leaf(Event::Bool(_)) => "bool",
            Node::Leaf(Event::Integer(_)) => "integer",
            Node::Leaf(Event::Float(_)) => "float",
            Node::Leaf(Event::Str(_)) => "string",
            Node::Leaf(_) => unreachable!("not a value-start event"),
            Node::Array(_) => "array",
            Node::Object(_) => "object",
        }
    }
}

impl<'a> InsertTable<'a> for InlineMap<'a> {
    type Value = Node<'a>;

    fn insert_leaf(
        &mut self,
        key: Cow<'a, str>,
        value: Node<'a>,
    ) -> Result<(), (OccupiedShape, Node<'a>)> {
        match self.entry(key) {
            indexmap::map::Entry::Occupied(e) => Err((
                OccupiedShape {
                    is_object: e.get().is_object(),
                    label: e.get().kind_label(),
                },
                value,
            )),
            indexmap::map::Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        }
    }

    fn descend(&mut self, key: Cow<'a, str>) -> Result<&mut Self, ()> {
        match self.entry(key) {
            indexmap::map::Entry::Occupied(e) => match e.into_mut() {
                Node::Object(sub) => Ok(sub),
                _ => Err(()),
            },
            indexmap::map::Entry::Vacant(e) => {
                Ok(match e.insert(Node::Object(InlineMap::default())) {
                    Node::Object(sub) => sub,
                    _ => unreachable!("just inserted an object"),
                })
            }
        }
    }
}

/// Scan a closed inline compound body (`body` starts with `{` or `[`)
/// into `out`, emitting exactly the event sequence the former
/// owned-`Value` detour (`parse_inline_object`/`parse_inline_array` +
/// `value_to_events`) produced.
///
/// The § 5.2 rules-6–9 triage runs FIRST — `BadEscapeSequence`
/// (rules-6–9 preamble precedence), then `UnterminatedInlineCompound`,
/// then `MalformedInlineCompound` for a closer followed by content —
/// before any event is emitted and before any key/path state is
/// touched.
pub(crate) fn scan_inline_events<'a, S: EventSink<'a>>(
    body: &'a str,
    kind: InlineBody,
    line_num: usize,
    span: Span,
    bump: &'a Bump,
    out: &mut S,
) -> Result<(), Error> {
    let (open, close) = if kind == InlineBody::Object {
        (b'{', b'}')
    } else {
        (b'[', b']')
    };
    match scan_inline_closer(body, open, close, line_num, span) {
        InlineCloserScan::BadEscape(err) => return Err(err),
        InlineCloserScan::NotFound => {
            return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            }));
        }
        InlineCloserScan::Found(idx) if idx != body.len() - 1 => {
            return Err(malformed_closer_not_at_end(line_num, span));
        }
        InlineCloserScan::Found(_) => {}
    }
    let mut buf: Vec<Event<'a>> = Vec::new();
    match kind {
        InlineBody::Object => {
            let node = scan_inline_object(body, line_num, span, 0, bump)?;
            emit_node(&node, bump, &mut buf);
        }
        InlineBody::Array => {
            scan_inline_array_into(body, line_num, span, 0, bump, &mut buf)?;
        }
    }
    for ev in buf {
        EventSink::push(out, ev);
    }
    Ok(())
}

fn malformed(line_num: usize, span: Span, detail: &str) -> Error {
    Error::Structured(ErrorKind::MalformedInlineCompound {
        line: line_num as u32,
        span,
        detail: detail.to_string(),
    })
}

/// Port of `parse_inline_object_inner` (owned) — same segment rules,
/// same error strings, same § 6.3 tables via the shared `insert_value`.
fn scan_inline_object<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
) -> Result<Node<'a>, Error> {
    if depth >= MAX_INLINE_DEPTH {
        return Err(malformed(
            line_num,
            span,
            "nesting depth exceeds limit (128)",
        ));
    }

    debug_assert!(input.starts_with('{') && input.ends_with('}'));
    let inner = &input[1..input.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Node::Object(InlineMap::default()));
    }

    let segments = split_top_level(inner, line_num, span, InlineBody::Object)?;
    let mut map = InlineMap::default();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            // Trailing comma (last segment empty) is OK
            if i == n - 1 {
                break;
            }
            return Err(malformed(
                line_num,
                span,
                "empty pair segment (leading comma, double comma, or missing pair)",
            ));
        }

        let colon_pos = match find_unescaped_colon_inline(trimmed) {
            Some(p) => p,
            None => {
                return Err(malformed(
                    line_num,
                    span,
                    &format!("inline object pair missing ':' separator in '{trimmed}'"),
                ));
            }
        };

        let raw_key = &trimmed[..colon_pos];
        let after_colon = &trimmed[colon_pos + 1..];

        let (is_raw, value_body) = match after_colon.strip_prefix(':') {
            Some(stripped) => (true, stripped),
            None => (false, after_colon),
        };

        let key = raw_key.trim();
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }

        let node = if is_raw {
            let processed = process_escapes(value_body.trim(), line_num, span)?;
            Node::Leaf(Event::Str(cow_to_bump(processed, bump)))
        } else {
            scan_inline_value(value_body, line_num, span, depth, bump)?
        };

        insert_value(&mut map, key, node, line_num, span)?;
    }

    Ok(Node::Object(map))
}

/// Port of `parse_inline_array_inner` (owned). Appends the array's
/// complete event block — `BeginArray`, each item's events in source
/// order (nested arrays recurse into the same buffer, nested objects
/// emit their finished key-table walk), `EndArray` — to `buf`.
/// Nothing is written to the parser's sink here; the caller copies
/// `buf` only after the whole compound validates.
fn scan_inline_array_into<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
    buf: &mut Vec<Event<'a>>,
) -> Result<(), Error> {
    if depth >= MAX_INLINE_DEPTH {
        return Err(malformed(
            line_num,
            span,
            "nesting depth exceeds limit (128)",
        ));
    }

    debug_assert!(input.starts_with('[') && input.ends_with(']'));
    let inner = &input[1..input.len() - 1];
    if inner.trim().is_empty() {
        buf.push(Event::BeginArray);
        buf.push(Event::EndArray);
        return Ok(());
    }

    let segments = split_top_level(inner, line_num, span, InlineBody::Array)?;
    buf.push(Event::BeginArray);
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            // Trailing comma (last segment empty) is OK
            if i == n - 1 {
                break;
            }
            return Err(malformed(
                line_num,
                span,
                "empty inline-array item (leading comma, double comma, or empty position)",
            ));
        }

        if let Some(rest) = trimmed.strip_prefix("::") {
            let processed = process_escapes(rest.trim(), line_num, span)?;
            buf.push(Event::Str(cow_to_bump(processed, bump)));
            continue;
        }

        match scan_inline_value_trimmed(trimmed, line_num, span, depth, bump)? {
            Node::Leaf(ev) => buf.push(ev),
            Node::Array(events) => buf.extend_from_slice(&events),
            Node::Object(map) => emit_node(&Node::Object(map), bump, buf),
        }
    }
    buf.push(Event::EndArray);
    Ok(())
}

/// Port of `parse_inline_value` (owned): the value body is NOT yet trimmed.
fn scan_inline_value<'a>(
    body: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
) -> Result<Node<'a>, Error> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Node::Leaf(Event::Str("")));
    }
    scan_inline_value_trimmed(trimmed, line_num, span, depth, bump)
}

/// Port of `parse_inline_value_raw` + `classify_inline_scalar`
/// (owned, lax mode — the thin path has no strict variant). Note the
/// nested-compound triage is find-then-scan, exactly as the owned code:
/// `find_matching_close` first, `scan_inline_closer` only as fallback.
fn scan_inline_value_trimmed<'a>(
    trimmed: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
) -> Result<Node<'a>, Error> {
    let first_byte = trimmed.as_bytes()[0];

    if first_byte == b'{' || first_byte == b'[' {
        let (open, close) = if first_byte == b'{' {
            (b'{', b'}')
        } else {
            (b'[', b']')
        };
        let close_idx = match find_matching_close(trimmed, open, close) {
            Some(close) => Some(close),
            None => match scan_inline_closer(trimmed, open, close, line_num, span) {
                InlineCloserScan::Found(idx) => Some(idx),
                InlineCloserScan::NotFound => None,
                InlineCloserScan::BadEscape(err) => return Err(err),
            },
        };
        return match close_idx {
            // § 6.11: the matching closer is the last byte.
            Some(idx) if idx == trimmed.len() - 1 => {
                let inner = &trimmed[1..trimmed.len() - 1];
                if first_byte == b'[' {
                    let mut events = Vec::new();
                    scan_inline_array_into(trimmed, line_num, span, depth + 1, bump, &mut events)?;
                    return Ok(Node::Array(events));
                }
                if inner.trim().is_empty() {
                    return Ok(Node::Object(InlineMap::default()));
                }
                scan_inline_object(trimmed, line_num, span, depth + 1, bump)
            }
            // § 6.12: closer found, but content follows it.
            Some(_) => Err(malformed_closer_not_at_end(line_num, span)),
            // § 6.11: no closer at all.
            None => Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            })),
        };
    }

    // § 5.2 rule 5: `()` / `(())` → empty String.
    if trimmed == "()" || trimmed == "(())" {
        return Ok(Node::Leaf(Event::Str("")));
    }

    // A recognised escape anywhere forces String (rule 15); the decoded
    // body is never re-classified. Borrowed ⇒ no escape ⇒ classify.
    let processed = process_escapes(trimmed, line_num, span)?;
    match processed {
        Cow::Owned(s) => Ok(Node::Leaf(Event::Str(bump.alloc_str(&s)))),
        Cow::Borrowed(b) => classify_inline_scalar(b, line_num, span, bump),
    }
}

/// Port of `classify_inline_scalar` (owned, `strict = false`): keywords,
/// canonical integer/float forms, String fallback. Canonical strings that
/// reproduce the source borrow it; only reformatting allocates.
fn classify_inline_scalar<'a>(
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
fn cow_to_bump<'a>(cow: Cow<'a, str>, bump: &'a Bump) -> &'a str {
    match cow {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => bump.alloc_str(&s),
    }
}

/// Emit a finished node into the flat scratch buffer in the exact
/// order the former `value_to_events` walked the owned `Value` tree:
/// insertion order per object, nested Begin/End brackets per compound.
fn emit_node<'a>(node: &Node<'a>, bump: &'a Bump, buf: &mut Vec<Event<'a>>) {
    match node {
        Node::Leaf(ev) => buf.push(*ev),
        Node::Array(events) => buf.extend_from_slice(events),
        Node::Object(map) => {
            buf.push(Event::BeginObject);
            for (k, v) in map {
                let key: &'a str = match k {
                    Cow::Borrowed(s) => s,
                    Cow::Owned(s) => bump.alloc_str(s),
                };
                buf.push(Event::Key(key));
                emit_node(v, bump, buf);
            }
            buf.push(Event::EndObject);
        }
    }
}
