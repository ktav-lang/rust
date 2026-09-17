use std::borrow::Cow;

use bumpalo::Bump;

use crate::error::{Error, ErrorKind, Span};
use crate::parser::inline::{
    find_matching_close, find_unescaped_colon_inline, has_quote_bytes, malformed_closer_not_at_end,
    process_escapes, scan_inline_closer, scan_inline_closer_with_bounds, split_top_level,
    InlineBody, InlineBounds, InlineCloserScan, MAX_INLINE_DEPTH,
};
use crate::parser::insert::insert_value;
use crate::whitespace::is_inline_whitespace;

use super::super::event::{Event, EventSink};
use super::emit::{classify_inline_scalar, cow_to_bump, emit_node};
use super::node::{InlineMap, Node};
use super::perf::record_event_transfers;

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
    // R10-F1, corrected R11-F1: quote presence is computed ONCE over
    // the root body and threaded down. The soundness argument is
    // ASYMMETRIC: every nested slice is a substring of this body, so
    // root-level ABSENCE guarantees descendant absence and the fast
    // machine is always safe; root-level PRESENCE implies nothing
    // about a descendant — a quote-free level may then run the
    // quote-aware machine, which is safe only because the two
    // machines agree on quote-free input (R8-F2; the last-segment
    // divergence R11-F1 closed lives in `split_top_level`).
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

pub(super) fn malformed(line_num: usize, span: Span, detail: &str) -> Error {
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
