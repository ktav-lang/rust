//! Inline compound parser (spec 0.5.0 section 5.8).
//!
//! Parses `{ key: value, ... }` into `Value::Object` and
//! `[ v1, v2, ... ]` into `Value::Array`.
//!
//! Escape sequences (section 3.7) are processed inside inline scalar values.

use std::borrow::Cow;

use memchr::memchr2;

use crate::error::{Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};
use crate::whitespace::{inline_whitespace_ascii, is_inline_whitespace};

use super::classify::{fast_plain_decimal_i64, is_float_literal, lossy_scalar, try_parse_integer};
use super::insert::insert_value;

/// Maximum nesting depth for inline compounds (per Q-3 decision).
pub(crate) const MAX_INLINE_DEPTH: usize = 128;

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

    // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
    if inner.trim_matches(is_inline_whitespace).is_empty() {
        return Ok(Value::Object(ObjectMap::default()));
    }

    let segments = split_top_level(inner, line_num, span, InlineBody::Object)?;

    let mut map = ObjectMap::default();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let trimmed = seg.trim_matches(is_inline_whitespace);
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
        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let key = raw_key.trim_matches(is_inline_whitespace);
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }

        // Process value
        let value = if is_raw {
            // Raw `::` — value is a String after escape processing + trim
            // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line;
            // `\n` escapes are two raw chars, not a raw LF byte).
            let processed = process_escapes(
                value_body.trim_matches(is_inline_whitespace),
                line_num,
                span,
            )?;
            Value::String(processed.into_owned().into())
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

    // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
    if inner.trim_matches(is_inline_whitespace).is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    let segments = split_top_level(inner, line_num, span, InlineBody::Array)?;

    let mut items: Vec<Value> = Vec::new();
    let n = segments.len();
    for (i, seg) in segments.into_iter().enumerate() {
        // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
        let trimmed = seg.trim_matches(is_inline_whitespace);
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
            // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line;
            // `\n` escapes are two raw chars, not a raw LF byte).
            let processed =
                process_escapes(rest.trim_matches(is_inline_whitespace), line_num, span)?;
            items.push(Value::String(processed.into_owned().into()));
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
    // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
    let trimmed = body.trim_matches(is_inline_whitespace);
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
        // § 6.11/§ 6.12 tri-state. Since R7-F1 `find_matching_close`
        // applies the SAME § 5.8.5 value-start / raw-mode / quoted-key
        // rules as `scan_inline_closer`; it differs only in never
        // validating escapes. The fallback exists so a
        // `BadEscapeSequence` (§ 6.13) still takes precedence over the
        // unterminated decision when no closer is found at all.
        let close_idx = match find_matching_close(trimmed, b'{', b'}') {
            Some(close) => Some(close),
            None => match scan_inline_closer(trimmed, b'{', b'}', line_num, span) {
                InlineCloserScan::Found(idx) => Some(idx),
                InlineCloserScan::NotFound => None,
                InlineCloserScan::BadEscape(err) => return Err(err),
            },
        };
        match close_idx {
            // § 6.11: the matching closer is the last byte.
            Some(idx) if idx == trimmed.len() - 1 => {
                // Empty object?
                let inner = &trimmed[1..trimmed.len() - 1];
                // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
                if inner.trim_matches(is_inline_whitespace).is_empty() {
                    return Ok(Value::Object(ObjectMap::default()));
                }
                // Nested inline object
                return parse_inline_object_inner(trimmed, line_num, span, depth + 1, strict);
            }
            // § 6.12: closer found, but content follows it.
            Some(_) => return Err(malformed_closer_not_at_end(line_num, span)),
            // § 6.11: no closer at all.
            None => {
                return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                    line: line_num as u32,
                    span,
                }));
            }
        }
    }

    if first_byte == b'[' {
        // Check for balanced closing `]` (see the `{` branch for the
        // escape-precedence fallback).
        // § 6.11/§ 6.12 tri-state (see the `{` branch).
        let close_idx = match find_matching_close(trimmed, b'[', b']') {
            Some(close) => Some(close),
            None => match scan_inline_closer(trimmed, b'[', b']', line_num, span) {
                InlineCloserScan::Found(idx) => Some(idx),
                InlineCloserScan::NotFound => None,
                InlineCloserScan::BadEscape(err) => return Err(err),
            },
        };
        match close_idx {
            // § 6.11: the matching closer is the last byte.
            Some(idx) if idx == trimmed.len() - 1 => {
                let inner = &trimmed[1..trimmed.len() - 1];
                // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
                if inner.trim_matches(is_inline_whitespace).is_empty() {
                    return Ok(Value::Array(Vec::new()));
                }
                // Nested inline array
                return parse_inline_array_inner(trimmed, line_num, span, depth + 1, strict);
            }
            // § 6.12: closer found, but content follows it.
            Some(_) => return Err(malformed_closer_not_at_end(line_num, span)),
            // § 6.11: no closer at all.
            None => {
                return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                    line: line_num as u32,
                    span,
                }));
            }
        }
    }

    // section 5.2 rule 5: `()` / `(())` → empty String, on the RAW body
    // (escape provenance: `\u0028\u0029` stays a literal String "()").
    if trimmed == "()" || trimmed == "(())" {
        return Ok(Value::String("".into()));
    }

    // Plain inline scalar — process escapes, then classify per section 5.2.
    // 0.7 § 3.7 / § 5.2: a recognised escape anywhere in the raw body
    // forces String (rule 15) — the decoded body is never re-classified as
    // keyword/numeric. `process_escapes` borrows iff the raw body
    // contains no `\` at all, and errors on every unrecognised `\X`
    // form, so an owned return means at least one recognised escape was
    // decoded. (Scanning the DECODED bytes for `\` would be wrong:
    // `\u0074` decodes to `t` with no backslash left.)
    let processed = process_escapes(trimmed, line_num, span)?;

    if matches!(&processed, Cow::Owned(_)) {
        return Ok(Value::String(processed.into_owned().into()));
    }

    classify_inline_scalar(&processed, line_num, span, strict)
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
            if strict {
                let rendered = crate::render::canonical::canonical_float(canonical);
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
pub(crate) fn parse_float_value(s: &str) -> Option<f64> {
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

/// One recognised § 3.7 escape, scanned at `bytes[i]` (which must be
/// `\`). The variant determines the sequence's total byte length.
enum RecognisedEscape {
    /// Single-character escape: the char to emit; total length is 2 bytes.
    Simple(char),
    /// `\uXXXX` mapping to an ordinary (non-surrogate) code point; total
    /// length is 6 bytes.
    Unicode(char),
    /// `\uXXXX\uXXXX` surrogate pair, combined; total length is 12 bytes.
    SurrogatePair(char),
}

impl RecognisedEscape {
    /// Total byte length of the escape sequence (2 for simple escapes,
    /// 6 for `\uXXXX`, 12 for a full surrogate pair).
    fn len(&self) -> usize {
        match self {
            RecognisedEscape::Simple(_) => 2,
            RecognisedEscape::Unicode(_) => 6,
            RecognisedEscape::SurrogatePair(_) => 12,
        }
    }
}

/// Scan one escape sequence. Returns the offending sequence text (the
/// exact `BadEscapeSequence` payload) on any unrecognised or malformed
/// form; the recognised table and every error string are identical to
/// `process_escapes`' decode — this is the single source of both.
fn scan_escape(bytes: &[u8], i: usize) -> Result<RecognisedEscape, String> {
    if i + 1 >= bytes.len() {
        // Backslash at end of line
        return Err("\\<end-of-line>".to_string());
    }
    let next = bytes[i + 1];
    let simple = match next {
        b'\\' => Some('\\'),
        b',' => Some(','),
        b'}' => Some('}'),
        b']' => Some(']'),
        b'{' => Some('{'),
        b'[' => Some('['),
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b'.' => Some('.'),
        b':' => Some(':'),
        b'"' => Some('"'),
        b'\'' => Some('\''),
        b'`' => Some('`'),
        _ => None,
    };
    if let Some(ch) = simple {
        return Ok(RecognisedEscape::Simple(ch));
    }
    if next == b'u' {
        // `\uXXXX`: exactly four ASCII hex digits (spec 0.7 § 3.7.1).
        // Validate up-front so nothing is consumed on a malformed escape.
        if i + 6 > bytes.len() || !bytes[i + 2..i + 6].iter().all(|b| b.is_ascii_hexdigit()) {
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        // All-ASCII validated slice: `from_utf8` cannot fail.
        let hex = std::str::from_utf8(&bytes[i + 2..i + 6]).unwrap();
        let value = u32::from_str_radix(hex, 16).expect("4 ASCII hex digits");
        if (0xD800..=0xDBFF).contains(&value) {
            // High surrogate: must pair with an immediately following
            // low-surrogate `\uXXXX` (12 bytes total).
            if i + 12 <= bytes.len()
                && bytes[i + 6] == b'\\'
                && bytes[i + 7] == b'u'
                && bytes[i + 8..i + 12].iter().all(|b| b.is_ascii_hexdigit())
            {
                let low_hex = std::str::from_utf8(&bytes[i + 8..i + 12]).unwrap();
                let low = u32::from_str_radix(low_hex, 16).expect("4 ASCII hex digits");
                if (0xDC00..=0xDFFF).contains(&low) {
                    let combined = 0x10000 + (value - 0xD800) * 0x400 + (low - 0xDC00);
                    // 0x10000..=0x10FFFF by construction.
                    let ch = char::from_u32(combined).expect("valid surrogate pair");
                    return Ok(RecognisedEscape::SurrogatePair(ch));
                }
            }
            // Lone high surrogate (end of input, non-`\u` text, malformed
            // second escape, or non-low value).
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        if (0xDC00..=0xDFFF).contains(&value) {
            // Lone low surrogate.
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        // Ordinary BMP code point.
        let ch = char::from_u32(value).expect("BMP non-surrogate value");
        return Ok(RecognisedEscape::Unicode(ch));
    }
    // Invalid escape
    if next < 0x80 {
        Err(format!("\\{}", next as char))
    } else {
        Err(format!("\\<0x{:02X}>", next))
    }
}

/// Process escape sequences in an inline scalar value.
///
/// Recognised sequences (14, spec 0.7 § 3.7 / § 3.7.1):
///   `\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:`,
///   `\"`, `\'`, `` \` `` and `\uXXXX` (exactly four hex digits,
///   case-insensitive). A `\uXXXX` in the high-surrogate range must be
///   immediately followed by a low surrogate `\uXXXX` (combined into a
///   single scalar value); lone surrogates and malformed `\u` forms are
///   `BadEscapeSequence`. The three quote escapes exist for the quoted
///   key form (§ 5.3.3) but are recognised in every context where
///   escapes are recognised at all (§ 3.7).
/// Any other `\X` is a `BadEscapeSequence` error.
///
/// Returns `Cow::Borrowed` iff the input contains no `\` at all;
/// `Cow::Owned` iff at least one recognised escape was decoded.
pub(crate) fn process_escapes<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
) -> Result<Cow<'a, str>, Error> {
    // Fast path: if no backslash, the input is already clean — return it
    // borrowed, no allocation at all.
    if !input.as_bytes().contains(&b'\\') {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let esc = match scan_escape(bytes, i) {
                Err(sequence) => {
                    return Err(Error::Structured(ErrorKind::BadEscapeSequence {
                        line: line_num as u32,
                        span,
                        sequence,
                    }));
                }
                Ok(esc) => esc,
            };
            let len = esc.len();
            match esc {
                RecognisedEscape::Simple(ch)
                | RecognisedEscape::Unicode(ch)
                | RecognisedEscape::SurrogatePair(ch) => out.push(ch),
            }
            i += len;
        } else {
            // Safe because we're iterating over valid UTF-8
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    Ok(Cow::Owned(out))
}

/// Render a malformed `\u` escape for the error payload: `\u` followed
/// by up to four raw bytes from the input; ASCII bytes as chars, others
/// as `<0xXX>`.
fn render_malformed_unicode_escape(bytes: &[u8], i: usize) -> String {
    let end = usize::min(i + 6, bytes.len());
    let mut seq = String::from("\\u");
    for &b in &bytes[i + 2..end] {
        if b < 0x80 {
            seq.push(b as char);
        } else {
            seq.push_str(&format!("<0x{:02X}>", b));
        }
    }
    seq
}

// ---------------------------------------------------------------------------
// Key-context escape processing (spec 0.6.0 § 3.7 + § 5.3)
// ---------------------------------------------------------------------------

/// Outcome of scanning key text for the pair separator (spec 0.7 § 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColonScan {
    /// Byte offset of the first unescaped `:` outside quoted segments.
    Found(usize),
    /// No unescaped `:` anywhere outside quoted segments.
    Absent,
    /// A quoted segment opened at a segment-start position and never
    /// closed: the rest of the line, colon included, is segment
    /// content (§ 5.3.3 "Unterminated quoted segments").
    UnterminatedQuote,
}

/// True iff `b` opens a quoted key segment (spec 0.7 § 5.3.3).
fn is_quote_byte(b: u8) -> bool {
    b == b'"' || b == b'\'' || b == b'`'
}

/// True iff `bytes` contains any quote byte. Cheap early-out for the
/// scanners: without a quote byte, quoted-segment tracking cannot
/// change the outcome, so callers keep their SIMD fast paths.
fn has_quote_bytes(bytes: &[u8]) -> bool {
    bytes.contains(&b'"') || bytes.contains(&b'\'') || bytes.contains(&b'`')
}

/// Skip line-bounded § 3.3 whitespace; LF/CR cannot occur here (lines
/// are
/// pre-split). Returns the index of the first non-whitespace byte at or
/// after `i`. Classification MUST NOT delegate to a host
/// Unicode-whitespace primitive (§ 3.3); it comes from the shared § 3.3
/// module ([`crate::whitespace`]) via [`inline_whitespace_at`].
fn skip_segment_ws(s: &str, mut i: usize) -> usize {
    while let Some(len) = inline_whitespace_at(s, i) {
        i += len;
    }
    i
}

/// Byte length of the § 3.3 whitespace code point starting at byte
/// offset `i`, or `None` if the code point at `i` is not whitespace.
/// Classification comes from the shared § 3.3 module
/// ([`crate::whitespace`]) — never a host Unicode-whitespace primitive
/// (`char::is_whitespace` MUST NOT be delegated to, even though it
/// matches the list today): the four single-byte ASCII members take the
/// byte fast path ([`inline_whitespace_ascii`]), the decoded-char check
/// is [`is_inline_whitespace`] (the § 3.3 set minus LF/CR; LF/CR cannot
/// occur here — lines are pre-split per § 3.2). A mid-code-point
/// (non-boundary) offset is never whitespace: callers scan
/// byte-at-a-time and may sit on a continuation byte of a
/// non-whitespace character.
fn inline_whitespace_at(s: &str, i: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let b = *bytes.get(i)?;
    if inline_whitespace_ascii(b) {
        return Some(1);
    }
    if b < 0x80 || !s.is_char_boundary(i) {
        return None;
    }
    let ch = s[i..].chars().next()?;
    if is_inline_whitespace(ch) {
        Some(ch.len_utf8())
    } else {
        None
    }
}

/// Scan key text for the pair separator, treating quoted-segment
/// content as opaque (spec 0.7 § 4's separator-scanning rule: the
/// separator is the first unescaped `:`, with the content of any
/// `<quoted-segment>` encountered along the way treated as opaque).
/// A quoted segment only OPENS at a segment-start position: the very
/// start of the text, or immediately after an unescaped `.` (plus
/// line-bounded whitespace, since `<raw-segment> ::= (ws) <segment>
/// (ws)`).
pub(crate) fn scan_unescaped_colon(s: &str) -> ColonScan {
    let bytes = s.as_bytes();
    if !has_quote_bytes(bytes) {
        // No quote bytes: segment tracking cannot change the outcome.
        return match find_unescaped_colon_fast(s) {
            Some(p) => ColonScan::Found(p),
            None => ColonScan::Absent,
        };
    }
    let mut i = 0;
    let mut seg_start = true; // position 0 is a segment start
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                return match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => ColonScan::UnterminatedQuote,
                };
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape lead: consume the escaped byte too (a lone trailing `\` overshoots `bytes.len()`, which the loop guard makes safe — scan just ends, as in `find_unescaped_colon`)
            b'.' => {
                seg_start = true;
                i += 1;
            }
            b':' => return ColonScan::Found(i),
            _ => i += 1,
        }
    }
    ColonScan::Absent
}

/// SIMD-accelerated escape-aware `:` scan: memchr2 jumps to the next
/// candidate byte (`\` or `:`). When we land on `\` we skip the
/// escaped byte and resume; when we land on `:` we return it.
fn find_unescaped_colon_fast(s: &str) -> Option<usize> {
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

/// Find the byte offset of the first **unescaped** `:` in `s`. Returns
/// `None` if every `:` is preceded by `\`, or if a quoted key segment
/// opened at a segment-start position and swallowed the rest of the
/// line (spec 0.7 § 5.3.3 "Unterminated quoted segments" — callers
/// needing to distinguish that case use [`scan_unescaped_colon`]).
/// Spec 0.6.0 § 5.3 — the pair separator is the first unescaped `:`
/// (or `::`).
///
/// `\` consumes the next byte; pairs of `\\` reset to "no pending
/// escape". This intentionally does not validate the escape sequence —
/// validation is deferred to `decode_key_segment` so a glued
/// `BadEscapeSequence` error fires at the right call site.
pub(crate) fn find_unescaped_colon(s: &str) -> Option<usize> {
    match scan_unescaped_colon(s) {
        ColonScan::Found(p) => Some(p),
        _ => None,
    }
}

/// Split a key string into dotted segments at **unescaped** `.` bytes
/// (spec 0.6.0 § 4 / § 5.3). The returned slices reference the input;
/// callers run `decode_key_segment` on each segment to materialise the
/// final byte form.
pub(crate) fn split_key_path(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    if !has_quote_bytes(bytes) {
        // SIMD-accelerated escape-aware split: memchr2 jumps to the next
        // candidate byte (`\` or `.`). On `\` skip the escaped byte; on
        // `.` cut a segment.
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
        return out;
    }
    // Slow path (quote bytes present): quoted segments are opaque to
    // `.` splitting (spec 0.7 § 5.3.3). A segment opens only at a
    // segment-start position: position 0 or after an unescaped `.`
    // (plus line-bounded whitespace).
    let mut start = 0;
    let mut i = 0;
    let mut seg_start = true;
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => {
                        // Defensive: callers only reach here with
                        // well-formed key text (the colon scan already
                        // proved the separator exists), so an unterminated
                        // span here keeps the remainder as one segment.
                        break;
                    }
                }
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape consumes the next byte
            b'.' => {
                out.push(&s[start..i]);
                start = i + 1;
                i += 1;
                seg_start = true;
            }
            _ => i += 1,
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
    let bytes = s.as_bytes();
    if !has_quote_bytes(bytes) {
        // SIMD-accelerated escape-aware scan via memchr2.
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
        return true;
    }
    // Slow path (quote bytes present): dots inside quoted segments are
    // not separators (spec 0.7 § 5.3.3). Segment-start tracking as in
    // [`split_key_path`].
    let mut i = 0;
    let mut seg_start = true;
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => {
                        // Defensive (see `split_key_path`): treat the
                        // unterminated span as one segment.
                        return true;
                    }
                }
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape consumes the next byte
            b'.' => return false,
            _ => i += 1,
        }
    }
    true
}

/// Find the matching unescaped closer for a quoted key segment opened
/// at `open_idx` (spec 0.7 § 5.3.3). `bytes[open_idx]` must be `"`,
/// `'`, or `` ` ``. Backslash-escaped bytes are skipped (`\"` does not
/// close a `"` segment). Returns `None` when no closer exists before
/// the end of input — the segment then swallows the entire remainder.
pub(crate) fn quoted_span_end(bytes: &[u8], open_idx: usize) -> Option<usize> {
    let quote = bytes[open_idx];
    let mut j = open_idx + 1;
    while j < bytes.len() {
        if bytes[j] == b'\\' {
            j += 2;
            continue;
        }
        if bytes[j] == quote {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Decode a single key segment per spec 0.7 § 3.7 / § 5.3.3. The
/// segment must not contain unescaped `.` or `:` (callers are expected
/// to split on those first). A segment whose first byte is `"`, `'`,
/// or `` ` `` is a `<quoted-segment>` (§ 5.3.3): the outer delimiter
/// pair is stripped and `process_escapes` is applied to the interior
/// only, with NO trimming of the interior (quoted content is never
/// trimmed — `" a "` decodes to the 3-char key ` a `). Callers must
/// have validated the segment with `validate::check_key` first.
/// Returns a borrowed slice when there is no escape at all: a quoted
/// segment with no `\` in its interior borrows the interior slice, and
/// a bare segment with no `\` borrows the input as-is — no allocation.
/// Any `\` decodes via [`process_escapes`] and yields an owned String,
/// including `\.`/`\:`, which decode to literal `.`/`:` that are NOT
/// separators. Errors with `BadEscapeSequence` on an unknown `\X`;
/// identical escape table to [`process_escapes`].
pub(crate) fn decode_key_segment<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
) -> Result<Cow<'a, str>, Error> {
    // Quoted segment (spec 0.7 § 5.3.3): strip the outer delimiter pair,
    // decode the interior only. Callers must have validated the segment
    // with `check_key` (properly closed, nothing after the closer).
    if !input.is_empty() {
        let first = input.as_bytes()[0];
        if first == b'"' || first == b'\'' || first == b'`' {
            debug_assert!(input.len() >= 2 && input.as_bytes()[input.len() - 1] == first);
            let interior = &input[1..input.len() - 1];
            if !interior.as_bytes().contains(&b'\\') {
                return Ok(Cow::Borrowed(interior));
            }
            return process_escapes(interior, line_num, span);
        }
    }
    // Bare segment path (unchanged)
    if !input.as_bytes().contains(&b'\\') {
        return Ok(Cow::Borrowed(input));
    }
    process_escapes(input, line_num, span)
}

// ---------------------------------------------------------------------------
// Splitting on top-level commas
// ---------------------------------------------------------------------------

/// Which inline-compound body is being split (spec 0.7 § 5.3.3
/// "Keys only"): in an object body, quotes at key-segment-start are
/// quoted KEYS and opaque to comma splitting; in an array body every
/// position is a value position, so quotes are ordinary content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineBody {
    Object,
    Array,
}

// ---------------------------------------------------------------------------
// Shared single-loop inline scanner
// ---------------------------------------------------------------------------
//
// `split_top_level`, `split_top_level_fast`, `find_matching_close` and
// `scan_inline_closer` are thin wrappers over ONE byte-at-a-time state
// machine ([`Scanner::run`]). The three former hand-written scanners
// duplicated overlapping state machines and successive review rounds
// each found a defect fixed in one copy but not another; the shared
// loop carries every policy decision as an associated `const` of the
// [`ScanCfg`] trait so each (entry point × quote-mode) instantiation
// monomorphizes into its own specialized copy where every policy check
// folds away at compile time — no `dyn`, no runtime enum dispatch in
// the byte loop.
//
// Preserved-quirk inventory (all LOAD-BEARING current behavior):
// 1. the fast configs (find-fast, scan-fast) leave `value_start`
//    untouched at a nested closer; the slow configs clear it
//    (`CLEAR_VS_ON_NESTED_CLOSE`).
// 2. a fast-config `[` opener leaves `in_key` untouched; a
//    quote-tracking config sets `in_key = (b == b'{')`.
// 3. split maps an unterminated quoted key to
//    `UnterminatedInlineCompound`, and dotted-key-then-EOF to
//    whitespace-only-rest → no trailing segment, else `EmptyKey`;
//    find/scan map both to `None`/`NotFound`.
// 4. find/scan commas set `value_start = true` even in object scopes;
//    split's comma sets `value_start = !body_object`.
// 5. scan validates escapes (full-length advance,
//    `BadEscapeSequence` precedence); find/split skip 2 bytes
//    unvalidated (validation happens later in `process_escapes`).
// 6. `prev` tracking (every TRACK_RAW config, i.e. find and scan):
//    ws arm sets prev to the run's last byte; quoted-span continue
//    sets prev to the closing quote byte; `\\` sets prev to `\\`.
// 7. the lone-`:`-in-value rule (R5-F2) clears `value_start` unless
//    the next byte is `:`.
// 8. The segment-start skip block runs only when
//     `TRACK_QUOTES && in_key && seg_start`.
//
// R7-F1: the find configs previously kept naive opener counting, no
// raw-marker tracking and no value-position tracking — frozen at the
// unification as `OPENER_GATED = false`. A literal mid-scalar `[`
// then pushed a PHANTOM Array scope; the next comma derived key
// context from it, quoted keys stopped being opaque, and a key's `}`
// was mistaken for the body closer. Byte meaning (opener
// structurality, raw mode, quoted-key opacity, crossed closers) is
// now IDENTICAL across find and scan; the configs differ only in
// result shape (`Option<usize>` vs [`InlineCloserScan`]) and
// validation mode (find never validates escapes).

/// Per open compound: the opener kind plus the enclosing key-position,
/// segment-start and raw state, packed into ONE byte. The scope stack
/// pushes/pops one element per nesting level, so the element size is
/// hot on deeply-nested documents; the former 4-byte struct
/// (kind + three `bool`s) regressed them measurably against the
/// pre-unification `Vec<u8>` opener stack.
///
/// Bit layout: bit 0 = opener kind (0: `{`-scope, 1: `[`-scope),
/// bit 1 = saved `in_key`, bit 2 = saved `seg_start`, bit 3 = saved `raw`.
#[derive(Clone, Copy)]
struct ScopeFrame(u8);

impl ScopeFrame {
    /// `kind` is the opener byte, `b'{'` or `b'['`.
    #[inline]
    fn pack(kind: u8, saved_in_key: bool, saved_seg_start: bool, saved_raw: bool) -> Self {
        ScopeFrame(
            ((kind == b'[') as u8)
                | ((saved_in_key as u8) << 1)
                | ((saved_seg_start as u8) << 2)
                | ((saved_raw as u8) << 3),
        )
    }

    /// The stored opener kind, as the opener byte it was packed from.
    #[inline]
    fn kind(self) -> u8 {
        if self.0 & 1 == 0 {
            b'{'
        } else {
            b'['
        }
    }

    #[inline]
    fn saved_in_key(self) -> bool {
        self.0 & 0b0010 != 0
    }

    #[inline]
    fn saved_seg_start(self) -> bool {
        self.0 & 0b0100 != 0
    }

    #[inline]
    fn saved_raw(self) -> bool {
        self.0 & 0b1000 != 0
    }
}

/// Why the shared scanner loop stopped.
enum ScanStop {
    /// Depth returned to 0 at `idx`; `byte` is the closer byte
    /// (`}`/`]`) seen there.
    Closer { idx: usize, byte: u8 },
    /// The seg-start whitespace skip consumed the rest of the input.
    EofAfterWsSkip,
    /// A quoted key segment never closed.
    UnterminatedQuote,
    /// [`scan_escape`] rejection with the ready-made error payload.
    BadEscape(Error),
    /// Input exhausted without any of the above.
    Exhausted,
}

/// Policy for `value_start` at a comma.
#[derive(Clone, Copy)]
enum CommaVs {
    /// scan: every loop-seen comma opens a value position.
    Always,
    /// split: `value_start = !body_object` (§ 5.3.3 "Keys only").
    BodyNegated,
}

/// Compile-time policy knobs of the shared scanner. Every `const` is
/// folded away by monomorphization, so each config's byte loop is the
/// specialized machine of exactly one former hand-written scanner.
trait ScanCfg {
    /// Quote/segment-start tracking (§ 5.3.3). `false` for every
    /// *Fast config.
    const TRACK_QUOTES: bool;
    /// Maintain the scope stack (find and scan, all four configs);
    /// `false` for split (its `OPENER_JUMP` sub-scans keep the stack empty).
    const USE_STACK: bool;
    /// split: sub-scan a candidate nested compound with its own pair
    /// ([`scan_inline_closer`]) and jump over it.
    const OPENER_JUMP: bool;
    /// find/scan: BOTH closer kinds decrement the shared depth and
    /// the depth-0 closer must be the body's own kind; split: unused
    /// (`}`/`]` are literal there).
    const DECREMENT_ANY_CLOSER: bool;
    /// split: `}`/`]` are ordinary content (§ 5.8.5 shields commas
    /// only inside value-start compounds).
    const CLOSER_LITERAL: bool;
    /// scan configs only ([`scan_inline_closer`]): [`scan_escape`]
    /// validation with `BadEscapeSequence` precedence (§ 5.2
    /// rules-6–9 preamble); find defers escape validation to
    /// `process_escapes` so the nested find-first dispatch keeps
    /// `BadEscapeSequence` precedence via its scan fallback.
    const VALIDATE_ESCAPES: bool;
    /// find + scan: `::` raw marker (§ 5.4) + `prev` byte tracking;
    /// split never tracks raw.
    const TRACK_RAW: bool;
    /// split: record a segment at every loop-seen comma.
    const COMMA_SPLITS: bool;
    /// find/scan: comma key context comes from the CURRENT scope (the
    /// stack top, R5-F1/R6-F1); split: from the body kind.
    const COMMA_CTX_SCOPE: bool;
    /// `value_start` policy at a comma (quirk 4).
    const COMMA_VS: CommaVs;
    /// find + scan: an unescaped comma ends raw mode (R5-F3).
    const COMMA_CLEARS_RAW: bool;
    /// a key `:` sets `value_start` (find + scan + split).
    const COLON_SETS_VS: bool;
    /// split: a non-key `:` clears `value_start` — `:` is never § 3.3
    /// whitespace, so split's old `_` fall-through applies.
    const COLON_ELSE_CLEAR_VS: bool;
    /// Restore `in_key` from a kind-matched popped scope.
    const RESTORE_IN_KEY_ON_MATCH: bool;
    /// scan-slow/find-slow: TRUE — a kind-matched closer restores the
    /// saved `seg_start`; the fast configs track no segments.
    const RESTORE_SEG_ON_MATCH: bool;
    /// scan-slow/find-slow: restore saved raw state on a kind match
    /// (§ 5.8.5, R4-F1).
    const RESTORE_RAW_ON_MATCH: bool;
    /// find-slow + scan-slow: a closer that matches NO scope kind
    /// still forces `seg_start = false`.
    const MISMATCH_SEG_FALSE: bool;
    /// the slow configs: a nested closer clears `value_start`; the
    /// fast configs deliberately leave it untouched (quirk 1).
    const CLEAR_VS_ON_NESTED_CLOSE: bool;
}

/// `find_matching_close`, quote-free fast path: byte-for-byte the
/// [`ScanFast`] machine minus escape validation (R7-F1). `in_key` and
/// `seg_start` are maintained but never read meaningfully (no quote
/// bytes exist); writes stay so find and scan share one code path.
struct FindFast;
impl ScanCfg for FindFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_VS: CommaVs = CommaVs::Always;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
}

/// `find_matching_close`, slow path (quote bytes present): the
/// [`ScanQ`] machine minus escape validation (R7-F1). The same
/// per-scope key-position tracking, value-position gating and
/// raw-marker rules as the root scan — only the result shape
/// (`Option<usize>`, no error construction) and the missing escape
/// validation differ.
struct FindQ;
impl ScanCfg for FindQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_VS: CommaVs = CommaVs::Always;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
}

/// `scan_inline_closer`, quote-free fast path. `value_start` marks an
/// unconsumed value position (§ 5.8.5); both closer kinds decrement
/// the shared depth. Quirks: opener `[` leaves `in_key` untouched (the
/// next `,` re-derives it) and a nested closer leaves `value_start`
/// untouched.
struct ScanFast;
impl ScanCfg for ScanFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = true;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_VS: CommaVs = CommaVs::Always;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
}

/// `scan_inline_closer`, slow path (quote bytes present): the
/// per-level key-position machine plus the same value-position /
/// raw-marker tracking as the fast path; closers restore the
/// enclosing scope's key, segment-start and raw state (R4-F2) and
/// clear `value_start`.
struct ScanQ;
impl ScanCfg for ScanQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = true;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_VS: CommaVs = CommaVs::Always;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
}

/// `split_top_level`, quote-free fast path (also serving object bodies
/// without quote bytes). Implements the § 5.8.5 value-start rule via
/// sub-scans; `}`/`]` are ordinary content.
struct SplitFast;
impl ScanCfg for SplitFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = false;
    const OPENER_JUMP: bool = true;
    const DECREMENT_ANY_CLOSER: bool = false;
    const CLOSER_LITERAL: bool = true;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = false;
    const COMMA_SPLITS: bool = true;
    const COMMA_CTX_SCOPE: bool = false;
    const COMMA_VS: CommaVs = CommaVs::BodyNegated;
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
}

/// `split_top_level`, object body with quote bytes: quoted KEYS are
/// comma-opaque (§ 5.3.3), quotes in value positions are content.
struct SplitQ;
impl ScanCfg for SplitQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = false;
    const OPENER_JUMP: bool = true;
    const DECREMENT_ANY_CLOSER: bool = false;
    const CLOSER_LITERAL: bool = true;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = false;
    const COMMA_SPLITS: bool = true;
    const COMMA_CTX_SCOPE: bool = false;
    const COMMA_VS: CommaVs = CommaVs::BodyNegated;
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
}

/// The shared byte-at-a-time scanner. All hot state lives in fields
/// that [`Scanner::run`] copies into LOCALS for the whole loop (the
/// former hand-written loops kept state in register-allocatable
/// locals; a field-per-iteration machine measured +15–20% slower on
/// deeply-nested quote-free documents) and writes back once after the
/// loop.
struct Scanner<'a, C: ScanCfg> {
    input: &'a str,
    bytes: &'a [u8],
    open: u8,
    close: u8,
    body_object: bool,
    line_num: usize,
    span: Span,
    i: usize,
    depth: i32,
    in_key: bool,
    seg_start: bool,
    value_start: bool,
    raw: bool,
    prev: u8,
    stack: Vec<ScopeFrame>,
    segments: Vec<&'a str>,
    seg_at: usize,
    _cfg: std::marker::PhantomData<C>,
}

impl<'a, C: ScanCfg> Scanner<'a, C> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        input: &'a str,
        open: u8,
        close: u8,
        body_object: bool,
        line_num: usize,
        span: Span,
    ) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            open,
            close,
            body_object,
            line_num,
            span,
            i: 0,
            depth: 0,
            in_key: false,
            seg_start: false,
            value_start: false,
            raw: false,
            prev: open,
            stack: Vec::new(),
            segments: Vec::new(),
            seg_at: 0,
            _cfg: std::marker::PhantomData,
        }
    }

    fn run(&mut self) -> ScanStop {
        let input = self.input;
        let bytes = self.bytes;
        let len = bytes.len();
        let open = self.open;
        let close = self.close;
        let body_object = self.body_object;
        let mut i = self.i;
        let mut depth = self.depth;
        let mut in_key = self.in_key;
        let mut seg_start = self.seg_start;
        let mut value_start = self.value_start;
        let mut raw = self.raw;
        let mut prev = self.prev;
        let mut seg_at = self.seg_at;
        let mut stack = std::mem::take(&mut self.stack);
        let mut segments = std::mem::take(&mut self.segments);

        let stop = loop {
            if i >= len {
                break ScanStop::Exhausted;
            }

            // (1) quoted key segment start: only when quote tracking,
            // in a key position, at a fresh segment start (quirk 10).
            if C::TRACK_QUOTES && in_key && seg_start {
                i = skip_segment_ws(input, i);
                // The skip may consume every byte that remains: after
                // a trailing comma (or a dotted-key `.`) only
                // whitespace can follow, leaving `i` at EOF;
                // `bytes[i]` below then indexed out of bounds
                // (R3-F1).
                if i >= len {
                    break ScanStop::EofAfterWsSkip;
                }
                if is_quote_byte(bytes[i]) {
                    match quoted_span_end(bytes, i) {
                        Some(end) => {
                            // `prev` takes the closing quote byte —
                            // what a per-byte loop would leave in
                            // `prev` after the span (TRACK_RAW only).
                            if C::TRACK_RAW {
                                prev = bytes[end];
                            }
                            i = end + 1;
                            seg_start = false;
                            continue;
                        }
                        None => break ScanStop::UnterminatedQuote,
                    }
                }
                seg_start = false;
            }

            let b = bytes[i];
            match b {
                b'\\' => {
                    if C::VALIDATE_ESCAPES {
                        // § 5.2 rules-6–9 preamble: an invalid escape
                        // beats the rule 8/9 decision
                        // (`BadEscapeSequence` precedence). Advance by
                        // the FULL escape length (unlike find/split's
                        // `i += 2`, which suffices there because hex
                        // digits are not structural and validation
                        // happens later in `process_escapes`). `prev`
                        // becomes `\\` so an escaped `:` never forms a
                        // `::` raw marker (R5-F4).
                        match scan_escape(bytes, i) {
                            Err(seq) => {
                                break ScanStop::BadEscape(Error::Structured(
                                    ErrorKind::BadEscapeSequence {
                                        line: self.line_num as u32,
                                        span: self.span,
                                        sequence: seq,
                                    },
                                ))
                            }
                            Ok(esc) => {
                                // A recognized escape in value
                                // position consumes the scalar start
                                // (§ 3.7 / § 5.8.5, R4-F4): the
                                // decoded byte cannot reopen
                                // structural dispatch.
                                value_start = false;
                                if C::TRACK_RAW {
                                    prev = b'\\';
                                }
                                i += esc.len();
                                continue;
                            }
                        }
                    }
                    // find/split: skip the escaped character, no
                    // validation (validation happens later in
                    // `process_escapes`).
                    value_start = false;
                    i += 2;
                    continue;
                }
                b':' => {
                    if in_key {
                        // Key/value boundary: quotes after this are
                        // content.
                        in_key = false;
                        if C::COLON_SETS_VS {
                            value_start = true;
                        }
                    } else if C::TRACK_RAW && prev == b':' && value_start {
                        // `::` raw marker (§ 5.4 — in array bodies
                        // too): the rest of the item's value is a
                        // String — braces/brackets in it are content.
                        raw = true;
                    } else if C::TRACK_RAW {
                        // A lone `:` opening the value's scalar
                        // (§ 5.8.5, R5-F2) consumes the value
                        // position: a following `{`/`[` is literal
                        // content, not a nested opener. A `:`
                        // immediately followed by another `:` stays
                        // armed for the `::` marker check above on
                        // the next iteration.
                        if value_start && bytes.get(i + 1) != Some(&b':') {
                            value_start = false;
                        }
                    } else if C::COLON_ELSE_CLEAR_VS {
                        // `:` is never § 3.3 whitespace: split's old
                        // `_` fall-through for a non-key colon.
                        value_start = false;
                    }
                }
                b',' => {
                    if C::COMMA_SPLITS {
                        segments.push(&input[seg_at..i]);
                        seg_at = i + 1;
                    }
                    if C::COMMA_CTX_SCOPE {
                        // Next pair / item begins: re-derive key
                        // context from the CURRENT scope (the
                        // innermost opener on the stack), not the
                        // outermost one (R5-F1/R6-F1) — an Object
                        // scope starts a key position, an Array
                        // scope stays a value position (§ 5.3.3
                        // "Keys only").
                        let scope_object = stack.last().map_or(body_object, |f| f.kind() == b'{');
                        in_key = scope_object;
                        if C::TRACK_QUOTES {
                            seg_start = scope_object;
                        }
                    } else {
                        // split: the body kind decides (the stack is
                        // always empty at a split comma).
                        in_key = body_object;
                        if C::TRACK_QUOTES {
                            seg_start = body_object;
                        }
                    }
                    // An unescaped comma ALWAYS ends raw mode (R5-F3).
                    if C::COMMA_CLEARS_RAW {
                        raw = false;
                    }
                    match C::COMMA_VS {
                        CommaVs::Always => value_start = true,
                        CommaVs::BodyNegated => value_start = !body_object,
                    }
                }
                b'.' if C::TRACK_QUOTES && in_key => {
                    seg_start = true;
                }
                b'{' | b'[' => {
                    let gated_open = value_start && !raw;
                    if !gated_open {
                        // Mid-scalar (or raw-mode) opener: a literal
                        // byte with no structural meaning; balancing
                        // is irrelevant (R3-F4, § 5.8.5) — commas
                        // inside it still split.
                        value_start = false;
                    } else if C::OPENER_JUMP {
                        // split: sub-scan the candidate nested
                        // compound with its own pair. The matching
                        // closer is found with the value-start-aware
                        // `scan_inline_closer` (a mid-scalar opener
                        // inside the span must not count);
                        // `find_matching_close` would balance naive
                        // byte counts and skip spans that per
                        // § 5.8.5 are NOT one compound.
                        let (o, c) = if b == b'{' {
                            (b'{', b'}')
                        } else {
                            (b'[', b']')
                        };
                        if let InlineCloserScan::Found(pos) =
                            scan_inline_closer(&input[i..], o, c, self.line_num, self.span)
                        {
                            // Skip over the entire nested compound.
                            i += pos + 1;
                            value_start = false;
                            if C::TRACK_QUOTES {
                                seg_start = false;
                            }
                            continue;
                        }
                        // No closer inside the body — treat as
                        // literal byte (mid-value brace). Escapes are
                        // validated later by `process_escapes`, so a
                        // `BadEscape` scan result is deliberately not
                        // propagated here.
                        value_start = false;
                        i += 1;
                        continue;
                    } else {
                        // Count in place.
                        if C::DECREMENT_ANY_CLOSER || b == open {
                            depth += 1;
                        }
                        if C::USE_STACK {
                            // Save the enclosing key-position and raw
                            // state (§ 5.8.5, R4-F1 / R6-F1).
                            stack.push(ScopeFrame::pack(b, in_key, seg_start, raw));
                        }
                        // After an array's `[` the next position is
                        // still a value position (its first item,
                        // § 5.8.5); after a nested `{` comes key
                        // context. find never reads `value_start`, so
                        // this write is harmless there.
                        value_start = b == b'[';
                        if C::TRACK_QUOTES {
                            in_key = b == b'{';
                            seg_start = b == b'{';
                        } else if b == b'{' {
                            // scan-fast quirk: `[` leaves `in_key`
                            // untouched; the next `,` re-derives it
                            // (quirk 2).
                            in_key = true;
                        }
                    }
                }
                b'}' | b']' if !C::CLOSER_LITERAL => {
                    if !C::DECREMENT_ANY_CLOSER && b != close {
                        // find's other-kind closer: restore-only, no
                        // depth change.
                        if C::TRACK_QUOTES {
                            let want = if b == b']' { b'[' } else { b'{' };
                            if stack.last().is_some_and(|f| f.kind() == want) {
                                let f = stack.pop().unwrap();
                                in_key = f.saved_in_key();
                                seg_start = if C::RESTORE_SEG_ON_MATCH {
                                    f.saved_seg_start()
                                } else {
                                    // The closer itself consumed a
                                    // position, so segment-start
                                    // tracking stays off until the
                                    // next re-arm (quirk 3).
                                    false
                                };
                            } else {
                                seg_start = false;
                            }
                        }
                    } else {
                        // Both closer kinds decrement the shared
                        // depth: a nested compound of the OTHER
                        // delimiter type still closes (an array item
                        // may be an object and vice versa). An
                        // unescaped closer ALWAYS ends raw mode and is
                        // structural (R5-F3):
                        // `<inline-raw-scalar>` terminates on the
                        // FIRST unescaped `,`, `}`, or `]` regardless
                        // of which scope it belongs to — raw mode only
                        // makes leading openers literal (§ 5.8.5).
                        raw = false;
                        depth -= 1;
                        if depth == 0 {
                            // A closer that returns depth to zero
                            // must be the body's own closer; a crossed
                            // one (e.g. `[{a: 1]`) is not a matching
                            // closer (§ 5.2's matching-closer rule).
                            break ScanStop::Closer { idx: i, byte: b };
                        }
                        // Nested closer: pop it only when it matches
                        // the most recently opened compound kind
                        // (compare the stored OPENER to the closer's
                        // matching opener, R4-F2), so crossed closers
                        // don't corrupt tracking.
                        let want = if b == b']' { b'[' } else { b'{' };
                        if stack.last().is_some_and(|f| f.kind() == want) {
                            let f = stack.pop().unwrap();
                            if C::RESTORE_IN_KEY_ON_MATCH {
                                in_key = f.saved_in_key();
                            }
                            if C::TRACK_QUOTES {
                                seg_start = if C::RESTORE_SEG_ON_MATCH {
                                    f.saved_seg_start()
                                } else {
                                    // The `}`/`]` itself consumed a
                                    // position, so segment-start
                                    // tracking stays off until the
                                    // next re-arm (quirk 3).
                                    false
                                };
                                if C::RESTORE_RAW_ON_MATCH {
                                    // Per-scope raw tracking
                                    // (§ 5.8.5, R4-F1 / R4-F2).
                                    raw = f.saved_raw();
                                }
                            }
                        } else if C::MISMATCH_SEG_FALSE && C::TRACK_QUOTES {
                            seg_start = false;
                        }
                        // scan-SLOW only; scan-fast deliberately
                        // leaves `value_start` untouched (quirk 1).
                        if C::CLEAR_VS_ON_NESTED_CLOSE {
                            value_start = false;
                        }
                    }
                }
                _ => {
                    // § 3.3 whitespace: skip the whole code point
                    // without consuming the value-start position
                    // (R4-F3) — NBSP's 0xC2 lead byte must not clear
                    // `value_start`, and a per-byte `i += 1` would
                    // strand its 0xA0 continuation byte mid-code-point.
                    // `prev` takes the run's LAST byte (TRACK_RAW
                    // only) — whitespace bytes are never `:` or `\`,
                    // so `::` raw-marker detection sees exactly what
                    // the per-byte loop would see.
                    if let Some(ws_len) = inline_whitespace_at(input, i) {
                        if C::TRACK_RAW {
                            prev = bytes[i + ws_len - 1];
                        }
                        i += ws_len;
                        continue;
                    }
                    // Non-whitespace content consumes the value-start
                    // position.
                    value_start = false;
                }
            }
            prev = b;
            i += 1;
        };

        self.i = i;
        self.depth = depth;
        self.in_key = in_key;
        self.seg_start = seg_start;
        self.value_start = value_start;
        self.raw = raw;
        self.prev = prev;
        self.seg_at = seg_at;
        self.stack = stack;
        self.segments = segments;
        stop
    }
}

// ---------------------------------------------------------------------------
// Splitting on top-level commas (wrappers over the shared machine)
// ---------------------------------------------------------------------------

/// Split `input` on unescaped `,` at nesting depth 0.
///
/// Unlike a naive brace-counting approach, this correctly handles the
/// section 5.8.5 "mid-value brace literal" rule: only a `{` or `[` that
/// is the first non-whitespace code point of a VALUE is skipped over as
/// a nested compound. A mid-scalar `{`/`[` is literal data even when it
/// happens to balance — it neither nests nor shields the commas inside
/// it, which still split. A value-start opener without a matching
/// closer is treated as literal (the value parser will handle it later
/// per the mid-value-brace rule).
///
/// In [`InlineBody::Object`] mode, quoted key segments (spec 0.7
/// § 5.3.3) are opaque to comma splitting — `{"a,b": 1, c: 2}`
/// splits into two pairs — while quotes in value positions are
/// ordinary content (`a: "x,y", b: 2` splits inside the quotes). An
/// unterminated quoted key segment raises `UnterminatedInlineCompound`.
// (`input: &str` instead of `<'a>(input: &'a str)`) — type-identical,
// no call-site changes.
pub(crate) fn split_top_level(
    input: &str,
    line_num: usize,
    span: Span,
    body: InlineBody,
) -> Result<Vec<&str>, Error> {
    let bytes = input.as_bytes();
    if body == InlineBody::Array || !has_quote_bytes(bytes) {
        return Ok(split_top_level_fast(input, line_num, span, body));
    }

    // Slow path (object body with quote bytes): track key/value and
    // segment-start state so quoted KEYS are comma-opaque.
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, SplitQ> = Scanner::new(input, open, close, object, line_num, span);
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::UnterminatedQuote => {
            // Unterminated quoted key segment (§ 5.3.3).
            Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            }))
        }
        ScanStop::EofAfterWsSkip => {
            // The skip may consume every byte that remains: after a
            // trailing comma (or a dotted-key `.`) only whitespace can
            // follow (R3-F1). Two possible outcomes:
            // Inline view trim: LF/CR cannot occur — this scanner works
            // within one § 3.2-pre-split line.
            if input[sc.seg_at..]
                .trim_matches(is_inline_whitespace)
                .is_empty()
            {
                // Only whitespace after the last comma: a valid
                // trailing comma — emit no final segment (the
                // callers treat an empty last segment identically).
                Ok(sc.segments)
            } else {
                // A `.` armed this segment start and only whitespace
                // followed: a dotted key whose final segment is empty
                // (`b.` / `.`) — EmptyKey, the same category
                // `insert_value` raises for `a.: 1` (spec 0.7 § 6.5).
                Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }))
            }
        }
        ScanStop::Exhausted => {
            // Last segment (after final comma, or the whole string if
            // no comma).
            sc.segments.push(&input[sc.seg_at..]);
            Ok(sc.segments)
        }
        ScanStop::Closer { .. } | ScanStop::BadEscape(_) => {
            unreachable!("split scanner cannot stop on a closer or bad escape")
        }
    }
}

/// Quote-free fast path for [`split_top_level`] (also serving object
/// bodies without quote bytes). Implements the § 5.8.5 value-start
/// rule: only a `{`/`[` at the first non-whitespace code point of a
/// value nests (skipped via the value-start-aware
/// `scan_inline_closer`); a mid-scalar opener is a literal byte even
/// when balanced, and a mid-scalar closer byte likewise has no
/// structural effect — commas after it still split.
/// In an object body a value starts only after `:`; in an array body
/// the body start and every position after `,` are value positions
/// (§ 5.3.3 "Keys only").
fn split_top_level_fast(input: &str, line_num: usize, span: Span, body: InlineBody) -> Vec<&str> {
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, SplitFast> = Scanner::new(input, open, close, object, line_num, span);
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::Exhausted => {
            // Last segment (after final comma, or the whole string if
            // no comma).
            sc.segments.push(&input[sc.seg_at..]);
            sc.segments
        }
        ScanStop::Closer { .. } => {
            unreachable!("quote-free split scanner cannot stop on a closer")
        }
        ScanStop::BadEscape(_) => {
            unreachable!("quote-free split scanner never validates escapes")
        }
        ScanStop::UnterminatedQuote | ScanStop::EofAfterWsSkip => {
            unreachable!("quote-free split scanner tracks no quoted key segments")
        }
    }
}
// ---------------------------------------------------------------------------
// Delimiter matching helpers
// ---------------------------------------------------------------------------

/// Check if `input` is a balanced inline compound: starts with `open`
/// and has a matching `close` at the very end. Returns the last byte
/// index if found.
///
/// For object bodies (`open == b'{'`), brackets inside quoted key
/// segments are opaque to bracket-balance counting (spec 0.7 § 5.3.3:
/// same reason an escaped bracket is). Array bodies' own positions are
/// value positions, so quotes there are content (§ 5.3.3 "Keys only"),
/// but Object scopes nested inside still track key positions — a quoted
/// key segment of a nested object is opaque to `]` counting too (R3-F2:
/// the gate is per nested scope, not the outer opener). Key-position
/// tracking is per nesting level, and BOTH compound kinds open a scope
/// whose kind decides what a `,` begins (R6-F1): an Object scope's comma
/// starts a fresh key position, an Array scope's comma stays a value
/// position, and the enclosing level's key context is saved and restored
/// around each nested body.
///
/// Per R7-F1, openers nest only at a value position (§ 5.8.5); `::`
/// raw values and quoted-key spans are honored exactly as in
/// [`scan_inline_closer`] — the only differences are the result shape
/// ([`Option<usize>`] instead of [`InlineCloserScan`], no error
/// construction) and that escapes are never validated here.
pub(crate) fn find_matching_close(input: &str, open: u8, close: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != open {
        return None;
    }

    // line_num/span: find's configs never build errors, so the
    // placeholder 0 / `Span::EMPTY` values are never observed.
    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking.
        run_find::<FindFast>(input, open, close, open == b'{')
    } else {
        // Slow path (quote bytes present): byte-identical rules for
        // object and array bodies (R7-F1) — key-position tracking is
        // per nested Object scope inside the machine itself.
        run_find::<FindQ>(input, open, close, open == b'{')
    }
}

fn run_find<C: ScanCfg>(input: &str, open: u8, close: u8, object: bool) -> Option<usize> {
    let mut sc: Scanner<'_, C> = Scanner::new(input, open, close, object, 0, Span::EMPTY);
    // Same seed state as `run_scan` (R7-F1): find is the identical
    // machine minus escape validation, so `Some(idx)` here is exactly
    // `scan_inline_closer`'s `Found(idx)` whenever no bad escape
    // intervenes.
    sc.i = 1;
    sc.depth = 1;
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::Closer { idx, byte } => (byte == close).then_some(idx),
        ScanStop::EofAfterWsSkip
        | ScanStop::UnterminatedQuote
        | ScanStop::BadEscape(_)
        | ScanStop::Exhausted => None,
    }
}
/// Find the first unescaped `:` in `s` that is at nesting depth 0.
/// Used to split inline pairs into key and value.
///
/// Quote-aware (spec 0.7 § 5.3.3): the content of a quoted key segment
/// opened at a segment-start position is opaque to both `:` and the
/// `{`/`[`/`}`/`]` depth counting. A span that never closes swallows
/// the rest of the input — `None` is returned and the caller maps that
/// to its unterminated/unparseable error of choice.
pub(crate) fn find_unescaped_colon_inline(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if !has_quote_bytes(bytes) {
        // Fast path: no quote bytes — the pre-0.7 loop, unchanged.
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
        return None;
    }
    // Slow path (quote bytes present): quoted segments are opaque.
    let mut depth: i32 = 0;
    let mut i = 0;
    let mut seg_start = true;
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            // The skip may consume every remaining byte (text ending in
            // whitespace after a dotted-key `.`); `bytes[i]` below then
            // indexed out of bounds (R3-F1). No colon exists after EOF.
            if i >= bytes.len() {
                return None;
            }
            if is_quote_byte(bytes[i]) {
                // Unterminated span: the rest of the input is segment
                // content.
                let end = quoted_span_end(bytes, i)?;
                i = end + 1;
                seg_start = false;
                continue;
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            b'.' => seg_start = true,
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

/// Error for § 5.2 rule 8's "closer followed by content" shape: the
/// scan found a matching closer, but not at the last byte of the body.
pub(crate) fn malformed_closer_not_at_end(line_num: usize, span: Span) -> Error {
    malformed(
        line_num,
        span,
        "matching closer is not the last byte of the body; non-whitespace content follows the closed inline compound",
    )
}

/// Outcome of the § 5.2 rules 6–9 same-line closer scan for a body
/// beginning with `{` or `[`.
pub(crate) enum InlineCloserScan {
    /// A matching closer (unescaped, depth back to zero) at this byte
    /// offset.
    Found(usize),
    /// No matching closer anywhere on the line — § 5.2 rule 9. Includes
    /// the unterminated-quoted-key case: § 6.16 keeps that
    /// `UnterminatedInlineCompound`, so the scan stops there without
    /// validating anything inside the swallowed span.
    NotFound,
    /// An invalid escape was met while scanning outside quoted
    /// segments — § 5.2 rules 6–9 preamble gives `BadEscapeSequence`
    /// precedence over the rule 8/9 decision. The payload is the
    /// ready-to-return error.
    BadEscape(Error),
}

/// Scan `input` (which MUST start with `open`) with § 5.8's quote-aware,
/// escape-aware delimiter rules and report the matching `close`, for the
/// § 5.2 rules 6–9 dispatch. Shares `find_matching_close`'s quote /
/// key-position state machine (quote tracking whenever quote bytes
/// are present — per nested Object scope, so an Array body still
/// becomes quote-aware inside a nested `{`; per-level key-position
/// tracking; unterminated quoted segment ⇒ `NotFound`) and
/// additionally validates every `\X` outside quoted
/// segments via [`scan_escape`] so `BadEscapeSequence` can take
/// precedence.
///
/// Unlike `find_matching_close`, openers only nest at a VALUE position
/// (§ 5.8.5 mid-value brace rule: only the first non-ws byte of a value
/// decides compound-vs-literal), and a `::` raw marker makes the rest of
/// the pair's value literal — so mid-value braces and raw strings never
/// swallow the body's matching closer. Raw tracking is per scope
/// (§ 5.8.5, R4-F1): a raw value is terminated only by the CURRENT
/// scope's own unescaped delimiter, and closers restore the enclosing
/// scope's key context (R4-F2).
pub(crate) fn scan_inline_closer(
    input: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
) -> InlineCloserScan {
    let bytes = input.as_bytes();
    let object = open == b'{';
    // The caller guarantees `input` starts with `open`; the opener
    // itself is depth 1, so scanning starts at byte 1.
    if bytes.is_empty() || bytes[0] != open {
        return InlineCloserScan::NotFound;
    }
    // R3-F2: the gate is "any quote byte anywhere" — quote tracking
    // itself is per nested Object scope in the machine, not per outer
    // opener.
    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking. `value_start` marks an
        // unconsumed value position (body start in arrays — including
        // the position right after the array's `[`, which is its first
        // item — and after `:`/`,` otherwise); per § 5.8.5 a `{`/`[`
        // only nests there.
        run_scan::<ScanFast>(input, open, close, object, line_num, span)
    } else {
        // Slow path (quote bytes present): the per-level key-position
        // machine plus the same value-position / raw-marker tracking.
        // Object bodies start at their first key segment; array bodies
        // start at their first item — a value position with no key
        // context (§ 5.3.3 "Keys only").
        run_scan::<ScanQ>(input, open, close, object, line_num, span)
    }
}

fn run_scan<C: ScanCfg>(
    input: &str,
    open: u8,
    close: u8,
    object: bool,
    line_num: usize,
    span: Span,
) -> InlineCloserScan {
    let mut sc: Scanner<'_, C> = Scanner::new(input, open, close, object, line_num, span);
    sc.i = 1;
    sc.depth = 1;
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        // § 5.2's matching-closer rule: a closer that returns depth to
        // zero must be the body's own closer kind (a crossed one, e.g.
        // `[{a: 1]`, is not a matching closer).
        ScanStop::Closer { idx, byte } => {
            if byte == close {
                InlineCloserScan::Found(idx)
            } else {
                InlineCloserScan::NotFound
            }
        }
        // Includes the unterminated-quoted-key case: § 6.16 keeps that
        // `UnterminatedInlineCompound` upstream, so the scan stops
        // there without validating anything inside the swallowed span.
        ScanStop::EofAfterWsSkip | ScanStop::UnterminatedQuote | ScanStop::Exhausted => {
            InlineCloserScan::NotFound
        }
        ScanStop::BadEscape(e) => InlineCloserScan::BadEscape(e),
    }
}
