//! R15-F2: map key-NAME serialization must be byte-identical across the
//! two writer paths (`ktav::to_string` direct text writer and
//! `ktav::ser::to_value` + `ktav::render::render`). Both paths share one
//! key-name policy (`ser::text_serializer::serialize_key_name`), so the
//! same key value always produces the same name text and the same set of
//! key types is accepted or rejected on both paths.

use std::collections::BTreeMap;

use serde::ser::{Serialize, SerializeMap, Serializer};

use ktav::value::Value;
use ktav::{from_str, parse, render, ser, to_string};

/// Custom wrapper emitting a one-entry map with a non-String runtime key
/// (std maps cannot hold f64 keys).
struct OneEntry<K>(K);

impl<K: Serialize> Serialize for OneEntry<K> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(Some(1))?;
        m.serialize_entry(&self.0, "v")?;
        m.end()
    }
}

#[derive(serde::Serialize)]
enum Tag {
    Alpha,
}

/// Parse `text` and return its sole top-level key.
fn parsed_sole_key(text: &str) -> String {
    let value = parse(text).unwrap_or_else(|e| panic!("parse({text:?}) failed: {e}"));
    match value {
        Value::Object(map) => {
            assert_eq!(map.len(), 1, "expected a one-entry object in {text:?}");
            map.keys().next().unwrap().to_string()
        }
        other => panic!("expected object, got {other:?} from {text:?}"),
    }
}

/// Assert both writer paths accept `key` and produce the same sole key
/// name, equal to `expected`.
fn assert_both_paths<K: Serialize>(key: &K, expected: &str) {
    let direct =
        to_string(&OneEntry(key)).unwrap_or_else(|e| panic!("to_string({expected}) failed: {e}"));
    let value = ser::to_value(&OneEntry(key))
        .unwrap_or_else(|e| panic!("to_value({expected}) failed: {e}"));
    let via_value = render::render(&value)
        .unwrap_or_else(|e| panic!("render(to_value({expected})) failed: {e}"));
    let direct_key = parsed_sole_key(&direct);
    let value_key = parsed_sole_key(&via_value);
    assert_eq!(direct_key, expected, "direct text path key name");
    assert_eq!(value_key, expected, "value path key name");
    assert_eq!(direct_key, value_key, "cross-path key name");
}

#[test]
fn float_key_names_identical_across_writer_paths() {
    assert_both_paths(&1.0_f64, "1");
    assert_both_paths(&1e20_f64, "100000000000000000000");
    assert_both_paths(&1e13_f32, "10000000000000");
    assert_both_paths(&1.5_f64, "1.5");
}

#[test]
fn string_key_literal_is_never_renormalized() {
    let k = "1.0";
    let direct = to_string(&OneEntry(k)).unwrap_or_else(|e| panic!("to_string failed: {e}"));
    let value = ser::to_value(&OneEntry(k)).unwrap_or_else(|e| panic!("to_value failed: {e}"));
    let via_value = render::render(&value).unwrap_or_else(|e| panic!("render failed: {e}"));
    assert_eq!(parsed_sole_key(&direct), "1.0");
    assert_eq!(parsed_sole_key(&via_value), "1.0");
}

#[test]
fn typed_roundtrip_numeric_and_bool_keys_match_across_paths() {
    // u32 keys
    let mut m: BTreeMap<u32, String> = BTreeMap::new();
    m.insert(1, "one".into());
    m.insert(42, "answer".into());
    let direct = to_string(&m).unwrap_or_else(|e| panic!("to_string failed: {e}"));
    assert_eq!(
        from_str::<BTreeMap<u32, String>>(&direct)
            .unwrap_or_else(|e| panic!("roundtrip failed: {e}")),
        m
    );
    let value = ser::to_value(&m).unwrap_or_else(|e| panic!("to_value failed: {e}"));
    let via_value = render::render(&value).unwrap_or_else(|e| panic!("render failed: {e}"));
    assert_eq!(
        from_str::<BTreeMap<u32, String>>(&via_value)
            .unwrap_or_else(|e| panic!("value-path roundtrip failed: {e}")),
        m
    );
    assert_eq!(
        parse(&direct).unwrap_or_else(|e| panic!("parse failed: {e}")),
        parse(&via_value).unwrap_or_else(|e| panic!("parse failed: {e}"))
    );

    // bool keys
    let mut b: BTreeMap<bool, String> = BTreeMap::new();
    b.insert(true, "yes".into());
    b.insert(false, "no".into());
    let direct = to_string(&b).unwrap_or_else(|e| panic!("to_string failed: {e}"));
    assert_eq!(
        from_str::<BTreeMap<bool, String>>(&direct)
            .unwrap_or_else(|e| panic!("roundtrip failed: {e}")),
        b
    );
    let value = ser::to_value(&b).unwrap_or_else(|e| panic!("to_value failed: {e}"));
    let via_value = render::render(&value).unwrap_or_else(|e| panic!("render failed: {e}"));
    assert_eq!(
        from_str::<BTreeMap<bool, String>>(&via_value)
            .unwrap_or_else(|e| panic!("value-path roundtrip failed: {e}")),
        b
    );
    assert_eq!(
        parse(&direct).unwrap_or_else(|e| panic!("parse failed: {e}")),
        parse(&via_value).unwrap_or_else(|e| panic!("parse failed: {e}"))
    );
}

#[test]
fn wide_int_keys_accepted_identically_on_both_paths() {
    assert_both_paths(&i128::MIN, "-170141183460469231731687303715884105728");
    assert_both_paths(&u128::MAX, "340282366920938463463374607431768211455");
    assert_both_paths(&1_u128, "1");
}

#[test]
fn rejected_key_types_are_rejected_symmetrically() {
    // unit (), serialize_none, and bytes must be rejected on both paths.
    assert!(to_string(&OneEntry(())).is_err());
    assert!(ser::to_value(&OneEntry(())).is_err());
    assert!(to_string(&OneEntry(None::<u8>)).is_err());
    assert!(ser::to_value(&OneEntry(None::<u8>)).is_err());
    let bytes_key = OneEntry::<&[u8]>(b"\x01");
    assert!(to_string(&bytes_key).is_err());
    assert!(ser::to_value(&bytes_key).is_err());

    for err in [
        to_string(&OneEntry(())).unwrap_err().to_string(),
        ser::to_value(&OneEntry(())).unwrap_err().to_string(),
        to_string(&OneEntry(None::<u8>)).unwrap_err().to_string(),
        ser::to_value(&OneEntry(None::<u8>))
            .unwrap_err()
            .to_string(),
        to_string(&bytes_key).unwrap_err().to_string(),
        ser::to_value(&bytes_key).unwrap_err().to_string(),
    ] {
        assert!(
            err.contains("map keys must serialize to strings"),
            "unexpected error text: {err}"
        );
    }
}

#[test]
fn some_and_char_and_unit_variant_keys_match() {
    assert_both_paths(&Some("k") as &Option<&str>, "k");
    assert_both_paths(&'x', "x");
    assert_both_paths(&Tag::Alpha, "Alpha");
}
