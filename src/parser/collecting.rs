//! State for collecting a multi-line string between `(` ... `)` or
//! `((` ... `))`.

use crate::whitespace::{inline_whitespace_ascii, is_inline_whitespace, is_ktav_whitespace};

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

/// Byte length of the longest leading-whitespace prefix shared by every
/// non-blank line, compared code-point-for-code-point (§ 5.6). The
/// result is the byte length of one code-point sequence that is a
/// prefix of each such line's own leading run, so it always lands on a
/// char boundary within every non-blank line. Does not allocate.
fn common_leading_whitespace_len(lines: &[&str]) -> usize {
    let mut iter = lines
        .iter()
        .filter(|l| !l.trim_matches(is_ktav_whitespace).is_empty());
    let first = match iter.next() {
        Some(l) => leading_whitespace_run(l),
        None => return 0,
    };
    let mut len = first.len();
    for line in iter {
        let other = leading_whitespace_run(line);
        len = shared_prefix_bytes(first, other, len);
        if len == 0 {
            break;
        }
    }
    len
}

/// Byte length of the longest common prefix of the leading runs `a` and
/// `b`, compared code-point-for-code-point (§ 5.6: "identical
/// code-point-for-code-point") and capped at `cap` (a previous result of
/// this function or a run length — in either case a char boundary in
/// `a`). ASCII bytes compare 1:1 against code points, so the fast path
/// compares bytes; it stops at the first non-ASCII lead byte and the
/// tail is compared in whole `char`s instead, because a shared UTF-8
/// byte prefix is not necessarily a shared code-point prefix (U+2000 and
/// U+2001 share their first two bytes) and the offset returned here must
/// fall inside the same code-point sequence in both runs. Both runs are
/// cut at char boundaries and only whole matched code points advance the
/// offset, so the result is a char boundary in `a` and `b`.
fn shared_prefix_bytes(a: &str, b: &str, cap: usize) -> usize {
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    let lim = cap.min(ab.len()).min(bb.len());
    let mut i = 0;
    while i < lim && ab[i] < 0x80 && ab[i] == bb[i] {
        i += 1;
    }
    if i < lim && ab[i] >= 0x80 {
        let mut ac = a[i..].chars();
        let mut bc = b[i..].chars();
        while i < lim {
            match (ac.next(), bc.next()) {
                (Some(x), Some(y)) if x == y => i += x.len_utf8(),
                _ => break,
            }
        }
    }
    i
}

/// Leading run of § 3.3 whitespace at the start of `s`, cut at a char
/// boundary (returned as `&str`; `.len()` is its byte length). Spec 0.7
/// § 5.6 measures the common leading whitespace "in whitespace code
/// points (§ 3.3)" and § 3.3 admits no narrower class, so the
/// multi-byte members (U+0085, U+00A0, U+1680, U+2000–U+200A, U+2028,
/// U+2029, U+202F, U+205F, U+3000) indent exactly like SP/TAB/VT/FF.
/// LF/CR cannot occur (§ 3.2 pre-split lines). Hot path: ASCII bytes
/// take the [`inline_whitespace_ascii`] byte test; a `char` is decoded
/// only when a non-ASCII lead byte is actually present.
fn leading_whitespace_run(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if inline_whitespace_ascii(b) {
            i += 1;
        } else if b < 0x80 {
            break;
        } else {
            // Non-ASCII lead byte at the char boundary `i` (only whole
            // matched chars are ever advanced past): decode the code
            // point and consult the char-level inline view.
            match s[i..].chars().next() {
                Some(c) if is_inline_whitespace(c) => i += c.len_utf8(),
                _ => break,
            }
        }
    }
    &s[..i]
}
