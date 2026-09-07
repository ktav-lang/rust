//! Regression guard for the bench fixture generator (review R6-F5).
//!
//! Includes `benches/fixtures.rs` by `#[path]` so the tests validate
//! the exact code the benches run — no copied fixture text that could
//! drift. Uses the small size only: the generator is size-driven and
//! every 12 indices cover the full branch mix, so a larger target adds
//! nothing but runtime.

#[path = "../benches/fixtures.rs"]
mod fixtures;

use ktav::{Error, ErrorKind};

#[test]
fn synth_small_parses_to_expected_object() {
    fixtures::validate_synth(&fixtures::small_1k());
}

#[test]
fn synth_output_is_byte_identical_across_calls() {
    assert_eq!(fixtures::small_1k(), fixtures::small_1k());
}

#[test]
fn injected_bad_line_fails_with_missing_separator_space_at_predicted_line() {
    let good = fixtures::small_1k();
    let (bad, bad_line) = fixtures::inject_bad_line(&good);
    match ktav::parse(&bad) {
        Err(Error::Structured(k)) => {
            assert!(
                matches!(k, ErrorKind::MissingSeparatorSpace { marker: ':', .. }),
                "unexpected category: {k:?}"
            );
            assert_eq!(k.line(), Some(bad_line), "unexpected line");
            assert_eq!(k.span().slice(&bad), Some("value"), "unexpected span");
        }
        other => panic!("expected MissingSeparatorSpace at line {bad_line}, got {other:?}"),
    }
}
