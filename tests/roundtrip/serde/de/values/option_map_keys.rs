//! R16-F1: `Some(...)` map keys are accepted by both writers, so both
//! readers must read them back. Inverse matrix: 2 readers × 2 writers per
//! key form. A key name always exists (§5: Object names are strings, not
//! scalar values), so the literal name `null` stays the string `"null"`
//! and is never read as `None`.

use std::collections::BTreeMap;
use std::fmt::Debug;

use ktav::{de::from_value, from_str, parse, render, ser, to_string};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

type Map<K> = BTreeMap<K, String>;

/// The full inverse matrix: both readers against both writers.
fn assert_read_back<K>(m: &Map<K>)
where
    K: Ord + Serialize + DeserializeOwned + PartialEq + Debug,
{
    let direct = to_string(m).unwrap();
    let owned_tree = ser::to_value(m).unwrap();
    let via_value = render::render(&owned_tree).unwrap();

    assert_eq!(
        from_str::<Map<K>>(&direct).unwrap(),
        *m,
        "thin reader × direct text writer"
    );
    assert_eq!(
        from_value::<Map<K>>(parse(&direct).unwrap()).unwrap(),
        *m,
        "owned reader × direct text writer"
    );
    assert_eq!(
        from_str::<Map<K>>(&via_value).unwrap(),
        *m,
        "thin reader × value writer"
    );
    assert_eq!(
        from_value::<Map<K>>(owned_tree).unwrap(),
        *m,
        "owned reader × value writer"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Id(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Tag {
    Alpha,
}

#[test]
fn some_u32_key_roundtrips_through_both_readers_and_writers() {
    let m = Map::from([(Some(42_u32), "v".to_owned())]);
    assert_eq!(to_string(&m).unwrap(), "42: v\n");
    assert_read_back(&m);
}

#[test]
fn some_bool_key_roundtrips() {
    assert_read_back(&Map::from([(Some(true), "v".to_owned())]));
}

#[test]
fn some_char_key_roundtrips() {
    assert_read_back(&Map::from([(Some('x'), "v".to_owned())]));
}

#[test]
fn some_u128_max_key_roundtrips() {
    assert_read_back(&Map::from([(Some(u128::MAX), "v".to_owned())]));
}

#[test]
fn some_newtype_key_roundtrips() {
    assert_read_back(&Map::from([(Some(Id(7)), "v".to_owned())]));
}

#[test]
fn some_unit_enum_key_roundtrips() {
    assert_read_back(&Map::from([(Some(Tag::Alpha), "v".to_owned())]));
}

#[test]
fn some_nested_option_key_roundtrips() {
    // `Some(Some(42))` writes the name "42" and reads back through two
    // `visit_some` hops.
    assert_read_back(&Map::from([(Some(Some(42_u32)), "v".to_owned())]));
}

#[test]
fn literal_null_name_is_some_string_not_none() {
    let m = Map::from([(Some("null".to_owned()), "v".to_owned())]);
    assert_eq!(to_string(&m).unwrap(), "null: v\n");
    assert_read_back(&m);

    // The read-back key must be `Some("null")` — a key name always exists;
    // the literal `null` is a string, not a scalar value (§5).
    let thin = from_str::<Map<Option<String>>>(&to_string(&m).unwrap()).unwrap();
    assert!(thin.contains_key(&Some("null".to_owned())));
    assert!(!thin.contains_key(&None));

    // For a non-String inner type the name still cannot become `None`:
    // it fails to parse as u32 instead.
    for err in [
        from_str::<Map<u32>>("null: v\n").unwrap_err().to_string(),
        from_value::<Map<u32>>(parse("null: v\n").unwrap())
            .unwrap_err()
            .to_string(),
    ] {
        assert!(
            err.contains("failed to parse map key 'null' as u32"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn writer_rejection_of_none_and_some_none_is_unchanged() {
    // Both writers reject `None` and `Some(None)` (at any depth) as keys.
    assert!(to_string(&Map::from([(None::<u32>, "v".to_owned())])).is_err());
    assert!(ser::to_value(&Map::from([(None::<u32>, "v".to_owned())])).is_err());
    assert!(to_string(&Map::from([(Some(None::<u32>), "v".to_owned())])).is_err());
    assert!(ser::to_value(&Map::from([(Some(None::<u32>), "v".to_owned())])).is_err());
    assert!(to_string(&Map::from([(Some(Some(None::<u32>)), "v".to_owned())])).is_err());
    assert!(ser::to_value(&Map::from([(Some(Some(None::<u32>)), "v".to_owned())])).is_err());
}
