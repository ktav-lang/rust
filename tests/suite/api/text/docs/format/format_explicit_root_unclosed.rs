//! `format_str` on explicit roots (spec § 5.0.1 rules 4-5): the
//! positioned formatter behind `format_str` is a separate state machine
//! from the owned parser -- an explicit root left open at EOF must be
//! rejected there too (spec § 6.1, review finding F2), while closed,
//! inline and implicit roots plus empty/comment-only documents keep
//! formatting.

use ktav::{CompoundKind, Error, ErrorKind, Span};

fn expect_unclosed(src: &str, kind: CompoundKind, start: u32, end: u32) {
    let err = match ktav::format_str(src) {
        Ok(out) => panic!("format_str accepted unclosed explicit root {src:?}: {out:?}"),
        Err(e) => e,
    };
    match &err {
        Error::Structured(ErrorKind::UnclosedCompound { kind: k, span }) => {
            assert_eq!(*k, kind, "kind mismatch for {src:?}");
            assert_eq!(*span, Span::new(start, end), "span mismatch for {src:?}");
        }
        other => panic!("expected UnclosedCompound for {src:?}, got {other:?}"),
    }
}

#[test]
fn format_str_explicit_root_unclosed_at_eof_is_error() {
    expect_unclosed("{", CompoundKind::Object, 0, 1);
    expect_unclosed("[", CompoundKind::Array, 0, 1);
    expect_unclosed("{\n", CompoundKind::Object, 0, 2);
    expect_unclosed("[\r\n", CompoundKind::Array, 0, 3);
    expect_unclosed("{\na: 1\n", CompoundKind::Object, 0, 7);
    expect_unclosed("[\n1\n2\n", CompoundKind::Array, 0, 6);
    // Inner compound closed, root still open at EOF: the error names the root.
    expect_unclosed("{\na: {\n    b: 1\n}\n", CompoundKind::Object, 0, 18);
}

#[test]
fn format_str_explicit_root_closed_forms_still_format() {
    for src in [
        "{\n}\n",
        "[\n]\n",
        "{\na: 1\n}\n",
        "{a: 1}",              // inline root needs no closer line
        "",                    // empty document
        "## only a comment\n", // comments-only document
    ] {
        let once = ktav::format_str(src)
            .unwrap_or_else(|e| panic!("format_str rejected valid root {src:?}: {e}"));
        // Idempotence is part of format_str's contract.
        let twice = ktav::format_str(&once)
            .unwrap_or_else(|e| panic!("format_str rejected its own output for {src:?}: {e}"));
        assert_eq!(once, twice, "format_str not idempotent for {src:?}");
    }
}

#[test]
fn format_str_explicit_root_pins_after_close() {
    // Pins TODAY's behaviour after a closed explicit root (unchanged by
    // the EOF fix): trailing content keeps its orphan-line error.
    let err = ktav::format_str("{\n}\nextra: 1\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::OrphanLineAfterTopLevelInline { line, .. }) => {
            assert_eq!(*line, 3);
        }
        other => panic!("expected OrphanLineAfterTopLevelInline, got {other:?}"),
    }
    // A mismatched closer stays UnbalancedBracket, not UnclosedCompound.
    let err = ktav::format_str("{\n]\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnbalancedBracket { line, .. }) => {
            assert_eq!(*line, 2);
        }
        other => panic!("expected UnbalancedBracket, got {other:?}"),
    }
}
