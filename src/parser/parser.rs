//! The parser state machine: a stack of [`Frame`]s plus a line dispatcher.

use crate::error::Error;
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
}

impl<'a> Parser<'a> {
    pub(super) fn new() -> Self {
        let mut stack = Vec::with_capacity(8);
        stack.push(Frame::new_object());
        Self {
            stack,
            collecting: None,
        }
    }

    pub(super) fn finish(mut self) -> Result<Value, Error> {
        if self.collecting.is_some() {
            return Err(Error::Syntax(
                "Unclosed multi-line string at end of input".to_string(),
            ));
        }
        if self.stack.len() > 1 {
            let kind = match self.stack.last().unwrap() {
                Frame::Object { .. } => "object",
                Frame::Array { .. } => "array",
            };
            return Err(Error::Syntax(format!("Unclosed {} at end of input", kind)));
        }
        Ok(self.stack.pop().unwrap().into_value())
    }

    pub(super) fn handle_line(&mut self, raw: &'a str, line_num: usize) -> Result<(), Error> {
        // Inside a multi-line string the line is raw content unless it is
        // the terminator — comments/brackets are NOT special here.
        if let Some(ref mut collecting) = self.collecting {
            let trimmed = raw.trim();
            if collecting.is_terminator(trimmed) {
                let finished = self.collecting.take().unwrap().finish();
                return self.attach_scalar_value(Value::String(finished.into()), line_num);
            }
            collecting.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        if trimmed == "}" {
            return self.close_frame(Bracket::Object, line_num);
        }
        if trimmed == "]" {
            return self.close_frame(Bracket::Array, line_num);
        }

        if matches!(self.stack.last(), Some(Frame::Array { .. })) {
            self.handle_array_item(trimmed, line_num)
        } else {
            self.handle_object_pair(trimmed, line_num)
        }
    }

    /// Routes a newly-computed scalar value into the current frame:
    /// - inside an Object with a `pending_key`: insert at that key.
    /// - inside an Array: push as item.
    fn attach_scalar_value(&mut self, value: Value, line_num: usize) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, pending_key } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Syntax(format!(
                        "Line {}: internal error — multi-line string closed without pending key",
                        line_num
                    ))
                })?;
                insert_value(pairs, key, value, line_num)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
    }

    fn handle_object_pair(&mut self, line: &'a str, line_num: usize) -> Result<(), Error> {
        let colon = line.find(':').ok_or_else(|| {
            Error::Syntax(format!(
                "Line {}: no ':' — object entries must be 'key: value' pairs",
                line_num
            ))
        })?;

        // `line` is already trim'ed by `handle_line`; only trailing
        // whitespace between the key and `:` is possible here.
        let key = line[..colon].trim_end();
        if key.is_empty() {
            return Err(Error::Syntax(format!("Empty key at line {}", line_num)));
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
        let sep = classify_separator(after_colon);

        match sep {
            Separator::Raw(after) => {
                require_sep_end(after, line_num)?;
                let value = Value::String(after.trim().into());
                self.insert_object_pair(key, value, line_num)
            }
            Separator::TypedInteger(body) => {
                // `body` is already guaranteed to be empty or ws-started by
                // `classify_separator`; no extra sep-end check here.
                let s = validate_typed_integer(body, line_num)?;
                self.insert_object_pair(key, Value::Integer(s), line_num)
            }
            Separator::TypedFloat(body) => {
                let s = validate_typed_float(body, line_num)?;
                self.insert_object_pair(key, Value::Float(s), line_num)
            }
            Separator::Plain(after) => {
                require_sep_end(after, line_num)?;
                match classify_value_start(after, line_num)? {
                    ValueStart::Scalar(s) => {
                        self.insert_object_pair(key, Value::String(s), line_num)
                    }
                    ValueStart::Null => self.insert_object_pair(key, Value::Null, line_num),
                    ValueStart::Bool(b) => self.insert_object_pair(key, Value::Bool(b), line_num),
                    ValueStart::EmptyObject => {
                        self.insert_object_pair(key, Value::Object(ObjectMap::default()), line_num)
                    }
                    ValueStart::EmptyArray => {
                        self.insert_object_pair(key, Value::Array(Vec::new()), line_num)
                    }
                    ValueStart::OpenObject => {
                        self.set_pending_key(key, line_num)?;
                        self.stack.push(Frame::new_object());
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.set_pending_key(key, line_num)?;
                        self.stack.push(Frame::new_array());
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        self.set_pending_key(key, line_num)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                        Ok(())
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        self.set_pending_key(key, line_num)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                        Ok(())
                    }
                }
            }
        }
    }

    fn handle_array_item(&mut self, line: &str, line_num: usize) -> Result<(), Error> {
        // Check typed-scalar prefixes before the general raw-string prefix.
        // Order matters: `::` before `:i`/`:f`/`:` — the `::` has two
        // colons, the others have one + letter / whitespace / EOL.
        //
        // Per spec § 5.4, every marker demands sep-end (whitespace or EOL);
        // a glued form like `::value` / `:i42` / `:f0.5` is a
        // MissingSeparatorSpace error (§ 6.10), not a String item.
        if let Some(rest) = line.strip_prefix("::") {
            require_sep_end(rest, line_num)?;
            let value = Value::String(rest.trim_start().into());
            return self.push_array_item(value);
        }

        if let Some(rest) = line.strip_prefix(":i") {
            require_sep_end(rest, line_num)?;
            let s = validate_typed_integer(rest, line_num)?;
            return self.push_array_item(Value::Integer(s));
        }
        if let Some(rest) = line.strip_prefix(":f") {
            require_sep_end(rest, line_num)?;
            let s = validate_typed_float(rest, line_num)?;
            return self.push_array_item(Value::Float(s));
        }

        match classify_value_start(line, line_num)? {
            ValueStart::Scalar(s) => self.push_array_item(Value::String(s)),
            ValueStart::Null => self.push_array_item(Value::Null),
            ValueStart::Bool(b) => self.push_array_item(Value::Bool(b)),
            ValueStart::EmptyObject => self.push_array_item(Value::Object(ObjectMap::default())),
            ValueStart::EmptyArray => self.push_array_item(Value::Array(Vec::new())),
            ValueStart::OpenObject => {
                self.stack.push(Frame::new_object());
                Ok(())
            }
            ValueStart::OpenArray => {
                self.stack.push(Frame::new_array());
                Ok(())
            }
            ValueStart::OpenMultilineStripped => {
                self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                Ok(())
            }
            ValueStart::OpenMultilineVerbatim => {
                self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                Ok(())
            }
        }
    }

    fn insert_object_pair(
        &mut self,
        key: &str,
        value: Value,
        line_num: usize,
    ) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, .. } => insert_value(pairs, key, value, line_num),
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

    fn set_pending_key(&mut self, key: &'a str, line_num: usize) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pending_key, .. } => {
                if pending_key.is_some() {
                    return Err(Error::Syntax(format!(
                        "Line {}: internal error — pending key already set",
                        line_num
                    )));
                }
                *pending_key = Some(key);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn close_frame(&mut self, expected: Bracket, line_num: usize) -> Result<(), Error> {
        if self.stack.len() <= 1 {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' without matching '{}'",
                line_num,
                expected.close(),
                expected.open()
            )));
        }
        let frame = self.stack.pop().unwrap();
        let frame_kind = match frame {
            Frame::Object { .. } => Bracket::Object,
            Frame::Array { .. } => Bracket::Array,
        };
        let matches_expected = matches!(
            (frame_kind, expected),
            (Bracket::Object, Bracket::Object) | (Bracket::Array, Bracket::Array)
        );
        if !matches_expected {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' does not match the open '{}'",
                line_num,
                expected.close(),
                frame_kind.open()
            )));
        }

        let value = frame.into_value();
        self.attach_child_value(value, line_num)
    }

    fn attach_child_value(&mut self, value: Value, line_num: usize) -> Result<(), Error> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, pending_key } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Syntax(format!(
                        "Line {}: internal error — closed compound without pending key",
                        line_num
                    ))
                })?;
                insert_value(pairs, key, value, line_num)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
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
fn require_sep_end(rest: &str, line_num: usize) -> Result<(), Error> {
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Ok(())
    } else {
        Err(Error::Syntax(format!(
            "Line {}: MissingSeparatorSpace: separator must be followed by whitespace or end of line",
            line_num,
        )))
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
