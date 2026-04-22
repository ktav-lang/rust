//! `null` / `true` / `false` keyword deserialization — strict lowercase.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn true_keyword_becomes_bool_true() {
    #[derive(Deserialize)]
    struct Cfg {
        on: bool,
    }
    let cfg: Cfg = from_str("on: true").unwrap();
    assert!(cfg.on);
}

#[test]
fn false_keyword_becomes_bool_false() {
    #[derive(Deserialize)]
    struct Cfg {
        on: bool,
    }
    let cfg: Cfg = from_str("on: false").unwrap();
    assert!(!cfg.on);
}

#[test]
fn null_keyword_becomes_none() {
    #[derive(Deserialize)]
    struct Cfg {
        x: Option<String>,
    }
    let cfg: Cfg = from_str("x: null").unwrap();
    assert!(cfg.x.is_none());
}

#[test]
fn capitalized_true_is_a_string_not_a_bool() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x: True").unwrap();
    assert_eq!(cfg.x, "True");
}

#[test]
fn uppercase_false_is_a_string_not_a_bool() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x: FALSE").unwrap();
    assert_eq!(cfg.x, "FALSE");
}

#[test]
fn capitalized_null_is_a_string_not_null() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x: Null").unwrap();
    assert_eq!(cfg.x, "Null");
}

#[test]
fn word_starting_with_true_is_a_string() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x: truesome").unwrap();
    assert_eq!(cfg.x, "truesome");
}

#[test]
fn true_inside_array_becomes_bool() {
    #[derive(Deserialize)]
    struct Cfg {
        flags: Vec<bool>,
    }
    let cfg: Cfg = from_str("flags: [\n  true\n  false\n  true\n]\n").unwrap();
    assert_eq!(cfg.flags, vec![true, false, true]);
}

#[test]
fn null_inside_array_becomes_none() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<Option<String>>,
    }
    let cfg: Cfg = from_str("items: [\n  a\n  null\n  b\n]\n").unwrap();
    assert_eq!(
        cfg.items,
        vec![Some("a".to_string()), None, Some("b".to_string())]
    );
}
