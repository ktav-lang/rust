//! Round-trip of typed scalars. Under spec 0.5.0, numbers are inferred
//! from lexical form (no `:i`/`:f` markers).

use ktav::{de::from_value, from_str, parse, render::render, ser::to_value, to_string, Value};
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

// ---------------------------------------------------------------------------
// 128-bit integers: full serde roundtrip through both read paths.
// The writer already emits every digit; these tests pin the read-back that
// R14-F1 found missing (serde default rejects i128/u128 unconditionally).
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wide {
    a: i128,
    b: u128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Bag {
    xs: Vec<u128>,
    ys: Vec<i128>,
}

const I128_SAMPLES: [i128; 7] = [
    0,
    1,
    -1,
    i64::MAX as i128 + 1,
    i64::MIN as i128 - 1,
    i128::MAX - 1,
    i128::MAX,
];

const U128_SAMPLES: [u128; 7] = [
    0,
    1,
    42,
    u64::MAX as u128,
    u64::MAX as u128 + 1,
    u128::MAX - 1,
    u128::MAX,
];

#[test]
fn i128_u128_object_root_roundtrip_thin() {
    for (a, b) in I128_SAMPLES.iter().zip(U128_SAMPLES.iter()) {
        let v = Wide { a: *a, b: *b };
        let text = to_string(&v).unwrap();
        let back: Wide = from_str(&text).unwrap();
        assert_eq!(v, back, "thin object roundtrip failed for {}", text);
    }
    let edge = Wide {
        a: i128::MIN,
        b: u128::MAX,
    };
    let text = to_string(&edge).unwrap();
    assert_eq!(text, format!("a: {}\nb: {}\n", i128::MIN, u128::MAX));
    let back: Wide = from_str(&text).unwrap();
    assert_eq!(edge, back);
}

#[test]
fn i128_u128_object_root_roundtrip_owned() {
    for (a, b) in I128_SAMPLES.iter().zip(U128_SAMPLES.iter()) {
        let v = Wide { a: *a, b: *b };
        let back: Wide = from_value(to_value(&v).unwrap()).unwrap();
        assert_eq!(v, back, "owned object roundtrip failed for {:?}", v);
    }
    // Integer payloads straight from the parser (in-i64 band).
    let back: Wide = from_value(parse("a: -1\nb: 2\n").unwrap()).unwrap();
    assert_eq!(back, Wide { a: -1, b: 2 });
    let edge = Wide {
        a: i128::MIN,
        b: u128::MAX,
    };
    let back: Wide = from_value(to_value(&edge).unwrap()).unwrap();
    assert_eq!(edge, back);
}

#[test]
fn i128_u128_array_root_roundtrip_thin() {
    let ys: Vec<i128> = vec![i128::MIN, -1, 0, 1, i128::MAX];
    let text = to_string(&ys).unwrap();
    assert_eq!(from_str::<Vec<i128>>(&text).unwrap(), ys);
    let xs: Vec<u128> = vec![0, 1, u64::MAX as u128 + 1, u128::MAX];
    let text = to_string(&xs).unwrap();
    assert_eq!(from_str::<Vec<u128>>(&text).unwrap(), xs);
    let bag = Bag {
        xs: xs.clone(),
        ys: ys.clone(),
    };
    let text = to_string(&bag).unwrap();
    assert_eq!(from_str::<Bag>(&text).unwrap(), bag);
}

#[test]
fn i128_u128_array_root_roundtrip_owned() {
    let ys: Vec<i128> = vec![i128::MIN, -1, 0, 1, i128::MAX];
    let xs: Vec<u128> = vec![0, 1, u64::MAX as u128 + 1, u128::MAX];
    assert_eq!(from_value::<Vec<i128>>(to_value(&ys).unwrap()).unwrap(), ys);
    assert_eq!(from_value::<Vec<u128>>(to_value(&xs).unwrap()).unwrap(), xs);
    let bag = Bag { xs, ys };
    assert_eq!(from_value::<Bag>(to_value(&bag).unwrap()).unwrap(), bag);
}

#[test]
fn i128_u128_via_raw_marker() {
    // `::` forces the scalar to a String; the target type still parses it
    // via FromStr (documented in README "Numbers").
    let src = format!("a:: {}\nb:: {}\n", i128::MAX, u128::MAX);
    let back: Wide = from_str(&src).unwrap();
    assert_eq!(
        back,
        Wide {
            a: i128::MAX,
            b: u128::MAX
        }
    );

    #[derive(Debug, Deserialize, PartialEq)]
    struct OnlyA {
        a: i128,
    }
    let src = format!("a:: {}\n", i128::MIN);
    let back: OnlyA = from_str(&src).unwrap();
    assert_eq!(back.a, i128::MIN);

    #[derive(Debug, Deserialize, PartialEq)]
    struct OnlyXs {
        xs: Vec<u128>,
    }
    let src = format!("xs: [\n  :: {}\n  :: {}\n]\n", u128::MAX - 1, u128::MAX);
    let back: OnlyXs = from_str(&src).unwrap();
    assert_eq!(back.xs, vec![u128::MAX - 1, u128::MAX]);
}

#[test]
fn i128_out_of_domain_rejected() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct OnlyA {
        a: i128,
    }
    for bad in [
        // 2^127 — one past i128::MAX
        "170141183460469231731687303715884105728",
        // one below i128::MIN
        "-170141183460469231731687303715884105729",
        // non-numeric text
        "abc",
    ] {
        let src = format!("a: {bad}\n");
        let msg = format!("failed to parse '{bad}' as i128");
        let err = from_str::<OnlyA>(&src).unwrap_err();
        assert!(err.to_string().contains(&msg), "thin: got: {err}");
        let err = from_value::<OnlyA>(parse(&src).unwrap()).unwrap_err();
        assert!(err.to_string().contains(&msg), "owned: got: {err}");
    }
}

#[test]
fn u128_out_of_domain_rejected() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct OnlyB {
        b: u128,
    }
    for bad in [
        // negative — unsigned domain
        "-1",
        // 2^128 — one past u128::MAX
        "340282366920938463463374607431768211456",
    ] {
        let src = format!("b: {bad}\n");
        let msg = format!("failed to parse '{bad}' as u128");
        let err = from_str::<OnlyB>(&src).unwrap_err();
        assert!(err.to_string().contains(&msg), "thin: got: {err}");
        let err = from_value::<OnlyB>(parse(&src).unwrap()).unwrap_err();
        assert!(err.to_string().contains(&msg), "owned: got: {err}");
    }
}
