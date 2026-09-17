use super::envelope::{envelope, message_envelope, not_utf8_envelope};
use super::wire::{value_to_json, wire_to_value};

/// Parse Ktav source text bytes into a JSON wire document.
pub fn loads(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let value = crate::parse(input).map_err(|err| envelope(&err, input))?;
    let json = value_to_json(&value);
    serde_json::to_vec(&json)
        .map_err(|err| message_envelope(format!("internal: encode JSON: {err}"), ""))
}

/// Like [`loads`], but with strict scalar inference: lossy numeric
/// literals (`1.10`, `+1`) are rejected instead of normalized.
pub fn loads_strict(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let value = crate::parse_strict(input).map_err(|err| envelope(&err, input))?;
    let json = value_to_json(&value);
    serde_json::to_vec(&json)
        .map_err(|err| message_envelope(format!("internal: encode JSON: {err}"), ""))
}

/// Render a JSON wire document back to Ktav source text.
pub fn dumps(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::render::render(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Render a JSON wire document to Ktav source, coercing every scalar to
/// a literal string.
pub fn dumps_force_strings(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::to_string_force_strings(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Render a JSON wire document to canonical Ktav text (fixed key order,
/// minimal quoting).
pub fn emit_canonical(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::emit_canonical(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Format Ktav source text bytes. The input is Ktav source text, NOT a
/// JSON wire value. Formatting preserves comments and blank-line
/// grouping and is a fixed point: `format(format(x)) == format(x)`.
pub fn format(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let text = crate::format_str(input).map_err(|err| envelope(&err, input))?;
    Ok(text.into_bytes())
}

/// Canonical form of a document, from its **source text** rather than
/// from a wire value: Ktav text in, canonical Ktav text out.
///
/// This is not redundant with [`emit_canonical`], which takes the JSON
/// wire form. The difference matters in host languages whose number
/// type cannot carry Ktav's Integer/Float distinction: routing a
/// document through such a value loses the distinction before the
/// writer ever sees it, so `1.0` comes back as `1` and § 5.9 becomes
/// unreachable. Going text to text never materialises a host value and
/// keeps every scalar spelling byte-exact.
///
/// Two bindings arrived at this independently while the shims were
/// still separate — the JavaScript one after a byte-exact canonical
/// check failed on ten float fixtures, and the Go one by the same
/// reasoning — which is why it belongs here rather than in either.
pub fn canonical_from_source(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let value = crate::parse(input).map_err(|err| envelope(&err, input))?;
    let text = crate::render::emit_canonical(&value).map_err(|err| envelope(&err, input))?;
    Ok(text.into_bytes())
}
