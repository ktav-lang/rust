use super::num::needs_raw_marker;
use super::shared::push_indent;
use crate::error::Result;

// ---------------------------------------------------------------------------
// § 5.9.7 String form selection — pair context
// ---------------------------------------------------------------------------

/// Emit a String value inside a pair. Chooses between:
/// - `key:` (empty string, no body)
/// - `key: body` (one-line plain)
/// - `key:: body` (one-line raw — would reclassify)
/// - `key: ((\n...\n))` (verbatim multi-line)
/// - `key: (\n...\n)` (stripped multi-line, fallback)
pub(crate) fn emit_string_in_pair(s: &str, indent: usize, out: &mut String) -> Result<()> {
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
pub(crate) fn emit_string_as_item(
    s: &str,
    indent: usize,
    is_root_array_first: bool,
    out: &mut String,
) -> Result<()> {
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
    //
    // § 5.9.6 / § 5.9.12: when this is the FIRST item of an Array
    // root, the bare form is additionally not used if the body
    // satisfies § 5.0.1 rule 6's phase-1 pair-candidate test (it
    // would otherwise be re-read as the root Object's first pair), OR
    // — independently of the pair-candidate test — the body begins
    // with U+FEFF (bare form would place it at byte offset 0, where
    // § 3.1 makes readers strip it as a metadata BOM). Both exclusions
    // sit after the empty (`::`) and multi-line branches: § 5.9.12
    // scopes them to bodies whose canonical form would otherwise be
    // the bare one-line form (multi-line bodies put `((` at byte 0;
    // empty bodies emit `::`).
    if crate::render::helpers::item_needs_raw_marker(s)
        || (is_root_array_first
            && (crate::render::helpers::bare_item_is_pair_candidate(s)
                || s.starts_with('\u{FEFF}')))
    {
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
    // § 5.9.7: prefer verbatim; fall back to stripped only when a `))`
    // content line makes verbatim impossible, and error when neither
    // form can hold the body losslessly.
    match crate::render::helpers::choose_multiline_form(s, false)? {
        crate::render::helpers::MultilineForm::Verbatim => {
            emit_multiline_verbatim(s, indent, is_pair, out);
        }
        crate::render::helpers::MultilineForm::Stripped => {
            emit_multiline_stripped(s, indent, is_pair, out);
        }
    }
    Ok(())
}

/// Verbatim multi-line `((…))`. Body lines at indent 0 (§ 5.9.6).
fn emit_multiline_verbatim(s: &str, indent: usize, is_pair: bool, out: &mut String) {
    if is_pair {
        out.push_str(": ((\n");
    } else {
        out.push_str("((\n");
    }
    // Body at indent 0 (verbatim preserves bytes exactly).
    out.push_str(s);
    out.push('\n');
    push_indent(out, indent);
    out.push_str("))\n");
}

/// Stripped multi-line `(…)` fallback. Body lines at indent 0
/// so the common-indent computation yields 0.
fn emit_multiline_stripped(s: &str, indent: usize, is_pair: bool, out: &mut String) {
    if is_pair {
        out.push_str(": (\n");
    } else {
        out.push_str("(\n");
    }
    // Body at indent 0, lines kept byte-for-byte: an unindented
    // line pins the parser's common-indent dedent to zero (the
    // form chooser guarantees one exists), so per-line leading
    // whitespace survives the round-trip.
    out.push_str(s);
    out.push('\n');
    push_indent(out, indent);
    out.push_str(")\n");
}
