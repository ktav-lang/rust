//! Scalar serialization.

use ktav::to_string;
use serde::Serialize;

#[test]
fn integer_scalar() {
    #[derive(Serialize)]
    struct Cfg {
        port: u16,
    }
    let s = to_string(&Cfg { port: 8080 }).unwrap();
    assert_eq!(s, "port:i 8080\n");
}

#[test]
fn string_scalar() {
    #[derive(Serialize)]
    struct Cfg {
        name: String,
    }
    let s = to_string(&Cfg {
        name: "demo".into(),
    })
    .unwrap();
    assert_eq!(s, "name: demo\n");
}

#[test]
fn float_scalar() {
    #[derive(Serialize)]
    struct Cfg {
        ratio: f64,
    }
    let s = to_string(&Cfg { ratio: 0.5 }).unwrap();
    assert_eq!(s, "ratio:f 0.5\n");
}

#[test]
fn two_fields_keep_order() {
    #[derive(Serialize)]
    struct Cfg {
        port: u16,
        name: String,
    }
    let s = to_string(&Cfg {
        port: 8080,
        name: "demo".into(),
    })
    .unwrap();
    assert_eq!(s, "port:i 8080\nname: demo\n");
}

#[test]
fn negative_integer() {
    #[derive(Serialize)]
    struct Cfg {
        x: i32,
    }
    let s = to_string(&Cfg { x: -42 }).unwrap();
    assert_eq!(s, "x:i -42\n");
}
