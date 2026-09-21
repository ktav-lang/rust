use indexmap::IndexMap;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map as JsonMap, Value as Json};

use super::envelope::message_envelope;
use crate::value::{ObjectMap, Value};

/// The dumps-family front half: JSON wire bytes -> `Value`, with every
/// failure — serde_json parse, tagged-payload validation, the
/// top-level object/array rule — reported as an envelope. The input
/// was a wire value, not Ktav text, so envelopes are built against an
/// empty source (honest nulls for `line` / `line_text` / `span`).
pub(super) fn wire_to_value(src: &[u8]) -> Result<Value, String> {
    let wire: WireValue = serde_json::from_slice(src)
        .map_err(|err| message_envelope(format!("input JSON: {err}"), ""))?;
    let value = wire.into_value().map_err(|e| message_envelope(e, ""))?;
    if !matches!(value, Value::Object(_) | Value::Array(_)) {
        return Err(message_envelope(
            "top-level Ktav document must be an object or array",
            "",
        ));
    }
    Ok(value)
}

pub(super) fn value_to_json(v: &Value) -> Json {
    match v {
        Value::Null => Json::Null,
        Value::Bool(b) => Json::Bool(*b),
        Value::Integer(s) => {
            let mut m = JsonMap::new();
            m.insert("$i".to_string(), Json::String(s.to_string()));
            Json::Object(m)
        }
        Value::Float(s) => {
            let mut m = JsonMap::new();
            m.insert("$f".to_string(), Json::String(s.to_string()));
            Json::Object(m)
        }
        Value::String(s) => Json::String(s.to_string()),
        Value::Array(a) => Json::Array(a.iter().map(value_to_json).collect()),
        Value::Object(o) => {
            let mut m = JsonMap::new();
            for (k, val) in o {
                m.insert(k.to_string(), value_to_json(val));
            }
            Json::Object(m)
        }
    }
}

/// A JSON wire value that understands both plain JSON values and the
/// `{"$i": ...}` / `{"$f": ...}` tagged wrappers, preserving object key
/// order via `indexmap`.
pub enum WireValue {
    /// JSON `null`.
    Null,
    /// A JSON boolean.
    Bool(bool),
    /// An integer literal, tagged `{"$i": "<digits>"}` on the wire.
    Integer(String),
    /// A float literal, tagged `{"$f": "<text>"}` on the wire.
    Float(String),
    /// A JSON string.
    String(String),
    /// A JSON array.
    Array(Vec<WireValue>),
    /// A JSON object with insertion order preserved.
    Object(IndexMap<String, WireValue>),
}

impl WireValue {
    fn into_value(self) -> Result<Value, String> {
        match self {
            WireValue::Null => Ok(Value::Null),
            WireValue::Bool(b) => Ok(Value::Bool(b)),
            WireValue::Integer(s) => Ok(Value::Integer(canonical_integer(&s)?.into())),
            WireValue::Float(s) => Ok(Value::Float(canonical_float(&s)?.into())),
            WireValue::String(s) => Ok(Value::String(s.into())),
            WireValue::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for w in items {
                    out.push(w.into_value()?);
                }
                Ok(Value::Array(out))
            }
            WireValue::Object(m) => {
                let mut obj = ObjectMap::with_capacity_and_hasher(m.len(), Default::default());
                for (k, v) in m {
                    obj.insert(k.into(), v.into_value()?);
                }
                Ok(Value::Object(obj))
            }
        }
    }
}

/// Normalize an `$i` payload to this crate's canonical Integer text: no
/// redundant leading zero, no explicit `+`, and no signed zero — the
/// same shape the internal text parser always stores (classify.rs never
/// lets a parsed Integer keep its original spelling). String-level, not
/// `i64`-parsed: the wire format deliberately carries integers beyond
/// `i64` (that is the whole reason `$i` exists instead of a bare JSON
/// number), so a bignum payload must pass through unchanged rather than
/// overflow.
///
/// Without this, both writers (`render` and the canonical one) trust
/// that stored Integer/Float text is already canonical and echo it
/// verbatim — `render`'s own comment on its `Value::Integer` arm says
/// so explicitly. A non-canonical wire payload breaks that trust
/// silently: `"01234"` gets written out unchanged, and since § 5.2 now
/// routes a redundant leading zero to String, re-parsing that output
/// reclassifies it — `dumps` -> `loads` silently turns the Integer into
/// a String.
fn canonical_integer(s: &str) -> Result<String, String> {
    let (sign, digits) = match s.as_bytes().first() {
        Some(b'-') => ("-", &s[1..]),
        Some(b'+') => ("", &s[1..]),
        _ => ("", s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("$i payload not an integer literal: {s:?}"));
    }
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return Ok("0".to_string());
    }
    Ok(format!("{sign}{trimmed}"))
}

/// True when a number's source text is a float lexical form (`.` or
/// exponent) rather than an integer literal.
fn looks_like_float(s: &str) -> bool {
    s.bytes().any(|b| b == b'.' || b == b'e' || b == b'E')
}

/// Normalize an `$f` payload to this crate's canonical Float text the
/// same way `classify.rs` does when it parses one from source: `f64`
/// parse, then `ryu`'s shortest round-trip decimal. A non-finite result
/// — reachable here via an extreme exponent like `"1e400"`, which
/// `f64::from_str` silently overflows to `Infinity` rather than erroring
/// — is passed through verbatim instead: `ryu` only documents finite
/// input, and a non-finite `Value::Float` is a deliberate wire-only
/// capability the § 5.9.0 writers reject at render time (or coerce
/// under `force_strings`), never something to normalize away here.
fn canonical_float(s: &str) -> Result<String, String> {
    if s.parse::<f64>().is_err() {
        return Err(format!("$f payload not a finite decimal: {s:?}"));
    }
    if !looks_like_float(s) {
        return Err(format!("$f payload must contain '.' or exponent: {s:?}"));
    }
    let val: f64 = s.parse().expect("just validated above");
    if !val.is_finite() {
        return Ok(s.to_string());
    }
    let mut buf = ryu::Buffer::new();
    Ok(buf.format(val).to_string())
}

/// The string payload of a `$i` / `$f` tag (or of serde_json's
/// arbitrary-precision sentinel): anything but string-like is a
/// wire-contract violation.
fn tagged_payload<E: de::Error>(tag: &str, v: WireValue) -> Result<String, E> {
    match v {
        WireValue::String(s) | WireValue::Integer(s) | WireValue::Float(s) => Ok(s),
        _ => Err(de::Error::custom(format!("{tag} payload must be a string"))),
    }
}

impl<'de> Deserialize<'de> for WireValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = WireValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_unit<E: de::Error>(self) -> Result<WireValue, E> {
                Ok(WireValue::Null)
            }
            fn visit_none<E: de::Error>(self) -> Result<WireValue, E> {
                Ok(WireValue::Null)
            }
            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<WireValue, D::Error> {
                WireValue::deserialize(d)
            }
            fn visit_bool<E: de::Error>(self, b: bool) -> Result<WireValue, E> {
                Ok(WireValue::Bool(b))
            }
            fn visit_i64<E: de::Error>(self, n: i64) -> Result<WireValue, E> {
                Ok(WireValue::Integer(n.to_string()))
            }
            fn visit_u64<E: de::Error>(self, n: u64) -> Result<WireValue, E> {
                Ok(WireValue::Integer(n.to_string()))
            }
            fn visit_f64<E: de::Error>(self, n: f64) -> Result<WireValue, E> {
                if !n.is_finite() {
                    return Err(E::custom("NaN / ±Infinity not allowed in Ktav"));
                }
                // Bare JSON floats get the ":f" wire form with a forced
                // decimal point so render's grammar check is satisfied.
                let mut s = format!("{n}");
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    s.push_str(".0");
                }
                Ok(WireValue::Float(s))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<WireValue, E> {
                Ok(WireValue::String(v.to_string()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<WireValue, E> {
                Ok(WireValue::String(v))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<WireValue, A::Error> {
                let mut out = Vec::new();
                while let Some(item) = seq.next_element()? {
                    out.push(item);
                }
                Ok(WireValue::Array(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<WireValue, A::Error> {
                let Some(k1) = map.next_key::<String>()? else {
                    return Ok(WireValue::Object(IndexMap::new()));
                };
                let v1: WireValue = map.next_value()?;
                let second_key: Option<String> = map.next_key()?;

                // serde_json built with `arbitrary_precision` — which this
                // crate's dependency graph can force via feature
                // unification — delivers every number that is not an exact
                // i64/u64 (all floats, integers beyond 64 bits) as a
                // single-entry map keyed by its crate-private sentinel, the
                // value being the number's exact source text. Recover the
                // number so bare JSON numbers keep decoding losslessly;
                // builds without the feature never hit this branch.
                if second_key.is_none() && k1 == "$serde_json::private::Number" {
                    let payload = tagged_payload(&k1, v1)?;
                    return Ok(if looks_like_float(&payload) {
                        WireValue::Float(payload)
                    } else {
                        WireValue::Integer(payload)
                    });
                }

                if second_key.is_none() && (k1 == "$i" || k1 == "$f") {
                    let payload = tagged_payload(&k1, v1)?;
                    return Ok(if k1 == "$i" {
                        WireValue::Integer(payload)
                    } else {
                        WireValue::Float(payload)
                    });
                }

                let mut out: IndexMap<String, WireValue> = IndexMap::new();
                out.insert(k1, v1);
                if let Some(k2) = second_key {
                    let v2: WireValue = map.next_value()?;
                    out.insert(k2, v2);
                    while let Some((k, v)) = map.next_entry::<String, WireValue>()? {
                        out.insert(k, v);
                    }
                }
                Ok(WireValue::Object(out))
            }
        }

        d.deserialize_any(V)
    }
}
