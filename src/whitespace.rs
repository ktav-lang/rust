//! The § 3.3 whitespace class: ONE definition, two documented views.
//!
//! Spec 0.7 § 3.3 freezes whitespace as a closed 25-code-point list and
//! forbids delegating to a host-language Unicode-whitespace primitive
//! (`char::is_whitespace()`), even one that currently matches the list
//! exactly — the list is exhaustive, no more and no fewer, and must stay
//! stable across toolchain and Unicode-version bumps. Every
//! character-level and byte-level whitespace classification in this
//! crate routes through this module (R7-P3 sweep). This includes the
//! § 5.6 multiline-dedent common-prefix scanners
//! ([`common_leading_whitespace_prefix_len`] and its
//! [`leading_whitespace_run`] building block), which live in THIS module
//! so the owned parser, the thin event parser and the writer share ONE
//! code-point-level notion of the common prefix (no fourth copy of the
//! § 3.3 list), and classify whole code points over the FULL § 3.3
//! class: § 5.6 measures the
//! stripped form's common leading whitespace "in whitespace code points
//! (§ 3.3) rather than bytes" and § 3.3 admits no separate, narrower
//! "structural" whitespace concept, so the ASCII members take the byte
//! view [`inline_whitespace_ascii`] and every non-ASCII member (U+0085,
//! U+00A0, U+1680, U+2000–U+200A, U+2028, U+2029, U+202F, U+205F,
//! U+3000) is decoded and checked against [`is_inline_whitespace`].
//! A new copy of the list must never be introduced.
//!
//! Views:
//! - [`is_ktav_whitespace`] — the full § 3.3 set, all 25 code points.
//!   The drop-in replacement for `char::is_whitespace` / `str::trim`.
//! - [`is_inline_whitespace`] — the line-bounded INLINE view: the § 3.3
//!   set minus LF (U+000A) and CR (U+000D), 23 code points. Used where
//!   § 3.2 line splitting has already consumed the terminators (inline
//!   scanning/trimming inside a single line).
//!
//! Performance contract: both predicates are `#[inline]`, branch-cheap,
//! allocation-free; the ASCII members (the overwhelmingly common case:
//! space and tab) take an explicit fast path before any non-ASCII match.
//! Code-point-level scanners use [`inline_whitespace_ascii`] on the
//! ASCII path and decode a `char` only when a non-ASCII lead byte is
//! actually present.

/// The full § 3.3 whitespace set: exactly twenty-five code points, the
/// Unicode `White_Space` property as of Unicode 6.3, fixed as a closed
/// list. The spec forbids delegating to a host Unicode-whitespace
/// primitive (`char::is_whitespace()`) even though it matches this list
/// today — host and list must be allowed to diverge.
#[inline]
pub(crate) fn is_ktav_whitespace(c: char) -> bool {
    // ASCII fast path: the six single-byte members (space, TAB, LF,
    // VT, FF, CR). Note `char::is_ascii_whitespace` deliberately
    // excludes VT (0x0B), so the members are spelled out.
    if c.is_ascii() {
        return matches!(
            c,
            '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
        );
    }
    matches!(
        c,
        '\u{0085}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// The line-bounded INLINE view of § 3.3 whitespace: the full set minus
/// LF (U+000A) and CR (U+000D) — 23 code points. For scanning and
/// trimming inside a single line, where § 3.2 line splitting has
/// already consumed the terminators, so LF/CR could never occur anyway;
/// excluding them keeps inline scanners from having to rule them out.
#[inline]
pub(crate) fn is_inline_whitespace(c: char) -> bool {
    is_ktav_whitespace(c) && c != '\u{000A}' && c != '\u{000D}'
}

/// ASCII byte fast path for [`is_inline_whitespace`]: the four
/// single-byte members (space, TAB, VT, FF). LF/CR are excluded by the
/// inline view itself.
///
/// Invariant (locked by a test below): for every ASCII byte `b`,
/// `inline_whitespace_ascii(b) == is_inline_whitespace(b as char)`.
/// Byte-level scanners take this path to avoid decoding a `char` for
/// ASCII input.
#[inline]
pub(crate) fn inline_whitespace_ascii(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == 0x0B || b == 0x0C
}

// ---------------------------------------------------------------------------
// § 5.6 multiline dedent: the shared common leading-whitespace prefix.
// ONE code-point-level implementation, called by BOTH multiline engines
// (src/parser/collecting.rs, src/thin/event_parser.rs) and by the
// writer (src/render/helpers.rs) — no fourth copy of the § 3.3 list,
// and no second notion of "common prefix".
// ---------------------------------------------------------------------------

/// Byte length of the longest leading-whitespace (§ 3.3) prefix shared
/// code-point-for-code-point (§ 5.6) by every NON-BLANK line of
/// `lines`; blank lines (only § 3.3 whitespace) do not participate.
/// `lines` is any iterator of line slices: the two parsers pass their
/// collected `&[&str]` via `.iter().copied()`, the writer passes
/// `str::split('\n')` directly, so no caller ever materialises an
/// intermediate vector of lines. The result is the byte length of one
/// code-point sequence that is a prefix of each such line's own
/// leading run, so it always lands on a char boundary within every
/// non-blank line. Does not allocate.
pub(crate) fn common_leading_whitespace_prefix_len<'a>(
    lines: impl IntoIterator<Item = &'a str>,
) -> usize {
    let mut iter = lines
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The 25 code points of § 3.3, enumerated independently of the
    /// implementation's ranges.
    const SET: [char; 25] = [
        '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}',
        '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
        '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}',
        '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
    ];

    #[test]
    fn full_set_is_exactly_the_25_listed_code_points() {
        for &c in &SET {
            assert!(
                is_ktav_whitespace(c),
                "U+{:04X} must be § 3.3 whitespace",
                c as u32
            );
        }
        // Near-miss negatives: range boundaries and historically
        // White_Space code points removed in Unicode 6.3 (U+180E), plus
        // U+200B (zero width space — famously NOT whitespace) and
        // U+FEFF. Each is one step outside the list.
        for c in [
            '\u{0008}', '\u{000E}', '\u{001F}', '\u{0021}', '\u{0084}', '\u{0086}', '\u{009F}',
            '\u{00A1}', '\u{167F}', '\u{1681}', '\u{180E}', '\u{1FFF}', '\u{200B}', '\u{2027}',
            '\u{202A}', '\u{202E}', '\u{2030}', '\u{205E}', '\u{2060}', '\u{2FFF}', '\u{3001}',
            '\u{FEFF}', '\u{E000}',
        ] {
            assert!(
                !is_ktav_whitespace(c),
                "U+{:04X} must NOT be § 3.3 whitespace",
                c as u32
            );
        }
    }

    #[test]
    fn inline_view_is_full_set_minus_exactly_lf_and_cr() {
        for &c in &SET {
            let expected = c != '\u{000A}' && c != '\u{000D}';
            assert_eq!(
                is_inline_whitespace(c),
                expected,
                "inline view diverges on U+{:04X}",
                c as u32
            );
        }
        assert!(!is_inline_whitespace('\u{000A}'));
        assert!(!is_inline_whitespace('\u{000D}'));
    }

    #[test]
    fn ascii_byte_fast_path_matches_the_inline_view() {
        for b in 0u8..128 {
            assert_eq!(
                inline_whitespace_ascii(b),
                is_inline_whitespace(b as char),
                "byte fast path diverges from the inline view at 0x{:02X}",
                b
            );
        }
    }

    /// Toolchain tripwire, NOT a delegation: the implementation never
    /// consults the host primitive, but § 3.3 was derived from the
    /// Unicode `White_Space` property as of Unicode 6.3, which is what
    /// `char::is_whitespace` implements today. If this test fails after
    /// a toolchain/Unicode bump, host and spec list have diverged — the
    /// fixed list above is authoritative; update THIS TEST only after a
    /// conscious decision, and never switch the implementation to the
    /// host.
    #[test]
    fn host_whitespace_currently_matches_the_frozen_list() {
        for cp in 0u32..=0x10FFFF {
            if let Some(c) = char::from_u32(cp) {
                assert_eq!(
                    is_ktav_whitespace(c),
                    c.is_whitespace(),
                    "host `char::is_whitespace` diverges from the § 3.3 list at U+{:04X}",
                    cp
                );
            }
        }
    }

    /// R8-F4 arithmetic control, at u32 width, over the review's real
    /// input shape: one non-blank line carrying a 65 536-code-point
    /// indent and 65 535 blank lines. The OLD capacity estimator formed
    /// `common_len * lines.len()` in plain usize before any saturating
    /// guard — at u32 width that product overflows (panics under
    /// overflow checks, wraps silently without). The replacement
    /// subtracts the prefix per NON-BLANK line, which has no product
    /// and stays within u32 width for this input. This is a width
    /// model of the estimator's formula over the real values, not a
    /// run on a 32-bit target.
    #[test]
    fn r8_f4_capacity_subtraction_stays_in_u32_width_where_the_product_does_not() {
        let indent = " ".repeat(65_536);
        let non_blank = format!("{indent}x");
        let lines: Vec<&str> = std::iter::once(non_blank.as_str())
            .chain(std::iter::repeat("").take(65_535))
            .collect();
        assert_eq!(lines.len(), 65_536);

        let common_len = common_leading_whitespace_prefix_len(lines.iter().copied());
        assert_eq!(common_len, 65_536);

        // The old formula's product, evaluated with checked u32 math —
        // the executable form of the review's arithmetic proof:
        let old = u32::checked_mul(common_len as u32, lines.len() as u32);
        assert_eq!(
            old, None,
            "premise: common_len * lines.len() overflows u32 here"
        );

        // The new per-non-blank-line subtraction, modelled at u32 width:
        let new_cap: u32 = lines
            .iter()
            .filter(|l| !l.trim_matches(is_ktav_whitespace).is_empty())
            .map(|l| l.len() as u32 - common_len as u32)
            .sum::<u32>()
            .checked_add(lines.len() as u32)
            .expect("new formula must stay within u32 width");
        assert_eq!(new_cap, 1 + 65_536);
    }
}
