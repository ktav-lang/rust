//! The parser state machine: a stack of [`Frame`]s plus a line dispatcher.

use crate::error::{CompoundKind, Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::bracket::Bracket;
use super::classify::classify_value_start;
use super::collecting::{Collecting, MultilineMode};
use super::frame::Frame;
use super::inline;
use super::inline::find_unescaped_colon;
use super::insert::insert_value;
use super::value_start::ValueStart;

pub(super) struct Parser<'a> {
    stack: Vec<Frame<'a>>,
    collecting: Option<Collecting<'a>>,
    /// Byte offset of the unclosed-compound's opener inside the
    /// original input, parallel to `stack`. Index `0` is unused (the
    /// implicit root object/array has no opener); each pushed child
    /// frame records its opener offset here so that an EOF-detected
    /// `UnclosedCompound` can produce a `Span` covering opener..EOF.
    opener_offsets: Vec<u32>,
    /// Byte offset of the multi-line opener (the line that began with
    /// `(` / `((`). Used by the EOF-detected unclosed-multiline span.
    multiline_opener: Option<u32>,
    /// `false` until the first content line (non-blank, non-comment)
    /// has been classified and the root frame pushed. Spec § 5.0.1
    /// (added in 0.1.1) determines the root kind from the first
    /// content line: pair-shape → Object, array-item-shape → Array.
    root_initialized: bool,
    /// Set to `true` after a top-level inline compound (§ 5.0.1 rules 2-3)
    /// or after the matching close of a top-level multi-line compound
    /// (§ 5.0.1 rules 4-5) has been fully consumed. Any subsequent
    /// non-blank, non-comment line is `OrphanLineAfterTopLevelInline`.
    root_consumed: bool,
    /// When the root is a top-level inline compound (§ 5.0.1 rules 2-3),
    /// this stores the parsed Value so `finish()` can return it.
    root_inline_value: Option<Value>,
    /// True when the root was opened by a lone `{` or `[` (§ 5.0.1
    /// rules 4-5). When the matching close is hit and the stack goes
    /// back to depth 1, `root_consumed` is set.
    root_is_explicit_compound: bool,
}

impl<'a> Parser<'a> {
    pub(super) fn new() -> Self {
        // Defer root frame construction until the first content line
        // is classified. `finish` falls back to an empty Object root
        // if no content line was ever encountered (preserves 0.3.0
        // behaviour on empty / comments-only docs).
        Self {
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
            root_initialized: false,
            root_consumed: false,
            root_inline_value: None,
            root_is_explicit_compound: false,
        }
    }

    pub(super) fn finish(mut self, eof_offset: u32) -> Result<Value, Error> {
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
                Frame::Array { .. } => CompoundKind::Array,
            };
            let start = *self.opener_offsets.last().unwrap();
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind,
                span: Span::new(start, eof_offset),
            }));
        }
        // If root was a top-level inline compound, return the stored value.
        if let Some(v) = self.root_inline_value {
            return Ok(v);
        }
        // Empty / comments-only document — root was never initialized;
        // fall back to an empty Object (spec § 5.0.1 rule 1).
        if self.stack.is_empty() {
            return Ok(Value::Object(crate::value::ObjectMap::default()));
        }
        Ok(self.stack.pop().unwrap().into_value())
    }

    pub(super) fn handle_line(
        &mut self,
        raw: &'a str,
        line_num: usize,
        line_start: u32,
    ) -> Result<(), Error> {
        // Inside a multi-line string the line is raw content unless it is
        // the terminator — comments/brackets are NOT special here.
        if let Some(ref mut collecting) = self.collecting {
            let trimmed = raw.trim();
            if collecting.is_terminator(trimmed) {
                let finished = self.collecting.take().unwrap().finish();
                self.multiline_opener = None;
                return self.attach_scalar_value(
                    Value::String(finished.into()),
                    line_num,
                    line_start,
                );
            }
            collecting.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with("##") {
            return Ok(());
        }

        // Compute the trimmed-line span (start at the first non-ws byte
        // in `raw`, end at the last non-ws byte). This is reused for
        // a few categories that span the whole logical line.
        let trimmed_span = trimmed_span_in(raw, trimmed, line_start);

        // § 5.0.1 — if root is already consumed (inline compound or
        // explicit-compound closed), any further content line is an
        // orphan.
        if self.root_consumed {
            return Err(Error::Structured(
                ErrorKind::OrphanLineAfterTopLevelInline {
                    line: line_num as u32,
                    span: trimmed_span,
                },
            ));
        }

        // Spec § 5.0.1 — first content line establishes the root kind.
        if !self.root_initialized {
            self.root_initialized = true;

            // `}` / `]` first content line — not a valid root kind;
            // fall through to the close-frame branch which will raise
            // UnbalancedBracket against the empty stack.
            if trimmed != "}" && trimmed != "]" {
                match classify_root_kind_050(trimmed, line_num, trimmed_span)? {
                    RootResult::InlineObject(value) => {
                        self.root_consumed = true;
                        self.root_inline_value = Some(value);
                        return Ok(());
                    }
                    RootResult::InlineArray(value) => {
                        self.root_consumed = true;
                        self.root_inline_value = Some(value);
                        return Ok(());
                    }
                    RootResult::ExplicitObject => {
                        self.root_is_explicit_compound = true;
                        self.stack.push(Frame::new_object());
                        self.opener_offsets.push(trimmed_span.start);
                        return Ok(());
                    }
                    RootResult::ExplicitArray => {
                        self.root_is_explicit_compound = true;
                        self.stack.push(Frame::new_array());
                        self.opener_offsets.push(trimmed_span.start);
                        return Ok(());
                    }
                    RootResult::Object => {
                        self.stack.push(Frame::new_object());
                        self.opener_offsets.push(0);
                    }
                    RootResult::Array => {
                        self.stack.push(Frame::new_array());
                        self.opener_offsets.push(0);
                    }
                }
            }
        }

        if trimmed == "}" {
            return self.close_frame(Bracket::Object, line_num, trimmed_span);
        }
        if trimmed == "]" {
            return self.close_frame(Bracket::Array, line_num, trimmed_span);
        }

        if matches!(self.stack.last(), Some(Frame::Array { .. })) {
            self.handle_array_item(trimmed, line_num, trimmed_span)
        } else {
            self.handle_object_pair(raw, trimmed, line_num, line_start, trimmed_span)
        }
    }

    /// Routes a newly-computed scalar value into the current frame:
    /// - inside an Object with a `pending_key`: insert at that key.
    /// - inside an Array: push as item.
    fn attach_scalar_value(
        &mut self,
        value: Value,
        line_num: usize,
        line_start: u32,
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object {
                pairs,
                pending_key,
                pending_key_span,
            } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} multi-line string closed without pending key",
                            line_num
                        ),
                        span: Span::new(line_start, line_start),
                    })
                })?;
                // Use the saved key span (set when the pair was opened)
                // so a duplicate-key / conflict error highlights the
                // offending key, not the location of its closing token.
                let key_span = pending_key_span
                    .take()
                    .unwrap_or_else(|| Span::new(line_start, line_start));
                insert_value(pairs, key, value, line_num, key_span)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
    }

    fn handle_object_pair(
        &mut self,
        raw: &'a str,
        line: &'a str,
        line_num: usize,
        line_start: u32,
        trimmed_span: Span,
    ) -> Result<(), Error> {
        // Byte offset (within `raw`) of where the trimmed line begins.
        let trimmed_off_in_raw = (trimmed_span.start - line_start) as usize;

        // Spec 0.6.0 § 5.3: the pair separator is the first UNescaped `:`.
        let colon = match find_unescaped_colon(line) {
            Some(c) => c,
            None => {
                return Err(Error::Structured(ErrorKind::MissingSeparator {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
        };

        // `line` is already trim'ed by `handle_line`; only trailing
        // whitespace between the key and `:` is possible here.
        let key = line[..colon].trim_end();
        let key_start = trimmed_span.start; // first non-ws byte of trimmed line
        let key_end = key_start + key.len() as u32;
        let _ = raw; // raw kept for parity with future refactors
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                // The colon is the offending byte for `: value`.
                span: Span::new(key_start, key_start + 1),
            }));
        }
        // Per-segment validation is folded into `insert_value`; it splits
        // the path anyway while descending, so a pre-pass here would scan
        // the key twice.

        // Separator analysis. Under 0.5.0 the byte immediately after the
        // first `:` may be `:` (raw marker `::`) or whitespace / EOL
        // (ordinary pair). The old `:i`/`:f` typed markers are removed.
        let after_colon = &line[colon + 1..];
        let after_colon_off_in_line = colon + 1;
        let after_colon_off = line_start + (trimmed_off_in_raw + after_colon_off_in_line) as u32;
        let sep = classify_separator(after_colon);

        // Column (within trimmed line) of the marker for MissingSep diagnostics.
        let marker_col = colon as u32; // 0-based within trimmed line

        match sep {
            Separator::Raw(after) => {
                require_sep_end(
                    after,
                    line_num,
                    line_start,
                    marker_col,
                    after_colon_off + 1,
                    trimmed_span,
                )?;
                let value = Value::String(after.trim().into());
                self.insert_object_pair(key, value, line_num, Span::new(key_start, key_end))
            }
            Separator::Plain(after) => {
                require_sep_end(
                    after,
                    line_num,
                    line_start,
                    marker_col,
                    after_colon_off,
                    trimmed_span,
                )?;
                let key_span = Span::new(key_start, key_end);
                match classify_value_start(after, line_num, trimmed_span)? {
                    ValueStart::Scalar(s) => {
                        self.insert_object_pair(key, Value::String(s), line_num, key_span)
                    }
                    ValueStart::Null => {
                        self.insert_object_pair(key, Value::Null, line_num, key_span)
                    }
                    ValueStart::Bool(b) => {
                        self.insert_object_pair(key, Value::Bool(b), line_num, key_span)
                    }
                    ValueStart::Integer(s) => {
                        self.insert_object_pair(key, Value::Integer(s), line_num, key_span)
                    }
                    ValueStart::Float(s) => {
                        self.insert_object_pair(key, Value::Float(s), line_num, key_span)
                    }
                    ValueStart::EmptyObject => self.insert_object_pair(
                        key,
                        Value::Object(ObjectMap::default()),
                        line_num,
                        key_span,
                    ),
                    ValueStart::EmptyArray => {
                        self.insert_object_pair(key, Value::Array(Vec::new()), line_num, key_span)
                    }
                    ValueStart::OpenObject => {
                        self.set_pending_key(key, line_num, line_start, key_span)?;
                        self.stack.push(Frame::new_object());
                        // The opener is the trailing '{' on the line.
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.set_pending_key(key, line_num, line_start, key_span)?;
                        self.stack.push(Frame::new_array());
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        self.set_pending_key(key, line_num, line_start, key_span)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                        self.multiline_opener = Some(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        self.set_pending_key(key, line_num, line_start, key_span)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                        self.multiline_opener = Some(trimmed_span.end - 2);
                        Ok(())
                    }
                    ValueStart::InlineValue(v) => {
                        self.insert_object_pair(key, v, line_num, key_span)
                    }
                }
            }
        }
    }

    fn handle_array_item(
        &mut self,
        line: &str,
        line_num: usize,
        trimmed_span: Span,
    ) -> Result<(), Error> {
        // Under 0.5.0, the only array-item prefix marker is `::` (raw
        // string). The old `:i`/`:f` typed markers are removed.
        //
        // Per spec § 5.4, the `::` marker demands sep-end (whitespace or
        // EOL); a glued form like `::value` is a MissingSeparatorSpace
        // error (§ 6.10), not a String item.
        let line_start = trimmed_span.start; // for arrays the "trimmed line start"

        if let Some(rest) = line.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start, 1, line_start + 2, trimmed_span)?;
            let value = Value::String(rest.trim_start().into());
            return self.push_array_item(value);
        }

        match classify_value_start(line, line_num, trimmed_span)? {
            ValueStart::Scalar(s) => self.push_array_item(Value::String(s)),
            ValueStart::Null => self.push_array_item(Value::Null),
            ValueStart::Bool(b) => self.push_array_item(Value::Bool(b)),
            ValueStart::Integer(s) => self.push_array_item(Value::Integer(s)),
            ValueStart::Float(s) => self.push_array_item(Value::Float(s)),
            ValueStart::EmptyObject => self.push_array_item(Value::Object(ObjectMap::default())),
            ValueStart::EmptyArray => self.push_array_item(Value::Array(Vec::new())),
            ValueStart::OpenObject => {
                self.stack.push(Frame::new_object());
                self.opener_offsets.push(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenArray => {
                self.stack.push(Frame::new_array());
                self.opener_offsets.push(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenMultilineStripped => {
                self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                self.multiline_opener = Some(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenMultilineVerbatim => {
                self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                self.multiline_opener = Some(trimmed_span.end - 2);
                Ok(())
            }
            ValueStart::InlineValue(v) => self.push_array_item(v),
        }
    }

    fn insert_object_pair(
        &mut self,
        key: &str,
        value: Value,
        line_num: usize,
        key_span: Span,
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, .. } => insert_value(pairs, key, value, line_num, key_span),
            Frame::Array { .. } => unreachable!("dispatched as object"),
        }
    }

    fn push_array_item(&mut self, value: Value) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
            Frame::Object { .. } => unreachable!("dispatched as array"),
        }
    }

    fn set_pending_key(
        &mut self,
        key: &'a str,
        line_num: usize,
        line_start: u32,
        key_span: Span,
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object {
                pending_key,
                pending_key_span,
                ..
            } => {
                if pending_key.is_some() {
                    return Err(Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} pending key already set",
                            line_num
                        ),
                        span: Span::new(line_start, line_start),
                    }));
                }
                *pending_key = Some(key);
                *pending_key_span = Some(key_span);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn close_frame(
        &mut self,
        expected: Bracket,
        line_num: usize,
        trimmed_span: Span,
    ) -> Result<(), Error> {
        if self.stack.len() <= 1 {
            // If the stack has exactly 1 frame and this is an explicit
            // compound root (§ 5.0.1 rules 4-5), closing it is valid.
            if self.stack.len() == 1 && self.root_is_explicit_compound {
                let frame = self.stack.last().unwrap();
                let frame_kind = match frame {
                    Frame::Object { .. } => Bracket::Object,
                    Frame::Array { .. } => Bracket::Array,
                };
                if !matches!(
                    (frame_kind, expected),
                    (Bracket::Object, Bracket::Object) | (Bracket::Array, Bracket::Array)
                ) {
                    return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                        line: line_num as u32,
                        span: trimmed_span,
                        expected: bracket_to_compound(frame_kind),
                        found: expected.close(),
                    }));
                }
                // Mark root as consumed — subsequent content lines are orphans.
                self.root_consumed = true;
                return Ok(());
            }
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: bracket_to_compound(expected),
                found: expected.close(),
            }));
        }
        let frame = self.stack.pop().unwrap();
        let _ = self.opener_offsets.pop();
        let frame_kind = match frame {
            Frame::Object { .. } => Bracket::Object,
            Frame::Array { .. } => Bracket::Array,
        };
        let matches_expected = matches!(
            (frame_kind, expected),
            (Bracket::Object, Bracket::Object) | (Bracket::Array, Bracket::Array)
        );
        if !matches_expected {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: bracket_to_compound(frame_kind),
                found: expected.close(),
            }));
        }

        let value = frame.into_value();
        self.attach_child_value(value, line_num, trimmed_span)
    }

    fn attach_child_value(
        &mut self,
        value: Value,
        line_num: usize,
        trimmed_span: Span,
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object {
                pairs,
                pending_key,
                pending_key_span,
            } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} closed compound without pending key",
                            line_num
                        ),
                        span: trimmed_span,
                    })
                })?;
                // Prefer the key's own span (saved when the pair was
                // opened) so duplicate-key errors point at the key
                // instead of the closing brace.
                let key_span = pending_key_span.take().unwrap_or(trimmed_span);
                insert_value(pairs, key, value, line_num, key_span)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
    }
}

fn bracket_to_compound(b: Bracket) -> CompoundKind {
    match b {
        Bracket::Object => CompoundKind::Object,
        Bracket::Array => CompoundKind::Array,
    }
}

/// Compute a span for the trimmed line content given the raw line and
/// its start offset. `raw` may have leading/trailing whitespace; `trimmed`
/// is its `.trim()` view.
fn trimmed_span_in(raw: &str, trimmed: &str, line_start: u32) -> Span {
    if trimmed.is_empty() {
        return Span::new(line_start, line_start);
    }
    // Locate `trimmed` inside `raw` via pointer arithmetic (both share
    // the same backing buffer).
    let raw_ptr = raw.as_ptr() as usize;
    let trim_ptr = trimmed.as_ptr() as usize;
    debug_assert!(trim_ptr >= raw_ptr && trim_ptr - raw_ptr <= raw.len());
    let off = (trim_ptr - raw_ptr) as u32;
    let start = line_start + off;
    Span::new(start, start + trimmed.len() as u32)
}

// ---------------------------------------------------------------------------
// Separator classification for pair-lines.
//
// Under 0.5.0, after the first `:` of `key: value`, the slice can begin
// with:
//   - `:` + whitespace/EOL   → raw-string marker `::`
//   - anything else          → plain `:` separator; the rest is the body
//
// The old `:i`/`:f` typed markers are removed in 0.5.0.
// ---------------------------------------------------------------------------

enum Separator<'a> {
    /// `::` followed by the body (leading whitespace not yet trimmed).
    Raw(&'a str),
    /// Plain `:` — body already lacks the leading separator char.
    Plain(&'a str),
}

/// Enforce the "separator followed by whitespace or end-of-line" rule
/// from spec § 5.3 / § 5.4. Returns `Err(MissingSeparatorSpace)` for the
/// `key:value` / `key::value` / `port:i42` / `ratio:f0.5` shapes where
/// the body is glued to the separator.
fn require_sep_end(
    rest: &str,
    line_num: usize,
    line_start: u32,
    column: u32,
    body_off: u32,
    trimmed_span: Span,
) -> Result<(), Error> {
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Ok(())
    } else {
        // Compute span: from the marker through the glued body chars
        // (i.e. up to the trimmed-line end).
        let _ = line_start;
        let span = Span::new(body_off, trimmed_span.end);
        Err(Error::Structured(ErrorKind::MissingSeparatorSpace {
            line: line_num as u32,
            column,
            marker: ':',
            span,
        }))
    }
}

/// Result of top-level kind detection (§ 5.0.1).
enum RootResult {
    /// § 5.0.1 rule 2: first content line is a closed inline object.
    InlineObject(Value),
    /// § 5.0.1 rule 3: first content line is a closed inline array.
    InlineArray(Value),
    /// § 5.0.1 rule 4: first content line is a lone `{`.
    ExplicitObject,
    /// § 5.0.1 rule 5: first content line is a lone `[`.
    ExplicitArray,
    /// § 5.0.1 rule 6: pair-shape → implicit Object root.
    Object,
    /// § 5.0.1 rule 7: array-item → implicit Array root.
    Array,
}

/// Top-level kind detection (spec § 5.0.1, 0.5.0).
///
/// Applies all 8 rules in order. Rules 1 (no content lines) and 8
/// (bare `}` / `]`) are handled by the caller.
fn classify_root_kind_050(
    trimmed: &str,
    line_num: usize,
    trimmed_span: Span,
) -> Result<RootResult, Error> {
    // Rule 4: lone `{`
    if trimmed == "{" {
        return Ok(RootResult::ExplicitObject);
    }
    // Rule 5: lone `[`
    if trimmed == "[" {
        return Ok(RootResult::ExplicitArray);
    }

    // Rule 2: closed inline object `{ ... }` — ends with `}`
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        // Could be empty `{}` / `{ }`
        if trimmed[1..trimmed.len() - 1].trim().is_empty() {
            let value = Value::Object(crate::value::ObjectMap::default());
            return Ok(RootResult::InlineObject(value));
        }
        let value = inline::parse_inline_object(trimmed, line_num, trimmed_span)?;
        return Ok(RootResult::InlineObject(value));
    }

    // Rule 3: closed inline array `[ ... ]` — ends with `]`
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if trimmed[1..trimmed.len() - 1].trim().is_empty() {
            let value = Value::Array(Vec::new());
            return Ok(RootResult::InlineArray(value));
        }
        let value = inline::parse_inline_array(trimmed, line_num, trimmed_span)?;
        return Ok(RootResult::InlineArray(value));
    }

    // Rule 8 for `{` / `[` that don't match 2-5: starts with brace
    // but not closed → unterminated inline compound.
    if trimmed.starts_with('{') {
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }
    if trimmed.starts_with('[') {
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }

    // Rule 6/7: pair-shape vs array-item. Use the same heuristic
    // as before.
    if is_pair_shape(trimmed) {
        Ok(RootResult::Object)
    } else {
        Ok(RootResult::Array)
    }
}

/// Check if the trimmed line looks like a pair shape (has a `:` with
/// a non-empty key before it and whitespace/EOL after it, or `::` raw
/// marker).
fn is_pair_shape(trimmed: &str) -> bool {
    // Spec 0.6.0 § 5.3 — pair separator is the first UNescaped `:`.
    let Some(colon_idx) = find_unescaped_colon(trimmed) else {
        return false;
    };
    // Empty prefix before `:` → array-item shape
    let key_part = trimmed[..colon_idx].trim_end();
    if key_part.is_empty() {
        return false;
    }
    let after = &trimmed[colon_idx + 1..];
    // `key::` (raw marker) → pair shape
    if after.starts_with(':') {
        return true;
    }
    // Plain `key: ` separator — body must start with whitespace or
    // be empty. Anything else (e.g. `http://...`) means the `:` is
    // part of a value and there's no real pair.
    after.is_empty() || after.starts_with([' ', '\t'])
}

fn classify_separator(after_colon: &str) -> Separator<'_> {
    if let Some(rest) = after_colon.strip_prefix(':') {
        return Separator::Raw(rest);
    }
    // Under spec 0.5.0, `:i` and `:f` typed markers are removed.
    // Everything that isn't `::` is a plain `:` separator.
    Separator::Plain(after_colon)
}
