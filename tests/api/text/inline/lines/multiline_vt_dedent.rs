//! Vertical tab (U+000B) in the stripped multi-line dedent — § 3.3 × § 5.6.
//!
//! Spec 0.7 § 3.3 enumerates the whitespace code points exhaustively
//! (U+000B among them) and states there is no separate, narrower
//! "structural" whitespace concept; § 5.6 measures the stripped form's
//! common leading whitespace "in whitespace code points (§ 3.3)".
//! A leading VT is therefore strippable indentation exactly like a
//! space. Every case below is asserted through ALL THREE parse engines
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

#[test]
fn leading_vt_dedents_like_space() {
    // § 3.3 lists U+000B as whitespace; § 5.6 computes the common
    // leading whitespace over § 3.3 code points, so a VT indent is
    // removed. (Before the § 3.3 conversion this yielded
    // "\u{0B}alpha\n\u{0B}beta".)
    assert_three_engines("value: (\n\u{0B}alpha\n\u{0B}beta\n)\n", "alpha\nbeta");
}

#[test]
fn leading_vt_and_space_share_no_common_prefix() {
    // § 5.6: the prefix must be identical code-point-for-code-point;
    // VT vs space at position 0 pins the common indent to zero, same
    // as tab vs space.
    assert_three_engines("value: (\n\u{0B}alpha\n beta\n)\n", "\u{0B}alpha\n beta");
}

#[test]
fn leading_vt_space_run_is_removed_from_both_lines() {
    // Common prefix "\u{0B} " (two § 3.3 code points) is stripped.
    assert_three_engines("value: (\n\u{0B} alpha\n\u{0B} beta\n)\n", "alpha\nbeta");
}

#[test]
fn vt_only_line_is_blank_and_skips_indent_computation() {
    // § 3.5: a line of only whitespace code points is blank — it
    // contributes an empty line and does not participate in the
    // common-indent computation.
    assert_three_engines("value: (\n\u{0B}\nalpha\n)\n", "\nalpha");
}

#[test]
fn single_line_vt_is_trimmed_by_the_full_class() {
    // The single-line finalize trims both edges with the full § 3.3
    // class, so a leading VT never survived there; pinned so the
    // single-line and dedent paths cannot diverge on VT.
    assert_three_engines("value: (\n\u{0B}alpha\n)\n", "alpha");
}

#[test]
fn trailing_vt_is_stripped_by_the_full_class() {
    // § 5.6 strips trailing § 3.3 whitespace per line — already true
    // before the dedent conversion; pinned for the VT edge.
    assert_three_engines("value: (\nalpha\u{0B}\nbeta\n)\n", "alpha\nbeta");
}

#[test]
fn verbatim_form_preserves_leading_vt() {
    // § 5.6 verbatim form: no stripping at all — the VT survives
    // byte-for-byte (guard against over-reach).
    assert_three_engines("value: ((\n\u{0B}alpha\n))\n", "\u{0B}alpha");
}
