//! Deep and mixed nesting — struct-in-array-in-struct-in-array, etc.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn four_level_nesting_round_trips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct L4 {
        leaf: u16,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct L3 {
        items: Vec<L4>,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct L2 {
        inner: L3,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct L1 {
        groups: Vec<L2>,
    }

    let cfg = L1 {
        groups: vec![
            L2 {
                inner: L3 {
                    items: vec![L4 { leaf: 1 }, L4 { leaf: 2 }],
                },
            },
            L2 {
                inner: L3 {
                    items: vec![L4 { leaf: 3 }],
                },
            },
        ],
    };
    let back: L1 = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn vec_of_vec_of_option() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        data: Vec<Vec<Option<u16>>>,
    }
    let cfg = Cfg {
        data: vec![vec![Some(1), None, Some(2)], vec![], vec![None]],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn enum_inside_array_inside_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        Fast,
        Slow,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        modes: Vec<Mode>,
    }
    let cfg = Cfg {
        modes: vec![Mode::Fast, Mode::Slow, Mode::Fast],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn newtype_variant_inside_array() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Event {
        Login(String),
        Count(u32),
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        events: Vec<Event>,
    }
    let cfg = Cfg {
        events: vec![
            Event::Login("alice".into()),
            Event::Count(42),
            Event::Login("bob".into()),
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn struct_variant_inside_array_inside_option() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Action {
        Move { x: i32, y: i32 },
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        actions: Option<Vec<Action>>,
    }

    let cfg = Cfg {
        actions: Some(vec![
            Action::Move { x: 1, y: 2 },
            Action::Move { x: -3, y: 4 },
        ]),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);

    let cfg_none = Cfg { actions: None };
    let back: Cfg = from_str(&to_string(&cfg_none).unwrap()).unwrap();
    assert_eq!(cfg_none, back);
}

#[test]
fn dotted_keys_at_same_level_merge_into_one_object() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Server {
        host: String,
        port: u16,
        timeout: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        server: Server,
    }

    // Multiple dotted pairs share the `server` parent — they must
    // coalesce, not conflict.
    let src = "\
server.host: 127.0.0.1
server.port: 8080
server.timeout: 30
";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.server.port, 8080);
    assert_eq!(cfg.server.timeout, 30);
}

#[test]
fn dotted_key_conflicts_with_scalar_at_shared_prefix() {
    // `a: 1` sets `a` as scalar; `a.b: 2` then tries to descend — must
    // produce a Syntax error, not silent corruption.
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _a: String,
    }
    let err = from_str::<Cfg>("a: 1\na.b: 2\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("conflict"), "got: {}", msg);
}
