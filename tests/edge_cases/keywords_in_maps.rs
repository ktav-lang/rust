//! Keywords (`null` / `true` / `false`) appearing as map keys, map values,
//! and both simultaneously.

use std::collections::BTreeMap;

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn keyword_strings_as_map_keys() {
    // Map keys are written as-is (identifier validation allows them).
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("true".into(), "yes".into());
    m.insert("false".into(), "no".into());
    m.insert("null".into(), "empty".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn keyword_strings_as_map_values_get_double_colon() {
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("a".into(), "true".into());
    m.insert("b".into(), "null".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("a:: true"));
    assert!(s.contains("b:: null"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn bool_values_in_map_emit_keyword() {
    let mut m: BTreeMap<String, bool> = BTreeMap::new();
    m.insert("on".into(), true);
    m.insert("off".into(), false);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, bool>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("on: true"));
    assert!(s.contains("off: false"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn integer_map_keys_serialize_as_strings_and_round_trip() {
    let mut m: BTreeMap<u32, String> = BTreeMap::new();
    m.insert(1, "one".into());
    m.insert(42, "answer".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<u32, String>,
    }
    let cfg = Cfg { m };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn capitalized_true_in_map_stays_plain_string() {
    // Only lowercase forms trigger `::`. `True` is just a string.
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("x".into(), "True".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("x: True"));
    assert!(!s.contains("x:: True"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}
