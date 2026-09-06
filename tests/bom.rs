//! Spec § 3.1 regression tests: leading byte-order mark (U+FEFF) handling.
//!
//! A parser-conforming implementation skips exactly one leading U+FEFF
//! if and only if it is the very first code point of the document; a
//! U+FEFF anywhere else is ordinary content, and the canonical writer
//! (§ 5.9) never emits a leading byte-order mark.

use serde::Deserialize;

use ktav::{emit_canonical, parse_events, ObjectMap, Value};

/// U+FEFF — its UTF-8 encoding is `EF BB BF`.
const BOM: &str = "\u{FEFF}";

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// The `Value` stored under a top-level key.
fn pair_value<'a>(value: &'a Value, key: &str) -> &'a Value {
    match value {
        Value::Object(map) => {
            map.get(key).unwrap_or_else(|| panic!("missing key {key:?}"))
        }
        _ => panic!("expected Object root, got {value:?}"),
    }
}

/// The string payload of a top-level `key: value` pair.
fn pair_string<'a>(value: &'a Value, key: &str) -> &'a str {
    pair_value(value, key)
        .as_str()
        .unwrap_or_else(|| panic!("{key} is not a String: {:?}", pair_value(value, key)))
}

/// The BOM must not appear anywhere in a parsed Value tree.
fn assert_no_bom(value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                assert!(!k.contains(BOM), "BOM leaked into key {k:?}");
                assert_no_bom(v);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_no_bom(item);
            }
        }
        Value::String(body) => {
            assert!(!body.contains(BOM), "BOM leaked into string {body:?}")
        }
        _ => {}
    }
}

#[test]
fn leading_bom_parses_identically_to_bom_less_document() {
    let plain = "port: 8080\nhost: example.com\n";
    let with_bom = format!("{BOM}{plain}");
    assert_eq!(&with_bom.as_bytes()[..3], &[0xEF, 0xBB, 0xBF]);

    let bare = ktav::parse(plain).unwrap();
    let bommed = ktav::parse(&with_bom).unwrap();
    assert_eq!(bare, bommed);
    assert_no_bom(&bommed);

    // The strict variant goes through the same entry point.
    assert_eq!(ktav::parse_strict(&with_bom).unwrap(), bare);
}

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    port: u16,
    host: String,
}

#[test]
fn leading_bom_is_skipped_on_serde_and_event_paths() {
    let plain = "port: 8080\nhost: example.com\n";
    let with_bom = format!("{BOM}{plain}");

    let bare: Config = ktav::from_str(plain).unwrap();
    let bommed: Config = ktav::from_str(&with_bom).unwrap();
    assert_eq!(bare, bommed);

    // The zero-copy event path tokenizes both documents identically.
    let mut with = Vec::new();
    let mut without = Vec::new();
    parse_events(&with_bom, |ev| with.push(format!("{ev:?}"))).unwrap();
    parse_events(plain, |ev| without.push(format!("{ev:?}"))).unwrap();
    assert_eq!(with, without);
    assert!(!with.iter().any(|e| e.contains(BOM)));
}

#[test]
fn bom_elsewhere_is_ordinary_content() {
    // Mid-string.
    let v = ktav::parse("note: a\u{FEFF}b\n").unwrap();
    assert_eq!(pair_string(&v, "note"), "a\u{FEFF}b");

    // A value beginning with U+FEFF — at absolute offset 7, not 0.
    let v = ktav::parse(&format!("note: {BOM}keep\n")).unwrap();
    assert_eq!(pair_string(&v, "note"), format!("{BOM}keep"));

    // A later line whose leading byte is U+FEFF: as an array item it is
    // the item's first content byte, not a document-level mark.
    let v = ktav::parse(&format!("items: [\n{BOM}alpha\n]\n")).unwrap();
    match pair_value(&v, "items") {
        Value::Array(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], s(&format!("{BOM}alpha")));
        }
        other => panic!("expected Array under items, got {other:?}"),
    }

    // Same on the serde path.
    let cfg: Config = ktav::from_str(&format!("port: 1\nhost: {BOM}proxy\n")).unwrap();
    assert_eq!(cfg.host, format!("{BOM}proxy"));
}

#[test]
fn canonical_writer_never_emits_leading_bom() {
    let value = obj(&[
        ("host", s(&format!("{BOM}proxy.example"))),
        ("port", Value::Integer("1080".parse().unwrap())),
    ]);

    let text = emit_canonical(&value).unwrap();
    // Canonical output never adds a byte-order mark: output offset 0
    // is the first key's `h`, never `EF BB BF`.
    assert!(!text.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(text.starts_with("host: "));
    // The U+FEFF itself is emitted literally, as ordinary content.
    assert!(text.contains(BOM));

    // Round-trip: the emitted BOM is not at byte offset 0, so parsing
    // the canonical text yields it verbatim.
    let back = ktav::parse(&text).unwrap();
    assert_eq!(pair_string(&back, "host"), format!("{BOM}proxy.example"));
}

#[test]
fn leading_bom_is_not_re_emitted_by_canonical_writer() {
    // § 3.1: a document that arrived with a leading BOM re-emits
    // canonically WITHOUT it.
    let parsed = ktav::parse(&format!("{BOM}host: value\n")).unwrap();
    assert_eq!(emit_canonical(&parsed).unwrap(), "host: value\n");
}
