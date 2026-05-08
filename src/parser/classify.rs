//! Classify the text following a `:` (or a bare array-line) into a
//! [`ValueStart`].

use crate::error::{Error, ErrorKind, Span};
use crate::value::Scalar;

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

    if trimmed.starts_with('{') {
        if trimmed.ends_with('}') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyObject);
        }
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "object".to_string(),
        }));
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']') && trimmed[1..trimmed.len() - 1].trim().is_empty() {
            return Ok(ValueStart::EmptyArray);
        }
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "array".to_string(),
        }));
    }

    // Multi-line string openers — exact tokens only.
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar(Scalar::new(""))),
        _ => {}
    }

    // Inline paren-string — visually ambiguous with multi-line openers.
    // `(value)`, `((value))`, and unclosed `(value` / `((value` all start
    // with `(` but aren't multi-line openers (those are exact tokens
    // matched above). Per spec § 5.2 + § 5.6, a string literal whose
    // first character is `(` MUST be marked raw with `::`. Reject the
    // `:`-form so users surface ambiguity at parse time rather than
    // silently storing a confusing scalar.
    if trimmed.starts_with('(') {
        return Err(Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: line_num as u32,
            span: trimmed_span,
            body: "paren-string".to_string(),
        }));
    }

    // JSON keywords
    match trimmed {
        "null" => return Ok(ValueStart::Null),
        "true" => return Ok(ValueStart::Bool(true)),
        "false" => return Ok(ValueStart::Bool(false)),
        _ => {}
    }

    Ok(ValueStart::Scalar(trimmed.into()))
}

// ---------------------------------------------------------------------------
// Typed-scalar (`:i` / `:f`) validation.
//
// These helpers parse the body that follows a `:i ` / `:f ` marker and
// return the normalized textual form (leading `+` stripped in the mantissa;
// leading `-` preserved; exponent sign preserved verbatim).
//
// Typed markers must NOT open a compound or a multi-line string — body
// starting with `{` / `[` / `(` / empty body is rejected here.
// ---------------------------------------------------------------------------

/// Validate the body of a `:i` typed-integer scalar. Returns the stripped
/// textual form on success; an `InvalidTypedScalar` error otherwise.
pub(super) fn validate_typed_integer(
    body: &str,
    line_num: usize,
    span: Span,
) -> Result<Scalar, Error> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            "integer body is empty",
            span,
        ));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            "typed marker `:i` cannot open a compound or multi-line value",
            span,
        ));
    }
    if !is_integer_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'i',
            &format!("'{}' is not a valid integer literal for `:i`", trimmed),
            span,
        ));
    }
    Ok(strip_leading_plus(trimmed).into())
}

/// Validate the body of a `:f` typed-float scalar. Returns the stripped
/// textual form on success; an `InvalidTypedScalar` error otherwise.
pub(super) fn validate_typed_float(
    body: &str,
    line_num: usize,
    span: Span,
) -> Result<Scalar, Error> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            "float body is empty",
            span,
        ));
    }
    if opens_compound_or_multiline(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            "typed marker `:f` cannot open a compound or multi-line value",
            span,
        ));
    }
    if !is_float_literal(trimmed) {
        return Err(invalid_typed_scalar(
            line_num,
            'f',
            &format!("'{}' is not a valid float literal for `:f`", trimmed),
            span,
        ));
    }
    Ok(strip_leading_plus(trimmed).into())
}

fn invalid_typed_scalar(line_num: usize, marker: char, detail: &str, span: Span) -> Error {
    Error::Structured(ErrorKind::InvalidTypedScalar {
        line: line_num as u32,
        marker,
        body: detail.to_string(),
        span,
    })
}

fn opens_compound_or_multiline(s: &str) -> bool {
    s.starts_with('{') || s.starts_with('[') || s.starts_with('(')
}

fn strip_leading_plus(s: &str) -> &str {
    s.strip_prefix('+').unwrap_or(s)
}

/// Matches `^[-+]?[0-9]+$`.
fn is_integer_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    if i == bytes.len() {
        return false; // sign only
    }
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return false;
        }
        i += 1;
    }
    true
}

/// Matches `^[-+]?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$`.
/// Decimal point is OPTIONAL — `42`, `42.0`, `42e3`, `42.5e-2` all valid.
/// (Integer literals coerce to float — same convention as JSON, TOML, YAML.)
fn is_float_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    // Integer part: at least one digit.
    let digits_before = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_before {
        return false;
    }
    // Optional decimal point + fractional digits.
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let digits_after = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == digits_after {
            // `42.` without fractional digits — invalid.
            return false;
        }
    }
    // Optional scientific exponent.
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
