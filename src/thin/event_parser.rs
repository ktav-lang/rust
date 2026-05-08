//! Tokenize Ktav text directly into a flat [`EventStream`] — no
//! intermediate tree. Mirrors the validation logic of [`super::parser`]
//! but emits a linear sequence of `Event`s into a single bump-arena
//! `Vec` instead of a recursive `ThinValue`.
//!
//! Dotted keys are resolved here, at tokenize time, by maintaining a
//! per-object-frame stack of currently-open synthetic prefixes. When a
//! new line's prefix diverges from the stack, the divergence point is
//! emitted as a sequence of `EndObject`s; the new tail is emitted as
//! `Key`+`BeginObject`s. Duplicates and path conflicts are caught the
//! same way the tree-builder catches them — through per-level
//! `seen_keys` sets, but using `FxHashSet` so the check is O(1) on wide
//! objects instead of O(K).

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::error::{CompoundKind, ConflictKind, Error, ErrorKind, Result, Span};

use super::event::{Event, EventSink, EventStream};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn parse_events<'a>(text: &'a str, bump: &'a Bump) -> Result<EventStream<'a>> {
    // Capacity hint: synth-config workloads measure roughly 1 event per
    // 5 source bytes (each `key: value\n` line is ~10–20 bytes and
    // produces 2 events). Underestimating used to trigger 8–10
    // realloc-and-copy steps inside the bump arena on a 500 KiB doc;
    // the over-estimate path just wastes a couple % of arena memory,
    // which the arena drops on scope exit anyway.
    let mut events: EventStream<'a> = BumpVec::with_capacity_in(text.len() / 4 + 64, bump);
    EventSink::push(&mut events, Event::BeginObject);

    let mut p = EventParser::new(bump);

    let bytes = text.as_bytes();
    let mut line_start: usize = 0;
    let mut line_num: usize = 0;
    while line_start <= bytes.len() {
        let nl = bytes[line_start..].iter().position(|&b| b == b'\n');
        let (end, next_start) = match nl {
            Some(rel) => (line_start + rel, line_start + rel + 1),
            None => {
                if line_start == bytes.len() {
                    break;
                }
                (bytes.len(), bytes.len() + 1)
            }
        };
        let content_end = if end > line_start && bytes[end - 1] == b'\r' {
            end - 1
        } else {
            end
        };
        let line: &'a str = &text[line_start..content_end];
        line_num += 1;
        p.handle_line(line, line_num, line_start as u32, &mut events)?;
        line_start = next_start;
    }

    p.finish(bytes.len() as u32, &mut events)?;
    Ok(events)
}

// ---------------------------------------------------------------------------
// Parser state
// ---------------------------------------------------------------------------

pub(crate) struct EventParser<'a> {
    pub(crate) bump: &'a Bump,
    pub(crate) stack: Vec<Frame<'a>>,
    pub(crate) collecting: Option<Collecting<'a>>,
    /// Byte offset (in original input) of the opener that started each
    /// frame. Index `0` is the implicit root (always `0`); subsequent
    /// entries are pushed for each child frame.
    pub(crate) opener_offsets: Vec<u32>,
    /// Byte offset of the `(` / `((` line that started a multi-line
    /// string, if one is currently being collected.
    pub(crate) multiline_opener: Option<u32>,
}

impl<'a> EventParser<'a> {
    pub(crate) fn new(bump: &'a Bump) -> Self {
        let mut p = EventParser {
            bump,
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
        };
        p.stack.push(Frame::new_object(bump));
        p.opener_offsets.push(0);
        p
    }
}

pub(crate) enum Frame<'a> {
    /// `levels` is parallel to "real frame + open synthetic prefixes".
    /// Index 0 is always the real object's namespace; subsequent entries
    /// are stacked synthetics with their own prefix and key sets. All
    /// vectors live in the bump arena — no per-frame heap allocation.
    Object {
        levels: BumpVec<'a, ObjectLevel<'a>>,
    },
    Array,
}

pub(crate) struct ObjectLevel<'a> {
    /// `None` for the real object level, `Some(prefix_segment)` for a
    /// synthetic dotted-key level.
    prefix: Option<&'a str>,
    /// Keys registered as plain scalars/compounds at this level. Linear
    /// scan dedup — fast for typical config shapes (K < ~20). Wider
    /// objects could justify a hash set, but the arena allocations
    /// would have to live in the arena too (FxHashSet doesn't), and
    /// the linear scan stays cache-friendly.
    leaf_keys: BumpVec<'a, &'a str>,
    /// Keys registered as a synthetic dotted-key prefix. May be re-
    /// opened later (a `prefix.x` line that pops back to this level
    /// can extend the same synthetic); cannot be re-used as a plain
    /// leaf.
    synthetic_keys: BumpVec<'a, &'a str>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new_object(bump: &'a Bump) -> Self {
        let mut levels = BumpVec::with_capacity_in(2, bump);
        levels.push(ObjectLevel {
            prefix: None,
            leaf_keys: BumpVec::with_capacity_in(8, bump),
            synthetic_keys: BumpVec::new_in(bump),
        });
        Frame::Object { levels }
    }
    pub(crate) fn new_array() -> Self {
        Frame::Array
    }
}

#[derive(Copy, Clone)]
pub(crate) enum MultilineMode {
    Stripped,
    Verbatim,
}

pub(crate) struct Collecting<'a> {
    pub(crate) mode: MultilineMode,
    pub(crate) lines: BumpVec<'a, &'a str>,
}

// ---------------------------------------------------------------------------
// Line dispatch
// ---------------------------------------------------------------------------

impl<'a> EventParser<'a> {
    pub(crate) fn finish<S: EventSink<'a>>(
        &mut self,
        eof_offset: u32,
        events: &mut S,
    ) -> Result<()> {
        if let Some(c) = &self.collecting {
            let kind = match c.mode {
                MultilineMode::Stripped => CompoundKind::MultilineStripped,
                MultilineMode::Verbatim => CompoundKind::MultilineVerbatim,
            };
            let start = self.multiline_opener.unwrap_or(eof_offset);
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind,
                span: Span::new(start, eof_offset),
            }));
        }
        if self.stack.len() > 1 {
            let kind = match self.stack.last().unwrap() {
                Frame::Object { .. } => CompoundKind::Object,
                Frame::Array => CompoundKind::Array,
            };
            let start = *self.opener_offsets.last().unwrap();
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind,
                span: Span::new(start, eof_offset),
            }));
        }
        // Close all synthetics still open in the root frame, then the
        // root object itself.
        self.close_synthetics_until(0, events);
        events.push(Event::EndObject);
        Ok(())
    }

    pub(crate) fn handle_line<S: EventSink<'a>>(
        &mut self,
        raw: &'a str,
        line_num: usize,
        line_start: u32,
        events: &mut S,
    ) -> Result<()> {
        if let Some(ref mut c) = self.collecting {
            if raw.as_bytes().contains(&b')') {
                let trimmed = raw.trim();
                let term = match c.mode {
                    MultilineMode::Stripped => ")",
                    MultilineMode::Verbatim => "))",
                };
                if trimmed == term {
                    let collecting = self.collecting.take().unwrap();
                    let s = finalize_multiline(collecting, self.bump);
                    self.multiline_opener = None;
                    return self.attach_scalar(Event::Str(s), line_num, events);
                }
            }
            c.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        let trimmed_span = trimmed_span_in(raw, trimmed, line_start);

        if trimmed == "}" {
            return self.close_frame(BracketKind::Object, line_num, trimmed_span, events);
        }
        if trimmed == "]" {
            return self.close_frame(BracketKind::Array, line_num, trimmed_span, events);
        }

        if matches!(self.stack.last(), Some(Frame::Array)) {
            self.handle_array_item(trimmed, line_num, trimmed_span, events)
        } else {
            self.handle_object_pair(trimmed, line_num, trimmed_span, events)
        }
    }

    // -----------------------------------------------------------------------
    // Object-pair dispatch (mirrors thin/parser.rs but emits events)
    // -----------------------------------------------------------------------

    fn handle_object_pair<S: EventSink<'a>>(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let colon = match trimmed.find(':') {
            Some(c) => c,
            None => {
                return Err(Error::Structured(ErrorKind::MissingSeparator {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
        };

        let key = trimmed[..colon].trim_end();
        let key_start = trimmed_span.start;
        let key_end = key_start + key.len() as u32;
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span: Span::new(key_start, key_start + 1),
            }));
        }

        let after_colon = &trimmed[colon + 1..];
        let after_colon_off = key_start + (colon as u32) + 1;
        let key_span = Span::new(key_start, key_end);

        match classify_separator(after_colon) {
            Separator::Raw(rest) => {
                require_sep_end(rest, line_num, after_colon_off + 1, trimmed_span)?;
                self.emit_keyed_scalar(key, Event::Str(rest.trim()), line_num, key_span, events)
            }
            Separator::TypedInteger(body) => {
                let body_span = body_span_for(body, after_colon, after_colon_off, trimmed_span);
                let normalized = validate_typed_integer(body, line_num, self.bump, body_span)?;
                self.emit_keyed_scalar(key, Event::Integer(normalized), line_num, key_span, events)
            }
            Separator::TypedFloat(body) => {
                let body_span = body_span_for(body, after_colon, after_colon_off, trimmed_span);
                let normalized = validate_typed_float(body, line_num, self.bump, body_span)?;
                self.emit_keyed_scalar(key, Event::Float(normalized), line_num, key_span, events)
            }
            Separator::Plain => {
                require_sep_end(after_colon, line_num, after_colon_off, trimmed_span)?;
                match classify(after_colon.trim_start(), line_num, trimmed_span)? {
                    ValueStart::Scalar(s) => {
                        self.emit_keyed_scalar(key, scalar_to_event(s), line_num, key_span, events)
                    }
                    ValueStart::EmptyObject => self.emit_keyed_compound(
                        key,
                        Event::BeginObject,
                        Event::EndObject,
                        line_num,
                        key_span,
                        events,
                    ),
                    ValueStart::EmptyArray => self.emit_keyed_compound(
                        key,
                        Event::BeginArray,
                        Event::EndArray,
                        line_num,
                        key_span,
                        events,
                    ),
                    ValueStart::OpenObject => {
                        self.emit_keyed_open(key, Event::BeginObject, line_num, key_span, events)?;
                        self.stack.push(Frame::new_object(self.bump));
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.emit_keyed_open(key, Event::BeginArray, line_num, key_span, events)?;
                        self.stack.push(Frame::new_array());
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        let r = self.emit_keyed_open_multiline(
                            key,
                            MultilineMode::Stripped,
                            line_num,
                            key_span,
                            events,
                        );
                        self.multiline_opener = Some(trimmed_span.end - 1);
                        r
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        let r = self.emit_keyed_open_multiline(
                            key,
                            MultilineMode::Verbatim,
                            line_num,
                            key_span,
                            events,
                        );
                        self.multiline_opener = Some(trimmed_span.end - 2);
                        r
                    }
                }
            }
        }
    }

    // Emits Key(leaf) + value-event after reconciling synthetic stack.
    fn emit_keyed_scalar<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        value: Event<'a>,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        self.register_leaf_key(leaf, line_num, key_span)?;
        events.push(Event::Key(leaf));
        events.push(value);
        Ok(())
    }

    // For empty inline compound `{}` / `[]`: emit Key + open + close.
    fn emit_keyed_compound<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        open: Event<'a>,
        close: Event<'a>,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        self.register_leaf_key(leaf, line_num, key_span)?;
        events.push(Event::Key(leaf));
        events.push(open);
        events.push(close);
        Ok(())
    }

    // For "key: {" or "key: [" — emit Key + open, push frame.
    fn emit_keyed_open<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        open: Event<'a>,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        self.register_leaf_key(leaf, line_num, key_span)?;
        events.push(Event::Key(leaf));
        events.push(open);
        Ok(())
    }

    fn emit_keyed_open_multiline<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        mode: MultilineMode,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        self.register_leaf_key(leaf, line_num, key_span)?;
        events.push(Event::Key(leaf));
        self.collecting = Some(Collecting {
            mode,
            lines: BumpVec::with_capacity_in(8, self.bump),
        });
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Array-item dispatch
    // -----------------------------------------------------------------------

    fn handle_array_item<S: EventSink<'a>>(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let line_start = trimmed_span.start;
        if let Some(rest) = trimmed.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start + 2, trimmed_span)?;
            events.push(Event::Str(rest.trim_start()));
            return Ok(());
        }
        if let Some(rest) = trimmed.strip_prefix(":i") {
            require_sep_end(rest, line_num, line_start + 2, trimmed_span)?;
            let body_span = Span::new(line_start + 2, trimmed_span.end);
            let normalized = validate_typed_integer(rest, line_num, self.bump, body_span)?;
            events.push(Event::Integer(normalized));
            return Ok(());
        }
        if let Some(rest) = trimmed.strip_prefix(":f") {
            require_sep_end(rest, line_num, line_start + 2, trimmed_span)?;
            let body_span = Span::new(line_start + 2, trimmed_span.end);
            let normalized = validate_typed_float(rest, line_num, self.bump, body_span)?;
            events.push(Event::Float(normalized));
            return Ok(());
        }

        match classify(trimmed, line_num, trimmed_span)? {
            ValueStart::Scalar(s) => events.push(scalar_to_event(s)),
            ValueStart::EmptyObject => {
                events.push(Event::BeginObject);
                events.push(Event::EndObject);
            }
            ValueStart::EmptyArray => {
                events.push(Event::BeginArray);
                events.push(Event::EndArray);
            }
            ValueStart::OpenObject => {
                events.push(Event::BeginObject);
                self.stack.push(Frame::new_object(self.bump));
                self.opener_offsets.push(trimmed_span.end - 1);
            }
            ValueStart::OpenArray => {
                events.push(Event::BeginArray);
                self.stack.push(Frame::new_array());
                self.opener_offsets.push(trimmed_span.end - 1);
            }
            ValueStart::OpenMultilineStripped => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Stripped,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                self.multiline_opener = Some(trimmed_span.end - 1);
            }
            ValueStart::OpenMultilineVerbatim => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Verbatim,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                self.multiline_opener = Some(trimmed_span.end - 2);
            }
        }
        Ok(())
    }

    // Multi-line / compound-child completion path: scalar attached as
    // the value for whatever keyed-or-array context we're in. Because
    // for keyed contexts the `Key(...)` was already emitted before the
    // multiline began, we just push the scalar event here.
    fn attach_scalar<S: EventSink<'a>>(
        &mut self,
        value: Event<'a>,
        _line_num: usize,
        events: &mut S,
    ) -> Result<()> {
        events.push(value);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Dotted-key reconciliation
    // -----------------------------------------------------------------------

    fn reconcile_dotted_key<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<&'a str> {
        if !key.as_bytes().contains(&b'.') {
            // Flat key: close any open synthetics in the current real
            // frame, then the leaf is just `key`.
            self.close_synthetics_to_real(events);
            if !is_valid_key(key) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            return Ok(key);
        }

        // Split off the leaf (the segment after the last dot). Don't
        // pre-collect the prefix segments — walk them lazily, comparing
        // against the current synthetic stack as we go (LCP fold). That
        // saves the heap-allocated `Vec<&str>` per dotted line.
        let (prefix_str, leaf) = key.rsplit_once('.').unwrap();
        if leaf.is_empty() || !is_valid_key(leaf) {
            return Err(Error::Structured(ErrorKind::InvalidKey {
                line: line_num as u32,
                key: key.to_string(),
                span: key_span,
            }));
        }

        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => unreachable!("dispatched as object"),
        };

        // Stage 1: walk new prefix segments and the current synthetic
        // stack in lockstep, advancing the LCP cursor. Stop at the
        // first mismatch.
        let mut new_iter = prefix_str.split('.');
        let mut lcp_count: usize = 0;
        let mut next_seg: Option<&'a str> = None;

        while lcp_count + 1 < cur_levels_len {
            let cur_prefix = match self.stack.last().unwrap() {
                Frame::Object { levels, .. } => levels[1 + lcp_count].prefix.unwrap(),
                _ => unreachable!(),
            };
            let seg = match new_iter.next() {
                Some(s) => s,
                None => break,
            };
            if !is_valid_key(seg) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            if seg != cur_prefix {
                next_seg = Some(seg);
                break;
            }
            lcp_count += 1;
        }

        // Stage 2: pop synthetic levels beyond LCP.
        let pops = cur_levels_len - 1 - lcp_count;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }

        // Stage 3: emit Key + BeginObject + push level for the carry-
        // over segment (if any), then for the rest of `new_iter`.
        if let Some(seg) = next_seg {
            self.push_synthetic(seg, line_num, key_span, events)?;
        }
        for seg in new_iter {
            if !is_valid_key(seg) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            self.push_synthetic(seg, line_num, key_span, events)?;
        }

        Ok(leaf)
    }

    #[inline]
    fn push_synthetic<S: EventSink<'a>>(
        &mut self,
        seg: &'a str,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        self.register_synthetic_prefix(seg, line_num, key_span)?;
        events.push(Event::Key(seg));
        events.push(Event::BeginObject);
        let bump = self.bump;
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => levels.push(ObjectLevel {
                prefix: Some(seg),
                leaf_keys: BumpVec::with_capacity_in(4, bump),
                synthetic_keys: BumpVec::new_in(bump),
            }),
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Mark a path segment as a synthetic dotted-key prefix at the
    /// current top level. Errors on:
    /// - existing leaf at the same name (existing scalar blocks the path)
    /// - existing synthetic at the same name (re-open after the
    ///   synthetic was closed by an intervening different prefix would
    ///   require buffering the whole document, which the event-stream
    ///   path explicitly avoids — group lines with the same prefix
    ///   together to keep them in one synthetic block)
    fn register_synthetic_prefix(
        &mut self,
        seg: &'a str,
        line_num: usize,
        key_span: Span,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                let level = levels.last_mut().unwrap();
                if level.leaf_keys.contains(&seg) {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: seg.to_string(),
                        kind: ConflictKind::BlockedByValue,
                        span: key_span,
                    }));
                }
                if level.synthetic_keys.contains(&seg) {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: seg.to_string(),
                        kind: ConflictKind::SyntheticReopen,
                        span: key_span,
                    }));
                }
                level.synthetic_keys.push(seg);
                Ok(())
            }
            _ => unreachable!("only objects have keys"),
        }
    }

    fn close_synthetics_to_real<S: EventSink<'a>>(&mut self, events: &mut S) {
        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => return,
        };
        let pops = cur_levels_len - 1;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }
    }

    pub(crate) fn close_synthetics_until<S: EventSink<'a>>(
        &mut self,
        target_synthetic_count: usize,
        events: &mut S,
    ) {
        loop {
            let cur = match self.stack.last() {
                Some(Frame::Object { levels, .. }) => levels.len() - 1,
                _ => return,
            };
            if cur <= target_synthetic_count {
                return;
            }
            self.pop_synthetic_level(events);
        }
    }

    fn pop_synthetic_level<S: EventSink<'a>>(&mut self, events: &mut S) {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                levels.pop();
                events.push(Event::EndObject);
            }
            _ => unreachable!(),
        }
    }

    /// Register a plain leaf key at the top-most open object level.
    /// Conflicts with both an existing leaf (duplicate) and an existing
    /// synthetic prefix (a sub-object would have to vanish to make room
    /// for the scalar). Linear scan — fine for K < ~20; worse for very
    /// wide objects, but those are rare in real configs.
    #[inline]
    fn register_leaf_key(&mut self, leaf: &'a str, line_num: usize, key_span: Span) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                let level = levels.last_mut().unwrap();
                if level.synthetic_keys.contains(&leaf) {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: leaf.to_string(),
                        kind: ConflictKind::BlockedByValue,
                        span: key_span,
                    }));
                }
                if level.leaf_keys.contains(&leaf) {
                    return Err(Error::Structured(ErrorKind::DuplicateKey {
                        line: line_num as u32,
                        key: leaf.to_string(),
                        span: key_span,
                    }));
                }
                level.leaf_keys.push(leaf);
                Ok(())
            }
            _ => unreachable!("only objects have keys"),
        }
    }

    // -----------------------------------------------------------------------
    // Frame close
    // -----------------------------------------------------------------------

    fn close_frame<S: EventSink<'a>>(
        &mut self,
        expected: BracketKind,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<()> {
        if self.stack.len() <= 1 {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: expected.to_compound(),
                found: expected.close(),
            }));
        }
        // Close any open synthetics in the current frame first (if it's
        // an object), then close the frame itself.
        if matches!(self.stack.last(), Some(Frame::Object { .. })) {
            self.close_synthetics_to_real(events);
        }
        let got = match self.stack.pop().unwrap() {
            Frame::Object { .. } => BracketKind::Object,
            Frame::Array => BracketKind::Array,
        };
        let _ = self.opener_offsets.pop();
        if got as u8 != expected as u8 {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: got.to_compound(),
                found: expected.close(),
            }));
        }
        let close_event = match got {
            BracketKind::Object => Event::EndObject,
            BracketKind::Array => Event::EndArray,
        };
        events.push(close_event);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bracket kind (same shape as the tree-builder's, deliberately repeated to
// keep the modules independently rip-out-able).
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum BracketKind {
    Object = 0,
    Array = 1,
}

impl BracketKind {
    fn close(self) -> char {
        match self {
            BracketKind::Object => '}',
            BracketKind::Array => ']',
        }
    }
    fn to_compound(self) -> CompoundKind {
        match self {
            BracketKind::Object => CompoundKind::Object,
            BracketKind::Array => CompoundKind::Array,
        }
    }
}

// ---------------------------------------------------------------------------
// Value-start classification (mirrors thin/parser.rs)
// ---------------------------------------------------------------------------

enum ValueStart<'a> {
    Scalar(&'a str),
    EmptyObject,
    EmptyArray,
    OpenObject,
    OpenArray,
    OpenMultilineStripped,
    OpenMultilineVerbatim,
}

enum Separator<'a> {
    Raw(&'a str),
    TypedInteger(&'a str),
    TypedFloat(&'a str),
    Plain,
}

#[inline]
fn require_sep_end(rest: &str, line_num: usize, body_off: u32, trimmed_span: Span) -> Result<()> {
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Ok(())
    } else {
        Err(Error::Structured(ErrorKind::MissingSeparatorSpace {
            line: line_num as u32,
            column: 0,
            marker: ':',
            span: Span::new(body_off, trimmed_span.end),
        }))
    }
}

/// Locate `trimmed` inside `raw` and produce its absolute span.
fn trimmed_span_in(raw: &str, trimmed: &str, line_start: u32) -> Span {
    if trimmed.is_empty() {
        return Span::new(line_start, line_start);
    }
    let raw_ptr = raw.as_ptr() as usize;
    let trim_ptr = trimmed.as_ptr() as usize;
    debug_assert!(trim_ptr >= raw_ptr && trim_ptr - raw_ptr <= raw.len());
    let off = (trim_ptr - raw_ptr) as u32;
    let start = line_start + off;
    Span::new(start, start + trimmed.len() as u32)
}

fn body_span_for(body: &str, after_colon: &str, after_colon_off: u32, fallback: Span) -> Span {
    if body.is_empty() {
        let p = after_colon_off + after_colon.len() as u32;
        return Span::new(p, p);
    }
    let after_ptr = after_colon.as_ptr() as usize;
    let body_ptr = body.as_ptr() as usize;
    if body_ptr >= after_ptr && body_ptr - after_ptr <= after_colon.len() {
        let off = (body_ptr - after_ptr) as u32;
        Span::new(
            after_colon_off + off,
            after_colon_off + off + body.len() as u32,
        )
    } else {
        fallback
    }
}

#[inline]
fn classify_separator<'a>(after_colon: &'a str) -> Separator<'a> {
    if let Some(rest) = after_colon.strip_prefix(':') {
        return Separator::Raw(rest);
    }
    if let Some(rest) = after_colon.strip_prefix('i') {
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            return Separator::TypedInteger(rest);
        }
    }
    if let Some(rest) = after_colon.strip_prefix('f') {
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            return Separator::TypedFloat(rest);
        }
    }
    Separator::Plain
}

fn validate_typed_integer<'a>(
    body: &'a str,
    line_num: usize,
    bump: &'a Bump,
    span: Span,
) -> Result<&'a str> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            "integer body is empty",
            span,
        ));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            "typed marker `:i` cannot open a compound or multi-line value",
            span,
        ));
    }
    if !is_integer_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            &format!("'{}' is not a valid integer literal for `:i`", trimmed),
            span,
        ));
    }
    Ok(strip_leading_plus(trimmed, bump))
}

fn validate_typed_float<'a>(
    body: &'a str,
    line_num: usize,
    bump: &'a Bump,
    span: Span,
) -> Result<&'a str> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            "float body is empty",
            span,
        ));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            "typed marker `:f` cannot open a compound or multi-line value",
            span,
        ));
    }
    if !is_float_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            &format!("'{}' is not a valid float literal for `:f`", trimmed),
            span,
        ));
    }
    Ok(strip_leading_plus(trimmed, bump))
}

fn invalid_typed_scalar(line_num: usize, marker: char, detail: &str, span: Span) -> Error {
    Error::Structured(ErrorKind::InvalidTypedScalar {
        line: line_num as u32,
        marker,
        body: detail.to_string(),
        span,
    })
}

fn opens_compound_or_multiline(s: &str) -> bool {
    s.starts_with('{') || s.starts_with('[') || s.starts_with('(')
}

fn strip_leading_plus<'a>(s: &'a str, bump: &'a Bump) -> &'a str {
    if let Some(stripped) = s.strip_prefix('+') {
        bump.alloc_str(stripped)
    } else {
        s
    }
}

fn is_integer_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    if i == bytes.len() {
        return false;
    }
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return false;
        }
        i += 1;
    }
    true
}

fn is_float_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let digits_before = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_before {
        return false;
    }
    if i == bytes.len() || bytes[i] != b'.' {
        return false;
    }
    i += 1;
    let digits_after = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_after {
        return false;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        let exp_digits = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == exp_digits {
            return false;
        }
    }
    i == bytes.len()
}

fn classify<'a>(trimmed: &'a str, line_num: usize, trimmed_span: Span) -> Result<ValueStart<'a>> {
    if trimmed == "{" {
        return Ok(ValueStart::OpenObject);
    }
    if trimmed == "[" {
        return Ok(ValueStart::OpenArray);
    }

    if trimmed.starts_with('{') {
        if trimmed.ends_with('}') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyObject);
        }
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "object".to_string(),
        }));
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "array".to_string(),
        }));
    }

    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar("")),
        _ => {}
    }

    Ok(ValueStart::Scalar(trimmed))
}

#[inline]
fn scalar_to_event(s: &str) -> Event<'_> {
    match s {
        "null" => Event::Null,
        "true" => Event::Bool(true),
        "false" => Event::Bool(false),
        _ => Event::Str(s),
    }
}

#[inline]
fn is_valid_key(k: &str) -> bool {
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            b.is_ascii_whitespace() || matches!(b, b'[' | b']' | b'{' | b'}' | b':' | b'#')
        })
}

// ---------------------------------------------------------------------------
// Multi-line finalize (identical semantics to parser.rs)
// ---------------------------------------------------------------------------

fn finalize_multiline<'a>(c: Collecting<'a>, bump: &'a Bump) -> &'a str {
    match c.mode {
        MultilineMode::Verbatim if c.lines.len() == 1 => c.lines[0],
        MultilineMode::Verbatim => {
            let joined = c.lines.join("\n");
            bump.alloc_str(&joined)
        }
        MultilineMode::Stripped if c.lines.len() == 1 => {
            let only = c.lines[0];
            if only.trim().is_empty() {
                ""
            } else {
                only.trim_start()
            }
        }
        MultilineMode::Stripped => {
            let dedented = dedent(&c.lines);
            bump.alloc_str(&dedented)
        }
    }
}

fn dedent(lines: &[&str]) -> String {
    let common_len = common_leading_whitespace_len(lines);

    let cap: usize = lines.iter().map(|l| l.len()).sum::<usize>() + lines.len();
    let mut out = String::with_capacity(cap.saturating_sub(common_len * lines.len()));

    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if l.trim().is_empty() {
            // blank line
        } else if common_len > 0 && l.len() >= common_len {
            out.push_str(&l[common_len..]);
        } else {
            out.push_str(l);
        }
    }
    out
}

fn common_leading_whitespace_len(lines: &[&str]) -> usize {
    let mut iter = lines.iter().filter(|l| !l.trim().is_empty());
    let first = match iter.next() {
        Some(l) => leading_whitespace_bytes(l),
        None => return 0,
    };
    let mut len = first.len();
    for line in iter {
        let other = leading_whitespace_bytes(line);
        let mut shared = 0;
        while shared < len && shared < other.len() && first[shared] == other[shared] {
            shared += 1;
        }
        len = shared;
        if len == 0 {
            break;
        }
    }
    len
}

fn leading_whitespace_bytes(s: &str) -> &[u8] {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    &bytes[..i]
}
