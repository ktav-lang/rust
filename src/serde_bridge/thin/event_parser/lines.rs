use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use memchr::{memchr, memchr2};

use crate::error::{CompoundKind, Error, ErrorKind, Result, Span};
use crate::parser::classify::is_pair_shape;
use crate::parser::inline::{scan_unescaped_colon, ColonScan, InlineBody};
use crate::parser::leading_bom_len;
use crate::whitespace::is_ktav_whitespace;

use super::super::inline_emit::scan_inline_events;
use super::super::{Event, EventSink, EventStream};

use super::classify::{
    classify, classify_separator, event_label, finalize_multiline, path_shape_of, require_sep_end,
    trimmed_span_in, BracketKind, Separator, ValueStart,
};
use super::state::{Collecting, EventParser, Frame, MultilineMode, NodeId, PathShape};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Returns the flat event stream plus the number of re-opened dotted-key
/// prefixes (spec 0.7 § 5.3.2 merge sites) encountered — callers that need
/// raw document order ignore the count; `from_str` uses it to decide
/// whether a reopen-merge normalization pass is required.
pub(crate) fn parse_events<'a>(text: &'a str, bump: &'a Bump) -> Result<(EventStream<'a>, usize)> {
    let mut events: EventStream<'a> = BumpVec::with_capacity_in(text.len() / 4 + 64, bump);

    // Spec § 5.0.1 (0.5.0): scan ahead to the first content line,
    // classify it per the 8 rules.
    let bytes = text.as_bytes();

    // Spec § 3.1: skip exactly one leading U+FEFF before any other
    // byte is examined — root-kind detection and line splitting
    // alike. Line offsets stay in original-input coordinates so
    // error Spans still slice the caller's text.
    let start = leading_bom_len(text);

    let mut p = EventParser::new(bump);

    // Line splitting: handle CR / CR LF / LF (spec § 3.2)
    //
    // Fast path for the overwhelmingly common case: LF-only input
    // (no CR bytes).  This avoids the per-byte branch on `\r` in the
    // inner scan loop and lets the compiler emit a tighter scan.
    if memchr(b'\r', bytes).is_none() {
        // LF-only fast path: memchr-backed `\n` splitting.
        let mut line_num: usize = 0;
        let mut line_start: usize = start;
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
        let mut line_start: usize = start;
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
    Ok((events, p.reopens))
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
        // § 5.0.1 rules 2-3: an inline root's events were already
        // emitted at line 1; rules 4-5: a closed explicit root's End
        // event was emitted by `close_frame`. Nothing more to emit.
        if self.root_consumed {
            debug_assert!(
                self.stack.is_empty(),
                "consumed root must have no open frames"
            );
            return Ok(());
        }
        // Close all synthetics still open in the root frame (only
        // applies to Object roots), then emit the matching close
        // event for whichever kind the root was. An explicit root
        // opened but never closed EOF-closes identically to an
        // implicit root (mirrors owned parser finish()).
        match self.stack.last() {
            Some(Frame::Object { .. }) => {
                self.close_synthetics_until(0, events);
                events.push(Event::EndObject);
            }
            Some(Frame::Array) => {
                events.push(Event::EndArray);
            }
            // Empty / comments-only document — root never initialized;
            // default to an empty implicit Object root (§ 5.0.1 rule 1).
            None => {
                events.push(Event::BeginObject);
                events.push(Event::EndObject);
            }
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
            // Pre-split line, exact trim parity (mirror parser.rs).
            let trimmed = raw.trim_matches(is_ktav_whitespace);
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

        // § 3.3: fixed 25-code-point class, never the host primitive.
        // Lines are pre-split on all three § 3.2 terminators (LF/CR/CRLF
        // — see parse_events above), so LF/CR cannot occur; the full
        // class is used for exact trim parity with str::trim.
        let trimmed = raw.trim_matches(is_ktav_whitespace);

        // Under 0.5.0: comments use `##` (not single `#`)
        if trimmed.is_empty() || trimmed.starts_with("##") {
            return Ok(());
        }

        let trimmed_span = trimmed_span_in(raw, trimmed, line_start);

        // § 5.0.1 — if root is already consumed (inline compound or
        // explicit-compound closed), any further content line is an
        // orphan. Comments stay legal, hence the ordering after the
        // blank/comment check above.
        if self.root_consumed {
            return Err(Error::Structured(
                ErrorKind::OrphanLineAfterTopLevelInline {
                    line: line_num as u32,
                    span: trimmed_span,
                },
            ));
        }

        // Spec § 5.0.1 — first content line establishes the root kind
        // (no pre-scan, mirroring the owned parser).
        if !self.root_initialized {
            self.root_initialized = true;
            // `}` / `]` first content line — not a valid root kind;
            // fall through to the close-frame branch which will raise
            // UnbalancedBracket against the empty stack.
            if trimmed != "}"
                && trimmed != "]"
                && self.classify_root(trimmed, line_num, trimmed_span, events)?
            {
                return Ok(());
            }
        }

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

    /// Classify the first content line (spec § 5.0.1) and push the root
    /// frame. Returns `Ok(true)` when the line was fully handled (inline
    /// root or explicit opener) and `Ok(false)` when an implicit root
    /// was pushed and the SAME line must fall through to ordinary
    /// closer/pair/item dispatch.
    fn classify_root<S: EventSink<'a>>(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<bool> {
        // § 5.0.1 rule 4: lone `{`
        if trimmed == "{" {
            self.root_is_explicit_compound = true;
            self.stack.push(Frame::new_object(self.bump, 0));
            self.opener_offsets.push(trimmed_span.start);
            EventSink::push(events, Event::BeginObject);
            return Ok(true);
        }
        // § 5.0.1 rule 5: lone `[`
        if trimmed == "[" {
            self.root_is_explicit_compound = true;
            self.stack.push(Frame::new_array());
            self.opener_offsets.push(trimmed_span.start);
            EventSink::push(events, Event::BeginArray);
            return Ok(true);
        }

        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            // § 5.0.1 rules 2/3 + the rules-2–5 addendum: the closer
            // triage inside `scan_inline_events` decides rule 6 (the
            // line IS the whole-document root) vs rule 8/9 errors.
            let kind = if trimmed.starts_with('{') {
                InlineBody::Object
            } else {
                InlineBody::Array
            };
            scan_inline_events(trimmed, kind, line_num, trimmed_span, self.bump, events)?;
            self.root_consumed = true;
            return Ok(true);
        }

        // § 5.0.1 rules 6/7: pair-shape → implicit Object root,
        // array-item-shape → implicit Array root.
        if is_pair_shape(trimmed) {
            self.stack.push(Frame::new_object(self.bump, 0));
            self.opener_offsets.push(0);
            EventSink::push(events, Event::BeginObject);
        } else {
            self.stack.push(Frame::new_array());
            self.opener_offsets.push(0);
            EventSink::push(events, Event::BeginArray);
        }
        Ok(false)
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
        // Spec 0.7 § 5.3.3 / § 6.16: a quoted segment that never closes
        // swallows the separator — that takes precedence over
        // MissingSeparator.
        let colon = match scan_unescaped_colon(trimmed) {
            ColonScan::Found(c) => c,
            ColonScan::UnterminatedQuote => {
                return Err(Error::Structured(ErrorKind::UnterminatedQuotedKey {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
            ColonScan::Absent => {
                return Err(Error::Structured(ErrorKind::MissingSeparator {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
        };

        // Pre-split line, exact trim parity (mirror parser.rs).
        let key = trimmed[..colon].trim_end_matches(is_ktav_whitespace);
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
                self.emit_keyed_scalar(
                    key,
                    // Pre-split line, exact trim parity (mirror parser.rs).
                    Event::Str(rest.trim_matches(is_ktav_whitespace)),
                    line_num,
                    key_span,
                    events,
                )
            }
            Separator::Plain => {
                require_sep_end(after_colon, line_num, after_colon_off, trimmed_span)?;
                // Pre-split line, exact trim parity (mirror parser.rs).
                let body = after_colon.trim_start_matches(is_ktav_whitespace);
                match classify(body, self.bump)? {
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
                        let node = self.emit_keyed_open(
                            key,
                            Event::BeginObject,
                            line_num,
                            key_span,
                            events,
                        )?;
                        self.stack.push(Frame::new_object(self.bump, node));
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
                    ValueStart::InlineCompound(kind) => {
                        // Scan the closed inline compound FIRST: its
                        // internal errors (BadEscapeSequence, inline
                        // duplicates, …) keep their precedence over this
                        // line's dotted-key reconciliation and path
                        // registration. Events are staged — reconcile's
                        // synthetic prefix events must precede the
                        // compound's events in the stream, and
                        // `register_inline_child_paths` must walk the
                        // finished event list before the first inline
                        // event is published.
                        let mut staging = std::mem::take(&mut self.staging);
                        staging.clear();
                        scan_inline_events(
                            body,
                            kind,
                            line_num,
                            trimmed_span,
                            self.bump,
                            &mut staging,
                        )?;
                        let (leaf, parent_node) =
                            self.reconcile_dotted_key(key, line_num, key_span, events)?;
                        let shape = match staging.first() {
                            Some(ev) => path_shape_of(ev),
                            None => unreachable!("inline compound always emits events"),
                        };
                        let node = self.register_value_path(
                            parent_node,
                            leaf,
                            shape,
                            key,
                            line_num,
                            key_span,
                        )?;
                        if matches!(shape, PathShape::Object) {
                            // § 5.3.2 / § 6.3: the inline compound's
                            // internal key paths must be visible to
                            // later dotted re-entry in THIS frame.
                            self.register_inline_child_paths(node, &staging, line_num, key_span)?;
                        }
                        events.push(Event::Key(leaf));
                        for ev in &staging {
                            events.push(*ev);
                        }
                        self.staging = staging;
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
        let (leaf, parent_node) = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        let label = event_label(&value);
        self.register_value_path(
            parent_node,
            leaf,
            PathShape::Leaf(label),
            key,
            line_num,
            key_span,
        )?;
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
        let (leaf, parent_node) = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        let shape = path_shape_of(&open);
        self.register_value_path(parent_node, leaf, shape, key, line_num, key_span)?;
        events.push(Event::Key(leaf));
        events.push(open);
        events.push(close);
        Ok(())
    }

    /// For `key: {` / `key: [` — register the key path and emit `Key` + open; returns the registered node for the caller's `frame_root`.
    fn emit_keyed_open<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        open: Event<'a>,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<NodeId> {
        let (leaf, parent_node) = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        let shape = path_shape_of(&open);
        let node = self.register_value_path(parent_node, leaf, shape, key, line_num, key_span)?;
        events.push(Event::Key(leaf));
        events.push(open);
        Ok(node)
    }

    fn emit_keyed_open_multiline<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        mode: MultilineMode,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<()> {
        let (leaf, parent_node) = self.reconcile_dotted_key(key, line_num, key_span, events)?;
        self.register_value_path(
            parent_node,
            leaf,
            PathShape::Leaf("string"),
            key,
            line_num,
            key_span,
        )?;
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
            // `rest` is a within-line slice after `::` on a pre-split
            // line — LF/CR cannot occur (mirror parser.rs site).
            events.push(Event::Str(rest.trim_start_matches(is_ktav_whitespace)));
            return Ok(());
        }

        match classify(trimmed, self.bump)? {
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
                // Array elements are unnamed; paths never cross an
                // array boundary (arrays are leaves), so the frame gets
                // a detached root node and nothing is linked to any
                // enclosing object on close.
                let fr = self.add_detached_root();
                self.stack.push(Frame::new_object(self.bump, fr));
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
            ValueStart::InlineCompound(kind) => {
                scan_inline_events(trimmed, kind, line_num, trimmed_span, self.bump, events)?
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
}
