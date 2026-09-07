//! Tokenize Ktav text directly into a flat [`EventStream`] — no
//! intermediate tree. Mirrors the validation logic of [`super::parser`]
//! but emits a linear sequence of `Event`s into a single bump-arena
//! `Vec` instead of a recursive `ThinValue`.
//!
//! Dotted keys are resolved here, at tokenize time, by maintaining a
//! per-object-frame stack of currently-open synthetic prefixes. When a
//! new line's prefix diverges from the stack, the divergence point is
//! emitted as a sequence of `EndObject`s; the new tail is emitted as
//! `Key`+`BeginObject`s.
//!
//! Duplicates and path conflicts are caught the same way the tree-builder
//! catches them (spec 0.7 § 5.3.2 / § 6.3): the parser maintains ONE
//! parse-wide shared node arena of key-path SHAPES — one `PathShape`
//! per real
//! key segment ever seen, labelled `Object` or `Leaf(<kind>)` — plus a
//! two-tier lookup index keyed on `(parent node id, decoded segment)`:
//! small documents scan a contiguous linear list with zero hashing and
//! zero heap allocation, and once entries exceed
//! `LINEAR_INDEX_THRESHOLD` the index spills (once) into a
//! `FxHashMap` for O(1) lookups on large documents. Identity is
//! STRUCTURAL: a node is its `(parent, segment)` pair, never a joined
//! string, because decoded segments may contain literal dots. A child
//! object's frame holds only the `NodeId` of its own key path, so its
//! subtree is visible to the enclosing frame with no copying on close.
//! A dotted key re-entering a path already shaped as an `Object`
//! (whether created by an earlier dotted pair or explicitly as
//! `a: { … }`) MERGES, regardless of intervening sibling pairs; a dotted
//! path descending through a non-Object leaf, or a plain pair naming an
//! earlier-established Object, raises `KeyPathConflict`. The synthetic
//! `ObjectLevel`s hold no key state at all — only the prefix needed for
//! longest-common-prefix comparison and emission bookkeeping.
//!
//! A compound value's INTERNAL key paths — for both inline (`a: {x: 1}`)
//! and explicit multi-line compounds — are registered into the shared
//! parse-wide shared node-shape arena under the compound's own node, recursively for nested
//! objects, and stop at array boundaries (arrays are leaves), so later
//! dotted re-entry sees them (§ 5.3.2 / § 6.3). Event order / first-
//! appearance order lives in the event stream and is independent of the
//! node index.

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use memchr::{memchr, memchr2};
use rustc_hash::FxHashMap;

use crate::error::{CompoundKind, ConflictKind, Error, ErrorKind, Result, Span};
use crate::parser::classify::{is_float_literal, is_pair_shape, try_parse_integer};
use crate::parser::inline::{
    decode_key_segment, key_is_single_segment, malformed_closer_not_at_end, scan_inline_closer,
    scan_unescaped_colon, split_key_path, ColonScan, InlineCloserScan,
};
use crate::parser::leading_bom_len;
use crate::parser::validate::{check_key, KeyValidity};
use crate::whitespace::is_ktav_whitespace;

use super::event::{Event, EventSink, EventStream};

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
// Parser state
// ---------------------------------------------------------------------------

/// Below this many registered `(parent, segment)` entries the parser
/// scans a small contiguous list instead of touching the hash map —
/// small documents (the common case) then pay no hashing and no heap
/// allocation at all. 8 matches the owned parser's per-frame
/// `ObjectMap` capacity precedent (`src/parser/frame.rs`).
const LINEAR_INDEX_THRESHOLD: usize = 8;

struct LinearEntry<'a> {
    parent: NodeId,
    segment: &'a str,
    id: NodeId,
}

pub(crate) struct EventParser<'a> {
    pub(crate) bump: &'a Bump,
    pub(crate) stack: Vec<Frame<'a>>,
    pub(crate) collecting: Option<Collecting<'a>>,
    /// Byte offset (in original input) of the opener that started each
    /// frame — one entry per open frame. The implicit root records `0`
    /// (it has no opener); an explicit `{`/`[`-opened root records the
    /// byte offset of its opener, mirroring parser.rs lines 167-186.
    pub(crate) opener_offsets: Vec<u32>,
    /// Byte offset of the `(` / `((` line that started a multi-line
    /// string, if one is currently being collected.
    pub(crate) multiline_opener: Option<u32>,
    /// `false` until the first content line is classified and the root
    /// frame pushed. Spec § 5.0.1 determines the root kind lazily, with
    /// no pre-scan — mirroring the owned parser.
    pub(crate) root_initialized: bool,
    /// Set after a top-level inline compound (§ 5.0.1 rules 2-3) or
    /// after the matching close of a lone-`{`/`[`-opened root (rules
    /// 4-5). Any further non-blank, non-comment line is then
    /// `OrphanLineAfterTopLevelInline`.
    pub(crate) root_consumed: bool,
    /// True when the root was opened by a lone `{` or `[` (§ 5.0.1
    /// rules 4-5); a depth-1 close then consumes the root instead of
    /// erroring.
    pub(crate) root_is_explicit_compound: bool,
    /// Number of dotted-key RE-OPENED prefixes emitted as synthetic
    /// `Key`+`BeginObject` pairs this parse — i.e. pushes of a prefix
    /// whose persistent path entry already existed as an Object before
    /// the current line (spec 0.7 § 5.3.2 merge case). Ordinary grouped
    /// dotted keys (`a.b: 1` then `a.c: 2`, no intervening sibling)
    /// re-use the still-open synthetic and do NOT count. `from_str`
    /// uses this to decide whether a reopen-merge pass is needed.
    pub(crate) reopens: usize,
    /// Shared parse-wide arena of key-path node SHAPES: slot 0 is a
    /// sentinel
    /// detached root serving the implicit root frame; every other slot
    /// holds one decoded key segment's shape, indexed by `NodeId`.
    /// Identity is the `(parent node id, decoded segment)` pair,
    /// enforced by the shared `index` — segments are NEVER
    /// joined into a `.`-separated string, because decoded segments may
    /// themselves contain literal dots. Ordering lives in the event
    /// stream, not here.
    pub(crate) nodes: BumpVec<'a, PathShape>,
    /// Two-tier lookup index over `nodes`, keyed on `(parent, segment)`
    /// — ONE per parse, shared by all frames. Below
    /// [`LINEAR_INDEX_THRESHOLD`] registered entries it is a contiguous
    /// linear list (`linear`) with zero hashing and zero heap
    /// allocation, which is what small documents (the common case) hit;
    /// the first insert past the threshold spills once into a
    /// `FxHashMap` (`index`) for O(1) lookups on large documents.
    /// Identity is the `(parent node id, decoded segment)` pair —
    /// segments are NEVER joined into a `.`-separated string, because
    /// decoded segments may themselves contain literal dots. Ordering
    /// lives in the event stream, not here.
    linear: BumpVec<'a, LinearEntry<'a>>,
    /// `None` until the linear tier spills (see [`LINEAR_INDEX_THRESHOLD`]).
    index: Option<FxHashMap<(NodeId, &'a str), NodeId>>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_node_allocs: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_index_probes: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_entry_compares: usize,
}

impl<'a> EventParser<'a> {
    pub(crate) fn new(bump: &'a Bump) -> Self {
        // The root frame is pushed lazily by `classify_root` once the
        // first content line is classified — no pre-scan. `finish`
        // falls back to an empty Object root if no content line was
        // ever encountered (§ 5.0.1 rule 1).
        let mut p = EventParser {
            bump,
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
            root_initialized: false,
            root_consumed: false,
            root_is_explicit_compound: false,
            reopens: 0,
            nodes: {
                let mut nodes = BumpVec::with_capacity_in(16, bump);
                // Sentinel detached root: `frame_root` of the implicit
                // root frame.
                nodes.push(PathShape::Object);
                nodes
            },
            linear: BumpVec::with_capacity_in(LINEAR_INDEX_THRESHOLD, bump),
            index: None,
            dbg_node_allocs: 0,
            dbg_index_probes: 0,
            dbg_entry_compares: 0,
        };
        // The sentinel push above counts as a node allocation, so the
        // counter reflects the exact arena size.
        p.dbg_node_allocs += 1;
        p
    }

    fn node_shape(&self, id: NodeId) -> PathShape {
        self.nodes[id as usize]
    }

    fn probe(&mut self, parent: NodeId, segment: &str) -> Option<NodeId> {
        self.dbg_index_probes += 1;
        if let Some(index) = &self.index {
            self.dbg_entry_compares += 1;
            return index.get(&(parent, segment)).copied();
        }
        for i in 0..self.linear.len() {
            let e = &self.linear[i];
            self.dbg_entry_compares += 1;
            if e.parent == parent && e.segment == segment {
                return Some(e.id);
            }
        }
        None
    }

    fn add_child(&mut self, parent: NodeId, segment: &'a str, shape: PathShape) -> NodeId {
        let id = u32::try_from(self.nodes.len()).expect("path node count overflow");
        self.nodes.push(shape);
        self.dbg_node_allocs += 1;
        if let Some(index) = &mut self.index {
            index.insert((parent, segment), id);
        } else if self.linear.len() >= LINEAR_INDEX_THRESHOLD {
            // Spill: build the hash from the retained linear entries.
            // The list itself is left in place — a few hundred bytes of
            // arena, never consulted again once `index` is `Some`.
            let mut map = FxHashMap::default();
            for e in &self.linear {
                map.insert((e.parent, e.segment), e.id);
            }
            map.insert((parent, segment), id);
            self.index = Some(map);
        } else {
            self.linear.push(LinearEntry {
                parent,
                segment,
                id,
            });
        }
        id
    }

    /// Detached root for frames whose key path lives in no enclosing
    /// object — array-element objects (arrays are leaves, so paths never
    /// cross an array boundary).
    fn add_detached_root(&mut self) -> NodeId {
        let id = u32::try_from(self.nodes.len()).expect("path node count overflow");
        self.nodes.push(PathShape::Object);
        self.dbg_node_allocs += 1;
        id
    }
}

pub(crate) type NodeId = u32;

pub(crate) enum Frame<'a> {
    /// `levels` is parallel to "real frame + open synthetic prefixes".
    /// Index 0 is always the real object's namespace; subsequent entries
    /// are stacked synthetics. Key state lives in the parse-wide shared
    /// node arena/index, NOT in the frame: the frame only holds the
    /// `NodeId` under which its own entries are keyed, so a dotted
    /// prefix closed by intervening siblings can still be re-entered (it
    /// merges) and duplicate/conflict detection below a reopened prefix
    /// stays exact — with no per-frame table and no copying on close.
    Object {
        levels: BumpVec<'a, ObjectLevel<'a>>,
        /// The node of this object's own key path in the shared node-shape
        /// arena — top-level entries of the frame are keyed
        /// `(frame_root, segment)`. Node `0` (sentinel detached root)
        /// for root frames and for objects opened inside arrays (array
        /// elements are unnamed, and arrays are leaves in the path
        /// model, so paths never cross an array boundary).
        frame_root: NodeId,
    },
    Array,
}

/// Shape a registered key path holds, mirroring the owned parser's
/// `Value` kinds in `parser::insert` (§ 6.3 conflict classification).
#[derive(Clone, Copy)]
pub(crate) enum PathShape {
    /// The path was established as a nested Object.
    Object,
    /// The path was established as a leaf value; the label is the same
    /// kind string the owned parser's `kind_label` produces (used in
    /// `ConflictKind::Overwrite` diagnostics).
    Leaf(&'static str),
}

pub(crate) struct ObjectLevel<'a> {
    /// `None` for the real object level, `Some(prefix_segment)` for a
    /// synthetic dotted-key level. Kept ONLY for the LCP comparison and
    /// emission bookkeeping — all key state lives in the shared node
    /// arena/index.
    prefix: Option<&'a str>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new_object(bump: &'a Bump, frame_root: NodeId) -> Self {
        let mut levels = BumpVec::with_capacity_in(2, bump);
        levels.push(ObjectLevel { prefix: None });
        Frame::Object { levels, frame_root }
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

        // § 5.0.1 rules 2/3 + the rules-2–5 addendum: a first content
        // line beginning with `{`/`[` is diagnosed by the same § 5.2
        // closer scan as a value body — closed at the end ⇒ the root IS
        // the inline value; such a line is never a pair candidate.
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            let (open, close) = if trimmed.starts_with('{') {
                (b'{', b'}')
            } else {
                (b'[', b']')
            };
            match scan_inline_closer(trimmed, open, close, line_num, trimmed_span) {
                InlineCloserScan::BadEscape(err) => return Err(err),
                InlineCloserScan::NotFound => {
                    return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                        line: line_num as u32,
                        span: trimmed_span,
                    }));
                }
                InlineCloserScan::Found(idx) if idx != trimmed.len() - 1 => {
                    return Err(malformed_closer_not_at_end(line_num, trimmed_span));
                }
                InlineCloserScan::Found(_) => {
                    // The line IS the whole-document root. Thin has no
                    // strict mode (same as `dispatch_inline_events`).
                    let value = if open == b'{' {
                        crate::parser::inline::parse_inline_object(
                            trimmed,
                            line_num,
                            trimmed_span,
                            false,
                        )?
                    } else {
                        crate::parser::inline::parse_inline_array(
                            trimmed,
                            line_num,
                            trimmed_span,
                            false,
                        )?
                    };
                    for ev in value_to_events(&value, self.bump) {
                        EventSink::push(events, ev);
                    }
                    self.root_consumed = true;
                    return Ok(true);
                }
            }
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
                    ValueStart::InlineEvents(inline_events) => {
                        let (leaf, parent_node) =
                            self.reconcile_dotted_key(key, line_num, key_span, events)?;
                        let shape = match inline_events.first() {
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
                            self.register_inline_child_paths(
                                node,
                                &inline_events,
                                line_num,
                                key_span,
                            )?;
                        }
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

    /// Returns `(leaf_segment, parent_node_id)`: the node under which
    /// the leaf value's segment will be registered (the current frame's
    /// `frame_root` for single-segment keys, or the deepest prefix node
    /// for dotted keys).
    fn reconcile_dotted_key<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<(&'a str, NodeId)> {
        // Single segment (no UNescaped `.`) fast path. Decode the
        // segment if it contains a `\`; otherwise reuse the source
        // borrow.
        if key_is_single_segment(key) {
            self.close_synthetics_to_real(events);
            // Validate the RAW segment (forbidden bytes must be
            // escaped, quoted segments checked against their own
            // class) before decoding — see `parser::validate`.
            match check_key(key) {
                KeyValidity::Valid => {}
                KeyValidity::Empty => {
                    return Err(Error::Structured(ErrorKind::EmptyKey {
                        line: line_num as u32,
                        span: key_span,
                    }));
                }
                KeyValidity::Invalid => {
                    return Err(Error::Structured(ErrorKind::InvalidKey {
                        line: line_num as u32,
                        key: key.to_string(),
                        span: key_span,
                    }));
                }
            }
            let leaf = self.decode_key_in_arena(key, line_num, key_span)?;
            let frame_root = match self.stack.last() {
                Some(Frame::Object { frame_root, .. }) => *frame_root,
                _ => unreachable!("dispatched as object"),
            };
            return Ok((leaf, frame_root));
        }

        // Multi-segment path — split on UNescaped `.`, decode each
        // segment (arena-allocated if it had `\`).
        let raw_segments = split_key_path(key);
        debug_assert!(raw_segments.len() >= 2);
        let mut decoded_segments: Vec<&'a str> = Vec::with_capacity(raw_segments.len());
        for seg in &raw_segments {
            let trimmed = seg.trim();
            // Empty segment → EmptyKey (`a..b`, leading/trailing `.`;
            // spec 0.7 § 6.5 names `a..b` explicitly as EmptyKey).
            match check_key(trimmed) {
                KeyValidity::Valid => {}
                KeyValidity::Empty => {
                    return Err(Error::Structured(ErrorKind::EmptyKey {
                        line: line_num as u32,
                        span: key_span,
                    }));
                }
                KeyValidity::Invalid => {
                    return Err(Error::Structured(ErrorKind::InvalidKey {
                        line: line_num as u32,
                        key: key.to_string(),
                        span: key_span,
                    }));
                }
            }
            let decoded = self.decode_key_in_arena(trimmed, line_num, key_span)?;
            decoded_segments.push(decoded);
        }

        let leaf = *decoded_segments.last().unwrap();

        // Spec 0.7 § 5.3.2: a dotted key re-entering an Object that
        // already exists — whether created by an earlier dotted pair or
        // explicitly as `a: { … }` / `a: {}` — MUST merge, regardless
        // of intervening sibling pairs. Descend the shared node arena
        // from the frame's `frame_root`: every proper prefix must
        // either already be an Object node (fine — merge) or be absent
        // (create it); descending through a Leaf is a BlockedByValue
        // conflict (§ 6.3). The arena is NOT per synthetic level, so a
        // prefix closed by an intervening sibling stays visible here and
        // duplicate detection beneath a reopened prefix stays exact.
        // Identity is `(parent node id, decoded segment)` — segments
        // are never joined into a `.`-separated string, because decoded
        // segments may contain literal dots.
        let frame_root = match self.stack.last() {
            Some(Frame::Object { frame_root, .. }) => *frame_root,
            _ => unreachable!("dispatched as object"),
        };
        let mut cur = frame_root;
        let mut prefix_existed = vec![false; decoded_segments.len()];
        for k in 0..decoded_segments.len() - 1 {
            let seg = decoded_segments[k];
            match self.probe(cur, seg) {
                Some(id) if matches!(self.node_shape(id), PathShape::Leaf(_)) => {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        // RAW key text (escapes intact), matching the
                        // owned parser's `full_path` reporting.
                        path: key.to_string(),
                        kind: ConflictKind::BlockedByValue,
                        span: key_span,
                    }));
                }
                Some(id) => {
                    prefix_existed[k + 1] = true;
                    cur = id;
                }
                None => {
                    cur = self.add_child(cur, seg, PathShape::Object);
                }
            }
        }

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
        for (i, seg) in prefix_segments.iter().enumerate().skip(push_start) {
            if prefix_existed[i + 1] {
                // The prefix was closed by intervening siblings (or was
                // an explicit `a: { … }`) and is being re-opened as a
                // synthetic — a § 5.3.2 merge site the deserializer's
                // merge pass must fold back together.
                self.reopens += 1;
            }
            self.push_synthetic(seg, events);
        }

        Ok((leaf, cur))
    }

    /// Decode a key segment per § 3.7 / § 5.3.3. Bare segments without
    /// a `\` return the source slice as-is (zero-copy fast path), as do
    /// quoted segments (§ 5.3.3) whose interior contains no `\` — the
    /// key is then the source slice between the delimiters, never
    /// trimmed. Any segment containing a `\` decodes via
    /// `decode_key_segment` and is allocated in the bump arena so the
    /// returned `&'a str` outlives the call.
    fn decode_key_in_arena(
        &self,
        seg: &'a str,
        line_num: usize,
        key_span: Span,
    ) -> Result<&'a str> {
        let is_quoted = seg
            .as_bytes()
            .first()
            .is_some_and(|&b| b == b'"' || b == b'\'' || b == b'`');
        if !is_quoted && !seg.as_bytes().contains(&b'\\') {
            return Ok(seg);
        }
        if is_quoted {
            debug_assert!(seg.len() >= 2 && seg.as_bytes()[seg.len() - 1] == seg.as_bytes()[0]);
            let interior = &seg[1..seg.len() - 1];
            if !interior.as_bytes().contains(&b'\\') {
                // Validated unescaped quoted interior: the key IS the source
                // slice between the quotes (§ 5.3.3 — quoted content is never
                // trimmed). No temporary String, no bump copy.
                return Ok(interior);
            }
        }
        let decoded = decode_key_segment(seg, line_num, key_span)?;
        Ok(self.bump.alloc_str(&decoded))
    }

    #[inline]
    fn push_synthetic<S: EventSink<'a>>(&mut self, seg: &'a str, events: &mut S) {
        // Emission-only: key state for the synthetic prefix is already
        // recorded in the shared node arena/index (or about to be by
        // `register_value_path`), so nothing can fail here.
        events.push(Event::Key(seg));
        events.push(Event::BeginObject);
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => levels.push(ObjectLevel { prefix: Some(seg) }),
            _ => unreachable!(),
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

    /// Register a fully-decoded leaf segment under `parent_node` with
    /// the value shape that now occupies it, implementing the owned
    /// parser's outcome tables (`parser::insert::insert_value` /
    /// `insert_dotted`, § 6.3). The parent node is looked up in the
    /// shared hash index — no path scan. The two entry points differ on
    /// an OCCUPIED slot:
    ///
    /// - Dotted key (`parent_node` deeper than the frame's
    ///   `frame_root`, mirrors `insert_dotted`): the descent was
    ///   already validated per-prefix by `reconcile_dotted_key`, so
    ///   ANY occupied final segment is a `DuplicateKey` —
    ///   unconditionally, regardless of shape.
    /// - Single-segment key (`parent_node == frame_root`, mirrors
    ///   `insert_value`'s four-arm table):
    ///
    ///   - leaf value onto existing Object → `KeyPathConflict`
    ///     `Overwrite { existing: "object", new_kind }`
    ///   - leaf value onto existing leaf → `DuplicateKey`
    ///   - Object onto existing Object → `DuplicateKey`
    ///   - Object onto existing leaf → `KeyPathConflict`
    ///     `Overwrite { existing: <kind>, new_kind: "object" }`
    ///
    /// - absent slot → record it. Returns the registered node's id.
    fn register_value_path(
        &mut self,
        parent_node: NodeId,
        leaf: &'a str,
        shape: PathShape,
        raw_key: &str,
        line_num: usize,
        key_span: Span,
    ) -> Result<NodeId> {
        let frame_root = match self.stack.last() {
            Some(Frame::Object { frame_root, .. }) => *frame_root,
            _ => unreachable!("only objects have keys"),
        };
        // Single-segment keys have parent == frame_root; every dotted
        // key and every inline-compound child has a deeper parent.
        let dotted = parent_node != frame_root;
        match self.probe(parent_node, leaf) {
            // `insert_dotted` parity: an occupied final segment
            // of a dotted key is ALWAYS a DuplicateKey — shape
            // conflicts along the way were already raised per-
            // prefix by `reconcile_dotted_key`.
            Some(_) if dotted => Err(Error::Structured(ErrorKind::DuplicateKey {
                line: line_num as u32,
                key: raw_key.to_string(),
                span: key_span,
            })),
            Some(id) => {
                let existing = self.node_shape(id);
                match existing {
                    PathShape::Object => match shape {
                        PathShape::Leaf(label) => {
                            Err(Error::Structured(ErrorKind::KeyPathConflict {
                                line: line_num as u32,
                                path: raw_key.to_string(),
                                kind: ConflictKind::Overwrite {
                                    existing: "object",
                                    new_kind: label,
                                },
                                span: key_span,
                            }))
                        }
                        PathShape::Object => Err(Error::Structured(ErrorKind::DuplicateKey {
                            line: line_num as u32,
                            key: raw_key.to_string(),
                            span: key_span,
                        })),
                    },
                    PathShape::Leaf(existing) => match shape {
                        PathShape::Leaf(_) => Err(Error::Structured(ErrorKind::DuplicateKey {
                            line: line_num as u32,
                            key: raw_key.to_string(),
                            span: key_span,
                        })),
                        PathShape::Object => Err(Error::Structured(ErrorKind::KeyPathConflict {
                            line: line_num as u32,
                            path: raw_key.to_string(),
                            kind: ConflictKind::Overwrite {
                                existing,
                                new_kind: "object",
                            },
                            span: key_span,
                        })),
                    },
                }
            }
            None => Ok(self.add_child(parent_node, leaf, shape)),
        }
    }

    /// Register the INTERNAL key paths of an inline compound value
    /// (`a: {x: 1}` — events already flattened by `value_to_events`)
    /// into the shared node arena, under `base_node` (the node just
    /// registered for the compound itself). Recurses into nested
    /// objects; arrays are leaves — nothing inside a bracketed array is
    /// registered (§ 5.3.2 / § 6.3).
    ///
    /// Registration is provably collision-free: the inline `Value` was
    /// already validated internally by `parse_inline_object`'s
    /// `insert_value` (each path appears exactly once), and `base_node`
    /// was just inserted absent. Errors are still propagated with `?`
    /// defensively rather than panicking.
    fn register_inline_child_paths(
        &mut self,
        base_node: NodeId,
        events: &[Event<'a>],
        line_num: usize,
        key_span: Span,
    ) -> Result<()> {
        debug_assert!(matches!(events.first(), Some(Event::BeginObject)));
        debug_assert!(matches!(events.last(), Some(Event::EndObject)));
        let inner = &events[1..events.len() - 1];
        let mut i = 0;
        while i < inner.len() {
            let k = match &inner[i] {
                Event::Key(k) => *k,
                other => unreachable!("pair position must be Key, got {other:?}"),
            };
            // Each Key is immediately followed by exactly one value
            // event or a bracketed compound — guaranteed by
            // `value_to_events`.
            let value_ev = &inner[i + 1];
            let shape = path_shape_of(value_ev);
            let child_node =
                self.register_value_path(base_node, k, shape, k, line_num, key_span)?;
            match value_ev {
                Event::BeginObject => {
                    let j = matching_bracket(inner, i + 1, b'o');
                    self.register_inline_child_paths(
                        child_node,
                        &inner[i + 1..=j],
                        line_num,
                        key_span,
                    )?;
                    i = j + 1;
                }
                Event::BeginArray => {
                    // Arrays are leaves — skip their interior.
                    let j = matching_bracket(inner, i + 1, b'a');
                    i = j + 1;
                }
                _ => i += 2,
            }
        }
        Ok(())
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
        // Depth-1 close of a lone-`{`/`[`-opened root (§ 5.0.1 rules
        // 4-5): a matching close consumes the root — the frame is
        // popped, its End event emitted, and `root_consumed` set
        // (mirrors owned parser close_frame). Mismatched kind at this
        // depth errors against the FRAME's kind.
        if self.stack.len() == 1 && self.root_is_explicit_compound {
            let frame_kind = match self.stack.last() {
                Some(Frame::Object { .. }) => BracketKind::Object,
                _ => BracketKind::Array,
            };
            if frame_kind as u8 != expected as u8 {
                return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                    line: line_num as u32,
                    span: trimmed_span,
                    expected: frame_kind.to_compound(),
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
            self.root_consumed = true;
            let close_event = match got {
                BracketKind::Object => Event::EndObject,
                BracketKind::Array => Event::EndArray,
            };
            events.push(close_event);
            return Ok(());
        }
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
        let popped = self.stack.pop().unwrap();
        let got = match &popped {
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
        // § 5.3.2: no fold is needed on close. The child frame's
        // `frame_root` IS the node of its own key path in the shared
        // global node arena/index; its top-level entries are keyed
        // `(frame_root, segment)` — exactly the key the parent's
        // dotted-key descent would probe. The child's subtree is
        // therefore already linked at the parent's descent point and
        // visible to later `a.x: 2` re-entry with no copying. Any
        // collision the old defensive fold arm claimed to catch is
        // impossible: the shared index enforces `(parent, segment)`
        // node identity at insertion time.
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
    // § 3.3 fixed class; `rest` is a suffix of one pre-split line (§ 3.2),
    // so LF/CR cannot occur.
    if rest.is_empty() || rest.starts_with(is_ktav_whitespace) {
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

/// Map a value event to the same kind label the owned parser's
/// `kind_label` (src/parser/insert.rs) produces — these strings appear
/// verbatim in `ConflictKind::Overwrite` diagnostics (§ 6.3).
#[inline]
fn event_label(ev: &Event<'_>) -> &'static str {
    match ev {
        Event::Null => "null",
        Event::Bool(_) => "bool",
        Event::Integer(_) => "integer",
        Event::Float(_) => "float",
        Event::Str(_) => "string",
        Event::BeginArray => "array",
        Event::BeginObject => "object",
        _ => unreachable!("not a value-start event"),
    }
}

/// The [`PathShape`] a keyed value establishes: an `Event::BeginObject`
/// opener makes the path an Object; anything else (including an array,
/// which is a leaf under the owned parser's model) is a leaf of the
/// event's kind.
#[inline]
fn path_shape_of(ev: &Event<'_>) -> PathShape {
    match ev {
        Event::BeginObject => PathShape::Object,
        other => PathShape::Leaf(event_label(other)),
    }
}

/// Index of the bracket event in `events` matching the opener at
/// `open_idx`. `kind` is `b'o'` for object brackets, `b'a'` for array
/// brackets — used by `register_inline_child_paths` to skip leaf-array
/// interiors and recurse into nested objects.
fn matching_bracket(events: &[Event<'_>], open_idx: usize, kind: u8) -> usize {
    let (open, close) = if kind == b'o' {
        (Event::BeginObject, Event::EndObject)
    } else {
        (Event::BeginArray, Event::EndArray)
    };
    let mut depth = 0usize;
    let mut j = open_idx;
    loop {
        match events[j] {
            ref ev if *ev == open => depth += 1,
            ref ev if *ev == close => {
                depth -= 1;
                if depth == 0 {
                    return j;
                }
            }
            _ => {}
        }
        j += 1;
    }
}

/// § 5.2 rules 6–9 for a non-empty `{`/`[`-prefixed value body, mirroring
/// the owned parser's `classify::dispatch_inline_compound`: one closer
/// scan decides closed shape (parse per § 5.8, re-emitted as events),
/// closer-followed-by-content (`MalformedInlineCompound`), no closer
/// (`UnterminatedInlineCompound`), with `BadEscapeSequence` taking
/// precedence per the rules-6–9 preamble.
fn dispatch_inline_events<'a>(
    trimmed: &'a str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
    bump: &'a Bump,
) -> Result<ValueStart<'a>> {
    match scan_inline_closer(trimmed, open, close, line_num, span) {
        InlineCloserScan::BadEscape(err) => Err(err),
        InlineCloserScan::NotFound => {
            Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            }))
        }
        InlineCloserScan::Found(idx) if idx == trimmed.len() - 1 => {
            let value = if open == b'{' {
                crate::parser::inline::parse_inline_object(trimmed, line_num, span, false)?
            } else {
                crate::parser::inline::parse_inline_array(trimmed, line_num, span, false)?
            };
            Ok(ValueStart::InlineEvents(value_to_events(&value, bump)))
        }
        InlineCloserScan::Found(_) => Err(malformed_closer_not_at_end(line_num, span)),
    }
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

    // § 5.2 rules 6-9: inline compounds — diagnosed by one closer scan
    // (see `dispatch_inline_events`). Empty compounds shortcut first.
    if trimmed.starts_with('{') {
        if trimmed.ends_with('}') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyObject);
        }
        return dispatch_inline_events(trimmed, b'{', b'}', line_num, trimmed_span, bump);
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        return dispatch_inline_events(trimmed, b'[', b']', line_num, trimmed_span, bump);
    }

    // Multi-line string openers
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar("")),
        _ => {}
    }

    // Spec 0.7 § 5.2: only the bare tokens `(` / `((` open multi-line
    // strings; anything else starting with `(` is an ordinary inline
    // scalar and falls through to classification below (mirrors
    // parser/classify.rs; fixture `inline/paren_scalar_is_string`).

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
                only.trim_start().trim_end()
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
            out.push_str(l[common_len..].trim_end());
        } else {
            out.push_str(l.trim_end());
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

// ---------------------------------------------------------------------------
// Tests: deterministic allocation/probe counters
// ---------------------------------------------------------------------------

#[cfg(test)]
mod counter_tests {
    use super::*;

    /// Runs a full LF-only parse (duplicating `parse_events`' fast-path
    /// line loop, since `parse_events` builds the parser locally) and
    /// returns the exact `(dbg_node_allocs, dbg_index_probes,
    /// dbg_entry_compares)` counts.
    /// All test docs are LF-only, so only the `memchr(b'\r', ..).is_none()`
    /// branch is needed.
    fn parse_counters(doc: &str) -> (usize, usize, usize) {
        let bump = Bump::new();
        let mut p = EventParser::new(&bump);
        let mut events: EventStream<'_> = BumpVec::with_capacity_in(64, &bump);
        let bytes = doc.as_bytes();
        let mut line_num: usize = 0;
        let mut line_start: usize = 0;
        while line_start <= bytes.len() {
            let end = memchr(b'\n', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            let line: &str = &doc[line_start..end];
            line_num += 1;
            p.handle_line(line, line_num, line_start as u32, &mut events)
                .unwrap();
            if end == bytes.len() {
                break;
            }
            line_start = end + 1;
        }
        p.finish(bytes.len() as u32, &mut events).unwrap();
        (p.dbg_node_allocs, p.dbg_index_probes, p.dbg_entry_compares)
    }

    /// Deep chain `a: { ... a: { x: 1 } ... }` of depth D.
    ///
    /// EXACT counts, derived from the code paths (not fitted):
    /// - `dbg_node_allocs == D + 2`: one node per `a` level (D), one for
    ///   `x` (leaf), plus the sentinel pushed in `new()` (which counts,
    ///   so totals stay exact). `x: 1` sits inside D open `a` objects,
    ///   so there is exactly one leaf node.
    /// - `dbg_index_probes == D + 1`: one probe per real key seen — D
    ///   for the `a` openers, 1 for `x`. `}` closers probe nothing.
    ///
    /// The PRE-FIX representation copied the whole key path as `&str`
    /// slots on every frame close, allocating Theta(D^3) path slots
    /// overall. Measured slot-copies at D = 4 / 8 / 16 / 32 were
    /// 35 / 165 / 969 / 6,545 (doubling ratios 4.7 / 5.9 / 6.8 —
    /// clearly super-quadratic, trending cubic). The shape arena makes
    /// allocation exactly linear in nodes: allocs(2D) <= 3 * allocs(D).
    #[test]
    fn deep_chain_path_metadata_is_linear() {
        for d in [4usize, 8, 16, 32] {
            let mut doc = String::new();
            for _ in 0..d {
                doc.push_str("a: {\n");
            }
            doc.push_str("x: 1\n");
            for _ in 0..d {
                doc.push_str("}\n");
            }
            let (allocs, probes, _) = parse_counters(&doc);
            assert_eq!(allocs, d + 2, "node allocs at D={d}");
            assert_eq!(probes, d + 1, "index probes at D={d}");
        }
        // Linear-growth check across consecutive pairs of the sizes above.
        let allocs_of = |d: usize| {
            let mut doc = String::new();
            for _ in 0..d {
                doc.push_str("a: {\n");
            }
            doc.push_str("x: 1\n");
            for _ in 0..d {
                doc.push_str("}\n");
            }
            parse_counters(&doc).0
        };
        let a4 = allocs_of(4);
        let a8 = allocs_of(8);
        let a16 = allocs_of(16);
        let a32 = allocs_of(32);
        assert!(a8 <= 3 * a4);
        assert!(a16 <= 3 * a8);
        assert!(a32 <= 3 * a16);
    }

    /// Flat object with K scalar keys, one per line.
    ///
    /// EXACT counts: one node per key + the sentinel in `new()`
    /// (`dbg_node_allocs == K + 1`), and one index probe per key
    /// (`dbg_index_probes == K`). No synthetic prefixes exist, so no
    /// extra nodes or probes appear.
    ///
    /// PRE-FIX, every duplicate/conflict check compared full joined
    /// key-path strings entry-by-entry against a per-frame table —
    /// exactly K(K-1)/2 entry comparisons for K keys: 28 / 120 / 496 /
    /// 2,016 at K = 8 / 16 / 32 / 64 — precisely quadratic. The shared
    /// index makes probes exactly linear: probes(2K) <= 3*probes(K).
    ///
    /// The hybrid two-tier index restores pre-fix EXACT comparison
    /// counts below the spill threshold and stays sub-quadratic above:
    /// the i-th key's lookup (0-based) scans i entries while linear, so
    /// for K <= 8: compares = K(K-1)/2 (K=8 → 28, exactly the pre-fix
    /// count). The 9th insert spills (linear holds 8), so for K >= 9:
    /// compares(K) = 28 + 8 + (K-9) → K=16 → 43, K=32 → 59, K=64 → 91.
    /// PRE-FIX comparisons were exactly K(K-1)/2 (28 / 120 / 496 / 2,016);
    /// the hybrid matches pre-fix below the threshold and is
    /// sub-quadratic above it.
    #[test]
    fn flat_object_index_probes_are_linear() {
        let build = |k: usize| {
            let mut doc = String::new();
            for i in 0..k {
                doc.push_str(&format!("k{i}: {i}\n"));
            }
            doc
        };
        for k in [8usize, 16, 32, 64] {
            let (allocs, probes, compares) = parse_counters(&build(k));
            assert_eq!(allocs, k + 1, "node allocs at K={k}");
            assert_eq!(probes, k, "index probes at K={k}");
            let expected_compares = if k <= 8 {
                k * (k - 1) / 2
            } else {
                28 + 8 + (k - 9)
            };
            assert_eq!(compares, expected_compares, "entry compares at K={k}");
        }
        let counters_of = |k: usize| parse_counters(&build(k));
        let probes_of = |k: usize| parse_counters(&build(k)).1;
        let p8 = probes_of(8);
        let p16 = counters_of(16).1;
        let p32 = counters_of(32).1;
        let p64 = counters_of(64).1;
        assert!(p16 <= 3 * p8);
        assert!(p32 <= 3 * p16);
        assert!(p64 <= 3 * p32);
        // Compares grow sub-quadratically across doubling pairs.
        assert!(counters_of(32).2 <= 3 * counters_of(16).2);
        assert!(counters_of(64).2 <= 3 * counters_of(32).2);
    }
}
