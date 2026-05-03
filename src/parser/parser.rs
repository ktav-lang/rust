//! The parser state machine: a stack of [`Frame`]s plus a line dispatcher.

use crate::error::{CompoundKind, Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::bracket::Bracket;
use super::classify::{classify_value_start, validate_typed_float, validate_typed_integer};
use super::collecting::{Collecting, MultilineMode};
use super::frame::Frame;
use super::insert::insert_value;
use super::value_start::ValueStart;

pub(super) struct Parser<'a> {
    stack: Vec<Frame<'a>>,
    collecting: Option<Collecting<'a>>,
    /// Byte offset of the unclosed-compound's opener inside the
    /// original input, parallel to `stack`. Index `0` is unused (the
    /// implicit root object has no opener); each pushed child frame
    /// records its opener offset here so that an EOF-detected
    /// `UnclosedCompound` can produce a `Span` covering opener..EOF.
    opener_offsets: Vec<u32>,
    /// Byte offset of the multi-line opener (the line that began with
    /// `(` / `((`). Used by the EOF-detected unclosed-multiline span.
    multiline_opener: Option<u32>,
}

impl<'a> Parser<'a> {
    pub(super) fn new() -> Self {
        let mut stack = Vec::with_capacity(8);
        stack.push(Frame::new_object());
        let mut opener_offsets = Vec::with_capacity(8);
        opener_offsets.push(0);
        Self {
            stack,
            collecting: None,
            opener_offsets,
            multiline_opener: None,
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

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        // Compute the trimmed-line span (start at the first non-ws byte
        // in `raw`, end at the last non-ws byte). This is reused for
        // a few categories that span the whole logical line.
        let trimmed_span = trimmed_span_in(raw, trimmed, line_start);

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
            Frame::Object { pairs, pending_key } => {
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
                insert_value(
                    pairs,
                    key,
                    value,
                    line_num,
                    Span::new(line_start, line_start),
                )
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

        let colon = match line.find(':') {
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

        // Separator analysis. The byte immediately after the first `:` may
        // be `:` (raw marker), `i` / `f` followed by space or EOL (typed
        // marker), or whitespace / EOL (ordinary pair). Anything else is
        // the classic ordinary-pair case — the marker-looking prefix is
        // simply the start of the value.
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
            Separator::TypedInteger(body) => {
                // `body` is already guaranteed to be empty or ws-started by
                // `classify_separator`; no extra sep-end check here.
                let body_span = body_span_for(body, after_colon, after_colon_off, trimmed_span);
                let s = validate_typed_integer(body, line_num, body_span)?;
                self.insert_object_pair(
                    key,
                    Value::Integer(s),
                    line_num,
                    Span::new(key_start, key_end),
                )
            }
            Separator::TypedFloat(body) => {
                let body_span = body_span_for(body, after_colon, after_colon_off, trimmed_span);
                let s = validate_typed_float(body, line_num, body_span)?;
                self.insert_object_pair(
                    key,
                    Value::Float(s),
                    line_num,
                    Span::new(key_start, key_end),
                )
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
                        self.set_pending_key(key, line_num, line_start)?;
                        self.stack.push(Frame::new_object());
                        // The opener is the trailing '{' on the line.
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.set_pending_key(key, line_num, line_start)?;
                        self.stack.push(Frame::new_array());
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        self.set_pending_key(key, line_num, line_start)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                        self.multiline_opener = Some(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        self.set_pending_key(key, line_num, line_start)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                        self.multiline_opener = Some(trimmed_span.end - 2);
                        Ok(())
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
        // Check typed-scalar prefixes before the general raw-string prefix.
        // Order matters: `::` before `:i`/`:f`/`:` — the `::` has two
        // colons, the others have one + letter / whitespace / EOL.
        //
        // Per spec § 5.4, every marker demands sep-end (whitespace or EOL);
        // a glued form like `::value` / `:i42` / `:f0.5` is a
        // MissingSeparatorSpace error (§ 6.10), not a String item.
        let line_start = trimmed_span.start; // for arrays the "trimmed line start"

        if let Some(rest) = line.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start, 1, line_start + 2, trimmed_span)?;
            let value = Value::String(rest.trim_start().into());
            return self.push_array_item(value);
        }

        if let Some(rest) = line.strip_prefix(":i") {
            require_sep_end(rest, line_num, line_start, 0, line_start + 2, trimmed_span)?;
            let body_span = Span::new(line_start + 2, trimmed_span.end);
            let s = validate_typed_integer(rest, line_num, body_span)?;
            return self.push_array_item(Value::Integer(s));
        }
        if let Some(rest) = line.strip_prefix(":f") {
            require_sep_end(rest, line_num, line_start, 0, line_start + 2, trimmed_span)?;
            let body_span = Span::new(line_start + 2, trimmed_span.end);
            let s = validate_typed_float(rest, line_num, body_span)?;
            return self.push_array_item(Value::Float(s));
        }

        match classify_value_start(line, line_num, trimmed_span)? {
            ValueStart::Scalar(s) => self.push_array_item(Value::String(s)),
            ValueStart::Null => self.push_array_item(Value::Null),
            ValueStart::Bool(b) => self.push_array_item(Value::Bool(b)),
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
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pending_key, .. } => {
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
            Frame::Object { pairs, pending_key } => {
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
                insert_value(pairs, key, value, line_num, trimmed_span)
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

/// Span of the body following a typed-scalar marker.
fn body_span_for(body: &str, after_colon: &str, after_colon_off: u32, fallback: Span) -> Span {
    if body.is_empty() {
        // Body is empty — point at the byte just past the marker.
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

// ---------------------------------------------------------------------------
// Separator classification for pair-lines.
//
// After the first `:` of `key: value`, this slice can begin with:
//   - `:` + whitespace/EOL   → raw-string marker `::`
//   - `i` + whitespace/EOL   → typed integer `:i`
//   - `f` + whitespace/EOL   → typed float   `:f`
//   - anything else          → plain `:` separator; the rest is the body
//
// The "typed" variants require that whatever follows the letter be either
// whitespace or end of line — so `:info: ...` stays a plain-`:` pair whose
// value begins with `info: ...`.
// ---------------------------------------------------------------------------

enum Separator<'a> {
    /// `::` followed by the body (leading whitespace not yet trimmed).
    Raw(&'a str),
    /// `:i` followed by the body (starting with the whitespace separator
    /// or empty → will be rejected downstream).
    TypedInteger(&'a str),
    /// `:f` followed by the body.
    TypedFloat(&'a str),
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

fn classify_separator(after_colon: &str) -> Separator<'_> {
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
    Separator::Plain(after_colon)
}
