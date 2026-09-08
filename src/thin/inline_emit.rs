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
    find_matching_close, find_unescaped_colon_inline, has_quote_bytes, malformed_closer_not_at_end,
    parse_float_value, process_escapes, scan_inline_closer, scan_inline_closer_with_bounds,
    split_top_level, InlineBody, InlineBounds, InlineCloserScan, MAX_INLINE_DEPTH,
};
use crate::parser::insert::{insert_value, InsertShape, InsertTable, OccupiedShape};
use crate::whitespace::is_inline_whitespace;

use super::event::{Event, EventSink};

// Deterministic instrument for the shared-buffer property: counts
// every `Event` value the scanner moves into a scratch buffer or
// sink, plus the block copies between buffers that nested-array
// staging used to make quadratic (3 + 5 + … + (2D−1) = D² − 1 extra
// copies for a chain of D nested arrays). A parse that lands every
// event in the shared compound scratch exactly once counts each
// event twice — once into the scratch, once into the sink.
// Accumulated in test builds only; in non-test builds
// `record_event_transfers` is a no-op and the counters optimize
// away.
#[cfg(test)]
thread_local! {
    static EVENT_TRANSFERS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Add `n` moved/copied events to this thread's total (test builds only).
#[cfg(test)]
fn record_event_transfers(n: u64) {
    EVENT_TRANSFERS.with(|total| total.set(total.get() + n));
}

/// Read and reset this thread's moved/copied-event total.
#[cfg(test)]
pub(crate) fn take_event_transfers() -> u64 {
    EVENT_TRANSFERS.with(|total| {
        let n = total.get();
        total.set(0);
        n
    })
}

#[cfg(not(test))]
#[inline(always)]
fn record_event_transfers(n: u64) {
    let _ = n;
}

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
///
/// That gate scan records the compound's boundary map once
/// (`InlineBounds`), which is handed down to the scan.
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
    let mut bounds_pairs = Vec::new();
    match scan_inline_closer_with_bounds(body, open, close, line_num, span, &mut bounds_pairs) {
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
    let bounds = InlineBounds::over(body, &bounds_pairs);
    // R10-F1: quote presence is computed ONCE over the root body and
    // threaded down — every nested slice is a substring of this body,
    // so a quote byte at the root implies one in every descendant
    // (false positives are harmless: the fast and quote-aware machines
    // are byte-identical on quote-free slices, R8-F2).
    let has_quotes = has_quote_bytes(body.as_bytes());
    let mut buf: Vec<Event<'a>> = Vec::new();
    let mut transfers: u64 = 0;
    match kind {
        InlineBody::Object => {
            let node = scan_inline_object(body, line_num, span, 0, bump, bounds, has_quotes)?;
            emit_node(&node, bump, &mut buf, &mut transfers);
        }
        InlineBody::Array => {
            scan_inline_array_into(
                body,
                line_num,
                span,
                0,
                bump,
                &mut buf,
                bounds,
                has_quotes,
                &mut transfers,
            )?;
        }
    }
    let sink_moves = buf.len() as u64;
    for ev in buf {
        EventSink::push(out, ev);
    }
    record_event_transfers(transfers + sink_moves);
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
///
/// `has_quotes` (R10-F1) is the root body's quote presence threaded
/// from [`scan_inline_events`]; computing it per level would re-scan
/// every descendant subtree once per nesting level.
fn scan_inline_object<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
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
    // Inline scan: trim with the crate-internal § 3.3 INLINE view —
    // segments are carved from one § 3.2-pre-split line, so raw LF/CR
    // bytes cannot occur here.
    if inner.trim_matches(is_inline_whitespace).is_empty() {
        return Ok(Node::Object(InlineMap::default()));
    }

    let segments = split_top_level(
        inner,
        line_num,
        span,
        InlineBody::Object,
        bounds,
        has_quotes,
    )?;
    let mut map = InlineMap::default();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let trimmed = seg.trim_matches(is_inline_whitespace);
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

        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let key = raw_key.trim_matches(is_inline_whitespace);
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }

        let node = if is_raw {
            // Inline view trim: value_body holds raw source chars from a
            // § 3.2-pre-split line (escape sequences like `\n` are two
            // raw chars, not a raw LF byte), so LF/CR cannot occur.
            let processed = process_escapes(
                value_body.trim_matches(is_inline_whitespace),
                line_num,
                span,
            )?;
            Node::Leaf(Event::Str(cow_to_bump(processed, bump)))
        } else {
            scan_inline_value(value_body, line_num, span, depth, bump, bounds, has_quotes)?
        };

        insert_value(&mut map, key, node, line_num, span)?;
    }

    Ok(Node::Object(map))
}

/// Port of `parse_inline_array_inner` (owned). Appends the array's
/// complete event block — `BeginArray`, each item's events in source
/// order, `EndArray` — to `buf`. Nested arrays run the SAME § 5.2
/// rules-6–9 closer triage as `scan_inline_value_trimmed`, then
/// recurse into this same `buf`: no per-item block is built and no
/// block is copied back into the parent, so each event of a nested
/// chain is moved exactly once into the buffer. Nested objects cannot
/// take that shortcut — their event sequence only exists once the
/// object's key table is final (insertion order, § 6.3 merge) — so
/// they come back as a finished key-table walk. Nothing is written to
/// the parser's sink here; the caller copies `buf` only after the
/// whole compound validates. `transfers` counts every `Event` value
/// moved into a scratch buffer or copied between buffers (the
/// deterministic instrument behind the `tests` module).
#[allow(clippy::too_many_arguments)]
fn scan_inline_array_into<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
    buf: &mut Vec<Event<'a>>,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
    transfers: &mut u64,
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
    // Inline scan: LF/CR cannot occur here — the input is carved from
    // one § 3.2-pre-split line.
    if inner.trim_matches(is_inline_whitespace).is_empty() {
        *transfers += 2;
        buf.push(Event::BeginArray);
        buf.push(Event::EndArray);
        return Ok(());
    }

    let segments = split_top_level(inner, line_num, span, InlineBody::Array, bounds, has_quotes)?;
    buf.push(Event::BeginArray);
    *transfers += 1;
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let trimmed = seg.trim_matches(is_inline_whitespace);
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

        // Nested array item: same memo-then-find-then-scan closer
        // triage as `scan_inline_value_trimmed`, and only once the
        // closer is known to END the item does the recursion run —
        // straight into THIS buffer, so the child never materializes
        // a block for the parent to copy.
        if trimmed.starts_with('[') {
            match find_inline_compound_close(trimmed, b'[', b']', line_num, span, bounds)? {
                // § 6.11: the matching closer is the last byte.
                Some(idx) if idx == trimmed.len() - 1 => {
                    scan_inline_array_into(
                        trimmed,
                        line_num,
                        span,
                        depth + 1,
                        bump,
                        buf,
                        bounds,
                        has_quotes,
                        transfers,
                    )?;
                }
                // § 6.12: closer found, but content follows it.
                Some(_) => return Err(malformed_closer_not_at_end(line_num, span)),
                // § 6.11: no closer at all.
                None => {
                    return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                        line: line_num as u32,
                        span,
                    }));
                }
            }
            continue;
        }

        match scan_inline_value_trimmed(trimmed, line_num, span, depth, bump, bounds, has_quotes)? {
            Node::Leaf(ev) => {
                buf.push(ev);
                *transfers += 1;
            }
            // Unreachable since nested-array items recurse above;
            // kept for exhaustiveness.
            Node::Array(events) => {
                *transfers += events.len() as u64;
                buf.extend_from_slice(&events);
            }
            Node::Object(map) => emit_node(&Node::Object(map), bump, buf, transfers),
        }
    }
    buf.push(Event::EndArray);
    *transfers += 1;
    Ok(())
}

/// Port of `parse_inline_value` (owned): the value body is NOT yet trimmed.
fn scan_inline_value<'a>(
    body: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Node<'a>, Error> {
    // Inline view trim: the body is a slice of one § 3.2-pre-split
    // line, so raw LF/CR bytes cannot occur.
    let trimmed = body.trim_matches(is_inline_whitespace);
    if trimmed.is_empty() {
        return Ok(Node::Leaf(Event::Str("")));
    }
    scan_inline_value_trimmed(trimmed, line_num, span, depth, bump, bounds, has_quotes)
}

/// The § 5.2 rules-6–9 closer triage for a compound-starting value,
/// shared verbatim by `scan_inline_value_trimmed` and the nested-array
/// path in `scan_inline_array_into`: the gate-scan memo
/// (`known_closer`) first, then `find_matching_close`, then
/// `scan_inline_closer` as the fallback that keeps
/// `BadEscapeSequence` precedence for non-memo spans. `Ok(Some(idx))`
/// is the closer's offset in `trimmed` (§ 6.11 wants
/// `idx == trimmed.len() - 1`), `Ok(None)` — no closer at all,
/// `Err` — a bad escape found on the way.
fn find_inline_compound_close(
    trimmed: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
    bounds: InlineBounds<'_>,
) -> Result<Option<usize>, Error> {
    match bounds.known_closer(trimmed) {
        Some(idx) => Ok(Some(idx)),
        None => match find_matching_close(trimmed, open, close) {
            Some(close) => Ok(Some(close)),
            None => match scan_inline_closer(trimmed, open, close, line_num, span) {
                InlineCloserScan::Found(idx) => Ok(Some(idx)),
                InlineCloserScan::NotFound => Ok(None),
                InlineCloserScan::BadEscape(err) => Err(err),
            },
        },
    }
}

/// Port of `parse_inline_value_raw` + `classify_inline_scalar`
/// (owned, lax mode — the thin path has no strict variant). Note the
/// nested-compound triage (`find_inline_compound_close`) is
/// memo-then-find-then-scan, exactly as the
/// owned code: the gate-scan memo hit short-circuits both scans with
/// the identical verdict; the fallback keeps `BadEscapeSequence`
/// precedence for non-memo spans (`find_matching_close` first,
/// `scan_inline_closer` only as fallback).
fn scan_inline_value_trimmed<'a>(
    trimmed: &'a str,
    line_num: usize,
    span: Span,
    depth: usize,
    bump: &'a Bump,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Node<'a>, Error> {
    let first_byte = trimmed.as_bytes()[0];

    if first_byte == b'{' || first_byte == b'[' {
        let (open, close) = if first_byte == b'{' {
            (b'{', b'}')
        } else {
            (b'[', b']')
        };
        return match find_inline_compound_close(trimmed, open, close, line_num, span, bounds)? {
            // § 6.11: the matching closer is the last byte.
            Some(idx) if idx == trimmed.len() - 1 => {
                let inner = &trimmed[1..trimmed.len() - 1];
                if first_byte == b'[' {
                    // An array here is an OBJECT member's value: it is
                    // staged as one flat block (`Node::Array`) until the
                    // enclosing object's insertion order is final. That
                    // staging stays — § 5.3.2 dotted re-entry can add
                    // merge-pairs to an already-seen key, so member
                    // values are emitted from the object's final walk,
                    // never at arrival time.
                    let mut events = Vec::new();
                    let mut block_transfers = 0u64;
                    scan_inline_array_into(
                        trimmed,
                        line_num,
                        span,
                        depth + 1,
                        bump,
                        &mut events,
                        bounds,
                        has_quotes,
                        &mut block_transfers,
                    )?;
                    record_event_transfers(block_transfers);
                    return Ok(Node::Array(events));
                }
                // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
                if inner.trim_matches(is_inline_whitespace).is_empty() {
                    return Ok(Node::Object(InlineMap::default()));
                }
                scan_inline_object(trimmed, line_num, span, depth + 1, bump, bounds, has_quotes)
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
/// `transfers` counts moves/copies as in `scan_inline_array_into`.
fn emit_node<'a>(node: &Node<'a>, bump: &'a Bump, buf: &mut Vec<Event<'a>>, transfers: &mut u64) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Collecting sink for direct `scan_inline_events` calls.
    struct CollectingSink<'a> {
        events: Vec<Event<'a>>,
    }

    impl<'a> EventSink<'a> for CollectingSink<'a> {
        fn push(&mut self, event: Event<'a>) {
            self.events.push(event);
        }
    }

    /// `[[[[…1…]]]]` with `depth` bracket pairs.
    fn array_chain(depth: usize) -> String {
        let mut body = String::new();
        body.push_str(&"[".repeat(depth));
        body.push('1');
        body.push_str(&"]".repeat(depth));
        body
    }

    /// Scan one inline array compound; returns (emitted events, transfers).
    fn scan_chain(body: &str) -> (usize, u64) {
        let bump = Bump::new();
        let mut sink = CollectingSink { events: Vec::new() };
        take_event_transfers();
        scan_inline_events(
            body,
            InlineBody::Array,
            1,
            Span::new(0, 0),
            &bump,
            &mut sink,
        )
        .expect("chain must scan");
        (sink.events.len(), take_event_transfers())
    }

    /// R8-F1 pin: in a chain of D nested arrays every event is moved
    /// exactly twice — once into the shared scratch buffer, once into
    /// the sink. The pre-fix staged-block path moved D² − 1 extra
    /// copies (D² + 4D + 1 transfers total: quadratic); any silent
    /// regression to per-item blocks breaks the exact count.
    #[test]
    fn nested_array_chain_moves_each_event_exactly_twice() {
        for depth in [1usize, 2, 3, 4, 8, 16, 32, 64] {
            let (events, transfers) = scan_chain(&array_chain(depth));
            assert_eq!(events, 2 * depth + 1, "events at depth {depth}");
            assert_eq!(
                transfers,
                (2 * (2 * depth + 1)) as u64,
                "transfers at depth {depth}"
            );
        }
    }

    /// Growth-shape pin independent of the exact per-event accounting:
    /// a 4× depth jump must multiply transfers by ~4 (linear), not
    /// ~13–16 (quadratic). Headroom 5× keeps the pin deterministic
    /// while a quadratic regression (≥ 13×) fails it loudly.
    #[test]
    fn nested_array_transfers_grow_linearly_with_depth() {
        let (_, small) = scan_chain(&array_chain(16));
        let (_, large) = scan_chain(&array_chain(64));
        assert_eq!(small, 66);
        assert_eq!(large, 258);
        assert!(
            large < 5 * small,
            "transfers grew super-linearly: {small} -> {large}"
        );
    }

    /// Positive control for the instrument: object-member arrays are
    /// still staged as one block (deliberately — § 5.3.2 dotted
    /// re-entry ordering), which shows up as block-build plus
    /// block-copy transfers ABOVE the once-into-scratch,
    /// once-into-sink floor of 2 × events.
    #[test]
    fn object_member_array_staging_is_visible_to_the_counter() {
        let bump = Bump::new();
        let mut sink = CollectingSink { events: Vec::new() };
        take_event_transfers();
        scan_inline_events(
            "{a: [1, [2]]}",
            InlineBody::Object,
            1,
            Span::new(0, 0),
            &bump,
            &mut sink,
        )
        .expect("object must scan");
        let events = sink.events.len();
        let transfers = take_event_transfers();
        assert_eq!(events, 9); // BeginObject, Key, 6 array events, EndObject
        assert!(transfers > 2 * events as u64);
    }
}
