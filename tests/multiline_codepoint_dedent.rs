//! Multi-byte § 3.3 whitespace in the stripped multi-line dedent —
//! § 3.3 × § 5.6, code-point-level measurement.
//!
//! Spec 0.7 § 3.3 enumerates the whitespace code points exhaustively,
//! including the nineteen non-single-byte members (U+0085, U+00A0,
//! U+1680, U+2000–U+200A, U+2028, U+2029, U+202F, U+205F, U+3000), and
//! states there is no separate, narrower "structural" whitespace
//! concept; § 5.6 measures the stripped form's common leading whitespace
//! "in whitespace code points (§ 3.3) rather than bytes … identical
//! code-point-for-code-point across every non-blank line's own leading
//! run". Both parsers carry a twin of the dedent helper, so every case
//! below is asserted through BOTH the owned parser (`ktav::from_str`)
//! and the thin event parser (`ktav::thin::parse_events`) — a divergence
//! between the two is the defect family these twins exist to prevent.

use ktav::from_str;
use ktav::thin::{parse_events, ParseEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct Cfg {
    value: String,
}

fn owned_scalar(src: &str) -> String {
    let cfg: Cfg = from_str(src).unwrap();
    cfg.value
}

fn thin_scalar(src: &str) -> String {
    let mut strs: Vec<String> = Vec::new();
    parse_events(src, |ev| {
        if let ParseEvent::Str(s) = ev {
            strs.push(s.to_string());
        }
    })
    .unwrap();
    assert_eq!(strs.len(), 1, "expected exactly one Str event: {strs:?}");
    strs[0].to_string()
}

fn assert_both_parsers(src: &str, expected: &str) {
    assert_eq!(owned_scalar(src), expected, "owned parser, src = {src:?}");
    assert_eq!(thin_scalar(src), expected, "thin parser, src = {src:?}");
}

/// The § 3.3 members that are NOT single-byte ASCII — the nineteen code
/// points the former byte-level scans skipped over entirely. (The four
/// single-byte members SP/TAB/VT/FF are pinned by
/// tests/multiline_vt_dedent.rs.)
const NON_ASCII_MEMBERS: [char; 19] = [
    '\u{0085}', '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
    '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}',
    '\u{202F}', '\u{205F}', '\u{3000}',
];

#[test]
fn each_non_ascii_member_indents_like_space() {
    // Positive control, one per newly-participating code point: a
    // leading run of exactly that code point is the common indent and
    // is removed. (Before the code-point scan this yielded
    // "{c}alpha\n{c}beta" — the run ended at the first non-ASCII byte.)
    for &c in &NON_ASCII_MEMBERS {
        let src = format!("value: (\n{c}alpha\n{c}beta\n)\n");
        let expected = "alpha\nbeta";
        assert_eq!(
            owned_scalar(&src),
            expected,
            "owned parser, indent U+{:04X}",
            c as u32
        );
        assert_eq!(
            thin_scalar(&src),
            expected,
            "thin parser, indent U+{:04X}",
            c as u32
        );
    }
}

#[test]
fn mixed_ascii_and_non_ascii_prefix_is_stripped_code_point_wise() {
    // Positive control: the shared prefix spans an ASCII byte and a
    // multi-byte member; the whole two-code-point prefix is removed.
    for &c in &NON_ASCII_MEMBERS {
        let src = format!("value: (\n {c}alpha\n {c}beta\n)\n");
        let expected = "alpha\nbeta";
        assert_eq!(
            owned_scalar(&src),
            expected,
            "owned parser, indent U+{:04X}",
            c as u32
        );
        assert_eq!(
            thin_scalar(&src),
            expected,
            "thin parser, indent U+{:04X}",
            c as u32
        );
    }
}

#[test]
fn u2000_u2001_share_two_bytes_but_no_code_point_prefix() {
    // Negative control: U+2000 (E2 80 80) and U+2001 (E2 80 81) share
    // their first two UTF-8 bytes but differ at code-point position 0,
    // so § 5.6 gives them NO common prefix. A byte-wise comparison
    // would either mis-strip or slice mid-code-point.
    assert_both_parsers(
        "value: (\n\u{2000}alpha\n\u{2001}beta\n)\n",
        "\u{2000}alpha\n\u{2001}beta",
    );
}

#[test]
fn common_prefix_caps_at_the_last_whole_matched_code_point() {
    // Positive + negative control: the lines share exactly one U+2000
    // (3 bytes) and then diverge (U+2000 vs U+2001). The common prefix
    // is one code point; a byte-wise comparison would take 5 bytes and
    // panic slicing the second line mid-code-point.
    assert_both_parsers(
        "value: (\n\u{2000}\u{2000}alpha\n\u{2000}\u{2001}beta\n)\n",
        "\u{2000}alpha\n\u{2001}beta",
    );
}

#[test]
fn non_whitespace_near_misses_do_not_indent() {
    // Negative control: one step outside the § 3.3 list on each side —
    // none of these starts a leading whitespace run, so the common
    // indent stays empty and each run survives verbatim.
    for c in [
        '\u{0084}', '\u{0086}', '\u{009F}', '\u{00A1}', '\u{167F}', '\u{1681}', '\u{180E}',
        '\u{1FFF}', '\u{200B}', '\u{2027}', '\u{202A}', '\u{202E}', '\u{2030}', '\u{205E}',
        '\u{2FFF}', '\u{3001}', '\u{FEFF}', '\u{E000}',
    ] {
        let src = format!("value: (\n{c}alpha\n{c}beta\n)\n");
        let expected = format!("{c}alpha\n{c}beta");
        assert_eq!(
            owned_scalar(&src),
            expected,
            "owned parser, U+{:04X}",
            c as u32
        );
        assert_eq!(
            thin_scalar(&src),
            expected,
            "thin parser, U+{:04X}",
            c as u32
        );
    }
}

#[test]
fn leading_run_ends_at_a_non_member_code_point() {
    // Negative control: space is § 3.3 whitespace, U+200B (zero width
    // space — famously NOT) is not, so each line's leading run is
    // exactly one byte and only the space is stripped.
    assert_both_parsers(
        "value: (\n \u{200B}alpha\n \u{200B}beta\n)\n",
        "\u{200B}alpha\n\u{200B}beta",
    );
}

#[test]
fn mismatched_member_prefix_shares_nothing() {
    // § 5.6: the prefix must be identical code-point-for-code-point —
    // space vs NBSP at position 0 pins the common indent to zero, same
    // as tab vs space in the § 5.6 example.
    assert_both_parsers(
        "value: (\n \u{00A0}alpha\n\u{00A0} beta\n)\n",
        " \u{00A0}alpha\n\u{00A0} beta",
    );
}

#[test]
fn common_prefix_is_capped_at_the_shortest_leading_run() {
    // The second line's leading run is shorter; the shared prefix is
    // " \u{2003}" (4 bytes) and the rest of the first line's run stays.
    assert_both_parsers(
        "value: (\n \u{2003}\u{2003}alpha\n \u{2003}beta\n)\n",
        "\u{2003}alpha\nbeta",
    );
}

#[test]
fn nbsp_only_line_is_blank_and_skips_indent_computation() {
    // § 3.5: a line of only whitespace code points is blank — NBSP
    // qualifies — and does not participate in the common indent.
    assert_both_parsers("value: (\n\u{00A0}\nalpha\n)\n", "\nalpha");
}

#[test]
fn verbatim_form_preserves_exotic_indents() {
    // § 5.6 verbatim form: no stripping at all — the NBSP indent
    // survives byte-for-byte (guard against over-reach).
    assert_both_parsers(
        "value: ((\n\u{00A0}alpha\n\u{00A0}beta\n))\n",
        "\u{00A0}alpha\n\u{00A0}beta",
    );
}

#[test]
fn trailing_non_ascii_ws_is_stripped_by_the_full_class() {
    // § 5.6 strips trailing § 3.3 whitespace per line — already true
    // via the full-class trim; pinned for the multi-byte edges.
    assert_both_parsers("value: (\nalpha\u{00A0}\nbeta\u{3000}\n)\n", "alpha\nbeta");
}
