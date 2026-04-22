//! Array serialization.

use ktav::to_string;
use serde::Serialize;

#[test]
fn empty_array_inline() {
    #[derive(Serialize)]
    struct Cfg {
        tags: Vec<String>,
    }
    let s = to_string(&Cfg { tags: vec![] }).unwrap();
    assert_eq!(s, "tags: []\n");
}

#[test]
fn array_of_strings_multiline() {
    #[derive(Serialize)]
    struct Cfg {
        tags: Vec<String>,
    }
    let s = to_string(&Cfg {
        tags: vec!["primary".into(), "eu".into()],
    })
    .unwrap();
    assert_eq!(s, "tags: [\n    primary\n    eu\n]\n");
}

#[test]
fn array_of_integers_multiline() {
    #[derive(Serialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let s = to_string(&Cfg {
        ports: vec![80, 443],
    })
    .unwrap();
    assert_eq!(s, "ports: [\n    :i 80\n    :i 443\n]\n");
}

#[test]
fn array_of_objects_multiline() {
    #[derive(Serialize)]
    struct Item {
        name: String,
    }
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<Item>,
    }
    let s = to_string(&Cfg {
        items: vec![Item { name: "a".into() }, Item { name: "b".into() }],
    })
    .unwrap();
    assert_eq!(
        s,
        "items: [\n    {\n        name: a\n    }\n    {\n        name: b\n    }\n]\n"
    );
}

#[test]
fn nested_array_multiline() {
    #[derive(Serialize)]
    struct Cfg {
        outer: Vec<Vec<u16>>,
    }
    let s = to_string(&Cfg {
        outer: vec![vec![1, 2], vec![3]],
    })
    .unwrap();
    assert_eq!(
        s,
        "outer: [\n    [\n        :i 1\n        :i 2\n    ]\n    [\n        :i 3\n    ]\n]\n"
    );
}
