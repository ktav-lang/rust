use serde::ser;

use crate::error::{Error, Result};

const INDENT: &str = "    ";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(super) fn write_indent(out: &mut String, level: usize) {
    // Push `level * INDENT.len()` spaces via slice copies of a static string.
    // Using a precomputed all-spaces slice lets the hot path be a single
    // `push_str` (vectorised memcpy) instead of a byte-by-byte loop —
    // the equivalent of the old `unsafe { as_mut_vec }.extend(repeat)`,
    // but with no unsafe and usually faster, since memcpy is SIMD-friendly.
    const SPACES: &str = "                                                                "; // 64
    let mut remaining = level * INDENT.len();
    if remaining == 0 {
        return;
    }
    out.reserve(remaining);
    while remaining > 0 {
        let chunk = remaining.min(SPACES.len());
        out.push_str(&SPACES[..chunk]);
        remaining -= chunk;
    }
}

pub(super) fn needs_raw_marker(s: &str) -> bool {
    crate::render::helpers::needs_raw_marker(s)
}

pub(super) fn top_err() -> Error {
    <Error as ser::Error>::custom("top-level value must be an object")
}

/// A bare scalar root is exactly the § 5.9.0 `ScalarRoot` case.
pub(super) fn scalar_root_err() -> Error {
    Error::Unrepresentable(crate::error::ReasonCode::ScalarRoot)
}

pub(super) fn key_err() -> Error {
    <Error as ser::Error>::custom("map keys must serialize to strings")
}

/// Append `s` (a ryu-formatted float) to `out`, ensuring a decimal point
/// is present in the mantissa — Ktav's Float grammar requires `N.N` at a
/// minimum. `ryu` emits `1.0` for `1.0f64` (good) but `1e100` without a
/// decimal point for very large values; insert `.0` before the exponent
/// in that case so the parser accepts the literal.
pub(super) fn push_float_body(out: &mut String, s: &str) {
    // Single pass over the mantissa bytes: ryu emits either `N.N`,
    // `N.Ne±E`, or `NeE` (no dot, exponent only) — the exponent always
    // comes after any dot, so the first `e`/`E` ends the scan.
    let bytes = s.as_bytes();
    let mut e_pos: Option<usize> = None;
    let mut has_dot = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'.' {
            has_dot = true;
        } else if b == b'e' || b == b'E' {
            e_pos = Some(i);
            break;
        }
    }
    match (e_pos, has_dot) {
        (_, true) => out.push_str(s),
        (Some(pos), false) => {
            out.push_str(&s[..pos]);
            out.push_str(".0");
            out.push_str(&s[pos..]);
        }
        (None, false) => {
            out.push_str(s);
            out.push_str(".0");
        }
    }
}

/// Fast path for pair-position integer emission: `: <digits>\n`.
/// Under spec 0.5.0, integers use plain `:` (no `:i` marker).
/// `itoa` avoids `fmt::Formatter` overhead.
pub(super) fn push_int_pair<I: itoa::Integer>(out: &mut String, v: I) {
    out.push_str(": ");
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Same, but for array-item position: bare `<digits>\n`.
/// Under spec 0.5.0, integer items are inferred from the lexical form.
pub(super) fn push_int_item<I: itoa::Integer>(out: &mut String, v: I) {
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Fast path for pair-position float emission via `ryu`.
/// Under spec 0.5.0, floats use plain `: ` (no `:f` marker).
pub(super) fn push_f64_pair(out: &mut String, v: f64) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    out.push_str(": ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

pub(super) fn push_f32_pair(out: &mut String, v: f32) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    out.push_str(": ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

/// Bare float item emission (no pair prefix). Under spec 0.5.0,
/// float items are inferred from the lexical form.
pub(super) fn push_f64_item(out: &mut String, v: f64) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

pub(super) fn push_f32_item(out: &mut String, v: f32) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}
