//! `parse_strict` — lossy-scalar detection (§ 3.6 / § 5.2 inference).
//!
//! A scalar is *lossy* when its lexical form differs from the canonical
//! form of the inferred number: type inference would silently rewrite
//! the value (`1.10` → `1.1`, `01234` → `1234`, `0x1A` → `26`, …).
//! `parse` accepts such documents unchanged; `parse_strict` rejects
//! them with [`ErrorKind::LossyScalar`] so the author can either append
//! `::` (keep a String) or write the canonical number.
//!
//! The data-driven cases (which literal spellings are lossy, in which
//! grammar position, with which exact body/canonical) live in the
//! shared spec corpus now (`versions/0.7/tests/strict-lossy/`, exercised
//! by `tests/spec_conformance/strict_lossy.rs`) rather than here, so
//! every language binding tests the same fixtures ktav itself does. What
//! remains in this file is Rust-implementation-specific: exact byte
//! spans, `Display` formatting, and internal `Value` representation
//! details no cross-language corpus fixture can pin.

use ktav::value::Value;
use ktav::{Error, ErrorKind};

fn lossy(src: &str) -> (u32, String, String) {
    match ktav::parse_strict(src) {
        Err(Error::Structured(ErrorKind::LossyScalar {
            line,
            body,
            canonical,
            ..
        })) => (line, body, canonical),
        other => panic!("expected LossyScalar, got {:?}", other),
    }
}

#[test]
fn error_reports_the_offending_line() {
    let (line, body, _) = lossy("service: web\nport: 8080\nversion: 1.10\n");
    assert_eq!((line, body.as_str()), (3, "1.10"));
}

#[test]
fn forced_string_marker_bypasses_the_check() {
    let doc = ktav::parse_strict("version:: 1.10\nzip:: 01234\n").expect("valid strict Ktav");
    let Value::Object(top) = &doc else {
        panic!("top-level must be an object");
    };
    assert_eq!(top.get("version"), Some(&Value::String("1.10".into())));
    assert_eq!(top.get("zip"), Some(&Value::String("01234".into())));
}

#[test]
fn canonical_scalars_pass_strict() {
    let src = "\
service: web
port: 8080
ratio: 0.75
count: -30
tls: true
nothing: null
note: plain text
tags: [
    prod
    eu-west-1
]
inline: {a: 1, b: 2.5}
";
    let strict = ktav::parse_strict(src).expect("canonical doc must pass strict");
    let lax = ktav::parse(src).expect("valid Ktav");
    assert_eq!(strict, lax);
}

#[test]
fn canonical_writer_float_forms_pass_strict() {
    let value = ktav::parse(
        "small: 0.001\nmid: 0.0015\nnegative: -0.001\nlarge: 10000000.0\ninline: {small: 0.0015}\n",
    )
    .expect("source must parse");
    let canonical = ktav::emit_canonical(&value).expect("canonical writer must succeed");

    assert_eq!(
        canonical,
        "small: 1e-3\nmid: 1.5e-3\nnegative: -1e-3\nlarge: 1e7\ninline: {\n    small: 1.5e-3\n}\n"
    );
    let strict = ktav::parse_strict(&canonical).expect("writer output must be strict-canonical");
    assert_eq!(strict, value);
}

/// Lax parse still canonicalises the spellings § 5.2 infers as numbers,
/// and still refuses to infer one for a redundant leading zero — the two
/// halves of the rule in one document, so a regression in either
/// direction fails here.
#[test]
fn lax_canonicalises_inferred_numbers_but_keeps_a_leading_zero_verbatim() {
    let doc = ktav::parse("version: 1.10\nzip: 01234\n").expect("valid lax Ktav");
    let Value::Object(top) = &doc else {
        panic!("top-level must be an object");
    };
    let Some(Value::Float(v)) = top.get("version") else {
        panic!("version must stay an inferred Float");
    };
    assert_eq!(v.as_str(), "1.1");
    // § 5.2 rule 13's exception: `01234` is never an Integer, so the
    // identifier survives the lax entry point byte for byte.
    let Some(Value::String(z)) = top.get("zip") else {
        panic!(
            "a redundant leading zero must stay a String, got {:?}",
            top.get("zip")
        );
    };
    assert_eq!(z.as_str(), "01234");
    // …and strict has nothing to reject there, because nothing is lost.
    assert!(ktav::parse_strict("zip: 01234\n").is_ok());
}

#[test]
fn lossy_error_display_names_body_and_canonical() {
    let err = ktav::parse_strict("version: 1.10\n").expect_err("must be lossy");
    let msg = err.to_string();
    assert!(msg.contains("LossyScalar"), "message: {msg}");
    assert!(msg.contains("1.10"), "message: {msg}");
    assert!(msg.contains("1.1"), "message: {msg}");
}

// --- multi-line float path (classify_value_start): diagnostics + lax forms ---

#[test]
fn lossy_float_error_payload_is_exact() {
    // Third line "ratio: 1.10" starts at byte 24; the span covers the
    // trimmed source line [24, 35).
    let src = "service: web\nport: 8080\nratio: 1.10\n";
    match ktav::parse_strict(src) {
        Err(Error::Structured(ErrorKind::LossyScalar {
            line,
            body,
            canonical,
            span,
        })) => {
            assert_eq!(line, 3);
            assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
            assert_eq!(span, ktav::error::Span::new(24, 35));
            assert_eq!(span.slice(src), Some("ratio: 1.10"));
        }
        other => panic!("expected LossyScalar, got {other:?}"),
    }
}

#[test]
fn lax_multiline_float_stores_ryu_form_at_magnitude_extremes() {
    // The lax branch never applies the § 5.9.8 rendering: stored forms
    // are the Ryu shortest decimals, never scientific rewrites.
    let doc = ktav::parse("big: 10000000.5\ntiny: 2e-3\nneg: -0.001\nzero: 0.000\n")
        .expect("valid lax Ktav");
    let Value::Object(top) = &doc else {
        panic!("top-level must be an object");
    };
    assert_eq!(
        top.get("big"),
        Some(&Value::Float("10000000.5".into())),
        "big must keep the Ryu form, not § 5.9.8's 1.00000005e7"
    );
    assert_eq!(
        top.get("tiny"),
        Some(&Value::Float("0.002".into())),
        "tiny must keep the Ryu form, not § 5.9.8's 2e-3"
    );
    assert_eq!(top.get("neg"), Some(&Value::Float("-0.001".into())));
    assert_eq!(top.get("zero"), Some(&Value::Float("0.0".into())));
}

#[test]
fn strict_scientific_region_canonical_form_passes_strict_as_the_same_ryu_value() {
    // The § 5.9.8 canonical source form for a float in the >= 1e7 region
    // (spec-corpus-covered: strict-lossy/float/large_magnitude_scientific_region
    // proves the fixed-point spelling is rejected with this exact rendering)
    // itself passes strict and is stored as the same Ryu value lax parsing
    // would produce from the fixed-point spelling.
    let v = ktav::parse_strict("big: 1.00000005e7\n").expect("§ 5.9.8 form must pass strict");
    let Value::Object(top) = &v else {
        panic!("top-level must be an object");
    };
    assert_eq!(
        top.get("big"),
        Some(&Value::Float("10000000.5".into())),
        "strict stores the same Ryu form as lax"
    );
}
