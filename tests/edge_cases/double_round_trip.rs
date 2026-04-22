//! Stability under repeated round-trips: text → T → text → T should
//! converge after one cycle (canonical form).

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

fn converges<T>(cfg: T) -> bool
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let s1 = to_string(&cfg).unwrap();
    let c1: T = from_str(&s1).unwrap();
    let s2 = to_string(&c1).unwrap();
    let c2: T = from_str(&s2).unwrap();
    s1 == s2 && c1 == c2
}

#[test]
fn simple_struct_converges() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        port: u16,
        name: String,
    }
    assert!(converges(Cfg {
        port: 8080,
        name: "demo".into()
    }));
}

#[test]
fn nested_arrays_of_structs_converge() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Item {
        id: u32,
        name: String,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<Item>,
    }
    assert!(converges(Cfg {
        items: vec![
            Item {
                id: 1,
                name: "a".into()
            },
            Item {
                id: 2,
                name: "b".into()
            },
        ],
    }));
}

#[test]
fn enum_variants_converge() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        On,
        Off,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Action {
        Log(String),
        Set { key: String, val: u32 },
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        mode: Mode,
        actions: Vec<Action>,
    }
    assert!(converges(Cfg {
        mode: Mode::On,
        actions: vec![
            Action::Log("hi".into()),
            Action::Set {
                key: "k".into(),
                val: 42
            },
        ],
    }));
}

#[test]
fn multiline_strings_converge() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        short: String,
        long: String,
    }
    assert!(converges(Cfg {
        short: "single-line".into(),
        long: "line 1\nline 2\nline 3\n".into(),
    }));
}

#[test]
fn keyword_like_strings_converge() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        a: String,
        b: String,
        c: String,
        d: String,
    }
    assert!(converges(Cfg {
        a: "true".into(), // literal string "true"
        b: "null".into(),
        c: "[foo]".into(),
        d: "(".into(),
    }));
}

#[test]
fn options_and_nulls_converge() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        some: Option<String>,
        none: Option<String>,
        maybe_vec: Option<Vec<u16>>,
    }
    assert!(converges(Cfg {
        some: Some("hello".into()),
        none: None,
        maybe_vec: Some(vec![1, 2, 3]),
    }));
    assert!(converges(Cfg {
        some: None,
        none: None,
        maybe_vec: None,
    }));
}
