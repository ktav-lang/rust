//! The § 3.3 whitespace class: ONE definition, two documented views.
//!
//! Spec 0.7 § 3.3 freezes whitespace as a closed 25-code-point list and
//! forbids delegating to a host-language Unicode-whitespace primitive
//! (`char::is_whitespace()`), even one that currently matches the list
//! exactly — the list is exhaustive, no more and no fewer, and must stay
//! stable across toolchain and Unicode-version bumps. Every
//! character-level whitespace classification in this crate routes
//! through this module (R7-P3 sweep). Two byte-level exceptions remain,
//! both the multiline-dedent helper `leading_whitespace_bytes`
//! (src/parser/collecting.rs, src/thin/event_parser.rs): they scan
//! leading bytes with `u8::is_ascii_whitespace`, which excludes VT
//! (U+000B) and therefore does NOT implement the § 3.3 list — left for
//! separate behavioral review rather than silently converted, since
//! widening it to the § 3.3 class would change `common_len` on input
//! containing VT. A new copy of the list must never be introduced.
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
//! Byte-level scanners use [`inline_whitespace_ascii`].

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
}
