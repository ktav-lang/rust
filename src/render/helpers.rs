//! Small primitives shared by the rendering functions.

use crate::parser::classify;

pub(super) const INDENT: &str = "    ";

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
