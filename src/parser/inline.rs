//! Inline compound parser (spec 0.5.0 section 5.8).
//!
//! Parses `{ key: value, ... }` into `Value::Object` and
//! `[ v1, v2, ... ]` into `Value::Array`.
//!
//! Escape sequences (section 3.7) are processed inside inline scalar values.

use memchr::memchr2;

use crate::error::{Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};
use crate::whitespace::{inline_whitespace_ascii, is_inline_whitespace};

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

    let segments = split_top_level(inner, line_num, span, InlineBody::Object)?;

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

    let segments = split_top_level(inner, line_num, span, InlineBody::Array)?;

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
        // Check for balanced closing `}`. `find_matching_close` counts
        // mid-scalar openers naively; when it finds no match, fall back
        // to the value-start-aware `scan_inline_closer` (§ 5.8.5) so a
        // body like `{a: hello{world, b: x}` — the trailing item of
        // `[{a: hello{world, b: x}]` — is still recognized as closed.
        // § 6.11/§ 6.12 tri-state: closer at last byte, closer with
        // trailing content, or no closer at all.
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
                if inner.trim().is_empty() {
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
        // value-start-aware fallback).
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
                if inner.trim().is_empty() {
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
    // keyword/numeric. `process_escapes` errors on every unrecognised `\X`
    // form, so a `\` byte surviving in `trimmed` is exactly the "had at
    // least one recognised escape" signal.
    let processed = process_escapes(trimmed, line_num, span)?;
    let body = processed;

    if trimmed.contains('\\') {
        return Ok(Value::String(body.into()));
    }

    classify_inline_scalar(&body, line_num, span, strict)
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

    Ok(out)
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
/// Returns the decoded String on success or `BadEscapeSequence` on an
/// unknown `\X`. Identical escape table to [`process_escapes`].
pub(crate) fn decode_key_segment(
    input: &str,
    line_num: usize,
    span: Span,
) -> Result<String, Error> {
    // Quoted segment (spec 0.7 § 5.3.3): strip the outer delimiter pair,
    // decode the interior only. Callers must have validated the segment
    // with `check_key` (properly closed, nothing after the closer).
    if !input.is_empty() {
        let first = input.as_bytes()[0];
        if first == b'"' || first == b'\'' || first == b'`' {
            debug_assert!(input.len() >= 2 && input.as_bytes()[input.len() - 1] == first);
            let interior = &input[1..input.len() - 1];
            if !interior.as_bytes().contains(&b'\\') {
                return Ok(interior.to_string());
            }
            return process_escapes(interior, line_num, span);
        }
    }
    // Bare segment path (unchanged)
    if !input.as_bytes().contains(&b'\\') {
        return Ok(input.to_string());
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
/// § 5.3.3) are opaque to comma splitting — `\{"a,b": 1, c: 2\}`
/// splits into two pairs — while quotes in value positions are
/// ordinary content (`a: "x,y", b: 2` splits inside the quotes). An
/// unterminated quoted key segment raises `UnterminatedInlineCompound`.
pub(crate) fn split_top_level<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    body: InlineBody,
) -> Result<Vec<&'a str>, Error> {
    let bytes = input.as_bytes();
    if body == InlineBody::Array || !has_quote_bytes(bytes) {
        return Ok(split_top_level_fast(input, line_num, span, body));
    }

    // Slow path (object body with quote bytes): track key/value and
    // segment-start state so quoted KEYS are comma-opaque.
    let mut segments: Vec<&'a str> = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut in_key = true;
    let mut seg_start = true;
    let mut value_start = false;

    while i < bytes.len() {
        if in_key && seg_start {
            i = skip_segment_ws(input, i);
            // The skip may consume every byte that remains: after a
            // trailing comma (or a dotted-key `.`) only whitespace can
            // follow, leaving `i` at EOF; `bytes[i]` below then indexed
            // out of bounds (R3-F1). Two possible outcomes:
            if i >= bytes.len() {
                if input[start..].trim().is_empty() {
                    // Only whitespace after the last comma: a valid
                    // trailing comma — emit no final segment (the
                    // callers treat an empty last segment identically).
                    return Ok(segments);
                }
                // A `.` armed this segment start and only whitespace
                // followed: a dotted key whose final segment is empty
                // (`b.` / `.`) — EmptyKey, the same category
                // `insert_value` raises for `a.: 1` (spec 0.7 § 6.5).
                return Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }));
            }
            if is_quote_byte(bytes[i]) {
                match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => {
                        // Unterminated quoted key segment (§ 5.3.3).
                        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                            line: line_num as u32,
                            span,
                        }));
                    }
                }
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => {
                // Skip escaped character. We validate escapes later
                // during process_escapes; here we just need to not
                // count `\,`, `\{`, `\}`, `\[`, `\]` as structural.
                // A recognized escape in value position consumes the
                // scalar start (§ 3.7 / § 5.8.5, R4-F4): the decoded
                // byte cannot reopen structural dispatch, so a later
                // `[`/`{` in the same value stays literal data.
                value_start = false;
                i += 2;
            }
            b'.' if in_key => {
                seg_start = true;
                i += 1;
            }
            b':' if in_key => {
                // Key/value boundary — quotes after this are content.
                in_key = false;
                value_start = true;
                i += 1;
            }
            b'{' | b'[' if value_start => {
                // Only a value-start opener opens a nested compound; its
                // commas stay internal (§ 5.8.5). The matching closer is
                // found with the value-start-aware `scan_inline_closer`
                // (a mid-scalar opener inside the span must not count);
                // `find_matching_close` would balance naive byte counts
                // and skip spans that per § 5.8.5 are NOT one compound.
                let open = bytes[i];
                let close = if open == b'{' { b'}' } else { b']' };
                if let InlineCloserScan::Found(close_pos) =
                    scan_inline_closer(&input[i..], open, close, line_num, span)
                {
                    // Skip over the entire nested compound.
                    i += close_pos + 1;
                    value_start = false;
                    seg_start = false;
                    continue;
                }
                // No closer inside the body — treat as literal byte
                // (mid-value brace). Escapes are validated later by
                // `process_escapes`, so a `BadEscape` scan result is
                // deliberately not propagated here.
                value_start = false;
                i += 1;
            }
            b'{' | b'[' => {
                // Mid-scalar opener: a literal byte with no structural
                // meaning; balancing is irrelevant (R3-F4, § 5.8.5) —
                // commas inside it still split.
                value_start = false;
                i += 1;
            }
            b',' => {
                segments.push(&input[start..i]);
                start = i + 1;
                i += 1;
                // Next pair begins: back to key context.
                in_key = true;
                seg_start = true;
                value_start = false;
                continue;
            }
            _ => {
                // § 3.3 whitespace: skip the whole code point without
                // consuming the value-start position (R4-F3) — NBSP's
                // 0xC2 lead byte must not clear `value_start`, and the
                // per-byte `i += 1` below would strand its 0xA0
                // continuation byte mid-code-point.
                if let Some(len) = inline_whitespace_at(input, i) {
                    i += len;
                    continue;
                }
                // Non-whitespace content consumes the value-start
                // position (mirrors `scan_inline_closer`'s `_` arm).
                value_start = false;
                i += 1;
            }
        }
    }

    // Last segment (after final comma, or the whole string if no comma)
    segments.push(&input[start..]);

    Ok(segments)
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
    let bytes = input.as_bytes();
    let object = body == InlineBody::Object;
    let mut segments: Vec<&str> = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut in_key = object;
    let mut value_start = !object;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // Consumed atomically (mirrors `scan_inline_closer`):
                // the escaped bracket byte is never seen here. A
                // recognized escape in value position consumes the
                // scalar start (§ 3.7 / § 5.8.5, R4-F4): the decoded
                // byte cannot reopen structural dispatch.
                value_start = false;
                i += 2;
                continue;
            }
            b'{' | b'[' if value_start => {
                let open = bytes[i];
                let close = if open == b'{' { b'}' } else { b']' };
                if let InlineCloserScan::Found(close_pos) =
                    scan_inline_closer(&input[i..], open, close, line_num, span)
                {
                    i += close_pos + 1;
                    value_start = false;
                    continue;
                }
                // Not closed inside the body — literal byte (mid-value
                // brace). Escapes are validated later by
                // `process_escapes`, so a `BadEscape` scan result is
                // deliberately not propagated here.
                value_start = false;
                i += 1;
                continue;
            }
            b'{' | b'[' => {
                // Mid-scalar opener: a literal byte with no structural
                // meaning; balancing is irrelevant (R3-F4, § 5.8.5) —
                // commas inside it still split.
                value_start = false;
            }
            b':' if in_key => {
                in_key = false;
                value_start = true;
            }
            // Literal `}`/`]` closers fall through to `_`: they have
            // no comma-shielding effect (§ 5.8.5 shields commas only
            // inside value-start compounds), matching the slow path.
            b',' => {
                segments.push(&input[start..i]);
                start = i + 1;
                i += 1;
                in_key = object;
                value_start = !object;
                continue;
            }
            _ => {
                // § 3.3 whitespace: whole-code-point skip that leaves
                // the value-start position intact (R4-F3, mirrors the
                // slow path).
                if let Some(len) = inline_whitespace_at(input, i) {
                    i += len;
                    continue;
                }
                value_start = false;
            }
        }
        i += 1;
    }

    segments.push(&input[start..]);
    segments
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
pub(crate) fn find_matching_close(input: &str, open: u8, close: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != open {
        return None;
    }

    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking — the pre-0.7 loop, unchanged.
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
        return None;
    }
    if open == b'{' {
        // Slow path (object body with quote bytes): key-position tracking,
        // PER NESTING LEVEL — every `{` opens a fresh pair list, so the
        // enclosing level's in_key state is saved and restored around it.
        let mut depth: i32 = 0;
        let mut i = 0;
        let mut in_key = true;
        let mut seg_start = true;
        // Per open compound: the opener byte plus the enclosing
        // key-position state (R6-F1). A `,` re-derives key context from
        // the CURRENT scope — the innermost opener on this stack — not
        // from the outermost one: a comma in a nested Array scope is a
        // value position (§ 5.3.3 "Keys only"), so a quote opening an
        // array item there is ordinary content, and only an Object
        // scope's comma begins a fresh key. Mirrors
        // `scan_inline_closer`'s `nested`/`open_stack` derivation.
        let mut scope_stack: Vec<(u8, bool, bool)> = Vec::new();
        while i < bytes.len() {
            if in_key && seg_start {
                i = skip_segment_ws(input, i);
                // The skip may consume every remaining byte (text ending in
                // whitespace after a trailing comma or dot); `bytes[i]`
                // below then indexed out of bounds (R3-F1). Nothing left to
                // scan — no matching close exists.
                if i >= bytes.len() {
                    return None;
                }
                if is_quote_byte(bytes[i]) {
                    // Unterminated span: the rest of the input is segment
                    // content (for bracket balance: no matching close).
                    let end = quoted_span_end(bytes, i)?;
                    i = end + 1;
                    seg_start = false;
                    continue;
                }
                seg_start = false;
            }
            match bytes[i] {
                b'\\' => {
                    i += 2; // skip escaped character
                    continue;
                }
                b'.' if in_key => {
                    seg_start = true;
                }
                b':' => {
                    // Key/value boundary: quotes after this are content.
                    in_key = false;
                }
                b',' => {
                    // Next pair / item begins: a fresh key position in an
                    // Object scope; an Array scope stays a value position
                    // (R6-F1, § 5.3.3 "Keys only").
                    let scope_object = scope_stack.last().is_some_and(|(k, _, _)| *k == b'{');
                    in_key = scope_object;
                    seg_start = scope_object;
                }
                b'[' => {
                    // A nested Array scope has no key context (§ 5.3.3
                    // "Keys only"); it takes no part in `{`/`}` depth
                    // counting, but the enclosing state is saved so `]`
                    // can restore it.
                    scope_stack.push((b'[', in_key, seg_start));
                    in_key = false;
                    seg_start = false;
                }
                b']' => {
                    if scope_stack.last().is_some_and(|(k, _, _)| *k == b'[') {
                        let (_, saved_in_key, _) = scope_stack.pop().unwrap();
                        in_key = saved_in_key;
                    }
                    // The `]` itself consumed a position, so segment-start
                    // tracking stays off until the next re-arm.
                    seg_start = false;
                }
                b if b == open => {
                    depth += 1;
                    // A `{` opens a fresh pair list at any level: save the
                    // enclosing key-position state and restart tracking for
                    // the nested body.
                    scope_stack.push((b'{', in_key, seg_start));
                    in_key = true;
                    seg_start = true;
                }
                b if b == close => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                    // Matching close of a nested object: restore the
                    // enclosing pair list's key-position state — only when
                    // the `{` scope is innermost, so a crossed closer
                    // doesn't corrupt tracking (mirrors
                    // `scan_inline_closer`). The `}` itself consumed a
                    // position, so segment-start tracking stays off until
                    // the next re-arm.
                    if scope_stack.last().is_some_and(|(k, _, _)| *k == b'{') {
                        let (_, saved_in_key, _) = scope_stack.pop().unwrap();
                        in_key = saved_in_key;
                    }
                    seg_start = false;
                }
                _ => {}
            }
            i += 1;
        }
        return None;
    }

    // Slow path (array body with quote bytes somewhere): depth still
    // counts only this `[`/`]` pair, but nested `{` objects track key
    // positions per scope (R3-F2): a quoted key segment of a nested
    // object is opaque to the `]` count, while quotes in the array's
    // own value positions stay ordinary content (§ 5.3.3 "Keys only").
    let mut depth: i32 = 0;
    let mut i = 0;
    let mut in_key = false;
    let mut seg_start = false;
    // Per open compound: the opener byte plus the enclosing key-position
    // state. A closer restores that state only when it matches the most
    // recently opened compound kind, so crossed closers don't corrupt
    // key tracking.
    let mut scope_stack: Vec<(u8, bool, bool)> = Vec::new();
    while i < bytes.len() {
        if in_key && seg_start {
            i = skip_segment_ws(input, i);
            // The skip may consume every remaining byte (text ending in
            // whitespace after a trailing comma or dot); `bytes[i]`
            // below then indexed out of bounds (R3-F1). Nothing left to
            // scan — no matching close exists.
            if i >= bytes.len() {
                return None;
            }
            if is_quote_byte(bytes[i]) {
                // Unterminated span: the rest of the input is segment
                // content (for bracket balance: no matching close).
                let end = quoted_span_end(bytes, i)?;
                i = end + 1;
                seg_start = false;
                continue;
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => {
                i += 2; // skip escaped character
                continue;
            }
            b'.' if in_key => {
                seg_start = true;
            }
            b':' if in_key => {
                // Key/value boundary: quotes after this are content.
                in_key = false;
            }
            b',' => {
                // Next pair / item begins: a fresh key position in an
                // object scope; array scope positions stay value
                // positions.
                let scope_object = scope_stack.last().is_some_and(|(k, _, _)| *k == b'{');
                in_key = scope_object;
                seg_start = scope_object;
            }
            b'{' => {
                // An object scope opens a fresh pair list at any level:
                // save the enclosing key-position state and restart
                // tracking for the nested body.
                scope_stack.push((b'{', in_key, seg_start));
                in_key = true;
                seg_start = true;
            }
            b'}' => {
                if scope_stack.last().is_some_and(|(k, _, _)| *k == b'{') {
                    let (_, saved_in_key, saved_seg_start) = scope_stack.pop().unwrap();
                    in_key = saved_in_key;
                    seg_start = saved_seg_start;
                } else {
                    seg_start = false;
                }
            }
            b'[' => {
                depth += 1;
                scope_stack.push((b'[', in_key, seg_start));
                in_key = false;
                seg_start = false;
            }
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                if scope_stack.last().is_some_and(|(k, _, _)| *k == b'[') {
                    let (_, saved_in_key, saved_seg_start) = scope_stack.pop().unwrap();
                    in_key = saved_in_key;
                    seg_start = saved_seg_start;
                } else {
                    seg_start = false;
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
    let bad_escape = |sequence: String| {
        InlineCloserScan::BadEscape(Error::Structured(ErrorKind::BadEscapeSequence {
            line: line_num as u32,
            span,
            sequence,
        }))
    };
    let bytes = input.as_bytes();
    let object = open == b'{';
    // The caller guarantees `input` starts with `open`; the opener itself
    // is depth 1, so scanning starts at byte 1.
    if bytes.is_empty() || bytes[0] != open {
        return InlineCloserScan::NotFound;
    }
    // R3-F2: the gate is "any quote byte anywhere" — quote tracking
    // itself is per nested Object scope below, not per outer opener.
    let track_quotes = has_quote_bytes(bytes);
    if !track_quotes {
        // Fast path: no quote tracking. `value_start` marks an unconsumed
        // value position (body start in arrays — including the position
        // right after the array's `[`, which is its first item — and
        // after `:`/`,` otherwise); per § 5.8.5 a `{`/`[` only nests
        // there.
        let mut depth: i32 = 1;
        let mut i = 1;
        let mut in_key = object;
        let mut value_start = !object;
        let mut raw = false;
        // Opener kinds of currently-open NESTED scopes; the body's own
        // kind is `open` (§ 5.8.5, R4-F1: raw is terminated by the
        // current scope's own closer, not the outermost one).
        let mut nested: Vec<u8> = Vec::new();
        let mut prev = open;
        while i < bytes.len() {
            let b = bytes[i];
            match b {
                b'\\' => {
                    // § 5.2 rules-6–9 preamble: an invalid escape beats
                    // the rule 8/9 decision. Advance by the FULL escape
                    // length (unlike `find_matching_close`'s `i += 2`,
                    // which suffices there because hex digits are not
                    // structural). `prev` becomes `\\` so an escaped
                    // `:` never forms a `::` raw marker.
                    match scan_escape(bytes, i) {
                        Err(seq) => return bad_escape(seq),
                        Ok(esc) => {
                            // A recognized escape in value position
                            // consumes the scalar start (§ 3.7 /
                            // § 5.8.5, R4-F4): the decoded byte cannot
                            // reopen structural dispatch.
                            value_start = false;
                            i += esc.len();
                        }
                    }
                    prev = b'\\';
                    continue;
                }
                b':' => {
                    if in_key {
                        // Key/value boundary: quotes after this are content.
                        in_key = false;
                        value_start = true;
                    } else if prev == b':' && value_start {
                        // `::` raw marker (§ 5.4 — in array bodies too):
                        // the rest of the item's value is a String —
                        // braces/brackets in it are content.
                        raw = true;
                    } else if value_start && bytes.get(i + 1) != Some(&b':') {
                        // A lone `:` opening the value's scalar (§ 5.8.5,
                        // R5-F2) consumes the value position: a following
                        // `{`/`[` is literal content, not a nested opener.
                        // A `:` immediately followed by another `:` stays
                        // armed for the `::` marker check above on the
                        // next iteration.
                        value_start = false;
                    }
                }
                b',' => {
                    // Next pair / item begins: re-derive key context from
                    // the CURRENT scope (`nested.last()`), not the
                    // outermost opener (R5-F1) — an Object scope starts a
                    // key position, an Array scope stays a value position
                    // (§ 5.3.3 "Keys only"). Mirrors the slow path's
                    // open_stack-based derivation.
                    let scope_object = nested.last().map_or(object, |k| *k == b'{');
                    in_key = scope_object;
                    raw = false;
                    value_start = true;
                }
                b'{' | b'[' if value_start && !raw => {
                    // Only a value-position opener nests (§ 5.8.5); a
                    // non-value-start (or raw-mode) opener is a literal
                    // byte that falls through to `_`.
                    depth += 1;
                    nested.push(b);
                    // After an array's `[` the next position is still a
                    // value position (its first item, § 5.8.5); after a
                    // nested `{` comes key context.
                    value_start = b == b'[';
                    if b == b'{' {
                        in_key = true;
                    }
                }
                b'}' | b']' => {
                    // Both closer kinds decrement the shared depth: a
                    // nested compound of the OTHER delimiter type still
                    // closes (an array item may be an object and vice
                    // versa). An unescaped closer ALWAYS ends raw mode
                    // and is structural (R5-F3): `<inline-raw-scalar>`
                    // terminates on the FIRST unescaped `,`, `}`, or `]`
                    // regardless of which scope it belongs to — raw mode
                    // only makes leading openers literal (§ 5.8.5). A
                    // closer that returns depth to zero must be the
                    // body's own closer; a crossed one (e.g. `[{a: 1]`)
                    // is not a matching closer (§ 5.2's matching-closer
                    // rule).
                    raw = false;
                    depth -= 1;
                    if depth == 0 {
                        return if b == close {
                            InlineCloserScan::Found(i)
                        } else {
                            InlineCloserScan::NotFound
                        };
                    }
                    // Nested closer matching the current scope's opener
                    // kind: pop it (compare stored OPENER to the closer's
                    // matching opener, R4-F2). (`raw` was already reset
                    // at the top of this arm — R5-F3.) `in_key`/`value_start`
                    // are deliberately not restored here: the next `,`
                    // re-derives key context fresh from the CURRENT scope
                    // via `nested.last()` (R5-F1); deriving it from the
                    // outermost opener instead is exactly the bug that made
                    // closer-site restoration look unnecessary, which is
                    // why R5-F1 is fixed at the comma site, not the closer
                    // site.
                    let kind_matched = match b {
                        b'}' => nested.last() == Some(&b'{'),
                        _ => nested.last() == Some(&b'['),
                    };
                    if kind_matched {
                        nested.pop();
                    }
                }
                _ => {
                    // § 3.3 whitespace: whole-code-point skip, leaving
                    // the value-start position intact (R4-F3). `prev`
                    // takes the sequence's last byte — whitespace bytes
                    // are never `:` or `\`, so `::` raw-marker detection
                    // sees exactly what the per-byte loop would see.
                    if let Some(len) = inline_whitespace_at(input, i) {
                        prev = bytes[i + len - 1];
                        i += len;
                        continue;
                    }
                    value_start = false;
                }
            }
            prev = b;
            i += 1;
        }
        return InlineCloserScan::NotFound;
    }

    // Slow path (object body with quote bytes): the per-level
    // key-position machine of `find_matching_close`, plus the same
    // value-position / raw-marker tracking as the fast path. Escapes
    // inside a quoted span are NOT validated — an unterminated quoted
    // key remains quote-opaque and stays `UnterminatedInlineCompound`
    // (§ 6.16), even if a bad escape occurs inside that unclosed
    // quoted segment.
    let mut depth: i32 = 1;
    let mut i = 1;
    // Object bodies start at their first key segment; array bodies
    // start at their first item — a value position with no key context
    // (§ 5.3.3 "Keys only").
    let mut in_key = object;
    let mut seg_start = object;
    let mut value_start = !object;
    let mut raw = false;
    let mut prev = open;
    // Per open compound: the opener byte plus the enclosing key-position
    // and raw state. A closer restores that state only when it matches
    // the most recently opened compound kind, so crossed closers don't
    // corrupt key tracking (R4-F2).
    let mut open_stack: Vec<(u8, bool, bool, bool)> = Vec::new();
    while i < bytes.len() {
        if in_key && seg_start {
            i = skip_segment_ws(input, i);
            // The skip may consume every remaining byte (text ending in
            // whitespace after a trailing comma or dot); `bytes[i]`
            // below then indexed out of bounds (R3-F1). Nothing left in
            // this text — no closer here (§ 5.2 rule 9 continues on the
            // next line).
            if i >= bytes.len() {
                return InlineCloserScan::NotFound;
            }
            if is_quote_byte(bytes[i]) {
                // Unterminated span: the rest of the input is segment
                // content (for bracket balance: no matching close).
                // Escapes inside it stay unvalidated (§ 6.16 opacity).
                let end = match quoted_span_end(bytes, i) {
                    Some(end) => end,
                    None => return InlineCloserScan::NotFound,
                };
                i = end + 1;
                seg_start = false;
                prev = bytes.get(i.wrapping_sub(1)).copied().unwrap_or(open);
                continue;
            }
            seg_start = false;
        }
        let b = bytes[i];
        match b {
            b'\\' => {
                // Outside quoted spans: validate (see fast path).
                match scan_escape(bytes, i) {
                    Err(seq) => return bad_escape(seq),
                    Ok(esc) => {
                        // A recognized escape in value position
                        // consumes the scalar start (§ 3.7 / § 5.8.5,
                        // R4-F4): the decoded byte cannot reopen
                        // structural dispatch.
                        value_start = false;
                        i += esc.len();
                    }
                }
                prev = b'\\';
                continue;
            }
            b'.' if in_key => {
                seg_start = true;
            }
            b':' => {
                if in_key {
                    // Key/value boundary: quotes after this are content.
                    in_key = false;
                    value_start = true;
                } else if prev == b':' && value_start {
                    // `::` raw marker — see fast path.
                    raw = true;
                } else if value_start && bytes.get(i + 1) != Some(&b':') {
                    // A lone `:` opening the value's scalar (§ 5.8.5,
                    // R5-F2) consumes the value position: a following
                    // `{`/`[` is literal content, not a nested opener.
                    // A `:` immediately followed by another `:` stays
                    // armed for the `::` marker check above on the
                    // next iteration.
                    value_start = false;
                }
            }
            b',' => {
                // Next pair / item begins: a fresh key position in an
                // object scope; array scope positions stay value
                // positions (§ 5.3.3 "Keys only").
                // After the R4-F2 fix the stack top correctly reflects
                // the ENCLOSING scope even after nested closers.
                let scope_object = open_stack.last().map_or(object, |(k, _, _, _)| *k == b'{');
                in_key = scope_object;
                seg_start = scope_object;
                raw = false;
                value_start = true;
            }
            b'{' | b'[' if value_start && !raw => {
                // Only a value-position opener nests (§ 5.8.5); a
                // non-value-start (or raw-mode) opener is a literal
                // byte that falls through to `_`.
                depth += 1;
                // After an array's `[` the next position is still a value
                // position (its first item); after a nested `{` comes a
                // fresh key position. Save the enclosing key-position
                // and raw state (§ 5.8.5, R4-F1); an array scope has no
                // key context (§ 5.3.3 "Keys only").
                open_stack.push((b, in_key, seg_start, raw));
                value_start = b == b'[';
                in_key = b == b'{';
                seg_start = b == b'{';
            }
            b'}' | b']' => {
                // Both closer kinds decrement (see fast path). An
                // unescaped closer ALWAYS ends raw mode and is
                // structural (R5-F3): `<inline-raw-scalar>` terminates
                // on the FIRST unescaped `,`, `}`, or `]` regardless of
                // which scope it belongs to — raw mode only makes
                // leading openers literal (§ 5.8.5).
                raw = false;
                depth -= 1;
                if depth == 0 {
                    // A closer that returns depth to zero must be the
                    // body's own closer kind, else the body has no
                    // matching closer.
                    return if b == close {
                        InlineCloserScan::Found(i)
                    } else {
                        InlineCloserScan::NotFound
                    };
                }
                // Matching close of a nested compound: restore the
                // enclosing key-position and raw state only when the
                // closer matches the most recently opened compound kind
                // (compare the stored OPENER against the closer's
                // matching opener kind, R4-F2 / `find_matching_close`).
                let kind_matched = match b {
                    b'}' => open_stack.last().is_some_and(|(k, _, _, _)| *k == b'{'),
                    _ => open_stack.last().is_some_and(|(k, _, _, _)| *k == b'['),
                };
                if kind_matched {
                    let (_, saved_in_key, saved_seg_start, saved_raw) = open_stack.pop().unwrap();
                    in_key = saved_in_key;
                    seg_start = saved_seg_start;
                    raw = saved_raw;
                } else {
                    seg_start = false;
                }
                value_start = false;
            }
            _ => {
                // § 3.3 whitespace: whole-code-point skip, leaving
                // the value-start position intact (R4-F3). `prev`
                // takes the sequence's last byte — whitespace bytes
                // are never `:` or `\`, so `::` raw-marker detection
                // sees exactly what the per-byte loop would see.
                if let Some(len) = inline_whitespace_at(input, i) {
                    prev = bytes[i + len - 1];
                    i += len;
                    continue;
                }
                value_start = false;
            }
        }
        prev = b;
        i += 1;
    }
    InlineCloserScan::NotFound
}
