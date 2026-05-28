//! Classify the text following a `:` (or a bare array-line) into a
//! [`ValueStart`].
//!
//! Under spec 0.5.0 this module handles:
//! - § 5.2 rules 1–5 (structural openers, empty strings)
//! - § 5.2 rules 10–12 (keywords: null / true / false)
//! - § 5.2 rules 13–14 (number inference via § 3.6 grammar)
//! - § 5.2 rule 15 (fallback to String)
//!
//! Rules 6–9 (inline compounds) are deferred to Phase 4.

use crate::error::{Error, ErrorKind, Span};
use crate::value::Scalar;

use super::inline;
use super::value_start::ValueStart;

/// `text` MUST already have trailing whitespace removed (guaranteed by
/// `handle_line`'s `raw.trim()` at the top of the pipeline). Only leading
/// whitespace — between `:` and the value — needs to be stripped here.
///
/// `trimmed_span` covers the trimmed source line; it is used as the
/// `Span` payload for any structured error emitted here.
pub(super) fn classify_value_start(
    text: &str,
    line_num: usize,
    trimmed_span: Span,
) -> Result<ValueStart, Error> {
    let trimmed = text.trim_start();

    if trimmed == "{" {
        return Ok(ValueStart::OpenObject);
    }
    if trimmed == "[" {
        return Ok(ValueStart::OpenArray);
    }

    // § 5.2 rules 6–9: inline compounds.
    if trimmed.starts_with('{') {
        // Empty object: `{}` or `{ }`
        if trimmed.ends_with('}') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyObject);
        }
        // § 5.2 rule 6 vs 8: if the body ends with `}`, try to parse
        // it as a closed inline object. The parser handles mid-value
        // braces (§ 5.8.5) correctly — a `{` that is NOT the first
        // non-ws byte of a value position is literal and does not
        // create nesting. If parsing succeeds → rule 6. If it fails
        // with a parse error, propagate that error (the user gets a
        // more specific diagnostic than "unterminated").
        if trimmed.ends_with('}') {
            let value = inline::parse_inline_object(trimmed, line_num, trimmed_span)?;
            return Ok(ValueStart::InlineValue(value));
        }
        // § 5.2 rule 8: starts with `{` but no `}` at the end → unterminated.
        // Before reporting unterminated, scan for trailing `\` which would
        // be a BadEscapeSequence (§ 3.7/§ 6.13) — spec says this error
        // triggers before UnterminatedInlineCompound.
        check_trailing_backslash(trimmed, line_num, trimmed_span)?;
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }

    if trimmed.starts_with('[') {
        // Empty array: `[]` or `[ ]`
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        // § 5.2 rule 7: if the body ends with `]`, try to parse it.
        if trimmed.ends_with(']') {
            let value = inline::parse_inline_array(trimmed, line_num, trimmed_span)?;
            return Ok(ValueStart::InlineValue(value));
        }
        // § 5.2 rule 9: starts with `[` but no `]` at the end → unterminated.
        check_trailing_backslash(trimmed, line_num, trimmed_span)?;
        return Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
            line: line_num as u32,
            span: trimmed_span,
        }));
    }

    // Multi-line string openers — exact tokens only.
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar(Scalar::new(""))),
        _ => {}
    }

    // Paren-prefixed text that is NOT a multi-line opener: under 0.5.0,
    // `(value)` etc. are still ambiguous with multi-line openers.
    // A string whose first byte is `(` MUST use `::`.
    if trimmed.starts_with('(') {
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "paren-string".to_string(),
        }));
    }

    // § 5.2 rules 10–12: JSON keywords
    match trimmed {
        "null" => return Ok(ValueStart::Null),
        "true" => return Ok(ValueStart::Bool(true)),
        "false" => return Ok(ValueStart::Bool(false)),
        _ => {}
    }

    // § 5.2 rule 13: integer literal (§ 3.6)
    // Fast path: plain ASCII decimal (most common in configs — ports,
    // counters, etc.). No sign, underscore, or base prefix. The input
    // is already canonical, so skip itoa formatting entirely.
    if let Some(_val) = fast_plain_decimal_i64(trimmed) {
        return Ok(ValueStart::Integer(trimmed.into()));
    }
    // General path: prefixed, signed, or underscored literals.
    if let Some(val) = try_parse_integer(trimmed) {
        let mut buf = itoa::Buffer::new();
        let canonical = buf.format(val);
        return Ok(ValueStart::Integer(canonical.into()));
    }

    // § 5.2 rule 14: float literal (§ 3.6)
    if is_float_literal(trimmed) {
        if let Some(val) = parse_float_value(trimmed) {
            let mut buf = ryu::Buffer::new();
            let canonical = buf.format(val);
            // If ryu reproduces the input, the original slice is already
            // canonical — skip the Scalar heap allocation.
            if canonical == trimmed {
                return Ok(ValueStart::Float(trimmed.into()));
            }
            return Ok(ValueStart::Float(canonical.into()));
        }
    }

    // § 5.2 rule 15: String
    Ok(ValueStart::Scalar(trimmed.into()))
}

// ---------------------------------------------------------------------------
// § 3.6 Integer Literal Grammar
//
// integer  ::= sign? ( hex | oct | bin | dec )
// sign     ::= "+" | "-"
// hex      ::= "0x" hex_digit (("_")? hex_digit)*
// oct      ::= "0o" oct_digit (("_")? oct_digit)*
// bin      ::= "0b" bin_digit (("_")? bin_digit)*
// dec      ::= dec_digit (("_")? dec_digit)*
//
// Underscore rules: allowed between two consecutive digits only.
// No leading `_`, no trailing `_`, no double `__`, no `_` right
// after the base prefix.
// ---------------------------------------------------------------------------

/// Try to parse `s` as a § 3.6 integer literal. Returns `Some(i64)` on
/// success, `None` if the grammar doesn't match or the value overflows i64.
pub(crate) fn try_parse_integer(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut i = 0;
    let negative = if bytes[i] == b'-' {
        i += 1;
        true
    } else if bytes[i] == b'+' {
        i += 1;
        false
    } else {
        false
    };
    if i >= bytes.len() {
        return None; // sign only
    }

    // Check for base prefix
    if bytes[i] == b'0' && i + 1 < bytes.len() {
        match bytes[i + 1] {
            b'x' | b'X' => return parse_prefixed_int(&bytes[i + 2..], 16, negative),
            b'o' | b'O' => return parse_prefixed_int(&bytes[i + 2..], 8, negative),
            b'b' | b'B' => return parse_prefixed_int(&bytes[i + 2..], 2, negative),
            _ => {}
        }
    }

    // Decimal
    parse_decimal_int(&bytes[i..], negative)
}

/// Parse digits after the base prefix (e.g. after `0x`). Returns None if
/// the digit sequence is invalid or the value overflows i64.
fn parse_prefixed_int(digits: &[u8], radix: u32, negative: bool) -> Option<i64> {
    if digits.is_empty() {
        return None; // e.g. just "0x"
    }
    // First byte must be a valid digit (not underscore)
    if !is_digit_for_radix(digits[0], radix) {
        return None;
    }

    let mut val: u64 = 0;
    let mut prev_was_underscore = false;
    for &b in digits {
        if b == b'_' {
            if prev_was_underscore {
                return None; // double underscore
            }
            prev_was_underscore = true;
            continue;
        }
        prev_was_underscore = false;
        let d = digit_value(b, radix)?;
        val = val.checked_mul(radix as u64)?.checked_add(d as u64)?;
    }
    // Trailing underscore
    if prev_was_underscore {
        return None;
    }

    if negative {
        // -val must fit in i64
        if val > (i64::MAX as u64) + 1 {
            return None;
        }
        if val == 0 {
            Some(0) // -0 → 0
        } else {
            Some(-(val as i64))
        }
    } else {
        if val > i64::MAX as u64 {
            return None;
        }
        Some(val as i64)
    }
}

fn parse_decimal_int(digits: &[u8], negative: bool) -> Option<i64> {
    if digits.is_empty() {
        return None;
    }
    // First byte must be a digit
    if !digits[0].is_ascii_digit() {
        return None;
    }

    let mut val: u64 = 0;
    let mut prev_was_underscore = false;
    let mut count = 0;
    for &b in digits {
        if b == b'_' {
            if prev_was_underscore || count == 0 {
                return None;
            }
            prev_was_underscore = true;
            continue;
        }
        if !b.is_ascii_digit() {
            return None; // non-digit, non-underscore → not a match
        }
        prev_was_underscore = false;
        let d = (b - b'0') as u64;
        val = val.checked_mul(10)?.checked_add(d)?;
        count += 1;
    }
    if prev_was_underscore || count == 0 {
        return None;
    }

    if negative {
        // i64::MIN magnitude is (i64::MAX as u64) + 1
        let min_mag = (i64::MAX as u64) + 1;
        if val > min_mag {
            return None;
        }
        if val == 0 {
            Some(0)
        } else if val == min_mag {
            Some(i64::MIN)
        } else {
            Some(-(val as i64))
        }
    } else {
        if val > i64::MAX as u64 {
            return None;
        }
        Some(val as i64)
    }
}

fn is_digit_for_radix(b: u8, radix: u32) -> bool {
    digit_value(b, radix).is_some()
}

fn digit_value(b: u8, radix: u32) -> Option<u32> {
    let v = match b {
        b'0'..=b'9' => (b - b'0') as u32,
        b'a'..=b'f' => (b - b'a') as u32 + 10,
        b'A'..=b'F' => (b - b'A') as u32 + 10,
        _ => return None,
    };
    if v < radix {
        Some(v)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// § 3.6 Float Literal Grammar
//
// float      ::= sign? dec_part "." dec_part exponent?
//              | sign? dec_part exponent
// dec_part   ::= dec_digit (("_")? dec_digit)*
// exponent   ::= ("e" | "E") sign? dec_part
//
// The first alternative requires a decimal point with digits on both sides.
// The second requires an exponent (no decimal point).
// Pure digits with no `.` and no exponent → integer, not float.
// ---------------------------------------------------------------------------

/// Check if `s` matches the § 3.6 float literal grammar.
pub(crate) fn is_float_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    // Fast reject: a float literal MUST start with a digit, '+', or '-'.
    // Strings like "host.example" skip the full grammar scan.
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_digit() && first != b'+' && first != b'-' {
        return false;
    }
    // Fast reject: a float literal MUST contain `.` or `e`/`E`.
    // Plain digit strings are integers, not floats.
    if !bytes.contains(&b'.') && !bytes.contains(&b'e') && !bytes.contains(&b'E') {
        return false;
    }
    let mut i = 0;
    if i >= bytes.len() {
        return false;
    }
    // Optional sign
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }
    // Integer part: at least one digit
    let (new_i, ok) = scan_dec_part(bytes, i);
    if !ok {
        return false;
    }
    i = new_i;

    if i < bytes.len() && bytes[i] == b'.' {
        // First alternative: decimal point required, digits on both sides
        i += 1;
        let (new_i, ok) = scan_dec_part(bytes, i);
        if !ok {
            return false; // no digits after dot
        }
        i = new_i;
        // Optional exponent
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            let (new_i, ok) = scan_exponent(bytes, i);
            if !ok {
                return false;
            }
            i = new_i;
        }
        return i == bytes.len();
    }

    // Second alternative: exponent required (no decimal point)
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let (new_i, ok) = scan_exponent(bytes, i);
        if !ok {
            return false;
        }
        i = new_i;
        return i == bytes.len();
    }

    // No dot, no exponent → not a float
    false
}

/// Scan a `dec_part`: one-or-more decimal digits with optional underscore
/// separators between consecutive digits.
fn scan_dec_part(bytes: &[u8], mut i: usize) -> (usize, bool) {
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return (i, false);
    }
    i += 1;
    let mut prev_was_underscore = false;
    while i < bytes.len() {
        if bytes[i] == b'_' {
            if prev_was_underscore {
                return (i, false); // double underscore
            }
            prev_was_underscore = true;
            i += 1;
            continue;
        }
        if bytes[i].is_ascii_digit() {
            prev_was_underscore = false;
            i += 1;
            continue;
        }
        break;
    }
    if prev_was_underscore {
        return (i, false); // trailing underscore
    }
    (i, true)
}

/// Scan an exponent: `[eE] sign? dec_part`.
fn scan_exponent(bytes: &[u8], mut i: usize) -> (usize, bool) {
    if i >= bytes.len() || (bytes[i] != b'e' && bytes[i] != b'E') {
        return (i, false);
    }
    i += 1; // skip 'e'/'E'
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    scan_dec_part(bytes, i)
}

/// Fast-path check: plain ASCII decimal integer (no sign, underscore, or
/// base prefix) that fits in i64. Returns `Some(val)` if `s` is a canonical
/// decimal integer, `None` otherwise. The caller can use the original `s`
/// directly as the canonical string, avoiding itoa formatting.
#[inline]
pub(super) fn fast_plain_decimal_i64(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0];
    // Leading zero is only valid for "0" itself.
    if first == b'0' {
        return if bytes.len() == 1 { Some(0) } else { None };
    }
    if !(b'1'..=b'9').contains(&first) {
        return None;
    }
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

/// Parse a float literal that has already been validated by `is_float_literal`
/// into an `f64`. Returns `None` on overflow / NaN / infinity.
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
        return None; // overflow → falls through to String
    }
    Some(val)
}

// ---------------------------------------------------------------------------
// Public helpers for render/helpers.rs and ser/text_serializer.rs
// ---------------------------------------------------------------------------

/// Check if a string matches the § 3.6 integer literal grammar. Used by
/// the renderer to decide when `::` is needed. This checks the grammar
/// syntactically — the value may overflow i64 and still match.
pub fn matches_integer_grammar(s: &str) -> bool {
    // If it parses to i64, it obviously matches.
    if try_parse_integer(s).is_some() {
        return true;
    }
    // Also check if it matches the grammar syntactically (for overflow values).
    matches_integer_grammar_syntax(s)
}

/// Check if `s` matches the § 3.6 integer literal grammar syntactically,
/// without requiring the value to fit in i64.
fn matches_integer_grammar_syntax(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }
    if i >= bytes.len() {
        return false;
    }
    // Check for base prefix
    if bytes[i] == b'0' && i + 1 < bytes.len() {
        match bytes[i + 1] {
            b'x' | b'X' => return check_prefixed_digits(&bytes[i + 2..], 16),
            b'o' | b'O' => return check_prefixed_digits(&bytes[i + 2..], 8),
            b'b' | b'B' => return check_prefixed_digits(&bytes[i + 2..], 2),
            _ => {}
        }
    }
    // Decimal
    check_decimal_digits(&bytes[i..])
}

fn check_prefixed_digits(digits: &[u8], radix: u32) -> bool {
    if digits.is_empty() {
        return false;
    }
    if !is_digit_for_radix(digits[0], radix) {
        return false;
    }
    let mut prev_underscore = false;
    for &b in &digits[1..] {
        if b == b'_' {
            if prev_underscore {
                return false;
            }
            prev_underscore = true;
            continue;
        }
        prev_underscore = false;
        if !is_digit_for_radix(b, radix) {
            return false;
        }
    }
    !prev_underscore
}

fn check_decimal_digits(digits: &[u8]) -> bool {
    if digits.is_empty() || !digits[0].is_ascii_digit() {
        return false;
    }
    let mut prev_underscore = false;
    for &b in &digits[1..] {
        if b == b'_' {
            if prev_underscore {
                return false;
            }
            prev_underscore = true;
            continue;
        }
        prev_underscore = false;
        if !b.is_ascii_digit() {
            return false;
        }
    }
    !prev_underscore
}

/// Check if a string matches the § 3.6 float literal grammar. Used by
/// the renderer to decide when `::` is needed.
pub fn matches_float_grammar(s: &str) -> bool {
    is_float_literal(s)
}

/// If the last non-escaped byte in `s` is `\`, return `BadEscapeSequence`.
/// Used to detect `\<EOL>` inside unterminated inline compounds — the spec
/// (§ 3.7 / § 6.13) says this error triggers before `UnterminatedInlineCompound`.
fn check_trailing_backslash(s: &str, line_num: usize, span: Span) -> Result<(), Error> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Ok(());
    }
    // Count consecutive trailing backslashes.
    let mut n = 0;
    for &b in bytes.iter().rev() {
        if b == b'\\' {
            n += 1;
        } else {
            break;
        }
    }
    // An odd number of trailing backslashes means the last one is unescaped.
    if n % 2 == 1 {
        return Err(Error::Structured(ErrorKind::BadEscapeSequence {
            line: line_num as u32,
            span,
            sequence: "\\<end-of-line>".to_string(),
        }));
    }
    Ok(())
}
