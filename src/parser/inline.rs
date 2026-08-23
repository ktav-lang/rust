//! Inline compound parser (spec 0.5.0 section 5.8).
//!
//! Parses `{ key: value, ... }` into `Value::Object` and
//! `[ v1, v2, ... ]` into `Value::Array`.
//!
//! Escape sequences (section 3.7) are processed inside inline scalar values.

use memchr::memchr2;

use crate::error::{Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::classify::{fast_plain_decimal_i64, is_float_literal, lossy_scalar, try_parse_integer};
use super::insert::insert_value;

/// Maximum nesting depth for inline compounds (per Q-3 decision).
const MAX_INLINE_DEPTH: usize = 128;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Parse a balanced inline object body. `input` is the full body
/// including the outer `{` and `}`. Returns `Value::Object`.
pub(crate) fn parse_inline_object(
    input: &str,
    line_num: usize,
    span: Span,
    strict: bool,
) -> Result<Value, Error> {
    parse_inline_object_inner(input, line_num, span, 0, strict)
}

/// Parse a balanced inline array body. `input` is the full body
/// including the outer `[` and `]`. Returns `Value::Array`.
pub(crate) fn parse_inline_array(
    input: &str,
    line_num: usize,
    span: Span,
    strict: bool,
) -> Result<Value, Error> {
    parse_inline_array_inner(input, line_num, span, 0, strict)
}

// ---------------------------------------------------------------------------
// Internal — depth-tracking variants
// ---------------------------------------------------------------------------

fn parse_inline_object_inner(
    input: &str,
    line_num: usize,
    span: Span,
    depth: usize,
    strict: bool,
) -> Result<Value, Error> {
    if depth >= MAX_INLINE_DEPTH {
        return Err(malformed(
            line_num,
            span,
            "nesting depth exceeds limit (128)",
        ));
    }

    // Strip outer `{` ... `}`
    debug_assert!(input.starts_with('{') && input.ends_with('}'));
    let inner = &input[1..input.len() - 1];

    if inner.trim().is_empty() {
        return Ok(Value::Object(ObjectMap::default()));
    }

    let segments = split_top_level(inner, line_num, span)?;

    let mut map = ObjectMap::default();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            // Trailing comma (last segment empty) is OK
            if i == n - 1 {
                break;
            }
            // Leading comma or double comma
            return Err(malformed(
                line_num,
                span,
                "empty pair segment (leading comma, double comma, or missing pair)",
            ));
        }

        // Find the first unescaped `:` to split key / value.
        let colon_pos = find_unescaped_colon_inline(trimmed);
        let colon_pos = match colon_pos {
            Some(p) => p,
            None => {
                return Err(malformed(
                    line_num,
                    span,
                    &format!("inline object pair missing ':' separator in '{}'", trimmed),
                ));
            }
        };

        let raw_key = &trimmed[..colon_pos];
        let after_colon = &trimmed[colon_pos + 1..];

        // Check for raw marker `::`
        let (is_raw, value_body) = if let Some(stripped) = after_colon.strip_prefix(':') {
            (true, stripped)
        } else {
            (false, after_colon)
        };

        // Trim the key (each segment individually per section 4)
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }

        // Process value
        let value = if is_raw {
            // Raw `::` — value is a String after escape processing + trim
            let processed = process_escapes(value_body.trim(), line_num, span)?;
            Value::String(processed.into())
        } else {
            // Plain `:` — parse inline value
            parse_inline_value(value_body, line_num, span, depth, strict)?
        };

        // Use insert_value for dotted key expansion
        insert_value(&mut map, key, value, line_num, span)?;
    }

    Ok(Value::Object(map))
}

fn parse_inline_array_inner(
    input: &str,
    line_num: usize,
    span: Span,
    depth: usize,
    strict: bool,
) -> Result<Value, Error> {
    if depth >= MAX_INLINE_DEPTH {
        return Err(malformed(
            line_num,
            span,
            "nesting depth exceeds limit (128)",
        ));
    }

    // Strip outer `[` ... `]`
    debug_assert!(input.starts_with('[') && input.ends_with(']'));
    let inner = &input[1..input.len() - 1];

    if inner.trim().is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    let segments = split_top_level(inner, line_num, span)?;

    let mut items: Vec<Value> = Vec::new();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            // Trailing comma (last segment empty) is OK
            if i == n - 1 {
                break;
            }
            // Leading comma or double comma or empty item
            return Err(malformed(
                line_num,
                span,
                "empty inline-array item (leading comma, double comma, or empty position)",
            ));
        }

        // Check for raw marker `::` at the start of an array item
        if let Some(rest) = trimmed.strip_prefix("::") {
            let processed = process_escapes(rest.trim(), line_num, span)?;
            items.push(Value::String(processed.into()));
            continue;
        }

        // Parse inline value (could be nested compound or scalar)
        let value = parse_inline_value_raw(trimmed, line_num, span, depth, strict)?;
        items.push(value);
    }

    Ok(Value::Array(items))
}

// ---------------------------------------------------------------------------
// Inline value parsing
// ---------------------------------------------------------------------------

/// Parse a single inline value. This is used for pair values after `:`.
/// The value body is NOT yet trimmed.
fn parse_inline_value(
    body: &str,
    line_num: usize,
    span: Span,
    depth: usize,
    strict: bool,
) -> Result<Value, Error> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        // Empty value after `:` → empty String
        return Ok(Value::String("".into()));
    }
    parse_inline_value_raw(trimmed, line_num, span, depth, strict)
}

/// Parse a single inline value that is already trimmed. This handles the
/// section 5.8.5 mid-value brace literal rule: only the FIRST non-ws byte
/// determines whether this is a nested compound or a plain inline scalar.
fn parse_inline_value_raw(
    trimmed: &str,
    line_num: usize,
    span: Span,
    depth: usize,
    strict: bool,
) -> Result<Value, Error> {
    let first_byte = trimmed.as_bytes()[0];

    if first_byte == b'{' {
        // Check for balanced closing `}`
        if let Some(close) = find_matching_close(trimmed, b'{', b'}') {
            if close == trimmed.len() - 1 {
                // Empty object?
                let inner = &trimmed[1..trimmed.len() - 1];
                if inner.trim().is_empty() {
                    return Ok(Value::Object(ObjectMap::default()));
                }
                // Nested inline object
                return parse_inline_object_inner(trimmed, line_num, span, depth + 1, strict);
            }
        }
        // Unterminated
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span,
        }));
    }

    if first_byte == b'[' {
        // Check for balanced closing `]`
        if let Some(close) = find_matching_close(trimmed, b'[', b']') {
            if close == trimmed.len() - 1 {
                let inner = &trimmed[1..trimmed.len() - 1];
                if inner.trim().is_empty() {
                    return Ok(Value::Array(Vec::new()));
                }
                // Nested inline array
                return parse_inline_array_inner(trimmed, line_num, span, depth + 1, strict);
            }
        }
        // Unterminated
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span,
        }));
    }

    // Plain inline scalar — process escapes, then classify per section 5.2
    let processed = process_escapes(trimmed, line_num, span)?;
    let body = processed.trim();

    classify_inline_scalar(body, line_num, span, strict)
}

/// Classify an inline scalar body (after escape processing and trimming)
/// per section 5.2 rules 10-15. Rules 1-9 don't apply inside inline
/// scalars (no multi-line openers, no nested compounds — those are
/// handled by the caller).
fn classify_inline_scalar(
    body: &str,
    line_num: usize,
    span: Span,
    strict: bool,
) -> Result<Value, Error> {
    if body.is_empty() {
        return Ok(Value::String("".into()));
    }

    // section 5.2 rules 10-12: keywords
    match body {
        "null" => return Ok(Value::Null),
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }

    // section 5.2 rule 13: integer literal
    // Fast path for plain decimal — input is already canonical.
    if let Some(_val) = fast_plain_decimal_i64(body) {
        return Ok(Value::Integer(body.into()));
    }
    if let Some(val) = try_parse_integer(body) {
        let mut buf = itoa::Buffer::new();
        let canonical = buf.format(val);
        if strict && canonical != body {
            return Err(lossy_scalar(body, canonical, line_num, span));
        }
        return Ok(Value::Integer(canonical.into()));
    }

    // section 5.2 rule 14: float literal
    if is_float_literal(body) {
        if let Some(val) = parse_float_value(body) {
            let mut buf = ryu::Buffer::new();
            let canonical = buf.format(val);
            let rendered = crate::render::canonical::canonical_float(canonical);
            if strict {
                if rendered != body {
                    return Err(lossy_scalar(body, &rendered, line_num, span));
                }
                // Strict acceptance still stores the same Ryu form as lax parse().
                return Ok(Value::Float(canonical.into()));
            }
            // If ryu reproduces the input, use it directly.
            if canonical == body {
                return Ok(Value::Float(body.into()));
            }
            return Ok(Value::Float(canonical.into()));
        }
    }

    // section 5.2 rule 15: String
    Ok(Value::String(body.into()))
}

/// Parse a float literal (already validated by is_float_literal) into f64.
fn parse_float_value(s: &str) -> Option<f64> {
    // Skip String allocation when there are no underscores
    if !s.as_bytes().contains(&b'_') {
        let val: f64 = s.parse().ok()?;
        if val.is_nan() || val.is_infinite() {
            return None;
        }
        return Some(val);
    }
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
    let val: f64 = cleaned.parse().ok()?;
    if val.is_nan() || val.is_infinite() {
        return None;
    }
    Some(val)
}

// ---------------------------------------------------------------------------
// Escape processing (section 3.7)
// ---------------------------------------------------------------------------

/// Process escape sequences in an inline scalar value.
///
/// Recognised sequences (10, spec 0.6.0 § 3.7):
///   `\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:`.
/// Any other `\X` is a `BadEscapeSequence` error.
pub(crate) fn process_escapes(input: &str, line_num: usize, span: Span) -> Result<String, Error> {
    // Fast path: if no backslash, the input is already clean — return a
    // single allocation rather than scanning byte-by-byte.
    if !input.as_bytes().contains(&b'\\') {
        return Ok(input.to_string());
    }

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 1 >= bytes.len() {
                // Backslash at end of inline body
                return Err(Error::Structured(ErrorKind::BadEscapeSequence {
                    line: line_num as u32,
                    span,
                    sequence: "\\<end-of-line>".to_string(),
                }));
            }
            let next = bytes[i + 1];
            match next {
                b'\\' => out.push('\\'),
                b',' => out.push(','),
                b'}' => out.push('}'),
                b']' => out.push(']'),
                b'{' => out.push('{'),
                b'[' => out.push('['),
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b'.' => out.push('.'),
                b':' => out.push(':'),
                _ => {
                    // Invalid escape
                    let seq = if next < 0x80 {
                        format!("\\{}", next as char)
                    } else {
                        format!("\\<0x{:02X}>", next)
                    };
                    return Err(Error::Structured(ErrorKind::BadEscapeSequence {
                        line: line_num as u32,
                        span,
                        sequence: seq,
                    }));
                }
            }
            i += 2;
        } else {
            // Safe because we're iterating over valid UTF-8
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Key-context escape processing (spec 0.6.0 § 3.7 + § 5.3)
// ---------------------------------------------------------------------------

/// Find the byte offset of the first **unescaped** `:` in `s`. Returns
/// `None` if every `:` is preceded by `\`. Spec 0.6.0 § 5.3 — the pair
/// separator is the first unescaped `:` (or `::`).
///
/// `\` consumes the next byte; pairs of `\\` reset to "no pending
/// escape". This intentionally does not validate the escape sequence —
/// validation is deferred to `decode_key_segment` so a glued
/// `BadEscapeSequence` error fires at the right call site.
pub(crate) fn find_unescaped_colon(s: &str) -> Option<usize> {
    // SIMD-accelerated escape-aware scan: memchr2 jumps to the next
    // candidate byte (`\` or `:`). When we land on `\` we skip the
    // escaped byte and resume; when we land on `:` we return it.
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let rel = memchr2(b'\\', b':', &bytes[i..])?;
        let abs = i + rel;
        if bytes[abs] == b':' {
            return Some(abs);
        }
        // `\` — skip the escaped byte (whatever it is). At EOL just
        // stop: the caller will report a key-without-separator error
        // of its own.
        i = abs + 2;
    }
    None
}

/// Split a key string into dotted segments at **unescaped** `.` bytes
/// (spec 0.6.0 § 4 / § 5.3). The returned slices reference the input;
/// callers run `decode_key_segment` on each segment to materialise the
/// final byte form.
pub(crate) fn split_key_path(s: &str) -> Vec<&str> {
    // SIMD-accelerated escape-aware split: memchr2 jumps to the next
    // candidate byte (`\` or `.`). On `\` skip the escaped byte; on
    // `.` cut a segment.
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        let rel = match memchr2(b'\\', b'.', &bytes[i..]) {
            Some(p) => p,
            None => break,
        };
        let abs = i + rel;
        if bytes[abs] == b'.' {
            out.push(&s[start..abs]);
            start = abs + 1;
            i = abs + 1;
        } else {
            // `\` — escape consumes the next byte.
            if abs + 1 < bytes.len() {
                i = abs + 2;
            } else {
                // Lone trailing `\` — let decoding report it.
                i = abs + 1;
            }
        }
    }
    out.push(&s[start..]);
    out
}

/// Returns `true` iff the key string contains no `.` separator at any
/// unescaped position. Used by callers that take a non-dotted fast
/// path; callers still need `decode_key_segment` to materialise the
/// final byte form when the segment contains a `\`.
pub(crate) fn key_is_single_segment(s: &str) -> bool {
    // SIMD-accelerated escape-aware scan via memchr2.
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let rel = match memchr2(b'\\', b'.', &bytes[i..]) {
            Some(p) => p,
            None => return true,
        };
        let abs = i + rel;
        if bytes[abs] == b'.' {
            return false;
        }
        // `\` — skip the escaped byte.
        i = abs + 2;
    }
    true
}

/// Decode a single key segment per spec 0.6.0 § 3.7. The segment must
/// not contain unescaped `.` or `:` (callers are expected to split on
/// those first). Returns the decoded String on success or
/// `BadEscapeSequence` on an unknown `\X`. Identical escape table to
/// [`process_escapes`].
pub(crate) fn decode_key_segment(
    input: &str,
    line_num: usize,
    span: Span,
) -> Result<String, Error> {
    // Fast path: no backslash → input is already final.
    if !input.as_bytes().contains(&b'\\') {
        return Ok(input.to_string());
    }
    process_escapes(input, line_num, span)
}

// ---------------------------------------------------------------------------
// Splitting on top-level commas
// ---------------------------------------------------------------------------

/// Split `input` on unescaped `,` at nesting depth 0.
///
/// Unlike a naive brace-counting approach, this correctly handles the
/// section 5.8.5 "mid-value brace literal" rule: a `{` or `[` that opens a
/// balanced compound is skipped over entirely. A `{` or `[` that doesn't
/// have a matching closer is treated as literal (the value parser will
/// handle it later per the mid-value-brace rule).
fn split_top_level<'a>(
    input: &'a str,
    _line_num: usize,
    _span: Span,
) -> Result<Vec<&'a str>, Error> {
    let bytes = input.as_bytes();
    let mut segments: Vec<&'a str> = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // Skip escaped character. We validate escapes later
                // during process_escapes; here we just need to not
                // count `\,`, `\{`, `\}`, `\[`, `\]` as structural.
                i += 2;
                continue;
            }
            b'{' | b'[' => {
                // Check if this opens a balanced nested compound.
                let open = bytes[i];
                let close = if open == b'{' { b'}' } else { b']' };
                if let Some(close_pos) = find_matching_close(&input[i..], open, close) {
                    // Skip over the entire nested compound.
                    i += close_pos + 1;
                    continue;
                }
                // Not balanced — treat as literal byte (mid-value brace).
                // The value parser will handle it correctly per section 5.8.5.
            }
            b',' => {
                segments.push(&input[start..i]);
                start = i + 1;
                i += 1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    // Last segment (after final comma, or the whole string if no comma)
    segments.push(&input[start..]);

    Ok(segments)
}

// ---------------------------------------------------------------------------
// Delimiter matching helpers
// ---------------------------------------------------------------------------

/// Check if `input` is a balanced inline compound: starts with `open`
/// and has a matching `close` at the very end. Returns the last byte
/// index if found.
fn find_matching_close(input: &str, open: u8, close: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != open {
        return None;
    }

    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2; // skip escaped character
                continue;
            }
            b if b == open => {
                depth += 1;
            }
            b if b == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find the first unescaped `:` in `s` that is at nesting depth 0.
/// Used to split inline pairs into key and value.
fn find_unescaped_colon_inline(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Error helpers
// ---------------------------------------------------------------------------

fn malformed(line_num: usize, span: Span, detail: &str) -> Error {
    Error::Structured(ErrorKind::MalformedInlineCompound {
        line: line_num as u32,
        span,
        detail: detail.to_string(),
    })
}
