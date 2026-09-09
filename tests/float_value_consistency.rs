//! `ser::to_value` must store the parser's normalized Float payload — the
//! raw `ryu` shortest form (`"1e100"`, `"5e-324"`), NOT a text-literal
//! variant with `.0` mantissa padding (`"1.0e100"`). The padding exists
//! only for the streaming text writer (`format_f64`); the parser never
//! produces it (see `src/parser/classify.rs` / `inline.rs`, which store the
//! raw ryu output). Since `Value`'s `PartialEq` compares stored strings,
//! the padding breaks `to_value -> emit_canonical/render -> parse`
//! equality for any float rendered in dot-less scientific form.
//!
//! Guards review finding **R12-F1** (payload normalization) and **R12-F2**
//! (canonical output bytes must not change when the payload is fixed).
//! The round-trip tests here are the review repro.

use std::collections::BTreeMap;

use ktav::render::{emit_canonical, render};
use ktav::ser::to_value;
use ktav::{parse, Error, ReasonCode, Value};
use serde::Serialize;

/// Extract the stored Float payload of the single value in a single-pair
/// map produced by `to_value`.
fn payload_of<T: Serialize + std::fmt::Debug>(map: &T) -> String {
    let v = to_value(map).unwrap_or_else(|e| panic!("to_value({map:?}): {e}"));
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("expected Object root, got {v:?}"));
    assert_eq!(obj.len(), 1, "expected single-pair map, got {v:?}");
    obj.values()
        .next()
        .and_then(Value::as_float)
        .unwrap_or_else(|| panic!("not a Float: {v:?}"))
        .to_string()
}

#[test]
fn to_value_stores_parser_normalized_f64_payload() {
    // Exact expected strings are the parser's stored forms (raw ryu).
    let cases: &[(f64, &str)] = &[
        (1e100, "1e100"),
        (1e-100, "1e-100"),
        (f64::from_bits(1), "5e-324"),
        (-f64::from_bits(1), "-5e-324"),
        (0.0, "0.0"),
        (-0.0, "-0.0"),
        (1.5, "1.5"),
        (1.0, "1.0"),
    ];
    for &(value, expected) in cases {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let got = payload_of(&map);
        assert_eq!(got, expected, "payload mismatch for f64 {value:?}");
    }
}

#[test]
fn to_value_stores_parser_normalized_f32_payload() {
    let cases: &[(f32, &str)] = &[
        (1e20, "1e20"),
        (1e-20, "1e-20"),
        (0.1, "0.1"),
        (1.5, "1.5"),
        (3.0, "3.0"),
    ];
    for &(value, expected) in cases {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let got = payload_of(&map);
        assert_eq!(got, expected, "payload mismatch for f32 {value:?}");
    }
}

/// R12-F1 repro: `to_value -> writer -> parse` must preserve `Value`
/// equality (derived PartialEq compares stored payload strings) for both
/// writers and both Object and Array roots.
#[test]
fn roundtrip_object_and_array_preserve_float_equality() {
    fn check(v: &ktav::Value) {
        let canon = emit_canonical(v).unwrap_or_else(|e| panic!("emit_canonical({v:?}): {e}"));
        let back = parse(&canon).unwrap_or_else(|e| panic!("parse({canon:?}): {e}"));
        assert_eq!(
            &back, v,
            "canonical round-trip broke equality (text {canon:?})"
        );
        let plain = render(v).unwrap_or_else(|e| panic!("render({v:?}): {e}"));
        let back = parse(&plain).unwrap_or_else(|e| panic!("parse({plain:?}): {e}"));
        assert_eq!(
            &back, v,
            "render round-trip broke equality (text {plain:?})"
        );
    }

    let f64_cases = [
        1e100,
        -1e100,
        1e-100,
        f64::from_bits(1),
        -f64::from_bits(1),
        0.0,
        -0.0,
        1.5,
        -2.78,
        0.01,
        0.001,
        9999999.0,
        1e7,
        1.0,
    ];
    for value in f64_cases {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);

        let v = to_value(&vec![value]).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);
    }

    let f32_cases = [1e20_f32, 1e-20, 0.1, 16777216.0];
    for value in f32_cases {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);

        let v = to_value(&vec![value]).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);
    }
}

/// R12-F2 guard: fixing the stored payload must not change the canonical
/// bytes. These assert CURRENT output and must pass before AND after the
/// fix.
#[test]
fn canonical_output_bytes_unchanged() {
    fn check_f64(value: f64, expected: &str) {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        let text = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical({value:?}): {e}"));
        assert_eq!(text, expected, "canonical bytes changed for f64 {value:?}");
    }
    fn check_f32(value: f32, expected: &str) {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        let text = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical({value:?}): {e}"));
        assert_eq!(text, expected, "canonical bytes changed for f32 {value:?}");
    }

    check_f64(0.0, "k: 0.0\n");
    check_f64(-0.0, "k: -0.0\n");
    check_f64(1.5, "k: 1.5\n");
    check_f64(-2.78, "k: -2.78\n");
    check_f64(1.0, "k: 1.0\n");
    check_f64(0.01, "k: 0.01\n"); // 1e-2 is the closed lower bound: decimal region
    check_f64(0.001, "k: 1e-3\n"); // below 1e-2: scientific
    check_f64(9999999.0, "k: 9999999.0\n"); // decimal region, < 1e7
    check_f64(1e7, "k: 1e7\n"); // upper bound excluded: scientific
    check_f64(1e100, "k: 1e100\n");
    check_f64(1e-100, "k: 1e-100\n");
    check_f64(f64::from_bits(1), "k: 5e-324\n");
    check_f64(-f64::from_bits(1), "k: -5e-324\n");

    check_f32(1e20, "k: 1e20\n");
    check_f32(1e-20, "k: 1e-20\n");
    check_f32(0.1, "k: 0.1\n");
    check_f32(16777216.0, "k: 1.6777216e7\n");

    // Root Array canonical form.
    let v = to_value(&vec![1.5_f64, 1e100]).unwrap_or_else(|e| panic!("to_value(array): {e}"));
    let text = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical(array): {e}"));
    assert_eq!(text, "1.5\n1e100\n");
}

/// Non-finite floats stay rejected by `to_value` with the exact reason
/// code — before and after the payload fix.
#[test]
fn non_finite_floats_still_rejected_in_to_value() {
    fn code_of<T: Serialize>(value: &T) -> ReasonCode {
        match to_value(value) {
            Err(e) => e
                .reason_code()
                .unwrap_or_else(|| panic!("expected a reason code, got: {e}")),
            Ok(v) => panic!("expected Unrepresentable(NonFiniteFloat), got: {v:?}"),
        }
    }

    let mut m = BTreeMap::new();
    m.insert("k", f64::NAN);
    assert_eq!(code_of(&m), ReasonCode::NonFiniteFloat);

    let mut m = BTreeMap::new();
    m.insert("k", f64::INFINITY);
    assert_eq!(code_of(&m), ReasonCode::NonFiniteFloat);

    let mut m = BTreeMap::new();
    m.insert("k", f64::NEG_INFINITY);
    assert_eq!(code_of(&m), ReasonCode::NonFiniteFloat);

    let mut m = BTreeMap::new();
    m.insert("k", f32::NAN);
    assert_eq!(code_of(&m), ReasonCode::NonFiniteFloat);

    let mut m = BTreeMap::new();
    m.insert("k", f32::INFINITY);
    assert_eq!(code_of(&m), ReasonCode::NonFiniteFloat);

    let err: Error = to_value(&vec![f64::NAN]).unwrap_err();
    assert_eq!(
        err.reason_code(),
        Some(ReasonCode::NonFiniteFloat),
        "array root: {err}"
    );
}
