//! State for collecting a multi-line string between `(` ... `)` or
//! `((` ... `))`.

use crate::whitespace::{common_leading_whitespace_prefix_len, is_ktav_whitespace};

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
                // whitespace, so the line is trimmed on both edges — trim_start
                // then trim_end, each via `is_ktav_whitespace` (spec 0.7 § 5.6) —
                // not the leading edge alone.
                if self.lines.len() == 1 {
                    let only = self.lines[0];
                    // LF/CR cannot occur (§ 3.2 pre-split lines, see
                    // parser.rs:126-129); full § 3.3 class for exact `trim` parity.
                    if only.trim_matches(is_ktav_whitespace).is_empty() {
                        String::new()
                    } else {
                        only.trim_start_matches(is_ktav_whitespace)
                            .trim_end_matches(is_ktav_whitespace)
                            .to_string()
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
// non-empty line, then strip trailing whitespace. Blank lines become the
// empty string.
// ---------------------------------------------------------------------------

fn dedent(lines: &[&str]) -> String {
    let common_len = common_leading_whitespace_prefix_len(lines.iter().copied());

    // The common prefix is removed only from non-blank lines, so it is
    // subtracted per line. This must stay a per-line subtraction, never
    // `common_len * lines.len()`: that product overflows 32-bit usize
    // on a valid document (R8-F4: common_len = lines.len() = 65_536
    // blank lines included) before any saturating guard could see it.
    // Every non-blank line's own leading run is at least `common_len`
    // bytes (the shared-prefix scan caps at the shortest run), so the
    // subtraction cannot underflow.
    let mut cap: usize = lines
        .iter()
        .filter(|l| !l.trim_matches(is_ktav_whitespace).is_empty())
        .map(|l| l.len() - common_len)
        .sum();
    cap = cap.saturating_add(lines.len());
    let mut out = String::with_capacity(cap);

    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // LF/CR cannot occur (§ 3.2 pre-split lines, see parser.rs:126-129);
        // full § 3.3 class for exact `trim` parity.
        if l.trim_matches(is_ktav_whitespace).is_empty() {
            // blank line → empty
        } else if common_len > 0 && l.len() >= common_len {
            // Char-boundary safety: `common_len` is the byte length of a
            // code-point sequence that § 5.6 makes a prefix of every
            // non-blank line's own leading run; each run is cut at a
            // char boundary by `leading_whitespace_run`, and the
            // shared-prefix comparison advances only over whole matched
            // code points (ASCII bytes 1:1, non-ASCII in `char` steps).
            // `common_len` therefore lands on a char boundary of this
            // line and the slice below cannot panic.
            out.push_str(l[common_len..].trim_end_matches(is_ktav_whitespace));
        } else {
            out.push_str(l.trim_end_matches(is_ktav_whitespace));
        }
    }
    out
}
