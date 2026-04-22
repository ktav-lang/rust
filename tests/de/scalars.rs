//! Scalar deserialization.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn i32_scalar() {
    #[derive(Deserialize)]
    struct Cfg {
        x: i32,
    }
    let cfg: Cfg = from_str("x: -42").unwrap();
    assert_eq!(cfg.x, -42);
}

#[test]
fn u16_scalar() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
    }
    let cfg: Cfg = from_str("port: 8080").unwrap();
    assert_eq!(cfg.port, 8080);
}

#[test]
fn string_scalar() {
    #[derive(Deserialize)]
    struct Cfg {
        name: String,
    }
    let cfg: Cfg = from_str("name: Russia").unwrap();
    assert_eq!(cfg.name, "Russia");
}

#[test]
fn string_value_is_trimmed() {
    #[derive(Deserialize)]
    struct Cfg {
        name: String,
    }
    let cfg: Cfg = from_str("name:    spaced    ").unwrap();
    assert_eq!(cfg.name, "spaced");
}

#[test]
fn string_value_may_contain_colon_after_the_first() {
    #[derive(Deserialize)]
    struct Cfg {
        pattern: String,
    }
    let cfg: Cfg = from_str("pattern: host:8080").unwrap();
    assert_eq!(cfg.pattern, "host:8080");
}

#[test]
fn float_scalar() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio: 2.56").unwrap();
    assert!((cfg.ratio - 2.56).abs() < 1e-9);
}

#[test]
fn dotted_key_creates_nested_object() {
    #[derive(Deserialize)]
    struct Inner {
        a: u16,
        b: u16,
    }
    #[derive(Deserialize)]
    struct Cfg {
        server: Inner,
    }
    let cfg: Cfg = from_str("server.a: 1\nserver.b: 2\n").unwrap();
    assert_eq!(cfg.server.a, 1);
    assert_eq!(cfg.server.b, 2);
}
