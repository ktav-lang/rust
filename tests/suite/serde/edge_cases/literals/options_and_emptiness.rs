//! Option, empty Vec, empty struct, skip_serializing_if combinations.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn all_none_fields_serialize_as_nulls() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        a: Option<String>,
        b: Option<u16>,
        c: Option<bool>,
    }
    let cfg = Cfg {
        a: None,
        b: None,
        c: None,
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "a: null\nb: null\nc: null\n");
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn all_none_with_skip_serialize_if_produces_empty_document() {
    #[derive(Debug, Serialize)]
    struct Cfg {
        #[serde(skip_serializing_if = "Option::is_none")]
        a: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        b: Option<u16>,
    }
    let cfg = Cfg { a: None, b: None };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "");
}

#[test]
fn empty_vec_vs_none_vec_are_distinguishable() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        xs: Option<Vec<u16>>,
    }

    let empty = Cfg { xs: Some(vec![]) };
    let none = Cfg { xs: None };

    let s_empty = to_string(&empty).unwrap();
    let s_none = to_string(&none).unwrap();
    assert_ne!(s_empty, s_none);
    assert_eq!(s_empty, "xs: []\n");
    assert_eq!(s_none, "xs: null\n");

    let r_empty: Cfg = from_str(&s_empty).unwrap();
    let r_none: Cfg = from_str(&s_none).unwrap();
    assert_eq!(r_empty, empty);
    assert_eq!(r_none, none);
}

#[test]
fn missing_field_is_none_without_default() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        x: Option<String>,
    }
    let cfg: Cfg = from_str("").unwrap();
    assert!(cfg.x.is_none());
}

#[test]
fn missing_vec_needs_serde_default() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        #[serde(default)]
        xs: Vec<String>,
    }
    let cfg: Cfg = from_str("").unwrap();
    assert!(cfg.xs.is_empty());
}

#[test]
fn empty_struct_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Empty {}
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        meta: Empty,
    }

    let cfg = Cfg { meta: Empty {} };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "meta: {}\n");
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn nested_empty_arrays_and_objects() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        arrays: Vec<Vec<u16>>,
        objects: Vec<()>, // Vec of unit = Vec of nulls
    }
    let cfg = Cfg {
        arrays: vec![vec![], vec![], vec![1]],
        objects: vec![(), ()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn empty_string_in_array_round_trips() {
    // Regression: an empty-string item inside an array used to render as a
    // bare indented blank line, which the parser then treated as decorative
    // and dropped — the array came back one item short.
    // The fix forces `::` for empty-string items so the line stays a
    // recognisable literal-string entry.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<String>,
    }

    let cfg = Cfg {
        items: vec!["plain".into(), "".into(), "after".into()],
    };
    let s = to_string(&cfg).unwrap();
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(back, cfg, "rendered text was:\n{}", s);

    // All-empty array — roundtrips without dropping items.
    let cfg = Cfg {
        items: vec!["".into(), "".into(), "".into()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(back, cfg);
}

#[test]
fn option_of_enum_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        On,
        Off,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        mode: Option<Mode>,
    }

    for m in [None, Some(Mode::On), Some(Mode::Off)] {
        let cfg = Cfg { mode: m };
        let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, back);
    }
}
