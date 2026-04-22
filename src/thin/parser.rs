//! Line-oriented parser producing a [`ThinValue<'a>`] that borrows
//! scalars and keys out of the input buffer. Mirrors the logic of the
//! `Value`-building parser but never allocates a key, avoids allocating
//! scalar strings whose content is already a contiguous substring of the
//! input, and places every compound vector inside a bump arena so the
//! entire parse frees in one drop.

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::error::{Error, Result};

use super::value::ThinValue;

// ---------------------------------------------------------------------------
// Frame stack — similar shape to the Value-building parser. Backing Vecs
// live in the bump arena `'a`, not on the heap.
// ---------------------------------------------------------------------------

enum Frame<'a> {
    Object {
        pairs: BumpVec<'a, (&'a str, ThinValue<'a>)>,
        pending_key: Option<&'a str>,
    },
    Array {
        items: BumpVec<'a, ThinValue<'a>>,
    },
}

impl<'a> Frame<'a> {
    fn new_object(bump: &'a Bump) -> Self {
        Frame::Object {
            pairs: BumpVec::with_capacity_in(4, bump),
            pending_key: None,
        }
    }
    fn new_array(bump: &'a Bump) -> Self {
        Frame::Array {
            items: BumpVec::with_capacity_in(4, bump),
        }
    }
    fn into_value(self) -> ThinValue<'a> {
        match self {
            Frame::Object { pairs, .. } => ThinValue::Object(pairs),
            Frame::Array { items } => ThinValue::Array(items),
        }
    }
}

// ---------------------------------------------------------------------------
// Collecting: multi-line string accumulation. Lines themselves are
// borrowed slices (`&'a str`); the outer Vec lives in the arena.
// ---------------------------------------------------------------------------

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
// Parser
// ---------------------------------------------------------------------------

/// Parse `text` into a [`ThinValue`] living in the provided bump arena.
/// Caller is responsible for keeping `bump` alive until the returned
/// value (and any `ThinDeserializer` built from it) is dropped.
pub(crate) fn parse_thin<'a>(text: &'a str, bump: &'a Bump) -> Result<ThinValue<'a>> {
    let mut p = Parser::new(bump);
    for (idx, line) in text.lines().enumerate() {
        p.handle_line(line, idx + 1)?;
    }
    p.finish()
}

struct Parser<'a> {
    bump: &'a Bump,
    stack: Vec<Frame<'a>>,
    collecting: Option<Collecting<'a>>,
}

impl<'a> Parser<'a> {
    fn new(bump: &'a Bump) -> Self {
        let mut stack = Vec::with_capacity(8);
        stack.push(Frame::new_object(bump));
        Self {
            bump,
            stack,
            collecting: None,
        }
    }

    fn finish(mut self) -> Result<ThinValue<'a>> {
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

    fn handle_line(&mut self, raw: &'a str, line_num: usize) -> Result<()> {
        // Inside a multi-line string the line is raw content unless it's
        // the terminator. Most lines inside a multiline have no `)` byte at
        // all, so check that first (memchr-backed) to skip the full `trim`
        // + equality compare on every content line.
        if let Some(ref mut collecting) = self.collecting {
            if raw.as_bytes().contains(&b')') {
                let trimmed = raw.trim();
                let term = match collecting.mode {
                    MultilineMode::Stripped => ")",
                    MultilineMode::Verbatim => "))",
                };
                if trimmed == term {
                    let collecting = self.collecting.take().unwrap();
                    let s = finalize_multiline(collecting, self.bump);
                    return self.attach_scalar(ThinValue::Str(s), line_num);
                }
            }
            collecting.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        if trimmed == "}" {
            return self.close_frame(BracketKind::Object, line_num);
        }
        if trimmed == "]" {
            return self.close_frame(BracketKind::Array, line_num);
        }

        if matches!(self.stack.last(), Some(Frame::Array { .. })) {
            self.handle_array_item(trimmed, raw, line_num)
        } else {
            self.handle_object_pair(trimmed, raw, line_num)
        }
    }

    fn attach_scalar(&mut self, value: ThinValue<'a>, line_num: usize) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, pending_key } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Syntax(format!(
                        "Line {}: internal error — value without pending key",
                        line_num
                    ))
                })?;
                insert_pair(pairs, key, value, line_num, self.bump)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
    }

    fn handle_object_pair(
        &mut self,
        trimmed: &'a str,
        _raw: &'a str,
        line_num: usize,
    ) -> Result<()> {
        let colon = trimmed.find(':').ok_or_else(|| {
            Error::Syntax(format!(
                "Line {}: no ':' — object entries must be 'key: value' pairs",
                line_num
            ))
        })?;

        // trimmed is already trim_start'ed; only trailing space between
        // key and ':' can exist.
        let key = trimmed[..colon].trim_end();
        if key.is_empty() {
            return Err(Error::Syntax(format!("Empty key at line {}", line_num)));
        }
        // Per-segment validation is folded into `insert_at_path`; it sees
        // every segment anyway while descending the path.

        let after_colon = &trimmed[colon + 1..];
        match classify_separator(after_colon) {
            Separator::Raw(rest) => {
                require_sep_end(rest, line_num)?;
                let val_text = rest.trim();
                let value = ThinValue::Str(val_text);
                return self.insert_object_pair(key, value, line_num);
            }
            Separator::TypedInteger(body) => {
                let normalized = validate_typed_integer(body, line_num, self.bump)?;
                return self.insert_object_pair(key, ThinValue::Integer(normalized), line_num);
            }
            Separator::TypedFloat(body) => {
                let normalized = validate_typed_float(body, line_num, self.bump)?;
                return self.insert_object_pair(key, ThinValue::Float(normalized), line_num);
            }
            Separator::Plain => {}
        }

        require_sep_end(after_colon, line_num)?;

        // after_colon has potential leading space but no trailing
        // (trimmed is already trim_end'ed).
        match classify(after_colon.trim_start(), line_num)? {
            ValueStart::Scalar(s) => self.insert_object_pair(key, scalar_to_value(s), line_num),
            ValueStart::EmptyObject => self.insert_object_pair(
                key,
                ThinValue::Object(BumpVec::new_in(self.bump)),
                line_num,
            ),
            ValueStart::EmptyArray => {
                self.insert_object_pair(key, ThinValue::Array(BumpVec::new_in(self.bump)), line_num)
            }
            ValueStart::OpenObject => {
                self.set_pending_key(key, line_num)?;
                self.stack.push(Frame::new_object(self.bump));
                Ok(())
            }
            ValueStart::OpenArray => {
                self.set_pending_key(key, line_num)?;
                self.stack.push(Frame::new_array(self.bump));
                Ok(())
            }
            ValueStart::OpenMultilineStripped => {
                self.set_pending_key(key, line_num)?;
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Stripped,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                Ok(())
            }
            ValueStart::OpenMultilineVerbatim => {
                self.set_pending_key(key, line_num)?;
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Verbatim,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                Ok(())
            }
        }
    }

    fn handle_array_item(
        &mut self,
        trimmed: &'a str,
        _raw: &'a str,
        line_num: usize,
    ) -> Result<()> {
        // Per spec § 5.4, every marker (`::`, `:i`, `:f`) demands sep-end —
        // whitespace or EOL. Glued forms are MissingSeparatorSpace errors,
        // not String fallbacks.
        if let Some(rest) = trimmed.strip_prefix("::") {
            require_sep_end(rest, line_num)?;
            let content = rest.trim_start();
            let value = ThinValue::Str(content);
            return self.push_item(value);
        }

        if let Some(rest) = trimmed.strip_prefix(":i") {
            require_sep_end(rest, line_num)?;
            let normalized = validate_typed_integer(rest, line_num, self.bump)?;
            return self.push_item(ThinValue::Integer(normalized));
        }
        if let Some(rest) = trimmed.strip_prefix(":f") {
            require_sep_end(rest, line_num)?;
            let normalized = validate_typed_float(rest, line_num, self.bump)?;
            return self.push_item(ThinValue::Float(normalized));
        }

        match classify(trimmed, line_num)? {
            ValueStart::Scalar(s) => self.push_item(scalar_to_value(s)),
            ValueStart::EmptyObject => {
                self.push_item(ThinValue::Object(BumpVec::new_in(self.bump)))
            }
            ValueStart::EmptyArray => self.push_item(ThinValue::Array(BumpVec::new_in(self.bump))),
            ValueStart::OpenObject => {
                self.stack.push(Frame::new_object(self.bump));
                Ok(())
            }
            ValueStart::OpenArray => {
                self.stack.push(Frame::new_array(self.bump));
                Ok(())
            }
            ValueStart::OpenMultilineStripped => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Stripped,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                Ok(())
            }
            ValueStart::OpenMultilineVerbatim => {
                self.collecting = Some(Collecting {
                    mode: MultilineMode::Verbatim,
                    lines: BumpVec::with_capacity_in(8, self.bump),
                });
                Ok(())
            }
        }
    }

    fn insert_object_pair(
        &mut self,
        key: &'a str,
        value: ThinValue<'a>,
        line_num: usize,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, .. } => insert_pair(pairs, key, value, line_num, self.bump),
            _ => unreachable!("dispatched as object"),
        }
    }

    fn push_item(&mut self, value: ThinValue<'a>) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
            _ => unreachable!("dispatched as array"),
        }
    }

    fn set_pending_key(&mut self, key: &'a str, line_num: usize) -> Result<()> {
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

    fn close_frame(&mut self, expected: BracketKind, line_num: usize) -> Result<()> {
        if self.stack.len() <= 1 {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' without matching '{}'",
                line_num,
                expected.close(),
                expected.open()
            )));
        }
        let frame = self.stack.pop().unwrap();
        let got = match frame {
            Frame::Object { .. } => BracketKind::Object,
            Frame::Array { .. } => BracketKind::Array,
        };
        if got as u8 != expected as u8 {
            return Err(Error::Syntax(format!(
                "Line {}: '{}' does not match the open '{}'",
                line_num,
                expected.close(),
                got.open()
            )));
        }
        let value = frame.into_value();
        self.attach_child(value, line_num)
    }

    fn attach_child(&mut self, value: ThinValue<'a>, line_num: usize) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            Frame::Object { pairs, pending_key } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Syntax(format!(
                        "Line {}: closed compound without pending key",
                        line_num
                    ))
                })?;
                insert_pair(pairs, key, value, line_num, self.bump)
            }
            Frame::Array { items } => {
                items.push(value);
                Ok(())
            }
        }
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
// Value-start classification
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

/// Spec § 5.3 / § 5.4 sep-end rule: a separator must be followed by at
/// least one ASCII-whitespace byte, or by the end of the line. Anything
/// else is a MissingSeparatorSpace error (§ 6.10).
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

// ---------------------------------------------------------------------------
// Typed-scalar validators. Borrowed input in the common case (no leading
// `+` to strip) — no allocation. On `+`-strip the normalized form gets
// copied into the bump arena.
// ---------------------------------------------------------------------------

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
        // Uncommon path: `+` had to go; copy the rest into the arena so
        // the returned slice is still `&'a str`.
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

/// `trimmed` MUST be trim_start'ed (no leading whitespace). Trailing
/// whitespace has been removed earlier in the pipeline (raw.trim() at
/// the top of handle_line). Don't trim again here — we'd pay O(len)
/// for a no-op on the hot path.
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

fn scalar_to_value(s: &str) -> ThinValue<'_> {
    match s {
        "null" => ThinValue::Null,
        "true" => ThinValue::Bool(true),
        "false" => ThinValue::Bool(false),
        _ => ThinValue::Str(s),
    }
}

// ---------------------------------------------------------------------------
// Insert pair — fast path for non-dotted keys + dotted fallback. Both
// allocate any new sub-objects inside the same bump arena as `target`.
// ---------------------------------------------------------------------------

fn insert_pair<'a>(
    target: &mut BumpVec<'a, (&'a str, ThinValue<'a>)>,
    path: &'a str,
    value: ThinValue<'a>,
    line_num: usize,
    bump: &'a Bump,
) -> Result<()> {
    // Fast path: non-dotted key — the vast majority of inserts.
    if !path.as_bytes().contains(&b'.') {
        if !is_valid_key(path) {
            return Err(Error::Syntax(format!(
                "Invalid key at line {}: '{}'",
                line_num, path
            )));
        }
        if target.iter().any(|(k, _)| *k == path) {
            return Err(Error::Syntax(format!(
                "Line {}: duplicate key '{}'",
                line_num, path
            )));
        }
        target.push((path, value));
        return Ok(());
    }
    insert_at_path(target, path, value, line_num, path, bump)
}

fn insert_at_path<'a>(
    target: &mut BumpVec<'a, (&'a str, ThinValue<'a>)>,
    path: &'a str,
    value: ThinValue<'a>,
    line_num: usize,
    full_path: &'a str,
    bump: &'a Bump,
) -> Result<()> {
    match path.split_once('.') {
        Some((head, rest)) => {
            if !is_valid_key(head) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, full_path
                )));
            }
            if let Some(existing) = target.iter_mut().find(|(k, _)| *k == head) {
                match &mut existing.1 {
                    ThinValue::Object(inner) => {
                        insert_at_path(inner, rest, value, line_num, full_path, bump)
                    }
                    _ => Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — an existing value blocks the path",
                        line_num, full_path
                    ))),
                }
            } else {
                let mut inner: BumpVec<'a, (&'a str, ThinValue<'a>)> = BumpVec::new_in(bump);
                insert_at_path(&mut inner, rest, value, line_num, full_path, bump)?;
                target.push((head, ThinValue::Object(inner)));
                Ok(())
            }
        }
        None => {
            if !is_valid_key(path) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, full_path
                )));
            }
            if target.iter().any(|(k, _)| *k == path) {
                return Err(Error::Syntax(format!(
                    "Line {}: duplicate key '{}'",
                    line_num, full_path
                )));
            }
            target.push((path, value));
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Key validation (same semantics as the owned-parser).
// ---------------------------------------------------------------------------

#[inline]
fn is_valid_key(k: &str) -> bool {
    // All forbidden characters are ASCII; scan bytes, and use
    // `is_ascii_whitespace` rather than the Unicode-wide
    // `char::is_whitespace` (spec §5.3.1 bans "ASCII whitespace"
    // specifically; NBSP etc. are allowed).
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            b.is_ascii_whitespace() || matches!(b, b'[' | b']' | b'{' | b'}' | b':' | b'#')
        })
}

// ---------------------------------------------------------------------------
// Multi-line finalization. Single-line cases return a slice directly from
// the input buffer — no arena touch. Multi-line forms allocate the
// joined/dedented text inside the arena so the returned `&'a str` is
// still arena-rooted.
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
