//! Scalar round-trips.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn integers() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        port: u16,
        id: i64,
    }
    let cfg = Cfg {
        port: 20082,
        id: -1,
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn strings() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        host: String,
        path: String,
    }
    let cfg = Cfg {
        host: "example.com".into(),
        path: "/api/v1".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn floats() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        ratio: f64,
    }
    let cfg = Cfg { ratio: 0.125 };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn bracket_literal_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        regex: String,
        ipv6: String,
    }
    let cfg = Cfg {
        regex: "[a-z]+".into(),
        ipv6: "[::1]:8080".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}
