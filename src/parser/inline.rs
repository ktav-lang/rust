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
    bounds: InlineBounds<'_>,
) -> Result<Value, Error> {
    // R10-F1: quote presence is computed ONCE over the root body and
    // threaded down — every nested slice is a substring of this body,
    // so a quote byte at the root implies one in every descendant
    // (false positives are harmless: the fast and quote-aware machines
    // are byte-identical on quote-free slices, R8-F2).
    let has_quotes = has_quote_bytes(input.as_bytes());
    parse_inline_object_inner(input, line_num, span, 0, strict, bounds, has_quotes)
}

/// Parse a balanced inline array body. `input` is the full body
/// including the outer `[` and `]`. Returns `Value::Array`.
pub(crate) fn parse_inline_array(
    input: &str,
    line_num: usize,
    span: Span,
    strict: bool,
    bounds: InlineBounds<'_>,
) -> Result<Value, Error> {
    // R10-F1: quote presence computed once at the root (see
    // `parse_inline_object`).
    let has_quotes = has_quote_bytes(input.as_bytes());
    parse_inline_array_inner(input, line_num, span, 0, strict, bounds, has_quotes)
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
    bounds: InlineBounds<'_>,
    has_quotes: bool,
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

    let segments = split_top_level(
        inner,
        line_num,
        span,
        InlineBody::Object,
        bounds,
        has_quotes,
    )?;

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
            parse_inline_value(
                value_body, line_num, span, depth, strict, bounds, has_quotes,
            )?
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
    bounds: InlineBounds<'_>,
    has_quotes: bool,
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

    let segments = split_top_level(inner, line_num, span, InlineBody::Array, bounds, has_quotes)?;

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

        // Parse inline value (could be nested compound or scalar)
        let value =
            parse_inline_value_raw(trimmed, line_num, span, depth, strict, bounds, has_quotes)?;
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
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Value, Error> {
    // Inline view trim: LF/CR cannot occur (§ 3.2-pre-split line).
    let trimmed = body.trim_matches(is_inline_whitespace);
    if trimmed.is_empty() {
        // Empty value after `:` → empty String
        return Ok(Value::String("".into()));
    }
    parse_inline_value_raw(trimmed, line_num, span, depth, strict, bounds, has_quotes)
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
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Value, Error> {
    let first_byte = trimmed.as_bytes()[0];

    if first_byte == b'{' {
        // § 6.11/§ 6.12 tri-state. Since R7-F1 `find_matching_close`
        // applies the SAME § 5.8.5 value-start / raw-mode / quoted-key
        // rules as `scan_inline_closer`; it differs only in never
        // validating escapes. The fallback exists so a
        // `BadEscapeSequence` (§ 6.13) still takes precedence over the
        // unterminated decision when no closer is found at all.
        // A gate-scan memo hit (`bounds.known_closer`) short-circuits both
        // scans with the identical verdict; the fallback keeps the live
        // find-first dispatch — and with it the `BadEscapeSequence`
        // precedence — for every non-memo span.
        let close_idx = match bounds.known_closer(trimmed) {
            // Gate-scan memo: identical to what the dispatch below computes.
            Some(idx) => Some(idx),
            None => match find_matching_close(trimmed, b'{', b'}') {
                Some(close) => Some(close),
                None => match scan_inline_closer(trimmed, b'{', b'}', line_num, span) {
                    InlineCloserScan::Found(idx) => Some(idx),
                    InlineCloserScan::NotFound => None,
                    InlineCloserScan::BadEscape(err) => return Err(err),
                },
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
                return parse_inline_object_inner(
                    trimmed,
                    line_num,
                    span,
                    depth + 1,
                    strict,
                    bounds,
                    has_quotes,
                );
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
        let close_idx = match bounds.known_closer(trimmed) {
            // Gate-scan memo: identical to what the dispatch below computes.
            Some(idx) => Some(idx),
            None => match find_matching_close(trimmed, b'[', b']') {
                Some(close) => Some(close),
                None => match scan_inline_closer(trimmed, b'[', b']', line_num, span) {
                    InlineCloserScan::Found(idx) => Some(idx),
                    InlineCloserScan::NotFound => None,
                    InlineCloserScan::BadEscape(err) => return Err(err),
                },
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
                return parse_inline_array_inner(
                    trimmed,
                    line_num,
                    span,
                    depth + 1,
                    strict,
                    bounds,
                    has_quotes,
                );
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
///
/// R10-F1: one call hands a slice of up to `bytes.len()` bytes to each
/// of the three `contains` passes, so the true byte-view count of one
/// call is between 1x and 3x the length recorded by
/// [`ix_probe::record_quote_prescan`]; comparisons across parser
/// versions hold because the factor is the same.
pub(crate) fn has_quote_bytes(bytes: &[u8]) -> bool {
    #[cfg(test)]
    ix_probe::record_quote_prescan(bytes.len());
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
///
/// R10-F1: candidate-driven instead of a full quote-prescan plus a
/// byte-at-a-time walk over the WHOLE pair text. The escape-aware fast
/// scan (`find_unescaped_colon_fast`, memchr) proposes the first
/// unescaped `:` candidate; only that candidate's KEY PREFIX
/// (`s[..cand]`, never the value) is checked for quote-opacity. A
/// candidate whose prefix ends outside every quoted segment is the
/// separator; a candidate inside a closed-or-open quoted span resumes
/// after the span's closer (never at a segment start); a span that
/// never closes is `UnterminatedQuote`. The full quote-aware walk
/// ([`scan_unescaped_colon_slow`]) remains only for the corner where
/// no unescaped candidate exists at all but quote bytes are present —
/// the `Absent` vs `UnterminatedQuote` distinction there does not
/// depend on finding a colon. Escape-awareness is carried by the fast
/// scan itself (`\` consumes the next byte); the prefix walk mirrors
/// the slow machine byte for byte over its range.
pub(crate) fn scan_unescaped_colon(s: &str) -> ColonScan {
    let bytes = s.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = find_unescaped_colon_fast(&s[from..]) {
        let cand = from + rel;
        match key_prefix_quote_state(s, from, cand) {
            KeyPrefixQuote::Opaque => return ColonScan::Found(cand),
            KeyPrefixQuote::OpenSegment { resume } => from = resume,
            KeyPrefixQuote::Unterminated => return ColonScan::UnterminatedQuote,
        }
    }
    // No further escape-aware colon candidate anywhere. Without quote
    // bytes nothing can be unterminated; with them the slow walk
    // decides Absent vs UnterminatedQuote (it cannot return `Found`:
    // the fast scan just proved no unescaped `:` exists).
    if has_quote_bytes(bytes) {
        return scan_unescaped_colon_slow(s);
    }
    ColonScan::Absent
}

/// Quote-state of `s[from..cand]` at the candidate offset `cand`
/// (R10-F1): is a `<quoted-segment>` open there, did one open at a
/// segment-start position and never close, or is the candidate outside
/// every segment? Walks ONLY the key prefix `[from, cand)` — the
/// value's own quotes and colons are irrelevant to where the key ends.
/// `from` is either 0 (segment start, the whole text's beginning) or
/// one past a quoted span's closer (NOT a segment start) — the same
/// resume state [`scan_unescaped_colon_slow`] carries.
enum KeyPrefixQuote {
    /// No quoted segment is open at the candidate offset.
    Opaque,
    /// A quoted segment is open at the candidate; `resume` is one past
    /// its closing quote.
    OpenSegment { resume: usize },
    /// A segment-start quote whose span never closes: the candidate is
    /// inside it (§ 5.3.3 "Unterminated quoted segments").
    Unterminated,
}

fn key_prefix_quote_state(s: &str, from: usize, cand: usize) -> KeyPrefixQuote {
    let bytes = s.as_bytes();
    // No quote byte in the prefix: no segment can open (openings need a
    // quote byte at a segment start) and none can be unterminated.
    if !has_quote_bytes(&bytes[from..cand]) {
        return KeyPrefixQuote::Opaque;
    }
    let mut i = from;
    let mut seg_start = from == 0;
    while i < cand {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < cand && is_quote_byte(bytes[i]) {
                return match quoted_span_end(bytes, i) {
                    // The colon is inside this span iff the span reaches
                    // past it (the closer cannot sit ON `cand` — that
                    // byte is `:`). A span closing before the candidate
                    // just resumes the walk after its closer.
                    Some(end) if end > cand => KeyPrefixQuote::OpenSegment { resume: end + 1 },
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => KeyPrefixQuote::Unterminated,
                };
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape lead: consume the escaped byte too (a lone trailing `\` overshoots the prefix, which the loop guard makes safe — the colon at `cand` is unescaped, so this cannot hide it)
            b'.' => {
                seg_start = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    KeyPrefixQuote::Opaque
}

/// The pre-R10-F1 full quote-aware walk (spec 0.7 § 4 + § 5.3.3), kept
/// for `scan_unescaped_colon`'s no-candidate corner: byte-at-a-time
/// segment tracking over the WHOLE text. Precondition: the caller has
/// established that no unescaped `:` candidate exists
/// ([`find_unescaped_colon_fast`] over the whole text returned `None`),
/// so this can only return `Absent` or `UnterminatedQuote`.
fn scan_unescaped_colon_slow(s: &str) -> ColonScan {
    let bytes = s.as_bytes();
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

/// Precomputed (opener, closer) byte offsets of every nested compound
/// span inside one top-level inline body, relative to the body's first
/// byte, sorted by opener. Produced by the § 5.2 gate scan
/// ([`scan_inline_closer_with_bounds`]); consumed as a pure memo by
/// split's opener jump and the per-value tri-state dispatch.
///
/// Purity invariant: a recorded pair `(o, c)` exists iff
/// [`scan_inline_closer`] over the same span returns `Found(c - o)`.
/// Two properties make this hold:
///
/// 1. ENTRY STATE. The recording walk and the standalone scan are the
///    same machine, and at the span's opener their states agree: same
///    seeds, and the opener arm re-derives in_key/seg_start/value_start/
///    prev identically in BOTH modes (R8-F2 closed the former
///    fast-only residue quirks), and gated openers always run with
///    `raw = false`. On a quote-free slice the fast and quote-aware
///    machines are byte-identical — no quote byte can arm the
///    quote/segment arms and every remaining policy const coincides —
///    so a recorded span is pure for EITHER consumer slice shape (the
///    bounded value and the split suffix), whichever mode the slice's
///    own byte set selects at dispatch time. Recording is
///    unconditional at push; the pop-time gates below still kill any
///    span whose own stop byte says the standalone would disagree.
/// 2. STOP BYTE. The standalone scan's counter is the walk's shared
///    `depth` offset by the value captured at the opener, so it returns
///    to zero exactly where global `depth` returns to `entry - 1`. That
///    byte is the standalone's stop, and the top frame is always the
///    span in question there (frames opened above die at strictly
///    higher depths). A kind-matched closer at that byte is what the
///    pop records; a crossed closer there is where the standalone
///    stops with `NotFound`, so the frame is marked dead and never
///    records, however far the walk continues past it — a later
///    kind-matched pop cannot satisfy `depth == entry - 1` (its depth
///    is strictly below), which is the second gate on recording.
///
/// No `unsafe`: every pointer use is an `as usize`
/// comparison/subtraction on slices of the same allocation — every
/// slice passed to [`InlineBounds::known_closer`] descends from the
/// body the bounds were built over, so the subtraction stays within
/// one allocation.
#[derive(Clone, Copy)]
pub(crate) struct InlineBounds<'a> {
    origin: usize, // address of the top body's first byte (coordinate origin)
    pairs: &'a [(usize, usize)],
}

impl<'a> InlineBounds<'a> {
    /// Empty bounds whose coordinate origin is `input` itself (gate
    /// scans and find: they never consult, and `input_off` becomes 0).
    pub(crate) fn for_input(input: &str) -> Self {
        InlineBounds {
            origin: input.as_ptr() as usize,
            pairs: &[],
        }
    }

    /// Bounds over `top`, offsets relative to `top`'s first byte.
    pub(crate) fn over<'b>(top: &str, pairs: &'b [(usize, usize)]) -> InlineBounds<'b> {
        InlineBounds {
            origin: top.as_ptr() as usize,
            pairs,
        }
    }

    /// If `s`'s first byte opens a recorded span whose closer lies
    /// within `s`, the closer's offset relative to `s`. `Some` here is
    /// byte-for-byte what the find-first dispatch's
    /// `find_matching_close` would return (identical machines; a
    /// recorded span provably contains no bad escape, the only
    /// find/scan divergence). `None` — no record, or the recorded
    /// closer falls beyond this body — means "not a memo for this
    /// slice": the caller MUST run the live dispatch.
    pub(crate) fn known_closer(&self, s: &str) -> Option<usize> {
        let off = s.as_ptr() as usize - self.origin;
        #[cfg(test)]
        if ix_probe::bypass_engaged() {
            return None;
        }
        #[cfg(test)]
        let idx = {
            // Counted twin of `binary_search_by_key` — the identical
            // probe closure `binary_search_by_key` itself uses — so
            // test builds see the same iteration sequence plus its
            // step count. Non-test builds keep the original line.
            let mut steps = 0usize;
            let idx = self.pairs.binary_search_by(|p| {
                steps += 1;
                p.0.cmp(&off)
            });
            ix_probe::record_kc_lookup(steps);
            idx
        };
        #[cfg(not(test))]
        let idx = self.pairs.binary_search_by_key(&off, |p| p.0);
        let idx = idx.ok()?;
        let rel = self.pairs[idx].1 - off;
        let hit = (rel < s.len()).then_some(rel);
        #[cfg(test)]
        if hit.is_some() {
            ix_probe::record_kc_hit();
        }
        hit
    }

    /// Closer absolute offset of a recorded span whose opener sits at
    /// absolute offset `abs`, for split's opener jump.
    fn opener_close_at(&self, abs: usize) -> Option<usize> {
        #[cfg(test)]
        if ix_probe::bypass_engaged() {
            return None;
        }
        #[cfg(test)]
        let idx = {
            let mut steps = 0usize;
            let idx = self.pairs.binary_search_by(|p| {
                steps += 1;
                p.0.cmp(&abs)
            });
            ix_probe::record_oca_lookup(steps);
            idx
        };
        #[cfg(not(test))]
        let idx = self.pairs.binary_search_by_key(&abs, |p| p.0);
        let idx = idx.ok()?;
        #[cfg(test)]
        ix_probe::record_oca_hit();
        Some(self.pairs[idx].1)
    }
}

// R8-F6 measurement-only counters. Compiled ONLY under `cfg(test)`; a
// release build carries none of this. They record the deterministic
// operation counts of the `InlineBounds` index — binary-search steps
// per lookup site (calls/hits/total/max steps), the boundary sort
// (calls/elements/comparisons), and the recorded bodies themselves
// (count, total body bytes, total and max recorded pairs) — so the
// index's cost can be measured against the O(N) walk instead of
// guessed at. See the ix_probe_* tests in src/parser/tests.rs.
//
// R9-F2: the state is THREAD-LOCAL. The former process-global atomics
// let any concurrent parse bump the counters and let a probe's BYPASS
// flip flip the lookup path of every other test in the process, so no
// probe's snapshot window was isolated from the rest of the suite (a
// mutex held only by probe tests does not close that interleaving).
// Each thread now sees only its own window, and `set_bypass` returns
// an RAII guard so an unwinding test cannot leak the bypassed mode to
// the next test on the same thread (the timing probes are `#[ignore]`d
// and run with `--test-threads=1`).
#[cfg(test)]
pub(crate) mod ix_probe {
    use std::cell::RefCell;

    #[derive(Clone, Copy, Default)]
    struct State {
        bypass: bool,
        kc_calls: u64,
        kc_hits: u64,
        kc_steps: u64,
        kc_steps_max: u64,
        oca_calls: u64,
        oca_hits: u64,
        oca_steps: u64,
        oca_steps_max: u64,
        sort_calls: u64,
        sort_elems: u64,
        sort_cmps: u64,
        bodies: u64,
        body_bytes: u64,
        pairs_total: u64,
        pairs_max: u64,
        hq_calls: u64,
        hq_bytes: u64,
        hq_max: u64,
    }

    thread_local! {
        static STATE: RefCell<State> = RefCell::new(State::default());
    }

    fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with(|s| f(&mut s.borrow_mut()))
    }

    /// RAII: restores `bypass = false` when dropped (even on unwind),
    /// so the bypassed mode cannot leak into another test sharing this
    /// thread under `--test-threads=1`.
    pub(crate) struct BypassGuard;

    impl Drop for BypassGuard {
        fn drop(&mut self) {
            with_state(|s| s.bypass = false);
        }
    }

    /// R8-F6 A/B switch: when set, both lookup sites return `None`
    /// without searching, forcing callers down the live-dispatch
    /// fallback (the memo is a proven pure memo — parser::tests
    /// `memo_bounds_are_a_pure_memo_of_the_live_dispatches` — so the
    /// parse result is byte-identical; only the index is skipped).
    /// Recording and sorting still happen. NOTE (R9-F3): this measures
    /// the memo's benefit (cached vs live re-scan), not the isolated
    /// cost of the binary search. Returns an RAII guard restoring
    /// `bypass = false` on drop.
    pub(crate) fn set_bypass(on: bool) -> BypassGuard {
        with_state(|s| s.bypass = on);
        BypassGuard
    }

    pub(crate) fn bypass_engaged() -> bool {
        with_state(|s| s.bypass)
    }

    /// One lookup's binary-search step count (loop iterations = element
    /// comparisons) plus whether it was a call/hit at all.
    pub(crate) fn record_kc_lookup(steps: usize) {
        with_state(|s| {
            s.kc_calls += 1;
            s.kc_steps += steps as u64;
            s.kc_steps_max = s.kc_steps_max.max(steps as u64);
        });
    }

    pub(crate) fn record_kc_hit() {
        with_state(|s| s.kc_hits += 1);
    }

    pub(crate) fn record_oca_lookup(steps: usize) {
        with_state(|s| {
            s.oca_calls += 1;
            s.oca_steps += steps as u64;
            s.oca_steps_max = s.oca_steps_max.max(steps as u64);
        });
    }

    pub(crate) fn record_oca_hit() {
        with_state(|s| s.oca_hits += 1);
    }

    pub(crate) fn record_sort(elems: usize, cmps: usize) {
        with_state(|s| {
            s.sort_calls += 1;
            s.sort_elems += elems as u64;
            s.sort_cmps += cmps as u64;
        });
    }

    pub(crate) fn record_body(input_len: usize, pairs: usize) {
        with_state(|s| {
            s.bodies += 1;
            s.body_bytes += input_len as u64;
            s.pairs_total += pairs as u64;
            s.pairs_max = s.pairs_max.max(pairs as u64);
        });
    }

    /// R10-F1: one quote-presence prescan (`has_quote_bytes`): `len` is
    /// the slice length handed to the check (up to three `contains`
    /// passes each). These are the PREscans — the repeated full-subtree
    /// quote checks this module's lookup counters do not see.
    pub(crate) fn record_quote_prescan(len: usize) {
        with_state(|s| {
            s.hq_calls += 1;
            s.hq_bytes += len as u64;
            s.hq_max = s.hq_max.max(len as u64);
        });
    }

    #[derive(Default, Clone, Copy)]
    pub(crate) struct Snapshot {
        pub kc_calls: u64,
        pub kc_hits: u64,
        pub kc_steps: u64,
        pub kc_steps_max: u64,
        pub oca_calls: u64,
        pub oca_hits: u64,
        pub oca_steps: u64,
        pub oca_steps_max: u64,
        pub sort_calls: u64,
        pub sort_elems: u64,
        pub sort_cmps: u64,
        pub bodies: u64,
        pub body_bytes: u64,
        pub pairs_total: u64,
        pub pairs_max: u64,
        pub hq_calls: u64,
        pub hq_bytes: u64,
        pub hq_max: u64,
    }

    pub(crate) fn snapshot() -> Snapshot {
        STATE.with(|s| {
            let s = s.borrow();
            Snapshot {
                kc_calls: s.kc_calls,
                kc_hits: s.kc_hits,
                kc_steps: s.kc_steps,
                kc_steps_max: s.kc_steps_max,
                oca_calls: s.oca_calls,
                oca_hits: s.oca_hits,
                oca_steps: s.oca_steps,
                oca_steps_max: s.oca_steps_max,
                sort_calls: s.sort_calls,
                sort_elems: s.sort_elems,
                sort_cmps: s.sort_cmps,
                bodies: s.bodies,
                body_bytes: s.body_bytes,
                pairs_total: s.pairs_total,
                pairs_max: s.pairs_max,
                hq_calls: s.hq_calls,
                hq_bytes: s.hq_bytes,
                hq_max: s.hq_max,
            }
        })
    }

    pub(crate) fn reset() {
        with_state(|s| *s = State::default());
    }
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
// Preserved-quirk inventory (all LOAD-BEARING current behavior).
// Former quirks 1-2 were REMOVED in R8-F2 — they made the fast and
// quote-aware machines disagree on the meaning of the same quote-free
// bytes (a `[` after a closed empty Array stayed a structural opener
// in fast mode, and key-position residue leaked into Array spans), so
// a memo recorded by one mode could lie about the other:
// 1. REMOVED R8-F2: every find/scan config now clears `value_start`
//    at a nested close (`CLEAR_VS_ON_NESTED_CLOSE`) — the closed
//    compound consumed the value position (§ 5.8.5).
// 2. REMOVED R8-F2: every config now sets `in_key = (b == b'{')` at
//    an opener and restores it from a kind-matched pop.
// 3. split maps an unterminated quoted key to
//    `UnterminatedInlineCompound`, and dotted-key-then-EOF to
//    whitespace-only-rest → no trailing segment, else `EmptyKey`;
//    find/scan map both to `None`/`NotFound`.
// 4. REMOVED R9-F1: every config now derives the comma's value_start
//    from the SAME scope kind as its in_key — after an Object comma
//    that position is a key position (§ 4: <inline-pair> begins with
//    <key>), so `value_start = !scope_kind`; an Array comma stays a
//    value position.
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
    /// every find/scan config (R8-F2, was fast-only quirk 1): a
    /// nested closer clears `value_start` — the closed compound
    /// consumed the value position (§ 5.8.5), so a following
    /// `{`/`[` is content after a closed value, not a new opener.
    const CLEAR_VS_ON_NESTED_CLOSE: bool;
    /// scan configs only: record every nested (opener, closer) pair
    /// into `pairs_out` as the walk pops each scope. Pure side output —
    /// changes no decision. Recording is only PURE for spans whose
    /// entry state equals the standalone scan's seed (see
    /// [`InlineBounds`]), which is why only the gate scans record.
    const RECORD_BOUNDS: bool;
}

/// `find_matching_close`, quote-free fast path: byte-for-byte the
/// [`ScanFast`] machine minus escape validation (R7-F1). `in_key` and
/// `seg_start` are maintained with the same semantics as [`FindQ`]
/// (R8-F2) even though no quote byte can exist: `in_key` still
/// classifies a `:` as key-separator vs value content, and find must
/// mean exactly what the quote-aware machine means by every byte.
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
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = false;
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
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = false;
}

/// `scan_inline_closer`, quote-free fast path. `value_start` marks an
/// unconsumed value position (§ 5.8.5); both closer kinds decrement
/// the shared depth; a nested close clears `value_start` and an opener
/// re-derives `in_key` — byte-identical to [`ScanQ`] on quote-free
/// slices (R8-F2 closed the former fast-only quirks), which is what
/// lets one recorded boundary memo serve a consumer in either mode.
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
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = true;
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
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = true;
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
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
    const RECORD_BOUNDS: bool = false;
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
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
    const RECORD_BOUNDS: bool = false;
}

/// The shared byte-at-a-time scanner. All hot state lives in fields
/// that [`Scanner::run`] copies into LOCALS for the whole loop (the
/// former hand-written loops kept state in register-allocatable
/// locals; a field-per-iteration machine measured +15–20% slower on
/// deeply-nested quote-free documents) and writes back once after the
/// loop.
struct Scanner<'a, 'b, C: ScanCfg> {
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
    input_off: usize, // offset of input[0] in bounds coordinates
    bounds: InlineBounds<'b>,
    open_at: Vec<(usize, bool, i32)>, // parallels `stack`: (opener offset, pure, entry depth)
    pairs_out: Vec<(usize, usize)>,   // recorded (opener, closer) pairs
    _cfg: std::marker::PhantomData<C>,
}

impl<'a, 'b, C: ScanCfg> Scanner<'a, 'b, C> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        input: &'a str,
        open: u8,
        close: u8,
        body_object: bool,
        line_num: usize,
        span: Span,
        bounds: InlineBounds<'b>,
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
            input_off: input.as_ptr() as usize - bounds.origin,
            bounds,
            open_at: Vec::new(),
            pairs_out: Vec::new(),
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
        let mut open_at = std::mem::take(&mut self.open_at);
        let mut pairs_out = std::mem::take(&mut self.pairs_out);

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
                    // Next pair / item begins: re-derive key context
                    // from the CURRENT scope (the innermost opener on
                    // the stack), not the outermost one (R5-F1/R6-F1)
                    // — an Object scope starts a key position, an
                    // Array scope stays a value position (§ 5.3.3
                    // "Keys only"). split has no stack (OPENER_JUMP):
                    // its commas always sit at body depth, so the body
                    // kind is the current scope kind there.
                    let scope_object = if C::COMMA_CTX_SCOPE {
                        stack.last().map_or(body_object, |f| f.kind() == b'{')
                    } else {
                        body_object
                    };
                    in_key = scope_object;
                    if C::TRACK_QUOTES {
                        seg_start = scope_object;
                    }
                    // An unescaped comma ALWAYS ends raw mode (R5-F3).
                    if C::COMMA_CLEARS_RAW {
                        raw = false;
                    }
                    // R9-F1: value_start mirrors the same scope kind —
                    // after an Object comma the next position is a KEY
                    // position (§ 4: <inline-pair> begins with <key>),
                    // so the opener gate must reject a bracket there
                    // instead of phantom-opening a compound whose
                    // closer then eats the body's own closer.
                    value_start = !scope_object;
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
                        // Consult the gate scan's boundary map first: a
                        // hit is a pure memo of the sub-scan below (see
                        // [`InlineBounds`]), so the jump target and the
                        // state writes are identical; a miss re-runs the
                        // live sub-scan. Only split configs consult
                        // (OPENER_JUMP): find-config walks never read
                        // the map, and find_matching_close itself is
                        // unchanged — a memo hit can therefore never
                        // mask a BadEscapeSequence the live find
                        // dispatch would raise, because
                        // known_closer/consult hits only ever return
                        // spans the validating scan walked without any
                        // bad escape.
                        if let Some(end_abs) = self.bounds.opener_close_at(self.input_off + i) {
                            let end = end_abs - self.input_off; // closer index within `input`
                            if end < len {
                                // Skip over the entire nested compound
                                // (same writes as the Found branch
                                // below).
                                i = end + 1;
                                value_start = false;
                                if C::TRACK_QUOTES {
                                    seg_start = false;
                                }
                                continue;
                            }
                        }
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
                            if C::RECORD_BOUNDS {
                                // Entry state of this span vs the
                                // standalone scan's seed: the opener
                                // arm re-derives in_key/seg_start/
                                // value_start/prev identically in both
                                // modes (R8-F2), so every span is
                                // recorded. Purity still has the two
                                // pop-time gates below (crossed-closer
                                // and stop-byte checks).
                                open_at.push((i, true, depth));
                            }
                        }
                        // After an array's `[` the next position is
                        // still a value position (its first item,
                        // § 5.8.5); after a nested `{` comes key
                        // context. find never reads `value_start`, so
                        // this write is harmless there. `in_key` is
                        // re-derived IDENTICALLY in both modes (R8-F2,
                        // was fast-only quirk 2): fast residue let a
                        // span's `:` be classified as a key separator
                        // in one mode and value content in the other.
                        value_start = b == b'[';
                        in_key = b == b'{';
                        if C::TRACK_QUOTES {
                            seg_start = b == b'{';
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
                        if C::RECORD_BOUNDS {
                            // The span's standalone scan stops at the FIRST
                            // closer that returns its own counter to zero —
                            // global `depth` back to `entry - 1`. That byte is
                            // now, and the top frame is the span in question
                            // (everything opened above it died at strictly
                            // higher depths). Kind match: the pop below records
                            // a pure pair. Crossed: the standalone said
                            // `NotFound` HERE, so the frame must never record,
                            // no matter how far the walk continues past it.
                            match open_at.last_mut() {
                                Some(top) if depth == top.2 - 1 => {
                                    if !stack.last().is_some_and(|f| f.kind() == want) {
                                        top.1 = false;
                                    }
                                }
                                // Defense in depth: the walk somehow passed
                                // the stop byte without stopping — treat the
                                // span as dead.
                                Some(top) if depth < top.2 - 1 => top.1 = false,
                                _ => {}
                            }
                        }
                        if stack.last().is_some_and(|f| f.kind() == want) {
                            let f = stack.pop().unwrap();
                            if C::RECORD_BOUNDS {
                                let (open, pure, entry) =
                                    open_at.pop().expect("open_at parallels the scope stack");
                                if pure && depth == entry - 1 {
                                    pairs_out.push((open, i));
                                }
                            }
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
                        // The closed compound consumed the value
                        // position (§ 5.8.5) — identical in every
                        // find/scan config (R8-F2, was quirk 1).
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
        self.open_at = open_at;
        self.pairs_out = pairs_out;
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
///
/// `has_quotes` (R10-F1) is the ROOT body's quote presence threaded
/// down from the parse entry — computing it here would re-scan every
/// descendant subtree once per nesting level (`O(N·D)`).
pub(crate) fn split_top_level<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    body: InlineBody,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Vec<&'a str>, Error> {
    // R10-F1: `has_quotes` is threaded from the parse entry, where it
    // was computed once over the ROOT body. Every slice split here
    // descends from that body, so root-level presence implies presence
    // here; the converse is allowed (a quote-free level may take the
    // quote-aware machine — byte-identical on quote-free slices,
    // R8-F2). No per-level `has_quote_bytes` re-scan remains.
    if body == InlineBody::Array || !has_quotes {
        return Ok(split_top_level_fast(input, line_num, span, body, bounds));
    }

    // Slow path (object body with quote bytes): track key/value and
    // segment-start state so quoted KEYS are comma-opaque.
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, '_, SplitQ> =
        Scanner::new(input, open, close, object, line_num, span, bounds);
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
fn split_top_level_fast<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    body: InlineBody,
    bounds: InlineBounds<'_>,
) -> Vec<&'a str> {
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, '_, SplitFast> =
        Scanner::new(input, open, close, object, line_num, span, bounds);
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
    let mut sc: Scanner<'_, '_, C> = Scanner::new(
        input,
        open,
        close,
        object,
        0,
        Span::EMPTY,
        InlineBounds::for_input(input),
    );
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
/// Find the byte offset of an inline pair's separator — the first
/// unescaped `:` outside quoted key segments (§ 4 `<inline-pair>`
/// starts with `<key> (ws) ":" (ws) ...`). Delegates to the shared
/// § 4/§ 5.3 key-separator scanner. NO compound-depth counting (R9-F1):
/// a raw bracket in the key prefix is forbidden by `<key-char>` and is
/// reported as `InvalidKey` at key validation; it must not hide the
/// separator behind phantom depth. Colons inside nested value
/// compounds cannot race this scan: the separator precedes the value,
/// so the first colon outside quoted segments is always the pair's own
/// separator for any prefix that could still be a valid key.
///
/// Quote-aware (spec 0.7 § 5.3.3): the content of a quoted key segment
/// opened at a segment-start position is opaque. A span that never
/// closes swallows the rest of the input — `None` is returned and the
/// caller maps that to its unterminated/unparseable error of choice.
pub(crate) fn find_unescaped_colon_inline(s: &str) -> Option<usize> {
    find_unescaped_colon(s)
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
    // opener. Recording is discarded; use
    // [`scan_inline_closer_with_bounds`] to keep it.
    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking. `value_start` marks an
        // unconsumed value position (body start in arrays — including
        // the position right after the array's `[`, which is its first
        // item — and after `:`/`,` otherwise); per § 5.8.5 a `{`/`[`
        // only nests there.
        run_scan::<ScanFast>(input, open, close, object, line_num, span, &mut Vec::new())
    } else {
        // Slow path (quote bytes present): the per-level key-position
        // machine plus the same value-position / raw-marker tracking.
        // Object bodies start at their first key segment; array bodies
        // start at their first item — a value position with no key
        // context (§ 5.3.3 "Keys only").
        run_scan::<ScanQ>(input, open, close, object, line_num, span, &mut Vec::new())
    }
}

/// [`scan_inline_closer`] plus the boundary recording: on `Found`,
/// `bounds_out` receives every nested compound span's (opener, closer)
/// byte offset relative to `input`'s first byte, sorted by opener — a
/// pure memo of the scans the callers would otherwise re-run (see
/// [`InlineBounds`] for the purity invariant). On every non-Found
/// outcome `bounds_out` is left empty.
pub(crate) fn scan_inline_closer_with_bounds(
    input: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
    bounds_out: &mut Vec<(usize, usize)>,
) -> InlineCloserScan {
    let bytes = input.as_bytes();
    let object = open == b'{';
    if bytes.is_empty() || bytes[0] != open {
        return InlineCloserScan::NotFound;
    }
    if !has_quote_bytes(bytes) {
        run_scan::<ScanFast>(input, open, close, object, line_num, span, bounds_out)
    } else {
        run_scan::<ScanQ>(input, open, close, object, line_num, span, bounds_out)
    }
}

fn run_scan<C: ScanCfg>(
    input: &str,
    open: u8,
    close: u8,
    object: bool,
    line_num: usize,
    span: Span,
    bounds_out: &mut Vec<(usize, usize)>,
) -> InlineCloserScan {
    let mut sc: Scanner<'_, '_, C> = Scanner::new(
        input,
        open,
        close,
        object,
        line_num,
        span,
        InlineBounds::for_input(input),
    );
    sc.i = 1;
    sc.depth = 1;
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    sc.pairs_out = std::mem::take(bounds_out);
    let stop = sc.run();
    let mut pairs = std::mem::take(&mut sc.pairs_out);
    match stop {
        // § 5.2's matching-closer rule: a closer that returns depth to
        // zero must be the body's own closer kind (a crossed one, e.g.
        // `[{a: 1]`, is not a matching closer).
        ScanStop::Closer { idx, byte } => {
            if byte == close {
                // Recorded in pop order (= closer order): sort by opener
                // once for the consumers' binary searches. Keys are
                // unique — one span per opener byte — so unstable is
                // fine.
                #[cfg(test)]
                {
                    // Counted twin of the production
                    // `sort_unstable_by_key` below — std implements
                    // that as exactly this `sort_unstable_by` closure —
                    // so test builds see the same algorithm, input and
                    // comparison count plus its count.
                    let mut cmps = 0usize;
                    pairs.sort_unstable_by(|a, b| {
                        cmps += 1;
                        a.0.cmp(&b.0)
                    });
                    ix_probe::record_sort(pairs.len(), cmps);
                    ix_probe::record_body(input.len(), pairs.len());
                }
                #[cfg(not(test))]
                pairs.sort_unstable_by_key(|p| p.0);
                *bounds_out = pairs;
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
