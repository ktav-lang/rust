//! Round-trip for nested structures, arrays of objects, and enums.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn nested_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Timeouts {
        read: u32,
        write: u32,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        name: String,
        timeouts: Timeouts,
    }
    let cfg = Cfg {
        name: "demo".into(),
        timeouts: Timeouts {
            read: 30,
            write: 10,
        },
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn array_of_objects() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Upstream {
        host: String,
        port: u16,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        upstreams: Vec<Upstream>,
    }
    let cfg = Cfg {
        upstreams: vec![
            Upstream {
                host: "a.example".into(),
                port: 1080,
            },
            Upstream {
                host: "b.example".into(),
                port: 1080,
            },
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn full_config_with_mixed_fields() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Timeouts {
        read: u32,
        write: u32,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Upstream {
        host: String,
        port: u16,
        timeouts: Option<Timeouts>,
        #[serde(default)]
        tags: Vec<String>,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        port: u16,
        banned: Vec<String>,
        upstreams: Vec<Upstream>,
    }
    let cfg = Cfg {
        port: 20082,
        banned: vec![".*\\.onion:\\d+".into()],
        upstreams: vec![
            Upstream {
                host: "a.example".into(),
                port: 1080,
                timeouts: Some(Timeouts {
                    read: 30,
                    write: 10,
                }),
                tags: vec![],
            },
            Upstream {
                host: "b.example".into(),
                port: 1080,
                timeouts: None,
                tags: vec!["backup".into(), "eu".into()],
            },
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn unit_variant_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        Fast,
        Slow,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        mode: Mode,
    }
    for m in [Mode::Fast, Mode::Slow] {
        let cfg = Cfg { mode: m };
        let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, back);
    }
}

#[test]
fn newtype_variant_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Action {
        Log(String),
        Count(u32),
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        action: Action,
    }

    let cfg = Cfg {
        action: Action::Log("hi".into()),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);

    let cfg = Cfg {
        action: Action::Count(7),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn struct_variant_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Message {
        Greet { who: String },
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        msg: Message,
    }
    let cfg = Cfg {
        msg: Message::Greet {
            who: "world".into(),
        },
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn nested_array() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        outer: Vec<Vec<u16>>,
    }
    let cfg = Cfg {
        outer: vec![vec![1, 2, 3], vec![], vec![4]],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}
