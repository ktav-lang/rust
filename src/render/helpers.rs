//! Small primitives shared by the rendering functions.

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
