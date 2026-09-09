//! `ser::to_value` must store the `Value` that `parse()` of the emitted
//! decimal literal would store: `Value::Integer` inside the parser's i64
//! domain, `Value::String` beyond it (see fixtures
//! `i64_overflow_to_string` / `big_overflow_to_string`). A variant flip
//! breaks `to_value -> emit_canonical/render -> parse` equality — derived
//! `PartialEq` compares stored payloads and the raw marker separator
//! (`k::`) that the String variant forces in canonical output.
//!
//! Guards review finding **R14-F2**: the `serialize_u64` / `serialize_i128`
//! / `serialize_u128` paths in `src/ser/value_serializer.rs` must consult
//! `crate::parser::classify::try_parse_integer` and store `Value::Integer`
//! exactly when the parser would.

use std::collections::BTreeMap;

use ktav::render::{emit_canonical, render};
use ktav::ser::to_value;
use ktav::{parse, Value};
use serde::Serialize;

#[test]
fn to_value_keeps_i64_bounds_integer() {
    fn check<T: Serialize + std::fmt::Debug + Copy>(value: T, expected: &str) {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({map:?}): {e}"));
        let got = v
            .as_object()
            .and_then(|o| o.get("k"))
            .and_then(Value::as_integer)
            .unwrap_or_else(|| panic!("expected Integer, got {v:?}"));
        assert_eq!(got, expected, "payload mismatch for {value:?}");
    }

    check(i64::MIN, "-9223372036854775808");
    check(i64::MAX, "9223372036854775807");
}

#[test]
fn to_value_stores_string_beyond_i64_domain() {
    fn check_beyond<T: Serialize + std::fmt::Debug>(value: T, expected: &str) {
        let debug = format!("{value:?}");
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({map:?}): {e}"));
        let got = v
            .as_object()
            .and_then(|o| o.get("k"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("expected String beyond i64 domain, got {v:?}"));
        assert_eq!(got, expected, "payload mismatch for {debug}");
    }

    check_beyond((i64::MAX as u64) + 1, "9223372036854775808");
    check_beyond((i64::MAX as u128) + 1, "9223372036854775808");
    check_beyond((i64::MAX as i128) + 1, "9223372036854775808");
    check_beyond((i64::MIN as i128) - 1, "-9223372036854775809");
    check_beyond(u64::MAX, "18446744073709551615");
    check_beyond(u128::MAX, "340282366920938463463374607431768211455");
    check_beyond(i128::MIN, "-170141183460469231731687303715884105728");
    check_beyond(i128::MAX, "170141183460469231731687303715884105727");
}

#[test]
fn roundtrip_object_and_array_preserve_integer_equality() {
    fn check(v: &ktav::Value) {
        let canon = emit_canonical(v).unwrap_or_else(|e| panic!("emit_canonical({v:?}): {e}"));
        let back = parse(&canon).unwrap_or_else(|e| panic!("parse({canon:?}): {e}"));
        assert_eq!(
            &back, v,
            "canonical round-trip broke equality (text {canon:?})"
        );
        // Idempotence: a variant flip would change the bytes (`k: <digits>`
        // becomes `k:: <digits>`).
        let canon2 =
            emit_canonical(&back).unwrap_or_else(|e| panic!("emit_canonical({back:?}): {e}"));
        assert_eq!(canon2, canon, "canonical not idempotent (text {canon:?})");

        let plain = render(v).unwrap_or_else(|e| panic!("render({v:?}): {e}"));
        let back = parse(&plain).unwrap_or_else(|e| panic!("parse({plain:?}): {e}"));
        assert_eq!(
            &back, v,
            "render round-trip broke equality (text {plain:?})"
        );
    }

    fn check_both_roots<T: Serialize + std::fmt::Debug + Clone>(value: &T) {
        let mut map = BTreeMap::new();
        map.insert("k", value.clone());
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);

        let v =
            to_value(&vec![value.clone()]).unwrap_or_else(|e| panic!("to_value({value:?}): {e}"));
        check(&v);
    }

    check_both_roots(&i64::MIN);
    check_both_roots(&i64::MAX);
    check_both_roots(&((i64::MAX as u64) + 1));
    check_both_roots(&u64::MAX);
    check_both_roots(&((i64::MAX as u128) + 1));
    check_both_roots(&u128::MAX);
    check_both_roots(&((i64::MAX as i128) + 1));
    check_both_roots(&((i64::MIN as i128) - 1));
    check_both_roots(&i128::MIN);
    check_both_roots(&i128::MAX);
}

#[test]
fn canonical_bytes_for_object_root_are_stable() {
    fn check<T: Serialize + std::fmt::Debug + Copy>(value: T, expected: &str) {
        let mut map = BTreeMap::new();
        map.insert("k", value);
        let v = to_value(&map).unwrap_or_else(|e| panic!("to_value({map:?}): {e}"));
        let text = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical({map:?}): {e}"));
        assert_eq!(text, expected, "canonical bytes changed for {value:?}");
    }

    check(i64::MAX, "k: 9223372036854775807\n");
    check(u64::MAX, "k:: 18446744073709551615\n");

    let v = to_value(&vec![u64::MAX]).unwrap_or_else(|e| panic!("to_value(array): {e}"));
    let text = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical(array): {e}"));
    let back = parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(
        back,
        Value::Array(vec![Value::String("18446744073709551615".into())]),
        "array round-trip broke variant (text {text:?})"
    );
}
