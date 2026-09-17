// ---------------------------------------------------------------------------
// Indent helper
// ---------------------------------------------------------------------------

pub(super) const INDENT: &str = "    ";

/// Push `level * 4` spaces into `out`.
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
