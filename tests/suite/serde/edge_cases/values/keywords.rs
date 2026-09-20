//! Round-trip for `null` / `true` / `false`.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn bool_true() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        on: bool,
    }
    let cfg = Cfg { on: true };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn bool_false() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        on: bool,
    }
    let cfg = Cfg { on: false };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn option_none_round_trips_as_null() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        label: Option<String>,
    }
    let cfg = Cfg { label: None };
    let text = to_string(&cfg).unwrap();
    assert_eq!(text, "label: null\n");
    let back: Cfg = from_str(&text).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn option_some_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        label: Option<String>,
    }
    let cfg = Cfg {
        label: Some("hello".into()),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn string_equal_to_keyword_round_trips_via_double_colon() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        x: String,
        y: String,
        z: String,
    }
    let cfg = Cfg {
        x: "true".into(),
        y: "false".into(),
        z: "null".into(),
    };
    let text = to_string(&cfg).unwrap();
    // All three strings emit with `::` so they survive a round-trip.
    assert!(text.contains("x:: true"));
    assert!(text.contains("y:: false"));
    assert!(text.contains("z:: null"));
    let back: Cfg = from_str(&text).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn array_of_bools_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        flags: Vec<bool>,
    }
    let cfg = Cfg {
        flags: vec![true, false, true, false],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn array_of_optionals_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<Option<String>>,
    }
    let cfg = Cfg {
        items: vec![Some("a".into()), None, Some("b".into()), None],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}
