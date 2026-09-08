//! State for collecting a multi-line string between `(` ... `)` or
//! `((` ... `))`.

use crate::whitespace::{inline_whitespace_ascii, is_ktav_whitespace};

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
        // LF/CR cannot occur (§ 3.2 pre-split lines, see parser.rs:126-129);
        // full § 3.3 class for exact `trim` parity.
        if l.trim_matches(is_ktav_whitespace).is_empty() {
            // blank line → empty
        } else if common_len > 0 && l.len() >= common_len {
            // SAFETY (soundness): common_len was computed from leading
            // whitespace bytes (the § 3.3 ASCII members SP/TAB/VT/FF via
            // `inline_whitespace_ascii`, all single-byte), so slicing at
            // that byte boundary is on a valid UTF-8 char boundary.
            out.push_str(l[common_len..].trim_end_matches(is_ktav_whitespace));
        } else {
            out.push_str(l.trim_end_matches(is_ktav_whitespace));
        }
    }
    out
}

/// Byte length of the longest leading-whitespace prefix shared by every
/// non-empty line. Does not allocate.
fn common_leading_whitespace_len(lines: &[&str]) -> usize {
    let mut iter = lines
        .iter()
        .filter(|l| !l.trim_matches(is_ktav_whitespace).is_empty());
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

/// Leading run of § 3.3 whitespace at the start of `s`, as bytes: the
/// byte-level inline view (`inline_whitespace_ascii`: SP, TAB, VT,
/// FF). Spec 0.7 § 5.6 measures the common leading whitespace "in
/// whitespace code points (§ 3.3)" and § 3.3 admits no narrower
/// class, so a leading VT is strippable indentation exactly like a
/// space. LF/CR cannot occur (§ 3.2 pre-split lines). Byte-level
/// limitation: the run ends at the first non-ASCII byte, so the
/// multi-byte § 3.3 members (U+0085, U+00A0, …) do not yet
/// participate in the dedent.
fn leading_whitespace_bytes(s: &str) -> &[u8] {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && inline_whitespace_ascii(bytes[i]) {
        i += 1;
    }
    &bytes[..i]
}
