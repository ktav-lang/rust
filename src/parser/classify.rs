//! Classify the text following a `:` (or a bare array-line) into a
//! [`ValueStart`].

use crate::error::Error;
use crate::value::Scalar;

use super::value_start::ValueStart;

/// `text` MUST already have trailing whitespace removed (guaranteed by
/// `handle_line`'s `raw.trim()` at the top of the pipeline). Only leading
/// whitespace — between `:` and the value — needs to be stripped here.
pub(super) fn classify_value_start(text: &str, line_num: usize) -> Result<ValueStart, Error> {
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

    // Multi-line string openers — exact tokens only.
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar(Scalar::new(""))),
        _ => {}
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
pub(super) fn validate_typed_integer(body: &str, line_num: usize) -> Result<Scalar, Error> {
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
    Ok(strip_leading_plus(trimmed).into())
}

/// Validate the body of a `:f` typed-float scalar. Returns the stripped
/// textual form on success; an `InvalidTypedScalar` error otherwise.
pub(super) fn validate_typed_float(body: &str, line_num: usize) -> Result<Scalar, Error> {
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
    Ok(strip_leading_plus(trimmed).into())
}

fn invalid_typed_scalar(line_num: usize, detail: &str) -> Error {
    Error::Syntax(format!("Line {}: InvalidTypedScalar: {}", line_num, detail))
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

/// Matches `^[-+]?[0-9]+\.[0-9]+([eE][-+]?[0-9]+)?$`. Mantissa MUST have a
/// decimal point — `42` (without `.`) is not a valid `:f` literal.
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
    // Mandatory decimal point.
    if i == bytes.len() || bytes[i] != b'.' {
        return false;
    }
    i += 1;
    // Fractional part: at least one digit.
    let digits_after = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_after {
        return false;
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
