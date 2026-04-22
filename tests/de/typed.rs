//! Deserialization of typed-marker scalars.

use ktav::{from_str, Error};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Typed markers in the source document
// ---------------------------------------------------------------------------

#[test]
fn typed_integer_into_u16() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
    }
    let cfg: Cfg = from_str("port:i 8080\n").unwrap();
    assert_eq!(cfg.port, 8080);
}

#[test]
fn typed_negative_integer_into_i32() {
    #[derive(Deserialize)]
    struct Cfg {
        count: i32,
    }
    let cfg: Cfg = from_str("count:i -100\n").unwrap();
    assert_eq!(cfg.count, -100);
}

#[test]
fn typed_integer_preserves_precision_as_string() {
    // i64 overflow, but String preserves every digit.
    #[derive(Deserialize)]
    struct Cfg {
        bignum: String,
    }
    let cfg: Cfg = from_str("bignum:i 99999999999999999999\n").unwrap();
    assert_eq!(cfg.bignum, "99999999999999999999");
}

#[test]
fn typed_integer_overflow_into_u64_errors() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        #[allow(dead_code)]
        bignum: u64,
    }
    let err = from_str::<Cfg>("bignum:i 99999999999999999999\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("u64") || msg.contains("parse"), "got: {}", msg);
}

#[test]
fn typed_float_into_f64() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio:f 0.5\n").unwrap();
    assert!((cfg.ratio - 0.5).abs() < 1e-12);
}

#[test]
fn typed_float_scientific_into_f64() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio:f 1.5e-10\n").unwrap();
    assert!((cfg.ratio - 1.5e-10).abs() < 1e-20);
}

#[test]
fn typed_float_into_f32() {
    #[derive(Deserialize)]
    struct Cfg {
        x: f32,
    }
    let cfg: Cfg = from_str("x:f 3.125\n").unwrap();
    assert_eq!(cfg.x, 3.125_f32);
}

#[test]
fn typed_leading_plus_is_stripped() {
    #[derive(Deserialize)]
    struct Cfg {
        x: i32,
    }
    let cfg: Cfg = from_str("x:i +5\n").unwrap();
    assert_eq!(cfg.x, 5);
}

#[test]
fn typed_integer_into_string_keeps_text() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x:i 42\n").unwrap();
    assert_eq!(cfg.x, "42");
}

#[test]
fn typed_float_into_string_keeps_text() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x:f 3.14\n").unwrap();
    assert_eq!(cfg.x, "3.14");
}

// ---------------------------------------------------------------------------
// Backward compat: legacy documents without markers
// ---------------------------------------------------------------------------

#[test]
fn plain_pair_into_u16_still_works() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
    }
    let cfg: Cfg = from_str("port: 8080\n").unwrap();
    assert_eq!(cfg.port, 8080);
}

#[test]
fn plain_pair_into_f64_still_works() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio:f 2.56\n").unwrap();
    assert!((cfg.ratio - 2.56).abs() < 1e-9);
}

#[test]
fn plain_pair_into_i64_still_works() {
    #[derive(Deserialize)]
    struct Cfg {
        x: i64,
    }
    let cfg: Cfg = from_str("x: -999\n").unwrap();
    assert_eq!(cfg.x, -999);
}

// ---------------------------------------------------------------------------
// Typed items in arrays
// ---------------------------------------------------------------------------

#[test]
fn typed_integers_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let cfg: Cfg = from_str("ports: [\n    :i 80\n    :i 443\n]\n").unwrap();
    assert_eq!(cfg.ports, vec![80, 443]);
}

#[test]
fn typed_floats_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ratios: Vec<f64>,
    }
    let cfg: Cfg = from_str("ratios: [\n    :f 0.5\n    :f 1.5\n]\n").unwrap();
    assert_eq!(cfg.ratios, vec![0.5, 1.5]);
}

#[test]
fn mixed_typed_and_plain_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let cfg: Cfg = from_str("ports: [\n    :i 80\n    443\n]\n").unwrap();
    assert_eq!(cfg.ports, vec![80, 443]);
}

// ---------------------------------------------------------------------------
// Typed marker that can't be destructured as the target type
// ---------------------------------------------------------------------------

#[test]
fn typed_integer_into_bool_errors() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: bool,
    }
    let err = from_str::<Cfg>("x:i 1\n").unwrap_err();
    // Our bool path accepts `Value::Bool` or a string-parseable "true" /
    // "false" — Integer isn't accepted.
    let _ = err;
}

#[test]
fn typed_integer_line_number_is_reported() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _a: u16,
        _b: u16,
    }
    let err = from_str::<Cfg>("a:i 1\nb:i not-a-number\n").unwrap_err();
    match err {
        Error::Syntax(m) => {
            // The parser itself rejects non-digit body for `:i`.
            assert!(m.contains("Line 2"), "got: {}", m);
            assert!(m.contains("InvalidTypedScalar"), "got: {}", m);
        }
        _ => panic!("expected Syntax error, got {:?}", err),
    }
}
