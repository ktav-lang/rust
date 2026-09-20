//! Inline compound entry points, depth-tracked variants, and inline scalar value parsing.

use std::borrow::Cow;

use crate::error::{Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};
use crate::whitespace::is_inline_whitespace;

use crate::parser::classify::{
    fast_plain_decimal_i64, has_redundant_leading_zero, is_float_literal, lossy_scalar,
    try_parse_integer,
};
use crate::parser::insert::insert_value;

use super::escapes::process_escapes;
use super::keys::has_quote_bytes;
use super::scan::split::{
    find_matching_close, find_unescaped_colon_inline, malformed, malformed_closer_not_at_end,
    scan_inline_closer, split_top_level, InlineBody, InlineBounds, InlineCloserScan,
};

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
    // R10-F1, corrected R11-F1: quote presence is computed ONCE over
    // the root body and threaded down. The soundness argument is
    // ASYMMETRIC: every nested slice is a substring of this body, so
    // root-level ABSENCE guarantees descendant absence and the fast
    // machine is always safe; root-level PRESENCE implies nothing
    // about a descendant — a quote-free level may then run the
    // quote-aware machine, which is safe only because the two
    // machines agree on quote-free input (R8-F2; the last-segment
    // divergence R11-F1 closed lives in `split_top_level`).
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

    // section 5.2 rules 13-14 exception: a redundant leading zero is never
    // a number, in an inline compound exactly as on a pair line.
    if has_redundant_leading_zero(body) {
        return Ok(Value::String(body.into()));
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
