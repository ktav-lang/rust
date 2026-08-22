//! Small primitives shared by the rendering functions.

use crate::error::{Error, Result};
use crate::parser::classify;

pub(super) const INDENT: &str = "    ";

/// Emit a key (flat, single-segment — the byte string stored on the
/// Object) into `out`, re-escaping `\`, `.`, `:` per spec 0.6.0 § 3.7
/// so the parser reads the same segment back.
///
/// This is used for the leaf key of a pair (and any non-dotted
/// segment). Callers that want to emit a dotted path (multiple
/// segments separated by an UNescaped `.`) join multiple calls with a
/// literal `.` between them.
pub(crate) fn push_escaped_key_segment(key: &str, out: &mut String) {
    // Fast path: most keys don't contain any of the three escape-
    // worthy bytes.
    let bytes = key.as_bytes();
    let needs_escape = bytes.iter().any(|&b| b == b'\\' || b == b'.' || b == b':');
    if !needs_escape {
        out.push_str(key);
        return;
    }
    out.reserve(key.len() + 4);
    for ch in key.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '.' => out.push_str("\\."),
            ':' => out.push_str("\\:"),
            other => out.push(other),
        }
    }
}

/// True if a one-line String body cannot be emitted on the value line
/// at all and must take a multi-line form: the parser trims the body
/// after `:` / `::` with `str::trim()` (§ 4) — the full Unicode
/// `White_Space` set, so an NBSP/NEL/ideographic-space edge is lost
/// the same way an ASCII one is; § 5.9.7 also routes control bytes
/// (other than `TAB`) to the multi-line form. Bodies containing `LF`
/// are multi-line by definition.
pub(crate) fn string_needs_multiline(s: &str) -> bool {
    s.chars().next().is_some_and(char::is_whitespace)
        || s.chars().next_back().is_some_and(char::is_whitespace)
        || s.bytes().any(|b| b < 0x20 && b != b'\t')
}

/// The § 5.9.7 error for a `CR` byte in a String: no text form can
/// hold it (verbatim blocks split on line terminators), so the writer
/// must reject the Value rather than emit a document that parses to a
/// different one.
pub(crate) fn cr_error() -> Error {
    Error::Message(
        "String containing CR (0x0D) is not representable in canonical form (§ 5.9.7)".into(),
    )
}

/// Which multi-line form a String body should take.
pub(crate) enum MultilineForm {
    /// `(` … `)` — parser dedents by the common leading whitespace.
    Stripped,
    /// `((` … `))` — parser copies content lines byte-for-byte.
    Verbatim,
}

/// Pick the multi-line form that reproduces `s` byte-for-byte on
/// re-parse, or error when neither form can (spec § 5.6.1).
///
/// Verbatim breaks on a content line trimming to `))` (it would close
/// the block). Stripped breaks on a line trimming to `)` (same), on a
/// whitespace-only line (§ 5.6 blanks it), and — because the parser
/// dedents by the COMMON leading whitespace — it loses per-line
/// indentation unless at least one non-blank line is unindented and
/// pins the common indent to zero.
///
/// `prefer_stripped` keeps the pretty renderers' historical choice
/// (indented stripped output when it is unconditionally safe); the
/// canonical writer passes `false` and prefers verbatim (§ 5.9.7).
pub(crate) fn choose_multiline_form(s: &str, prefer_stripped: bool) -> Result<MultilineForm> {
    let mut sole_single = false;
    let mut sole_double = false;
    let mut ws_only_line = false;
    let mut indented_line = false;
    let mut unindented_line = false;
    for line in s.split('\n') {
        let trimmed = line.trim();
        match trimmed {
            ")" => sole_single = true,
            "))" => sole_double = true,
            _ => {}
        }
        if line.is_empty() {
            continue;
        }
        if trimmed.is_empty() {
            ws_only_line = true;
        } else if line.starts_with(|c: char| c.is_whitespace()) {
            indented_line = true;
        } else {
            unindented_line = true;
        }
    }

    let stripped_safe = !sole_single && !ws_only_line && !indented_line;
    let stripped_lossless = !sole_single && !ws_only_line && unindented_line;
    let verbatim_ok = !sole_double;

    if prefer_stripped && stripped_safe {
        Ok(MultilineForm::Stripped)
    } else if verbatim_ok {
        Ok(MultilineForm::Verbatim)
    } else if stripped_lossless {
        Ok(MultilineForm::Stripped)
    } else {
        Err(Error::Message(
            "String has no lossless multi-line form (§ 5.6.1): a sole-`))` \
             content line closes the verbatim block, and the stripped block \
             cannot hold this body (a sole-`)` line, a whitespace-only line, \
             or every line indented). Split the value across adjacent \
             multi-line pairs."
                .into(),
        ))
    }
}

/// Item-context variant of [`needs_raw_marker`]: a bare array-item
/// line has extra collisions a pair body does not — a leading `##` is
/// a comment line (§ 3.4), a leading `::` is the raw-marker itself,
/// and a sole `]` / `}` closes the enclosing compound.
pub(crate) fn item_needs_raw_marker(s: &str) -> bool {
    needs_raw_marker(s) || s.starts_with("##") || s.starts_with("::") || matches!(s, "]" | "}")
}

/// True if the value must be emitted with `::` so that the parser does not
/// re-interpret it as a compound (`{...}` / `[...]`), a JSON keyword
/// (`null` / `true` / `false`), a number literal (§ 3.6), or a multi-line
/// opener (`(` / `((`).
pub(crate) fn needs_raw_marker(s: &str) -> bool {
    // Fast path: most scalars don't start with whitespace, so we can check
    // the first byte directly and skip `trim_start`'s whole-string scan.
    match s.as_bytes().first() {
        None => false,
        Some(&b' ') | Some(&b'\t') => needs_raw_marker_slow(s.trim_start()),
        Some(&b'{') | Some(&b'[') => true,
        Some(_) => needs_raw_marker_content(s),
    }
}

fn needs_raw_marker_content(s: &str) -> bool {
    if matches!(s, "null" | "true" | "false" | "(" | "((" | "()" | "(())") {
        return true;
    }
    if s.starts_with('(') {
        return true;
    }
    // § 5.2 rules 13–14: number literals must be forced to String via `::`
    if classify::matches_integer_grammar(s) || classify::matches_float_grammar(s) {
        return true;
    }
    false
}

#[cold]
#[inline(never)]
fn needs_raw_marker_slow(t: &str) -> bool {
    t.starts_with('{') || t.starts_with('[') || needs_raw_marker_content(t)
}

/// Push `level * INDENT.len()` spaces into `out`. Uses slice copies of a
/// const all-spaces string so the hot path is a single `push_str` →
/// vectorised memcpy instead of a per-level loop of 4-byte pushes.
pub(super) fn push_indent(out: &mut String, level: usize) {
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
