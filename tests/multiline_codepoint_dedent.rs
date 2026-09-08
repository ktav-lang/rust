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
//! run". Every case below is asserted through ALL THREE parse engines
//! (R8-F3: the former `owned_scalar` helper called `from_str`, which
//! drives the THIN EventParser via `parse_events_merged`, so the owned
//! engine was never executed by these tests):
//! - the OWNED tree engine `ktav::parse()` (src/parser/collecting.rs),
//! - the THIN merged-event engine `ktav::from_str`,
//! - the THIN event engine `ktav::thin::parse_events`,
//!
//! Each engine is compared against the same independent expected value —
//! a divergence in either engine's dedent twin is the defect family
//! these tests exist to prevent, and a mutual comparison alone could
//! not catch a defect the engines' shared code has.

use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, Value};
use serde::Deserialize;

#[derive(Deserialize)]
struct Cfg {
    value: String,
}

/// The OWNED tree engine: `ktav::parse()` through
/// src/parser/collecting.rs.
fn owned_parse_scalar(src: &str) -> String {
    match parse(src).unwrap() {
        Value::Object(map) => match map.get("value") {
            Some(Value::String(s)) => s.to_string(),
            other => panic!("expected value: String scalar, got {other:?}"),
        },
        other => panic!("expected a top-level object, got {other:?}"),
    }
}

/// The THIN merged-event engine: `ktav::from_str` deserializes over
/// `thin::parse_events_merged` — a thin EventParser path, NOT the owned
/// tree (R8-F3).
fn from_str_scalar(src: &str) -> String {
    let cfg: Cfg = from_str(src).unwrap();
    cfg.value
}

/// The THIN event engine: `ktav::thin::parse_events`.
fn events_scalar(src: &str) -> String {
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

/// All three engines, each compared against the INDEPENDENT expected
/// value — never against each other.
fn assert_three_engines(src: &str, expected: &str) {
    assert_eq!(
        owned_parse_scalar(src),
        expected,
        "owned parse(), src = {src:?}"
    );
    assert_eq!(
        from_str_scalar(src),
        expected,
        "from_str (thin merged), src = {src:?}"
    );
    assert_eq!(
        events_scalar(src),
        expected,
        "parse_events (thin), src = {src:?}"
    );
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
        assert_three_engines(&src, "alpha\nbeta");
    }
}

#[test]
fn mixed_ascii_and_non_ascii_prefix_is_stripped_code_point_wise() {
    // Positive control: the shared prefix spans an ASCII byte and a
    // multi-byte member; the whole two-code-point prefix is removed.
    for &c in &NON_ASCII_MEMBERS {
        let src = format!("value: (\n {c}alpha\n {c}beta\n)\n");
        assert_three_engines(&src, "alpha\nbeta");
    }
}

#[test]
fn u2000_u2001_share_two_bytes_but_no_code_point_prefix() {
    // Negative control: U+2000 (E2 80 80) and U+2001 (E2 80 81) share
    // their first two UTF-8 bytes but differ at code-point position 0,
    // so § 5.6 gives them NO common prefix. A byte-wise comparison
    // would either mis-strip or slice mid-code-point.
    assert_three_engines(
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
    assert_three_engines(
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
        assert_three_engines(&src, &expected);
    }
}

#[test]
fn leading_run_ends_at_a_non_member_code_point() {
    // Negative control: space is § 3.3 whitespace, U+200B (zero width
    // space — famously NOT) is not, so each line's leading run is
    // exactly one byte and only the space is stripped.
    assert_three_engines(
        "value: (\n \u{200B}alpha\n \u{200B}beta\n)\n",
        "\u{200B}alpha\n\u{200B}beta",
    );
}

#[test]
fn mismatched_member_prefix_shares_nothing() {
    // § 5.6: the prefix must be identical code-point-for-code-point —
    // space vs NBSP at position 0 pins the common indent to zero, same
    // as tab vs space in the § 5.6 example.
    assert_three_engines(
        "value: (\n \u{00A0}alpha\n\u{00A0} beta\n)\n",
        " \u{00A0}alpha\n\u{00A0} beta",
    );
}

#[test]
fn common_prefix_is_capped_at_the_shortest_leading_run() {
    // The second line's leading run is shorter; the shared prefix is
    // " \u{2003}" (4 bytes) and the rest of the first line's run stays.
    assert_three_engines(
        "value: (\n \u{2003}\u{2003}alpha\n \u{2003}beta\n)\n",
        "\u{2003}alpha\nbeta",
    );
}

#[test]
fn nbsp_only_line_is_blank_and_skips_indent_computation() {
    // § 3.5: a line of only whitespace code points is blank — NBSP
    // qualifies — and does not participate in the common indent.
    assert_three_engines("value: (\n\u{00A0}\nalpha\n)\n", "\nalpha");
}

#[test]
fn verbatim_form_preserves_exotic_indents() {
    // § 5.6 verbatim form: no stripping at all — the NBSP indent
    // survives byte-for-byte (guard against over-reach).
    assert_three_engines(
        "value: ((\n\u{00A0}alpha\n\u{00A0}beta\n))\n",
        "\u{00A0}alpha\n\u{00A0}beta",
    );
}

#[test]
fn trailing_non_ascii_ws_is_stripped_by_the_full_class() {
    // § 5.6 strips trailing § 3.3 whitespace per line — already true
    // via the full-class trim; pinned for the multi-byte edges.
    assert_three_engines("value: (\nalpha\u{00A0}\nbeta\u{3000}\n)\n", "alpha\nbeta");
}

#[test]
fn r8_f4_many_blank_lines_capacity_input_parses_through_all_engines() {
    // R8-F4 regression input at full scale: one non-blank line with a
    // 65 536-code-point indent, plus 65 535 blank lines that do not
    // participate in the common-prefix computation but DID count in the
    // old `common_len * lines.len()` capacity product — 65_536 *
    // 65_536 = 2^32 overflows 32-bit usize. The estimate now subtracts
    // the prefix per non-blank line; parsing must stay correct (and
    // panic-free under overflow checks) on every engine.
    let indent = " ".repeat(65_536);
    let mut lines: Vec<String> = Vec::with_capacity(65_536);
    lines.push(format!("{indent}x"));
    lines.extend(std::iter::repeat(String::new()).take(65_535));
    let src = format!("value: (\n{}\n)\n", lines.join("\n"));
    assert_eq!(
        src.len(),
        131_084,
        "input premise: the review's ~131 KB shape"
    );
    // Blank lines are NOT removed: § 5.6 keeps each as an empty line,
    // so the result is "x" followed by 65 535 newlines — exactly the
    // 65 536-char result String the review observed on x64.
    let expected = format!("x{}", "\n".repeat(65_535));
    assert_eq!(expected.len(), 65_536, "review's observed result length");
    assert_three_engines(&src, &expected);
}
