//! Serialization of `null` / `true` / `false` — emitted as keywords.

use ktav::to_string;
use serde::Serialize;

#[test]
fn true_bool_emits_keyword() {
    #[derive(Serialize)]
    struct Cfg {
        on: bool,
    }
    let s = to_string(&Cfg { on: true }).unwrap();
    assert_eq!(s, "on: true\n");
}

#[test]
fn false_bool_emits_keyword() {
    #[derive(Serialize)]
    struct Cfg {
        on: bool,
    }
    let s = to_string(&Cfg { on: false }).unwrap();
    assert_eq!(s, "on: false\n");
}

#[test]
fn option_none_emits_null_keyword() {
    #[derive(Serialize)]
    struct Cfg {
        label: Option<String>,
    }
    let s = to_string(&Cfg { label: None }).unwrap();
    assert_eq!(s, "label: null\n");
}

#[test]
fn option_none_can_be_skipped_via_serde_attr() {
    #[derive(Serialize)]
    struct Cfg {
        port: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    }
    let s = to_string(&Cfg {
        port: 8080,
        label: None,
    })
    .unwrap();
    assert_eq!(s, "port:i 8080\n");
}

#[test]
fn string_equal_to_true_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        x: String,
    }
    let s = to_string(&Cfg { x: "true".into() }).unwrap();
    assert_eq!(s, "x:: true\n");
}

#[test]
fn string_equal_to_false_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        x: String,
    }
    let s = to_string(&Cfg { x: "false".into() }).unwrap();
    assert_eq!(s, "x:: false\n");
}

#[test]
fn string_equal_to_null_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        x: String,
    }
    let s = to_string(&Cfg { x: "null".into() }).unwrap();
    assert_eq!(s, "x:: null\n");
}

#[test]
fn string_capital_true_stays_plain() {
    // "True" is not a keyword, so no `::` needed.
    #[derive(Serialize)]
    struct Cfg {
        x: String,
    }
    let s = to_string(&Cfg { x: "True".into() }).unwrap();
    assert_eq!(s, "x: True\n");
}

#[test]
fn bool_in_array_emits_keyword() {
    #[derive(Serialize)]
    struct Cfg {
        flags: Vec<bool>,
    }
    let s = to_string(&Cfg {
        flags: vec![true, false, true],
    })
    .unwrap();
    assert_eq!(s, "flags: [\n    true\n    false\n    true\n]\n");
}

#[test]
fn null_in_array_emits_keyword() {
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<Option<String>>,
    }
    let s = to_string(&Cfg {
        items: vec![Some("a".into()), None, Some("b".into())],
    })
    .unwrap();
    assert_eq!(s, "items: [\n    a\n    null\n    b\n]\n");
}

#[test]
fn string_equal_to_keyword_in_array_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let s = to_string(&Cfg {
        items: vec!["true".into(), "null".into(), "ok".into()],
    })
    .unwrap();
    assert_eq!(s, "items: [\n    :: true\n    :: null\n    ok\n]\n");
}
