//! R16-F3: the owned reader hands a long String key's heap buffer to the
//! target instead of copying it, while short numeric key names keep their
//! allocation-free borrowed parse. The buffer check is a mechanism control
//! (review round 16), not an allocator benchmark: a key longer than the
//! 24-byte inline capacity (x64) is heap-backed, and while the source
//! buffer is alive the allocator cannot reuse its address, so address
//! equality can only come from a genuine move, never from allocator reuse.

use std::collections::BTreeMap;

use ktav::value::{ObjectMap, Value};
use ktav::{de::from_value, from_str, parse, ser};

fn object_with_key(key: String) -> Value {
    let mut obj = ObjectMap::default();
    obj.insert(key.into(), Value::String("v".into()));
    Value::Object(obj)
}

#[test]
fn long_owned_string_key_buffer_is_moved_not_copied() {
    // 52 bytes — heap-backed, beyond the 24-byte inline capacity (x64).
    let long = "k".repeat(52);
    let value = object_with_key(long);
    let key_ptr = match &value {
        Value::Object(obj) => obj.keys().next().unwrap().as_str().as_ptr() as usize,
        other => panic!("expected object, got {other:?}"),
    };

    let map: BTreeMap<String, String> = from_value(value).unwrap();
    let key = map.into_keys().next().unwrap();
    assert_eq!(
        key.as_ptr() as usize,
        key_ptr,
        "long owned key buffer must be handed to the target String, not copied"
    );
    assert_eq!(key.len(), 52);
}

#[test]
fn short_and_long_string_keys_roundtrip_via_owned_path() {
    let mut src = BTreeMap::new();
    src.insert("short".to_owned(), "a".to_owned()); // inline (< 24 bytes)
    src.insert("k".repeat(52), "b".to_owned()); // heap-backed
    let back: BTreeMap<String, String> = from_value(ser::to_value(&src).unwrap()).unwrap();
    assert_eq!(back, src);

    let thin: BTreeMap<String, String> =
        from_str("short: a\nkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk: b\n").unwrap();
    assert_eq!(thin.get("short").map(String::as_str), Some("a"));
    assert_eq!(thin.values().count(), 2);
}

#[test]
fn short_numeric_key_names_keep_borrowed_parse_via_owned_path() {
    // Numeric targets parse the borrowed slice directly; the key-adapter
    // ownership change (R16-F3) must not disturb that path.
    let m: BTreeMap<u32, String> = from_value(parse("1: a\n42: b\n").unwrap()).unwrap();
    assert_eq!(m, BTreeMap::from([(1, "a".into()), (42, "b".into())]));

    let thin: BTreeMap<u32, String> = from_str("1: a\n42: b\n").unwrap();
    assert_eq!(thin, m);
}
