//! Vertical tab (U+000B) in the stripped multi-line dedent — § 3.3 × § 5.6.
//!
//! Spec 0.7 § 3.3 enumerates the whitespace code points exhaustively
//! (U+000B among them) and states there is no separate, narrower
//! "structural" whitespace concept; § 5.6 measures the stripped form's
//! common leading whitespace "in whitespace code points (§ 3.3)".
//! A leading VT is therefore strippable indentation exactly like a
//! space. Both parsers carry a twin of the dedent helper, so every
//! case below is asserted through BOTH the owned parser
//! (`ktav::from_str`) and the thin event parser
//! (`ktav::thin::parse_events`) — a divergence between the two is the
//! defect family these twins exist to prevent.

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

#[test]
fn leading_vt_dedents_like_space() {
    // § 3.3 lists U+000B as whitespace; § 5.6 computes the common
    // leading whitespace over § 3.3 code points, so a VT indent is
    // removed. (Before the § 3.3 conversion this yielded
    // "\u{0B}alpha\n\u{0B}beta".)
    assert_both_parsers("value: (\n\u{0B}alpha\n\u{0B}beta\n)\n", "alpha\nbeta");
}

#[test]
fn leading_vt_and_space_share_no_common_prefix() {
    // § 5.6: the prefix must be identical code-point-for-code-point;
    // VT vs space at position 0 pins the common indent to zero, same
    // as tab vs space.
    assert_both_parsers("value: (\n\u{0B}alpha\n beta\n)\n", "\u{0B}alpha\n beta");
}

#[test]
fn leading_vt_space_run_is_removed_from_both_lines() {
    // Common prefix "\u{0B} " (two § 3.3 code points) is stripped.
    assert_both_parsers("value: (\n\u{0B} alpha\n\u{0B} beta\n)\n", "alpha\nbeta");
}

#[test]
fn vt_only_line_is_blank_and_skips_indent_computation() {
    // § 3.5: a line of only whitespace code points is blank — it
    // contributes an empty line and does not participate in the
    // common-indent computation.
    assert_both_parsers("value: (\n\u{0B}\nalpha\n)\n", "\nalpha");
}

#[test]
fn single_line_vt_is_trimmed_by_the_full_class() {
    // The single-line finalize trims both edges with the full § 3.3
    // class, so a leading VT never survived there; pinned so the
    // single-line and dedent paths cannot diverge on VT.
    assert_both_parsers("value: (\n\u{0B}alpha\n)\n", "alpha");
}

#[test]
fn trailing_vt_is_stripped_by_the_full_class() {
    // § 5.6 strips trailing § 3.3 whitespace per line — already true
    // before the dedent conversion; pinned for the VT edge.
    assert_both_parsers("value: (\nalpha\u{0B}\nbeta\n)\n", "alpha\nbeta");
}

#[test]
fn verbatim_form_preserves_leading_vt() {
    // § 5.6 verbatim form: no stripping at all — the VT survives
    // byte-for-byte (guard against over-reach).
    assert_both_parsers("value: ((\n\u{0B}alpha\n))\n", "\u{0B}alpha");
}
