//! Oracle conversion and comparison helpers (ktav Value <-> JSON).

use ktav::{ObjectMap, Value};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

/// Convert a `ktav::Value` into a `serde_json::Value` using the 1:1 mapping
/// from the spec.
pub(crate) fn ktav_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::String(s) => JsonValue::String(s.to_string()),
        Value::Integer(s) => {
            // Under 0.5.0, Integer stores canonical base-10 decimal.
            let n = JsonNumber::from_string_unchecked(s.to_string());
            JsonValue::Number(n)
        }
        Value::Float(s) => {
            // Under 0.5.0, Float stores canonical form via ryu.
            // ryu may output forms like "0.5" which are valid JSON numbers.
            let text = s.to_string();
            let n = JsonNumber::from_string_unchecked(text);
            JsonValue::Number(n)
        }
        Value::Array(items) => JsonValue::Array(items.iter().map(ktav_to_json).collect()),
        Value::Object(obj) => {
            let mut map = JsonMap::new();
            for (k, v) in obj {
                map.insert(k.to_string(), ktav_to_json(v));
            }
            JsonValue::Object(map)
        }
    }
}

/// Ordered recursive comparison. Numbers are kind-aware and exact:
/// an integer-spelled token only matches an integer-spelled token and
/// a float-spelled token only a float-spelled one (`1` != `1.0`);
/// integers compare exactly at any magnitude (no f64 round-trip);
/// floats compare as binary64 values with the sign of zero preserved
/// (`0.0` != `-0.0`, spec § 5.9.8), so different spellings of the same
/// value (`1e9` vs `1000000000.0`) still compare equal.
pub(crate) fn json_eq_ordered(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(x), JsonValue::Bool(y)) => x == y,
        (JsonValue::Number(x), JsonValue::Number(y)) => {
            let x_str = x.to_string();
            let y_str = y.to_string();
            match (
                json_number_is_float_spelled(&x_str),
                json_number_is_float_spelled(&y_str),
            ) {
                (false, false) => json_integers_equal(&x_str, &y_str),
                (true, true) => json_floats_equal(&x_str, &y_str),
                // Kind mismatch: a ktav Integer must never equal a Float
                // regardless of numeric value, in either direction.
                (false, true) | (true, false) => false,
            }
        }
        (JsonValue::String(x), JsonValue::String(y)) => x == y,
        (JsonValue::Array(x), JsonValue::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq_ordered(a, b))
        }
        (JsonValue::Object(x), JsonValue::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .zip(y.iter())
                    .all(|((ka, va), (kb, vb))| ka == kb && json_eq_ordered(va, vb))
        }
        _ => false,
    }
}

/// Classify a JSON number token by its lexical spelling, the same rule
/// `json_to_ktav` applies in the oracle→Value direction: `.`/`e`/`E`
/// makes it float-spelled, otherwise integer-spelled. This recovers the
/// original `ktav::Value` kind on the ktav side because both scalar
/// kinds store canonical spellings (§ 5.9.8: Integer is base-10
/// decimal; Float is ryu decimal/scientific and always carries `.` or
/// `e`), and `serde_json` is compiled with `arbitrary_precision`, so
/// `Number::to_string()` returns the exact token.
fn json_number_is_float_spelled(token: &str) -> bool {
    token.contains(['.', 'e', 'E'])
}

/// Exact integer comparison. JSON integers carry no exponent or
/// leading zeros, so after normalising `-0`, string equality is exact
/// numeric equality at any magnitude — no f64 round-trip, which cannot
/// represent all i64 values (e.g. `9223372036854775807` parses to the
/// same f64 as `9223372036854775806`).
fn json_integers_equal(x: &str, y: &str) -> bool {
    let norm = if x == "-0" { "0" } else { x };
    let normy = if y == "-0" { "0" } else { y };
    norm == normy
}

/// Float comparison within the crate's binary64 domain. Values compare
/// as `f64` (so `1e9` == `1000000000.0`), except that the sign of zero
/// is significant: § 5.9.8 keeps `0.0` and `-0.0` as distinct canonical
/// spellings, so the comparator must not collapse them through IEEE
/// 754 `0.0 == -0.0`. Spellings outside the finite binary64 domain
/// (e.g. `1e400`, which parses to infinity) have no value to compare —
/// they match only textually.
fn json_floats_equal(x: &str, y: &str) -> bool {
    match (x.parse::<f64>(), y.parse::<f64>()) {
        (Ok(xf), Ok(yf)) if xf.is_finite() && yf.is_finite() => {
            if xf == 0.0 && yf == 0.0 {
                xf.is_sign_negative() == yf.is_sign_negative()
            } else {
                xf == yf
            }
        }
        _ => x == y,
    }
}

#[test]
pub(crate) fn json_eq_ordered_number_semantics() {
    let n = |s: &str| JsonValue::Number(JsonNumber::from_string_unchecked(s.to_string()));

    // Kind is respected in both directions (review finding F5).
    assert!(json_eq_ordered(&n("1"), &n("1")));
    assert!(!json_eq_ordered(&n("1"), &n("1.0")));
    assert!(!json_eq_ordered(&n("1.0"), &n("1")));
    assert!(!json_eq_ordered(&n("0"), &n("0.0")));

    // Integers compare exactly — no f64 round-trip: both spellings
    // parse to the same f64 but are different integers.
    assert!(json_eq_ordered(
        &n("9223372036854775806"),
        &n("9223372036854775806")
    ));
    assert!(!json_eq_ordered(
        &n("9223372036854775806"),
        &n("9223372036854775807")
    ));
    assert!(json_eq_ordered(
        &n("-9223372036854775808"),
        &n("-9223372036854775808")
    ));

    // Floats compare as binary64 values — different spellings of the
    // same value are equal...
    assert!(json_eq_ordered(&n("1e9"), &n("1000000000.0")));
    assert!(json_eq_ordered(
        &n("9007199254740992.0"),
        &n("9.007199254740992e15")
    ));
    // ...but the sign of zero is significant (spec § 5.9.8).
    assert!(json_eq_ordered(&n("0.0"), &n("0.0")));
    assert!(json_eq_ordered(&n("-0.0"), &n("-0.0")));
    assert!(!json_eq_ordered(&n("0.0"), &n("-0.0")));
    assert!(!json_eq_ordered(&n("-0.0"), &n("0.0")));

    // Non-number leaves are unaffected.
    assert!(json_eq_ordered(
        &JsonValue::Array(vec![n("1"), n("2")]),
        &JsonValue::Array(vec![n("1"), n("2")])
    ));
    assert!(!json_eq_ordered(
        &JsonValue::Array(vec![n("1"), n("2")]),
        &JsonValue::Array(vec![n("1"), n("2.0")])
    ));
}

/// Convert a `serde_json::Value` (a fixture oracle) into a `ktav::Value`.
pub(crate) fn json_to_ktav(v: &JsonValue) -> Value {
    match v {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::String(s) => {
            Value::String(s.parse().unwrap_or_else(|_| panic!("string scalar: {s:?}")))
        }
        JsonValue::Number(n) => {
            // serde_json is compiled with `arbitrary_precision`, so
            // `to_string()` returns the exact lexical token.
            let token = n.to_string();
            if !token.contains(['.', 'e', 'E']) {
                Value::Integer(
                    token
                        .parse()
                        .unwrap_or_else(|_| panic!("integer scalar: {token:?}")),
                )
            } else {
                Value::Float(
                    token
                        .parse()
                        .unwrap_or_else(|_| panic!("float scalar: {token:?}")),
                )
            }
        }
        JsonValue::Array(items) => Value::Array(items.iter().map(json_to_ktav).collect()),
        JsonValue::Object(map) => {
            // `$float` sentinel: only meaningful in the `unrepresentable/`
            // category — records NaN / ±Infinity as a one-field object.
            if map.len() == 1 {
                if let Some(JsonValue::String(f)) = map.get("$float") {
                    return Value::Float(
                        f.parse()
                            .unwrap_or_else(|_| panic!("$float sentinel: {f:?}")),
                    );
                }
            }
            let mut m = ObjectMap::default();
            for (k, val) in map {
                m.insert(k.as_str().into(), json_to_ktav(val));
            }
            Value::Object(m)
        }
    }
}

/// Serde mirror of a `ktav::Value`: lets the serde text serializer
/// (`ktav::to_string`) be exercised on Values (which deliberately do not
/// implement `Serialize`). Scalars go through their canonical text form;
/// non-finite floats surface as raw `f64`s so the serializer's own
/// rejection (`NonFiniteFloat`) fires exactly as it would for a real
/// `Serialize` type holding the same value.
pub(crate) struct SerValue<'a>(pub(crate) &'a Value);

impl Serialize for SerValue<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Null => ser.serialize_none(),
            Value::Bool(b) => ser.serialize_bool(*b),
            Value::Integer(i) => {
                let n: i64 = i.to_string().parse().map_err(serde::ser::Error::custom)?;
                ser.serialize_i64(n)
            }
            Value::Float(f) => {
                ser.serialize_f64(f.to_string().parse().map_err(serde::ser::Error::custom)?)
            }
            Value::String(s) => ser.serialize_str(s.as_ref()),
            Value::Array(items) => ser.collect_seq(items.iter().map(SerValue)),
            Value::Object(obj) => {
                let mut map = ser.serialize_map(Some(obj.len()))?;
                for (k, v) in obj {
                    map.serialize_entry(&k.to_string(), &SerValue(v))?;
                }
                map.end()
            }
        }
    }
}
