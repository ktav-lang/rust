//! Small primitives shared by the rendering functions.

pub(super) const INDENT: &str = "    ";

/// True if the value must be emitted with `::` so that the parser does not
/// re-interpret it as a compound (`{...}` / `[...]`) or a JSON keyword
/// (`null` / `true` / `false`).
pub(super) fn needs_raw_marker(s: &str) -> bool {
    // Fast path: most scalars don't start with whitespace, so we can check
    // the first byte directly and skip `trim_start`'s whole-string scan.
    match s.as_bytes().first() {
        None => false,
        Some(&b' ') | Some(&b'\t') => needs_raw_marker_slow(s.trim_start()),
        Some(&b'{') | Some(&b'[') => true,
        Some(_) => {
            matches!(s, "null" | "true" | "false" | "(" | "((" | "()" | "(())")
        }
    }
}

#[cold]
#[inline(never)]
fn needs_raw_marker_slow(t: &str) -> bool {
    t.starts_with('{')
        || t.starts_with('[')
        || matches!(t, "null" | "true" | "false" | "(" | "((" | "()" | "(())")
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
