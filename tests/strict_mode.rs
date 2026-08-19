//! `parse_strict` — lossy-scalar detection (§ 3.6 / § 5.2 inference).
//!
//! A scalar is *lossy* when its lexical form differs from the canonical
//! form of the inferred number: type inference would silently rewrite
//! the value (`1.10` → `1.1`, `01234` → `1234`, `0x1A` → `26`, …).
//! `parse` accepts such documents unchanged; `parse_strict` rejects
//! them with [`ErrorKind::LossyScalar`] so the author can either append
//! `::` (keep a String) or write the canonical number.

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
fn trailing_zero_float_is_lossy() {
    let (line, body, canonical) = lossy("version: 1.10\n");
    assert_eq!(
        (line, body.as_str(), canonical.as_str()),
        (1, "1.10", "1.1")
    );
}

#[test]
fn leading_zero_integer_is_lossy() {
    let (_, body, canonical) = lossy("zip: 01234\n");
    assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
}

#[test]
fn plus_sign_integer_is_lossy() {
    let (_, body, canonical) = lossy("phone: +79991234567\n");
    assert_eq!(
        (body.as_str(), canonical.as_str()),
        ("+79991234567", "79991234567")
    );
}

#[test]
fn base_prefixed_integers_are_lossy() {
    for (src, want_body, want_canonical) in [
        ("umask: 0o755\n", "0o755", "493"),
        ("sha: 0x1A2B\n", "0x1A2B", "6699"),
        ("bits: 0b1010\n", "0b1010", "10"),
    ] {
        let (_, body, canonical) = lossy(src);
        assert_eq!(
            (body.as_str(), canonical.as_str()),
            (want_body, want_canonical),
            "src: {src:?}"
        );
    }
}

#[test]
fn underscored_integer_is_lossy() {
    let (_, body, canonical) = lossy("budget: 1_000_000\n");
    assert_eq!(
        (body.as_str(), canonical.as_str()),
        ("1_000_000", "1000000")
    );
}

#[test]
fn exponent_float_is_lossy() {
    let (_, body, canonical) = lossy("build: 5e3\n");
    assert_eq!((body.as_str(), canonical.as_str()), ("5e3", "5000.0"));
}

#[test]
fn negative_zero_is_lossy() {
    let (_, body, canonical) = lossy("offset: -0\n");
    assert_eq!((body.as_str(), canonical.as_str()), ("-0", "0"));
}

#[test]
fn error_reports_the_offending_line() {
    let (line, body, _) = lossy("service: web\nport: 8080\nversion: 1.10\n");
    assert_eq!((line, body.as_str()), (3, "1.10"));
}

#[test]
fn lossy_inside_inline_object_is_rejected() {
    let (_, body, canonical) = lossy("cfg: {version: 1.10, port: 8080}\n");
    assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
}

#[test]
fn lossy_inside_inline_array_is_rejected() {
    let (_, body, _) = lossy("versions: [1.9, 1.10]\n");
    assert_eq!(body, "1.10");
}

#[test]
fn lossy_bare_array_item_is_rejected() {
    let (line, body, _) = lossy("zips: [\n    98101\n    01234\n]\n");
    assert_eq!((line, body.as_str()), (3, "01234"));
}

#[test]
fn lossy_in_top_level_inline_document_is_rejected() {
    let (_, body, _) = lossy("{version: 1.10}\n");
    assert_eq!(body, "1.10");
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
fn non_strict_parse_behaviour_is_unchanged() {
    let doc = ktav::parse("version: 1.10\nzip: 01234\n").expect("valid lax Ktav");
    let Value::Object(top) = &doc else {
        panic!("top-level must be an object");
    };
    let Some(Value::Float(v)) = top.get("version") else {
        panic!("version must stay an inferred Float");
    };
    assert_eq!(v.as_str(), "1.1");
    let Some(Value::Integer(z)) = top.get("zip") else {
        panic!("zip must stay an inferred Integer");
    };
    assert_eq!(z.as_str(), "1234");
}

#[test]
fn lossy_error_display_names_body_and_canonical() {
    let err = ktav::parse_strict("version: 1.10\n").expect_err("must be lossy");
    let msg = err.to_string();
    assert!(msg.contains("LossyScalar"), "message: {msg}");
    assert!(msg.contains("1.10"), "message: {msg}");
    assert!(msg.contains("1.1"), "message: {msg}");
}
