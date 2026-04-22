//! State for collecting a multi-line string between `(` ... `)` or
//! `((` ... `))`.

#[derive(Copy, Clone)]
pub(super) enum MultilineMode {
    /// `(` ... `)`: strip common leading whitespace from the collected lines.
    Stripped,
    /// `((` ... `))`: keep lines exactly as they appear.
    Verbatim,
}

impl MultilineMode {
    pub(super) fn terminator(self) -> &'static str {
        match self {
            MultilineMode::Stripped => ")",
            MultilineMode::Verbatim => "))",
        }
    }
}

pub(super) struct Collecting<'a> {
    pub(super) mode: MultilineMode,
    pub(super) lines: Vec<&'a str>,
}

impl<'a> Collecting<'a> {
    pub(super) fn new(mode: MultilineMode) -> Self {
        Self {
            mode,
            lines: Vec::with_capacity(8),
        }
    }

    pub(super) fn is_terminator(&self, trimmed: &str) -> bool {
        trimmed == self.mode.terminator()
    }

    pub(super) fn finish(self) -> String {
        match self.mode {
            MultilineMode::Verbatim => {
                // `Vec::join` always allocates; for the single-line case we
                // can skip the separator logic and the two-pass length
                // computation entirely.
                if self.lines.len() == 1 {
                    self.lines[0].to_string()
                } else {
                    self.lines.join("\n")
                }
            }
            MultilineMode::Stripped => {
                // Avoid the full dedent scan when there is only one line —
                // the common leading whitespace is just that line's leading
                // whitespace, so the result is `line.trim_start()`.
                if self.lines.len() == 1 {
                    let only = self.lines[0];
                    if only.trim().is_empty() {
                        String::new()
                    } else {
                        only.trim_start().to_string()
                    }
                } else {
                    dedent(&self.lines)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dedent: strip the longest common leading-whitespace prefix from every
// non-empty line. Blank lines become the empty string.
// ---------------------------------------------------------------------------

fn dedent(lines: &[&str]) -> String {
    let common_len = common_leading_whitespace_len(lines);

    let mut cap: usize = lines.iter().map(|l| l.len()).sum();
    cap = cap
        .saturating_sub(common_len * lines.len())
        .saturating_add(lines.len());
    let mut out = String::with_capacity(cap);

    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if l.trim().is_empty() {
            // blank line → empty
        } else if common_len > 0 && l.len() >= common_len {
            // SAFETY (soundness): common_len was computed from leading
            // whitespace bytes (ASCII by construction), so slicing at that
            // byte boundary is on a valid UTF-8 char boundary.
            out.push_str(&l[common_len..]);
        } else {
            out.push_str(l);
        }
    }
    out
}

/// Byte length of the longest leading-whitespace prefix shared by every
/// non-empty line. Does not allocate.
fn common_leading_whitespace_len(lines: &[&str]) -> usize {
    let mut iter = lines.iter().filter(|l| !l.trim().is_empty());
    let first = match iter.next() {
        Some(l) => leading_whitespace_bytes(l),
        None => return 0,
    };
    let mut len = first.len();
    for line in iter {
        let other = leading_whitespace_bytes(line);
        let mut shared = 0;
        while shared < len && shared < other.len() && first[shared] == other[shared] {
            shared += 1;
        }
        len = shared;
        if len == 0 {
            break;
        }
    }
    len
}

fn leading_whitespace_bytes(s: &str) -> &[u8] {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    &bytes[..i]
}
