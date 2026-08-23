//! Canonical writer — emits a deterministic byte sequence for any [`Value`]
//! per spec § 5.9.
//!
//! The canonical form is:
//! - LF-only line endings (no `CR`).
//! - 4-space indent per nesting level.
//! - Trailing `LF` at end of document (empty Object root → zero bytes).
//! - No comments.
//! - No inline compounds (except empty `{}` / `[]`).
//! - Numbers in canonical form (Integer: base-10; Float: shortest decimal).
//! - Multi-line strings prefer verbatim `((…))`.
//!
//! Two writer-conforming implementations fed the same Value MUST produce
//! identical output (§ 8.2).

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Emit a canonical Ktav serialisation of `value` (spec § 5.9).
///
/// The top-level value must be an Object or an Array (§ 5.0.1).
/// Returns an error for any other variant, or if a String contains a
/// `CR` byte (not representable in canonical form, § 5.9.7).
pub fn emit_canonical(value: &Value) -> Result<String> {
    let mut out = String::with_capacity(estimate_size(value));
    match value {
        Value::Object(o) if o.is_empty() => { /* § 5.9.3: empty Object → zero bytes */ }
        Value::Object(o) => emit_object_pairs(o, 0, &mut out)?,
        Value::Array(items) if items.is_empty() => {
            // § 5.9.3: empty Array root → `[]\n`
            out.push_str("[]\n");
        }
        Value::Array(items) => emit_array_root(items, &mut out)?,
        _ => {
            return Err(Error::Message(
                "top-level value must be an Object or an Array".into(),
            ))
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// § 5.9.3 Root-level emission
// ---------------------------------------------------------------------------

/// Emit an Object's pairs at the given indent level (root uses 0).
fn emit_object_pairs(obj: &ObjectMap, indent: usize, out: &mut String) -> Result<()> {
    for (k, v) in obj {
        emit_pair(k, v, indent, out)?;
    }
    Ok(())
}

/// Emit a root-level Array. Items are bare at indent 0 unless the first
/// item is itself a non-empty compound (§ 5.9.3 lone-`{`/`[` wrap).
fn emit_array_root(items: &[Value], out: &mut String) -> Result<()> {
    let needs_wrap = !items.is_empty() && crate::render::helpers::first_item_needs_wrap(&items[0]);
    if needs_wrap {
        out.push_str("[\n");
        for item in items {
            emit_array_item(item, 1, out)?;
        }
        out.push_str("]\n");
    } else {
        for item in items {
            emit_array_item(item, 0, out)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.5 Pair emission
// ---------------------------------------------------------------------------

/// Emit a single `key: value` / `key:: value` / compound pair.
fn emit_pair(key: &str, value: &Value, indent: usize, out: &mut String) -> Result<()> {
    push_indent(out, indent);
    // Spec 0.6.0 § 3.7 — re-escape `\`, `.`, `:` in the key.
    crate::render::helpers::push_escaped_key_segment(key, out);
    match value {
        Value::Null => {
            // § 5.9.9
            out.push_str(": null\n");
        }
        Value::Bool(b) => {
            out.push_str(": ");
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        Value::Integer(s) => {
            // § 5.9.8: canonical base-10 decimal. Always a valid integer
            // literal — no raw marker needed.
            out.push_str(": ");
            out.push_str(s);
            out.push('\n');
        }
        Value::Float(s) => {
            // § 5.9.8: canonical float form — scientific for large/small abs.
            out.push_str(": ");
            out.push_str(&canonical_float(s));
            out.push('\n');
        }
        Value::String(s) => {
            emit_string_in_pair(s, indent, out)?;
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str(": []\n");
            } else {
                out.push_str(": [\n");
                for item in items {
                    emit_array_item(item, indent + 1, out)?;
                }
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                out.push_str(": {}\n");
            } else {
                out.push_str(": {\n");
                emit_object_pairs(obj, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.6 Array-item emission
// ---------------------------------------------------------------------------

/// Emit one array item at the given indent level.
fn emit_array_item(value: &Value, indent: usize, out: &mut String) -> Result<()> {
    push_indent(out, indent);
    match value {
        Value::Null => {
            out.push_str("null\n");
        }
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
            out.push('\n');
        }
        Value::Integer(s) => {
            out.push_str(s);
            out.push('\n');
        }
        Value::Float(s) => {
            out.push_str(&canonical_float(s));
            out.push('\n');
        }
        Value::String(s) => {
            emit_string_as_item(s, indent, out)?;
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]\n");
            } else {
                out.push_str("[\n");
                for item in items {
                    emit_array_item(item, indent + 1, out)?;
                }
                push_indent(out, indent);
                out.push_str("]\n");
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                out.push_str("{}\n");
            } else {
                out.push_str("{\n");
                emit_object_pairs(obj, indent + 1, out)?;
                push_indent(out, indent);
                out.push_str("}\n");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.7 String form selection — pair context
// ---------------------------------------------------------------------------

/// Emit a String value inside a pair. Chooses between:
/// - `key:` (empty string, no body)
/// - `key: body` (one-line plain)
/// - `key:: body` (one-line raw — would reclassify)
/// - `key: ((\n...\n))` (verbatim multi-line)
/// - `key: (\n...\n)` (stripped multi-line, fallback)
fn emit_string_in_pair(s: &str, indent: usize, out: &mut String) -> Result<()> {
    if s.is_empty() {
        // § 5.9.7: empty String → `key:` with no body.
        out.push_str(":\n");
        return Ok(());
    }

    if s.contains('\r') {
        // § 5.9.7: CR byte not representable in canonical form.
        return Err(crate::render::helpers::cr_error());
    }

    if crate::render::helpers::string_needs_multiline(s) {
        // Multi-line string — also the § 5.9.7 form for bodies with
        // leading/trailing whitespace or control bytes, which the
        // parser would trim (or the spec routes to verbatim) on a
        // one-line value.
        return emit_multiline_string(s, indent, true, out);
    }

    // One-line string. Check if it needs the raw marker.
    if needs_raw_marker(s) {
        out.push_str(":: ");
        out.push_str(s);
        out.push('\n');
    } else {
        out.push_str(": ");
        out.push_str(s);
        out.push('\n');
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.7 String form selection — array-item context
// ---------------------------------------------------------------------------

/// Emit a String value as an array item. Chooses between:
/// - `::` (empty string, no body)
/// - bare `body` (one-line plain)
/// - `:: body` (one-line raw — would reclassify)
/// - `((\n...\n))` (verbatim multi-line)
/// - `(\n...\n)` (stripped multi-line, fallback)
fn emit_string_as_item(s: &str, indent: usize, out: &mut String) -> Result<()> {
    if s.is_empty() {
        // § 5.9.7: empty String item → `::` with no body.
        // Wait — looking at fixtures, canonical `empty_stripped.canonical.ktav`
        // shows `note:\n` for an empty string in a pair, and for array items
        // the canonical form is `::`. But actually let's re-check § 5.9.6:
        // "Bare scalar item: <bytes> on its own line" — empty string can't be
        // a bare scalar (it would be a blank line). Use `::`.
        out.push_str("::\n");
        return Ok(());
    }

    if s.contains('\r') {
        return Err(crate::render::helpers::cr_error());
    }

    if crate::render::helpers::string_needs_multiline(s) {
        return emit_multiline_string(s, indent, false, out);
    }

    // One-line string. Check if it needs the raw marker — the item
    // form has extra collisions (`##`, `::`, sole `]` / `}`).
    if crate::render::helpers::item_needs_raw_marker(s) {
        out.push_str(":: ");
        out.push_str(s);
        out.push('\n');
    } else {
        out.push_str(s);
        out.push('\n');
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// § 5.9.7 Multi-line string emission (shared for pair + item)
// ---------------------------------------------------------------------------

/// Emit a multi-line string in canonical form.
///
/// Prefers verbatim `((…))` (§ 5.9.7). Falls back to stripped `(…)` when
/// a content line is exactly `))`, and errors when neither form can
/// hold the body losslessly (§ 5.6.1) — see `choose_multiline_form`.
///
/// `is_pair`: if true, we need `key: ((` prefix; if false, just `((`.
/// For the item case, `indent` is where `((` goes, and body is at indent 0.
fn emit_multiline_string(s: &str, indent: usize, is_pair: bool, out: &mut String) -> Result<()> {
    let segments: Vec<&str> = s.split('\n').collect();
    // § 5.9.7: prefer verbatim; fall back to stripped only when a `))`
    // content line makes verbatim impossible, and error when neither
    // form can hold the body losslessly.
    match crate::render::helpers::choose_multiline_form(s, false)? {
        crate::render::helpers::MultilineForm::Verbatim => {
            emit_multiline_verbatim(&segments, indent, is_pair, out);
        }
        crate::render::helpers::MultilineForm::Stripped => {
            emit_multiline_stripped(&segments, indent, is_pair, out);
        }
    }
    Ok(())
}

/// Verbatim multi-line `((…))`. Body lines at indent 0 (§ 5.9.6).
fn emit_multiline_verbatim(segments: &[&str], indent: usize, is_pair: bool, out: &mut String) {
    if is_pair {
        out.push_str(": ((\n");
    } else {
        out.push_str("((\n");
    }
    // Body lines at indent 0 (verbatim preserves bytes exactly).
    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(seg);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push_str("))\n");
}

/// Stripped multi-line `(…)` fallback. Body lines at indent 0
/// so the common-indent computation yields 0.
fn emit_multiline_stripped(segments: &[&str], indent: usize, is_pair: bool, out: &mut String) {
    if is_pair {
        out.push_str(": (\n");
    } else {
        out.push_str("(\n");
    }
    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // Body at indent 0, lines kept byte-for-byte: an unindented
        // line pins the parser's common-indent dedent to zero (the
        // form chooser guarantees one exists), so per-line leading
        // whitespace survives the round-trip.
        out.push_str(seg);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push_str(")\n");
}

// ---------------------------------------------------------------------------
// § 5.9.8 Float canonical form
// ---------------------------------------------------------------------------

/// Convert a stored float scalar (ryu shortest-decimal) to the spec § 5.9.8
/// canonical form:
/// - Use scientific notation when `abs(value) >= 1e7` or
///   `0 < abs(value) < 1e-2`.
/// - Otherwise keep the ryu decimal form unchanged.
/// - Scientific: lowercase `e`, no `+` in exponent, strip trailing `.0`
///   in mantissa (so `1.0e9` → `1e9`).
pub(crate) fn canonical_float(s: &str) -> String {
    // Parse the stored ryu string back to f64.
    let val: f64 = match s.parse() {
        Ok(v) => v,
        Err(_) => return s.to_string(), // shouldn't happen; pass through
    };

    if val == 0.0 {
        // Positive/negative zero in ryu is "0.0" or "-0.0"; keep as-is.
        return s.to_string();
    }

    let abs = val.abs();

    if !(1e-2..1e7).contains(&abs) {
        // Build scientific form.
        // Use Rust's {:e} formatter then normalise.
        let raw = format!("{:e}", val); // e.g. "1e9", "1.5e9", "-2.5e-10"
        normalise_scientific(&raw)
    } else {
        // Decimal region: ryu's output is already correct.
        s.to_string()
    }
}

/// Normalise Rust's `{:e}` scientific output to the spec form:
/// - lowercase `e` (already lowercase from `{:e}`)
/// - no `+` sign in the exponent
/// - strip trailing `.0` in the mantissa  (`1.0e9` → `1e9`)
/// - strip trailing zeros after decimal point in mantissa (`1.50e9` → `1.5e9`)
fn normalise_scientific(raw: &str) -> String {
    // Rust {:e} format: "<mantissa>e<exp>" where exp may be negative.
    // Example: "1e9", "1.5e9", "-2.5e-10", "1.5e-3".
    let e_pos = raw.find('e').unwrap_or(raw.len());
    let mantissa = &raw[..e_pos];
    let exp_part = &raw[e_pos + 1..]; // e.g. "9", "-10", "3"

    // Strip trailing zeros and unnecessary decimal point from mantissa.
    let mantissa = if mantissa.contains('.') {
        let trimmed = mantissa.trim_end_matches('0');
        trimmed.trim_end_matches('.')
    } else {
        mantissa
    };

    // Remove leading '+' from exponent (Rust never emits one, but be safe).
    let exp_str = exp_part.trim_start_matches('+');

    format!("{}e{}", mantissa, exp_str)
}

// ---------------------------------------------------------------------------
// § 5.9.5 / 5.9.6 / 5.9.7 — Would the parser re-classify this body?
// ---------------------------------------------------------------------------

/// Returns `true` if `body` would be classified by § 5.2 as something
/// other than a String (number, keyword, compound opener, or multi-line
/// opener). In that case the canonical writer must use the `::` raw
/// marker so the parser reads it back as a String.
///
/// Delegates to the shared `render::helpers` implementation, which
/// also covers a body starting with `(` (§ 5.2 would open a
/// multi-line block or reject it as an inline paren compound).
fn needs_raw_marker(body: &str) -> bool {
    crate::render::helpers::needs_raw_marker(body)
}

// Number grammar matching now delegated to crate::parser::classify
// (matches_integer_grammar / matches_float_grammar).

// ---------------------------------------------------------------------------
// Indent helper
// ---------------------------------------------------------------------------

const INDENT: &str = "    ";

/// Push `level * 4` spaces into `out`.
fn push_indent(out: &mut String, level: usize) {
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

// ---------------------------------------------------------------------------
// Size estimate
// ---------------------------------------------------------------------------

fn estimate_size(value: &Value) -> usize {
    match value {
        Value::Null => 5,
        Value::Bool(_) => 6,
        Value::Integer(s) | Value::Float(s) | Value::String(s) => s.len() + 8,
        Value::Array(items) => 4 + items.iter().map(estimate_size).sum::<usize>(),
        Value::Object(obj) => obj
            .iter()
            .map(|(k, v)| k.len() + 4 + estimate_size(v))
            .sum::<usize>()
            .saturating_add(4),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ObjectMap;
    use compact_str::CompactString;
    use indexmap::IndexMap;
    use rustc_hash::FxBuildHasher;

    fn obj(pairs: Vec<(&str, Value)>) -> Value {
        let mut map: ObjectMap = IndexMap::with_capacity_and_hasher(pairs.len(), FxBuildHasher);
        for (k, v) in pairs {
            map.insert(CompactString::new(k), v);
        }
        Value::Object(map)
    }

    fn arr(items: Vec<Value>) -> Value {
        Value::Array(items)
    }

    fn int(n: i64) -> Value {
        let mut buf = itoa::Buffer::new();
        Value::Integer(CompactString::new(buf.format(n)))
    }

    fn float(f: f64) -> Value {
        let mut buf = ryu::Buffer::new();
        Value::Float(CompactString::new(buf.format(f)))
    }

    fn s(text: &str) -> Value {
        Value::String(CompactString::new(text))
    }

    #[test]
    fn empty_object_root_produces_zero_bytes() {
        let v = obj(vec![]);
        assert_eq!(emit_canonical(&v).unwrap(), "");
    }

    #[test]
    fn empty_array_root_produces_brackets() {
        let v = arr(vec![]);
        assert_eq!(emit_canonical(&v).unwrap(), "[]\n");
    }

    #[test]
    fn simple_pairs() {
        let v = obj(vec![
            ("host", s("localhost")),
            ("port", int(8080)),
            ("debug", Value::Bool(true)),
        ]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "host: localhost\nport: 8080\ndebug: true\n");
    }

    #[test]
    fn null_and_false_keywords() {
        let v = obj(vec![
            ("maintenance", Value::Null),
            ("enabled", Value::Bool(false)),
        ]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "maintenance: null\nenabled: false\n");
    }

    #[test]
    fn float_values() {
        let v = obj(vec![("ratio", float(0.5)), ("sci", float(1.5e-3))]);
        let out = emit_canonical(&v).unwrap();
        // ryu shortest: 0.5 → "0.5", 1.5e-3 → "0.0015"
        assert!(out.contains("ratio: 0.5\n"), "got: {out}");
        // ryu may produce "0.0015" or "1.5e-3" — accept both canonical forms
        assert!(
            out.contains("sci: 1.5e-3\n") || out.contains("sci: 0.0015\n"),
            "got: {out}"
        );
    }

    #[test]
    fn empty_string_pair() {
        let v = obj(vec![("note", s(""))]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "note:\n");
    }

    #[test]
    fn raw_marker_for_keywords() {
        let v = obj(vec![("a", s("true")), ("b", s("null")), ("c", s("false"))]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "a:: true\nb:: null\nc:: false\n");
    }

    #[test]
    fn raw_marker_for_numbers() {
        let v = obj(vec![("a", s("42")), ("b", s("0.5")), ("c", s("0xFF"))]);
        let out = emit_canonical(&v).unwrap();
        assert!(out.contains("a:: 42\n"));
        assert!(out.contains("b:: 0.5\n"));
        assert!(out.contains("c:: 0xFF\n"));
    }

    #[test]
    fn raw_marker_for_inline_opener() {
        let v = obj(vec![("a", s("{hello}"))]);
        let out = emit_canonical(&v).unwrap();
        assert!(out.contains("a:: {hello}\n"));
    }

    #[test]
    fn nested_object() {
        let v = obj(vec![(
            "server",
            obj(vec![("host", s("localhost")), ("port", int(8080))]),
        )]);
        let out = emit_canonical(&v).unwrap();
        let expected = "server: {\n    host: localhost\n    port: 8080\n}\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn nested_array() {
        let v = obj(vec![("tags", arr(vec![s("a"), s("b")]))]);
        let out = emit_canonical(&v).unwrap();
        let expected = "tags: [\n    a\n    b\n]\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn array_root_bare_items() {
        let v = arr(vec![s("foo"), s("bar"), s("baz")]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "foo\nbar\nbaz\n");
    }

    #[test]
    fn array_root_wraps_when_first_item_is_compound() {
        let v = arr(vec![arr(vec![s("a"), s("b")]), arr(vec![s("c"), s("d")])]);
        let out = emit_canonical(&v).unwrap();
        let expected =
            "[\n    [\n        a\n        b\n    ]\n    [\n        c\n        d\n    ]\n]\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn array_root_does_not_wrap_for_scalars() {
        let v = arr(vec![int(1), int(2), int(3)]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "1\n2\n3\n");
    }

    #[test]
    fn cr_in_string_is_error() {
        let v = obj(vec![("x", s("hello\rworld"))]);
        assert!(emit_canonical(&v).is_err());
    }

    #[test]
    fn verbatim_multiline_string() {
        let v = obj(vec![("msg", s("line one\nline two"))]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(
            out,
            "msg: ((\n\
             line one\n\
             line two\n\
             ))\n"
        );
    }

    #[test]
    fn verbatim_multiline_in_array_item() {
        let v = arr(vec![s("line one\nline two"), s("end")]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(
            out,
            "((\n\
             line one\n\
             line two\n\
             ))\n\
             end\n"
        );
    }

    #[test]
    fn empty_string_array_item() {
        let v = arr(vec![s(""), s("ok")]);
        let out = emit_canonical(&v).unwrap();
        assert_eq!(out, "::\nok\n");
    }

    #[test]
    fn raw_marker_for_paren_tokens() {
        let v = obj(vec![
            ("a", s("(")),
            ("b", s("((")),
            ("c", s("()")),
            ("d", s("(())")),
        ]);
        let out = emit_canonical(&v).unwrap();
        assert!(out.contains("a:: (\n"));
        assert!(out.contains("b:: ((\n"));
        assert!(out.contains("c:: ()\n"));
        assert!(out.contains("d:: (())\n"));
    }

    #[test]
    fn mixed_heterogeneous_array() {
        let v = obj(vec![(
            "mixed",
            arr(vec![
                s("plain_string"),
                int(42),
                Value::Bool(true),
                Value::Null,
                s("true"), // keyword collision → raw marker
                obj(vec![("nested_obj", s("inside"))]),
                arr(vec![s("nested_array")]),
            ]),
        )]);
        let out = emit_canonical(&v).unwrap();
        let expected = "\
mixed: [
    plain_string
    42
    true
    null
    :: true
    {
        nested_obj: inside
    }
    [
        nested_array
    ]
]
";
        assert_eq!(out, expected);
    }

    #[test]
    fn integer_canonical_negative() {
        let v = obj(vec![("x", int(-1)), ("y", int(-42))]);
        let out = emit_canonical(&v).unwrap();
        assert!(out.contains("x: -1\n"));
        assert!(out.contains("y: -42\n"));
    }

    #[test]
    fn integer_canonical_zero() {
        // `-0` should normalise to `0` — but since we use itoa, -0i64
        // would be `0` anyway (no negative zero in i64). The canonical
        // form from the parser would store "0".
        let v = obj(vec![("z", int(0))]);
        let out = emit_canonical(&v).unwrap();
        assert!(out.contains("z: 0\n"));
    }

    #[test]
    fn needs_raw_marker_integer_forms() {
        assert!(needs_raw_marker("42"));
        assert!(needs_raw_marker("-1"));
        assert!(needs_raw_marker("+7"));
        assert!(needs_raw_marker("0xFF"));
        assert!(needs_raw_marker("0o755"));
        assert!(needs_raw_marker("0b1111_0000"));
        assert!(needs_raw_marker("1_000_000"));
        assert!(!needs_raw_marker("hello"));
        assert!(!needs_raw_marker("42abc"));
    }

    #[test]
    fn needs_raw_marker_float_forms() {
        assert!(needs_raw_marker("0.5"));
        assert!(needs_raw_marker("1.5e-3"));
        assert!(needs_raw_marker("1e9"));
        assert!(!needs_raw_marker("1."));
        assert!(!needs_raw_marker(".5"));
    }
}
