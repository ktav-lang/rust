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
use memchr::{memchr, memchr2};

use crate::error::{CompoundKind, ConflictKind, Error, ErrorKind, Result, Span};
use crate::parser::classify::{is_float_literal, try_parse_integer};
use crate::parser::inline::{
    decode_key_segment, find_unescaped_colon, key_is_single_segment, split_key_path,
};

use super::event::{Event, EventSink, EventStream};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn parse_events<'a>(text: &'a str, bump: &'a Bump) -> Result<EventStream<'a>> {
    let mut events: EventStream<'a> = BumpVec::with_capacity_in(text.len() / 4 + 64, bump);

    // Spec § 5.0.1 (0.5.0): scan ahead to the first content line,
    // classify it per the 8 rules.
    let bytes = text.as_bytes();
    let root_kind = detect_root_kind(text, bytes);
    EventSink::push(
        &mut events,
        match root_kind {
            RootKind::Object => Event::BeginObject,
            RootKind::Array => Event::BeginArray,
        },
    );
    let mut p = EventParser::new(bump, root_kind);

    // Line splitting: handle CR / CR LF / LF (spec § 3.2)
    //
    // Fast path for the overwhelmingly common case: LF-only input
    // (no CR bytes).  This avoids the per-byte branch on `\r` in the
    // inner scan loop and lets the compiler emit a tighter scan.
    if memchr(b'\r', bytes).is_none() {
        // LF-only fast path: memchr-backed `\n` splitting.
        let mut line_num: usize = 0;
        let mut line_start: usize = 0;
        while line_start <= bytes.len() {
            let end = memchr(b'\n', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            let line: &'a str = &text[line_start..end];
            line_num += 1;
            p.handle_line(line, line_num, line_start as u32, &mut events)?;
            if end == bytes.len() {
                break;
            }
            line_start = end + 1;
        }
    } else {
        let mut line_start: usize = 0;
        let mut line_num: usize = 0;
        while line_start < bytes.len() {
            // memchr2 → SIMD-accelerated scan for next `\n` or `\r`.
            let end = memchr2(b'\n', b'\r', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            let content_end = end;
            let next_start = if end < bytes.len() {
                if bytes[end] == b'\r' && end + 1 < bytes.len() && bytes[end + 1] == b'\n' {
                    end + 2 // CR LF
                } else {
                    end + 1 // CR or LF alone
                }
            } else {
                end // EOF
            };
            let line: &'a str = &text[line_start..content_end];
            line_num += 1;
            p.handle_line(line, line_num, line_start as u32, &mut events)?;
            line_start = next_start;
        }
    }

    p.finish(bytes.len() as u32, &mut events)?;
    Ok(events)
}

/// Per spec § 5.0.1: scan forward to the first non-blank, non-comment
/// line and classify it as Object (pair shape) or Array (anything
/// else). Empty / comments-only documents default to Object.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RootKind {
    Object,
    Array,
}

fn detect_root_kind(text: &str, bytes: &[u8]) -> RootKind {
    let mut i = 0;
    while i < bytes.len() {
        let line_start = i;
        // Find next line terminator (CR / LF / CR LF) via SIMD memchr2.
        i = memchr2(b'\n', b'\r', &bytes[i..])
            .map(|p| line_start + p)
            .unwrap_or(bytes.len());
        let content_end = i;
        // Skip terminator
        if i < bytes.len() {
            if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
        }
        let line = &text[line_start..content_end];
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("##") {
            // First content line — classify (mirrors parser.rs)
            if trimmed == "}" || trimmed == "]" {
                return RootKind::Object; // bare closer → main loop emits error
            }
            return classify_first_line_root_kind(trimmed);
        }
    }
    RootKind::Object // empty or comments-only
}

fn classify_first_line_root_kind(trimmed: &str) -> RootKind {
    // § 5.0.1 rule 4: lone `{`
    if trimmed == "{" {
        return RootKind::Object;
    }
    // § 5.0.1 rule 5: lone `[`
    if trimmed == "[" {
        return RootKind::Array;
    }
    // § 5.0.1 rule 2: closed inline object
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return RootKind::Object;
    }
    // § 5.0.1 rule 3: closed inline array
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return RootKind::Array;
    }
    // § 5.0.1 rules 6/7: pair shape vs array item
    if is_pair_shape(trimmed) {
        RootKind::Object
    } else {
        RootKind::Array
    }
}

/// Check if the trimmed line looks like a pair shape.
/// Spec 0.6.0 § 5.3 — the separator is the first UNescaped `:`.
fn is_pair_shape(trimmed: &str) -> bool {
    let Some(colon_idx) = find_unescaped_colon(trimmed) else {
        return false;
    };
    let key_part = trimmed[..colon_idx].trim_end();
    if key_part.is_empty() {
        return false;
    }
    let after = &trimmed[colon_idx + 1..];
    if after.starts_with(':') {
        return true;
    }
    after.is_empty() || after.starts_with([' ', '\t'])
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
    pub(crate) fn new(bump: &'a Bump, root_kind: RootKind) -> Self {
        let mut p = EventParser {
            bump,
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
        };
        match root_kind {
            RootKind::Object => p.stack.push(Frame::new_object(bump)),
            RootKind::Array => p.stack.push(Frame::new_array()),
        }
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
    /// Keys registered as plain scalars/compounds at this level.
    leaf_keys: BumpVec<'a, &'a str>,
    /// Keys registered as a synthetic dotted-key prefix.
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
        // Close all synthetics still open in the root frame (only
        // applies to Object roots), then emit the matching close
        // event for whichever kind the root was.
        match self.stack.last() {
            Some(Frame::Object { .. }) => {
                self.close_synthetics_until(0, events);
                events.push(Event::EndObject);
            }
            Some(Frame::Array) => {
                events.push(Event::EndArray);
            }
            None => unreachable!("root frame is always present after construction"),
        }
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
            let trimmed = raw.trim();
            let term = match c.mode {
                MultilineMode::Stripped => ")",
                MultilineMode::Verbatim => "))",
            };
            // Fast reject: terminator must equal the trimmed line.
            // Most collection-body lines are NOT the terminator, so
            // the length check eliminates the vast majority before
            // the byte-comparison.
            if trimmed.len() <= 2 && trimmed == term {
                let collecting = self.collecting.take().unwrap();
                let s = finalize_multiline(collecting, self.bump);
                self.multiline_opener = None;
                return self.attach_scalar(Event::Str(s), line_num, events);
            }
            c.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim();

        // Under 0.5.0: comments use `##` (not single `#`)
        if trimmed.is_empty() || trimmed.starts_with("##") {
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
    // Object-pair dispatch
    // -----------------------------------------------------------------------

    fn handle_object_pair<S: EventSink<'a>>(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<()> {
        // Spec 0.6.0 § 5.3 — pair separator is the first UNescaped `:`.
        let colon = match find_unescaped_colon(trimmed) {
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
            Separator::Plain => {
                require_sep_end(after_colon, line_num, after_colon_off, trimmed_span)?;
                let body = after_colon.trim_start();
                match classify(body, line_num, trimmed_span, self.bump)? {
                    ValueStart::Scalar(s) => {
                        self.emit_keyed_scalar(key, Event::Str(s), line_num, key_span, events)
                    }
                    ValueStart::Integer(s) => {
                        self.emit_keyed_scalar(key, Event::Integer(s), line_num, key_span, events)
                    }
                    ValueStart::Float(s) => {
                        self.emit_keyed_scalar(key, Event::Float(s), line_num, key_span, events)
                    }
                    ValueStart::Null => {
                        self.emit_keyed_scalar(key, Event::Null, line_num, key_span, events)
                    }
                    ValueStart::Bool(b) => {
                        self.emit_keyed_scalar(key, Event::Bool(b), line_num, key_span, events)
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
                    ValueStart::InlineEvents(inline_events) => {
                        let leaf = self.reconcile_dotted_key(key, line_num, key_span, events)?;
                        self.register_leaf_key(leaf, line_num, key_span)?;
                        events.push(Event::Key(leaf));
                        for ev in inline_events {
                            events.push(ev);
                        }
                        Ok(())
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

        // Under 0.5.0: only `::` raw marker for arrays. No `:i`/`:f`.
        if let Some(rest) = trimmed.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start + 2, trimmed_span)?;
            events.push(Event::Str(rest.trim_start()));
            return Ok(());
        }

        match classify(trimmed, line_num, trimmed_span, self.bump)? {
            ValueStart::Scalar(s) => events.push(Event::Str(s)),
            ValueStart::Integer(s) => events.push(Event::Integer(s)),
            ValueStart::Float(s) => events.push(Event::Float(s)),
            ValueStart::Null => events.push(Event::Null),
            ValueStart::Bool(b) => events.push(Event::Bool(b)),
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
            ValueStart::InlineEvents(inline_events) => {
                for ev in inline_events {
                    events.push(ev);
                }
            }
        }
        Ok(())
    }

    // Multi-line / compound-child completion path
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
        // Single segment (no UNescaped `.`) fast path. Decode the
        // segment if it contains a `\`; otherwise reuse the source
        // borrow.
        if key_is_single_segment(key) {
            self.close_synthetics_to_real(events);
            let leaf = self.decode_key_in_arena(key, line_num, key_span)?;
            if !is_valid_key(leaf) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            return Ok(leaf);
        }

        // Multi-segment path — split on UNescaped `.`, decode each
        // segment (arena-allocated if it had `\`).
        let raw_segments = split_key_path(key);
        debug_assert!(raw_segments.len() >= 2);
        let mut decoded_segments: Vec<&'a str> = Vec::with_capacity(raw_segments.len());
        for seg in &raw_segments {
            // Empty segment → InvalidKey (`a..b`, leading/trailing `.`).
            let trimmed = seg.trim();
            if trimmed.is_empty() {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            let decoded = self.decode_key_in_arena(trimmed, line_num, key_span)?;
            if !is_valid_key(decoded) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: key.to_string(),
                    span: key_span,
                }));
            }
            decoded_segments.push(decoded);
        }

        let leaf = *decoded_segments.last().unwrap();
        let prefix_segments = &decoded_segments[..decoded_segments.len() - 1];

        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => unreachable!("dispatched as object"),
        };

        let mut lcp_count: usize = 0;
        let mut pending_seg_idx: Option<usize> = None;

        for (i, seg) in prefix_segments.iter().enumerate() {
            if lcp_count + 1 >= cur_levels_len {
                pending_seg_idx = Some(i);
                break;
            }
            let cur_prefix = match self.stack.last().unwrap() {
                Frame::Object { levels, .. } => levels[1 + lcp_count].prefix.unwrap(),
                _ => unreachable!(),
            };
            if *seg != cur_prefix {
                pending_seg_idx = Some(i);
                break;
            }
            lcp_count += 1;
        }

        let pops = cur_levels_len - 1 - lcp_count;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }

        let push_start = pending_seg_idx.unwrap_or(prefix_segments.len());
        for seg in &prefix_segments[push_start..] {
            self.push_synthetic(seg, line_num, key_span, events)?;
        }

        Ok(leaf)
    }

    /// Decode a key segment per § 3.7. If the input contains no `\`,
    /// the source slice is returned as-is (zero-copy fast path); else
    /// the decoded byte string is allocated in the bump arena so the
    /// returned `&'a str` outlives the call.
    fn decode_key_in_arena(
        &self,
        seg: &'a str,
        line_num: usize,
        key_span: Span,
    ) -> Result<&'a str> {
        if !seg.as_bytes().contains(&b'\\') {
            return Ok(seg);
        }
        let decoded = decode_key_segment(seg, line_num, key_span)?;
        Ok(self.bump.alloc_str(&decoded))
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
// Bracket kind
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
// Value-start classification (mirrors parser/classify.rs for 0.5.0)
// ---------------------------------------------------------------------------

enum ValueStart<'a> {
    Scalar(&'a str),
    Integer(&'a str),
    Float(&'a str),
    Null,
    Bool(bool),
    EmptyObject,
    EmptyArray,
    OpenObject,
    OpenArray,
    OpenMultilineStripped,
    OpenMultilineVerbatim,
    /// Inline compound parsed into a sequence of events
    InlineEvents(Vec<Event<'a>>),
}

enum Separator<'a> {
    Raw(&'a str),
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

#[inline]
fn classify_separator<'a>(after_colon: &'a str) -> Separator<'a> {
    if let Some(rest) = after_colon.strip_prefix(':') {
        return Separator::Raw(rest);
    }
    // Under spec 0.5.0, `:i` and `:f` typed markers are removed.
    Separator::Plain
}

/// Classify a value body per § 5.2 rules 1-15 (0.5.0).
#[inline]
fn classify<'a>(
    trimmed: &'a str,
    line_num: usize,
    trimmed_span: Span,
    bump: &'a Bump,
) -> Result<ValueStart<'a>> {
    if trimmed == "{" {
        return Ok(ValueStart::OpenObject);
    }
    if trimmed == "[" {
        return Ok(ValueStart::OpenArray);
    }

    // § 5.2 rules 6-9: inline compounds
    if trimmed.starts_with('{') {
        if trimmed.ends_with('}') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyObject);
        }
        if trimmed.ends_with('}') {
            // Try to parse inline object → emit as events
            let value =
                crate::parser::inline::parse_inline_object(trimmed, line_num, trimmed_span, false)?;
            let events = value_to_events(&value, bump);
            return Ok(ValueStart::InlineEvents(events));
        }
        // Unterminated inline compound
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        if trimmed.ends_with(']') {
            let value =
                crate::parser::inline::parse_inline_array(trimmed, line_num, trimmed_span, false)?;
            let events = value_to_events(&value, bump);
            return Ok(ValueStart::InlineEvents(events));
        }
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }

    // Multi-line string openers
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar("")),
        _ => {}
    }

    // Paren-prefixed text — under 0.5.0, `(` starts a multiline opener,
    // so a string starting with `(` that isn't one of the exact openers
    // above is an error (parser rejects it via InlineNonEmptyCompound).
    if trimmed.starts_with('(') {
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "paren-string".to_string(),
        }));
    }

    // § 5.2 rules 10-12: keywords
    match trimmed {
        "null" => return Ok(ValueStart::Null),
        "true" => return Ok(ValueStart::Bool(true)),
        "false" => return Ok(ValueStart::Bool(false)),
        _ => {}
    }

    // § 5.2 rule 13: integer literal
    // Fast path for plain decimal (most common case in configs): ASCII
    // digits only, no sign / underscore / base prefix. The input is
    // already canonical — skip itoa formatting and bump allocation.
    if let Some(_val) = fast_plain_decimal_i64(trimmed) {
        return Ok(ValueStart::Integer(trimmed));
    }
    // General path: prefixed, signed, or underscored literals.
    if let Some(val) = try_parse_integer(trimmed) {
        let mut buf = itoa::Buffer::new();
        let canonical = buf.format(val);
        let s = bump.alloc_str(canonical);
        return Ok(ValueStart::Integer(s));
    }

    // § 5.2 rule 14: float literal
    if is_float_literal(trimmed) {
        // If the literal has no underscores we can parse it directly
        // without allocating a cleaned String.
        let has_underscore = trimmed.as_bytes().contains(&b'_');
        if has_underscore {
            let cleaned: String = trimmed.chars().filter(|&c| c != '_').collect();
            if let Ok(val) = cleaned.parse::<f64>() {
                if !val.is_nan() && !val.is_infinite() {
                    let mut buf = ryu::Buffer::new();
                    let canonical = buf.format(val);
                    let s = bump.alloc_str(canonical);
                    return Ok(ValueStart::Float(s));
                }
            }
        } else if let Ok(val) = trimmed.parse::<f64>() {
            if !val.is_nan() && !val.is_infinite() {
                let mut buf = ryu::Buffer::new();
                let canonical = buf.format(val);
                // If ryu reproduces the input, the original slice is
                // canonical — skip the bump allocation.
                if canonical == trimmed {
                    return Ok(ValueStart::Float(trimmed));
                }
                let s = bump.alloc_str(canonical);
                return Ok(ValueStart::Float(s));
            }
        }
    }

    // § 5.2 rule 15: String
    Ok(ValueStart::Scalar(trimmed))
}

/// Convert a `Value` to a flat sequence of events (for inline compounds).
fn value_to_events<'a>(value: &crate::value::Value, bump: &'a Bump) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    value_to_events_inner(value, bump, &mut events);
    events
}

fn value_to_events_inner<'a>(
    value: &crate::value::Value,
    bump: &'a Bump,
    events: &mut Vec<Event<'a>>,
) {
    use crate::value::Value;
    match value {
        Value::Null => events.push(Event::Null),
        Value::Bool(b) => events.push(Event::Bool(*b)),
        Value::Integer(s) => {
            let s = bump.alloc_str(s.as_str());
            events.push(Event::Integer(s));
        }
        Value::Float(s) => {
            let s = bump.alloc_str(s.as_str());
            events.push(Event::Float(s));
        }
        Value::String(s) => {
            let s = bump.alloc_str(s.as_str());
            events.push(Event::Str(s));
        }
        Value::Object(obj) => {
            events.push(Event::BeginObject);
            for (k, v) in obj {
                let k = bump.alloc_str(k.as_str());
                events.push(Event::Key(k));
                value_to_events_inner(v, bump, events);
            }
            events.push(Event::EndObject);
        }
        Value::Array(items) => {
            events.push(Event::BeginArray);
            for item in items {
                value_to_events_inner(item, bump, events);
            }
            events.push(Event::EndArray);
        }
    }
}

#[inline]
fn is_valid_key(k: &str) -> bool {
    let k = k.trim();
    if k.is_empty() {
        return false;
    }
    let bytes = k.as_bytes();
    // Spec 0.6.0 — `:` and `.` are allowed in a DECODED key segment
    // (the user expressed them via `\:` / `\.`). The remaining
    // structural bytes are still forbidden.
    for &b in bytes {
        if matches!(b, b'[' | b']' | b'{' | b'}' | b',' | b'(' | b')') {
            return false;
        }
    }
    true
}

/// Fast-path check: plain ASCII decimal integer (no sign, underscore, or
/// base prefix) that fits in i64. Returns `Some(val)` if `s` is a
/// canonical decimal integer, `None` otherwise. The caller can use the
/// original `s` directly as the canonical string, avoiding itoa + bump
/// allocation.
#[inline]
fn fast_plain_decimal_i64(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    // Leading zero is only valid for "0" itself.
    let first = bytes[0];
    if first == b'0' {
        return if bytes.len() == 1 { Some(0) } else { None };
    }
    if !(b'1'..=b'9').contains(&first) {
        return None;
    }
    // All remaining bytes must be digits; accumulate value.
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
