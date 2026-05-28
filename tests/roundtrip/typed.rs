//! Round-trip of typed scalars. Under spec 0.5.0, numbers are inferred
//! from lexical form (no `:i`/`:f` markers).

use ktav::{from_str, parse, render::render, to_string, Value};
use serde::{Deserialize, Serialize};

#[test]
fn struct_with_numeric_fields_roundtrips() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        port: u16,
        ratio: f64,
        name: String,
    }
    let cfg = Cfg {
        port: 8080,
        ratio: 0.5,
        name: "demo".into(),
    };
    let text = to_string(&cfg).unwrap();
    assert!(text.contains("port: 8080"), "got: {}", text);
    assert!(text.contains("ratio: 0.5"), "got: {}", text);
    assert!(text.contains("name: demo"), "got: {}", text);
    let back: Cfg = from_str(&text).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn value_level_roundtrip_integer() {
    // Under 0.5.0, `port: 8080` is inferred as Integer
    let text = "port: 8080\n";
    let v1 = parse(text).unwrap();
    let rendered = render(&v1).unwrap();
    let v2 = parse(&rendered).unwrap();
    assert_eq!(v1, v2);
    let port = v2.as_object().unwrap().get("port").unwrap();
    assert_eq!(port, &Value::Integer("8080".into()));
}

#[test]
fn value_level_roundtrip_float() {
    let text = "ratio: 0.5\n";
    let v1 = parse(text).unwrap();
    let rendered = render(&v1).unwrap();
    let v2 = parse(&rendered).unwrap();
    assert_eq!(v1, v2);
    let ratio = v2.as_object().unwrap().get("ratio").unwrap();
    assert_eq!(ratio, &Value::Float("0.5".into()));
}

#[test]
fn value_level_roundtrip_preserves_big_integer() {
    // Under 0.5.0, numbers that overflow i64 become String per § 5.2 rule 15.
    let text = "id: 99999999999999999999\n";
    let v1 = parse(text).unwrap();
    let rendered = render(&v1).unwrap();
    let v2 = parse(&rendered).unwrap();
    assert_eq!(v1, v2);
    let id = v2.as_object().unwrap().get("id").unwrap();
    // Under 0.5.0, i64 overflow → String (not Integer)
    assert_eq!(id, &Value::String("99999999999999999999".into()));
}

#[test]
fn array_of_integers_roundtrips_via_value() {
    // Under 0.5.0, integers are inferred from lexical form
    let text = "xs: [\n    1\n    -2\n    3\n]\n";
    let v1 = parse(text).unwrap();
    let rendered = render(&v1).unwrap();
    let v2 = parse(&rendered).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn mixed_struct_roundtrips_via_serde() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
        label: String,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        version: u32,
        points: Vec<Point>,
    }
    let cfg = Cfg {
        version: 2,
        points: vec![
            Point {
                x: 1.0,
                y: 2.5,
                label: "a".into(),
            },
            Point {
                x: -2.78,
                y: 0.0,
                label: "b".into(),
            },
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn integer_and_float_signs_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        neg_i: i64,
        neg_f: f64,
        pos_i: u64,
    }
    let cfg = Cfg {
        neg_i: -1_000_000,
        neg_f: -2.5,
        pos_i: 7_000,
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}
