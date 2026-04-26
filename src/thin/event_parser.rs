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

use crate::error::{Error, Result};

use super::event::{Event, EventStream};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn parse_events<'a>(text: &'a str, bump: &'a Bump) -> Result<EventStream<'a>> {
    // Heuristic capacity: ~1 event per ~10 source bytes. Empirically
    // close to right for typical config shapes; the BumpVec just bumps
    // in the arena if we underestimate.
    let mut events: EventStream<'a> = BumpVec::with_capacity_in(text.len() / 8 + 16, bump);
    events.push(Event::BeginObject);

    let mut p = EventParser {
        bump,
        stack: Vec::with_capacity(8),
        collecting: None,
    };
    p.stack.push(Frame::new_object(bump));

    for (idx, line) in text.lines().enumerate() {
        p.handle_line(line, idx + 1, &mut events)?;
    }

    p.finish(&mut events)?;
    Ok(events)
}

// ---------------------------------------------------------------------------
// Parser state
// ---------------------------------------------------------------------------

struct EventParser<'a> {
    bump: &'a Bump,
    stack: Vec<Frame<'a>>,
    collecting: Option<Collecting<'a>>,
}

enum Frame<'a> {
    /// `levels` is parallel to "real frame + open synthetic prefixes".
    /// Index 0 is always the real object's namespace; subsequent entries
    /// are stacked synthetics with their own prefix and key sets. All
    /// vectors live in the bump arena — no per-frame heap allocation.
    Object {
        levels: BumpVec<'a, ObjectLevel<'a>>,
    },
    Array,
}

struct ObjectLevel<'a> {
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
    fn new_object(bump: &'a Bump) -> Self {
        let mut levels = BumpVec::with_capacity_in(2, bump);
        levels.push(ObjectLevel {
            prefix: None,
            leaf_keys: BumpVec::with_capacity_in(8, bump),
            synthetic_keys: BumpVec::new_in(bump),
        });
        Frame::Object { levels }
    }
    fn new_array() -> Self {
        Frame::Array
    }
}

#[derive(Copy, Clone)]
enum MultilineMode {
    Stripped,
    Verbatim,
}

struct Collecting<'a> {
    mode: MultilineMode,
    lines: BumpVec<'a, &'a str>,
}

// ---------------------------------------------------------------------------
// Line dispatch
// ---------------------------------------------------------------------------

impl<'a> EventParser<'a> {
    fn finish(&mut self, events: &mut EventStream<'a>) -> Result<()> {
        if self.collecting.is_some() {
            return Err(Error::Syntax(
                "Unclosed multi-line string at end of input".to_string(),
            ));
        }
        if self.stack.len() > 1 {
            let kind = match self.stack.last().unwrap() {
                Frame::Object { .. } => "object",
                Frame::Array => "array",
            };
            return Err(Error::Syntax(format!("Unclosed {} at end of input", kind)));
        }
        // Close all synthetics still open in the root frame, then the
        // root object itself.
        self.close_synthetics_until(0, events);
        events.push(Event::EndObject);
        Ok(())
    }

    fn handle_line(
        &mut self,
        raw: &'a str,
        line_num: usize,
        events: &mut EventStream<'a>,
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

        if trimmed == "}" {
            return self.close_frame(BracketKind::Object, line_num, events);
        }
        if trimmed == "]" {
            return self.close_frame(BracketKind::Array, line_num, events);
        }

        if matches!(self.stack.last(), Some(Frame::Array)) {
            self.handle_array_item(trimmed, line_num, events)
        } else {
            self.handle_object_pair(trimmed, line_num, events)
        }
    }

    // -----------------------------------------------------------------------
    // Object-pair dispatch (mirrors thin/parser.rs but emits events)
    // -----------------------------------------------------------------------

    fn handle_object_pair(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        let colon = trimmed.find(':').ok_or_else(|| {
            Error::Syntax(format!(
                "Line {}: no ':' — object entries must be 'key: value' pairs",
                line_num
            ))
        })?;

        let key = trimmed[..colon].trim_end();
        if key.is_empty() {
            return Err(Error::Syntax(format!("Empty key at line {}", line_num)));
        }

        let after_colon = &trimmed[colon + 1..];

        match classify_separator(after_colon) {
            Separator::Raw(rest) => {
                require_sep_end(rest, line_num)?;
                self.emit_keyed_scalar(key, Event::Str(rest.trim()), line_num, events)
            }
            Separator::TypedInteger(body) => {
                let normalized = validate_typed_integer(body, line_num, self.bump)?;
                self.emit_keyed_scalar(key, Event::Integer(normalized), line_num, events)
            }
            Separator::TypedFloat(body) => {
                let normalized = validate_typed_float(body, line_num, self.bump)?;
                self.emit_keyed_scalar(key, Event::Float(normalized), line_num, events)
            }
            Separator::Plain => {
                require_sep_end(after_colon, line_num)?;
                match classify(after_colon.trim_start(), line_num)? {
                    ValueStart::Scalar(s) => {
                        self.emit_keyed_scalar(key, scalar_to_event(s), line_num, events)
                    }
                    ValueStart::EmptyObject => self.emit_keyed_compound(
                        key,
                        Event::BeginObject,
                        Event::EndObject,
                        line_num,
                        events,
                    ),
                    ValueStart::EmptyArray => self.emit_keyed_compound(
                        key,
                        Event::BeginArray,
                        Event::EndArray,
                        line_num,
                        events,
                    ),
                    ValueStart::OpenObject => {
                        self.emit_keyed_open(key, Event::BeginObject, line_num, events)?;
                        self.stack.push(Frame::new_object(self.bump));
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.emit_keyed_open(key, Event::BeginArray, line_num, events)?;
                        self.stack.push(Frame::new_array());
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        self.emit_keyed_open_multiline(
                            key,
                            MultilineMode::Stripped,
                            line_num,
                            events,
                        )
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        self.emit_keyed_open_multiline(
                            key,
                            MultilineMode::Verbatim,
                            line_num,
                            events,
                        )
                    }
                }
            }
        }
    }

    // Emits Key(leaf) + value-event after reconciling synthetic stack.
    fn emit_keyed_scalar(
        &mut self,
        key: &'a str,
        value: Event<'a>,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, events)?;
        self.register_leaf_key(leaf, line_num)?;
        events.push(Event::Key(leaf));
        events.push(value);
        Ok(())
    }

    // For empty inline compound `{}` / `[]`: emit Key + open + close.
    fn emit_keyed_compound(
        &mut self,
        key: &'a str,
        open: Event<'a>,
        close: Event<'a>,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, events)?;
        self.register_leaf_key(leaf, line_num)?;
        events.push(Event::Key(leaf));
        events.push(open);
        events.push(close);
        Ok(())
    }

    // For "key: {" or "key: [" — emit Key + open, push frame.
    fn emit_keyed_open(
        &mut self,
        key: &'a str,
        open: Event<'a>,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, events)?;
        self.register_leaf_key(leaf, line_num)?;
        events.push(Event::Key(leaf));
        events.push(open);
        Ok(())
    }

    fn emit_keyed_open_multiline(
        &mut self,
        key: &'a str,
        mode: MultilineMode,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        let leaf = self.reconcile_dotted_key(key, line_num, events)?;
        self.register_leaf_key(leaf, line_num)?;
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

    fn handle_array_item(
        &mut self,
        trimmed: &'a str,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        if let Some(rest) = trimmed.strip_prefix("::") {
            require_sep_end(rest, line_num)?;
            events.push(Event::Str(rest.trim_start()));
            return Ok(());
        }
        if let Some(rest) = trimmed.strip_prefix(":i") {
            require_sep_end(rest, line_num)?;
            let normalized = validate_typed_integer(rest, line_num, self.bump)?;
            events.push(Event::Integer(normalized));
            return Ok(());
        }
        if let Some(rest) = trimmed.strip_prefix(":f") {
            require_sep_end(rest, line_num)?;
            let normalized = validate_typed_float(rest, line_num, self.bump)?;
            events.push(Event::Float(normalized));
            return Ok(());
        }

        match classify(trimmed, line_num)? {
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
            }
            ValueStart::OpenArray => {
                events.push(Event::BeginArray);
                self.stack.push(Frame::new_array());
            }
            ValueStart::OpenMultilineStripped => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Stripped,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
            }
            ValueStart::OpenMultilineVerbatim => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Verbatim,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
            }
        }
        Ok(())
    }

    // Multi-line / compound-child completion path: scalar attached as
    // the value for whatever keyed-or-array context we're in. Because
    // for keyed contexts the `Key(...)` was already emitted before the
    // multiline began, we just push the scalar event here.
    fn attach_scalar(
        &mut self,
        value: Event<'a>,
        _line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        events.push(value);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Dotted-key reconciliation
    // -----------------------------------------------------------------------

    fn reconcile_dotted_key(
        &mut self,
        key: &'a str,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<&'a str> {
        if !key.as_bytes().contains(&b'.') {
            // Flat key: close any open synthetics in the current real
            // frame, then the leaf is just `key`.
            self.close_synthetics_to_real(events);
            if !is_valid_key(key) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, key
                )));
            }
            return Ok(key);
        }

        // Split off the leaf (the segment after the last dot). Don't
        // pre-collect the prefix segments — walk them lazily, comparing
        // against the current synthetic stack as we go (LCP fold). That
        // saves the heap-allocated `Vec<&str>` per dotted line.
        let (prefix_str, leaf) = key.rsplit_once('.').unwrap();
        if leaf.is_empty() || !is_valid_key(leaf) {
            return Err(Error::Syntax(format!(
                "Invalid key at line {}: '{}'",
                line_num, key
            )));
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
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, key
                )));
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
            self.push_synthetic(seg, line_num, events)?;
        }
        for seg in new_iter {
            if !is_valid_key(seg) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, key
                )));
            }
            self.push_synthetic(seg, line_num, events)?;
        }

        Ok(leaf)
    }

    #[inline]
    fn push_synthetic(
        &mut self,
        seg: &'a str,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        self.register_synthetic_prefix(seg, line_num)?;
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
    fn register_synthetic_prefix(&mut self, seg: &'a str, line_num: usize) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                let level = levels.last_mut().unwrap();
                if level.leaf_keys.iter().any(|k| *k == seg) {
                    return Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — an existing value blocks the path",
                        line_num, seg
                    )));
                }
                if level.synthetic_keys.iter().any(|k| *k == seg) {
                    return Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — synthetic dotted-key prefix already closed by an intervening different prefix; group lines with the same prefix together",
                        line_num, seg
                    )));
                }
                level.synthetic_keys.push(seg);
                Ok(())
            }
            _ => unreachable!("only objects have keys"),
        }
    }

    fn close_synthetics_to_real(&mut self, events: &mut EventStream<'a>) {
        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => return,
        };
        let pops = cur_levels_len - 1;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }
    }

    fn close_synthetics_until(&mut self, target_synthetic_count: usize, events: &mut EventStream<'a>) {
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

    fn pop_synthetic_level(&mut self, events: &mut EventStream<'a>) {
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
    fn register_leaf_key(&mut self, leaf: &'a str, line_num: usize) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                let level = levels.last_mut().unwrap();
                if level.synthetic_keys.iter().any(|k| *k == leaf) {
                    return Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — an existing value blocks the path",
                        line_num, leaf
                    )));
                }
                if level.leaf_keys.iter().any(|k| *k == leaf) {
                    return Err(Error::Syntax(format!(
                        "Line {}: duplicate key '{}'",
                        line_num, leaf
                    )));
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

    fn close_frame(
        &mut self,
        expected: BracketKind,
        line_num: usize,
        events: &mut EventStream<'a>,
    ) -> Result<()> {
        if self.stack.len() <= 1 {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' without matching '{}'",
                line_num,
                expected.close(),
                expected.open()
            )));
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
        if got as u8 != expected as u8 {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' does not match the open '{}'",
                line_num,
                expected.close(),
                got.open()
            )));
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
    fn open(self) -> char {
        match self {
            BracketKind::Object => '{',
            BracketKind::Array => '[',
        }
    }
    fn close(self) -> char {
        match self {
            BracketKind::Object => '}',
            BracketKind::Array => ']',
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
fn require_sep_end(rest: &str, line_num: usize) -> Result<()> {
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Ok(())
    } else {
        Err(Error::Syntax(format!(
            "Line {}: MissingSeparatorSpace: separator must be followed by whitespace or end of line",
            line_num,
        )))
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

fn validate_typed_integer<'a>(body: &'a str, line_num: usize, bump: &'a Bump) -> Result<&'a str> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(line_num, "integer body is empty"));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            "typed marker `:i` cannot open a compound or multi-line value",
        ));
    }
    if !is_integer_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            &format!("'{}' is not a valid integer literal for `:i`", trimmed),
        ));
    }
    Ok(strip_leading_plus(trimmed, bump))
}

fn validate_typed_float<'a>(body: &'a str, line_num: usize, bump: &'a Bump) -> Result<&'a str> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(line_num, "float body is empty"));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            "typed marker `:f` cannot open a compound or multi-line value",
        ));
    }
    if !is_float_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            &format!("'{}' is not a valid float literal for `:f`", trimmed),
        ));
    }
    Ok(strip_leading_plus(trimmed, bump))
}

fn invalid_typed_scalar(line_num: usize, detail: &str) -> Error {
    Error::Syntax(format!("Line {}: InvalidTypedScalar: {}", line_num, detail))
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

fn classify<'a>(trimmed: &'a str, line_num: usize) -> Result<ValueStart<'a>> {
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
        return Err(Error::Syntax(format!(
            "Line {}: inline non-empty object is not supported; put entries on separate lines",
            line_num
        )));
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        return Err(Error::Syntax(format!(
            "Line {}: inline non-empty array is not supported; put items on separate lines",
            line_num
        )));
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
