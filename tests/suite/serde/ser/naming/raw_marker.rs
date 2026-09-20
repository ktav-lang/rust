//! The serializer emits `::` automatically when a string value starts with
//! `{` or `[`, or equals one of the JSON keywords.

use ktav::to_string;
use serde::Serialize;

#[test]
fn bracket_value_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        regex: String,
    }
    let s = to_string(&Cfg {
        regex: "[a-z]+".into(),
    })
    .unwrap();
    assert_eq!(s, "regex:: [a-z]+\n");
}

#[test]
fn brace_value_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        template: String,
    }
    let s = to_string(&Cfg {
        template: "{issue.id}.tpl".into(),
    })
    .unwrap();
    assert_eq!(s, "template:: {issue.id}.tpl\n");
}

#[test]
fn ipv6_literal_gets_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        addr: String,
    }
    let s = to_string(&Cfg {
        addr: "[::1]:8080".into(),
    })
    .unwrap();
    assert_eq!(s, "addr:: [::1]:8080\n");
}

#[test]
fn ordinary_string_does_not_get_double_colon() {
    #[derive(Serialize)]
    struct Cfg {
        host: String,
    }
    let s = to_string(&Cfg {
        host: "example.com".into(),
    })
    .unwrap();
    assert_eq!(s, "host: example.com\n");
}

#[test]
fn value_with_inner_bracket_does_not_need_marker() {
    // Only LEADING brackets trigger the marker.
    #[derive(Serialize)]
    struct Cfg {
        label: String,
    }
    let s = to_string(&Cfg {
        label: "hello[world]".into(),
    })
    .unwrap();
    assert_eq!(s, "label: hello[world]\n");
}

#[test]
fn bracket_item_in_array_gets_double_colon_prefix() {
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let s = to_string(&Cfg {
        items: vec!["ok".into(), "[::1]".into(), "{x}".into()],
    })
    .unwrap();
    assert_eq!(s, "items: [\n    ok\n    :: [::1]\n    :: {x}\n]\n");
}
