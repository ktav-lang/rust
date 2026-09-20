//! Deserialization of typed scalars. Under spec 0.5.0, numbers are
//! inferred from lexical form (no `:i`/`:f` markers).

use ktav::from_str;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Numbers inferred from the document
// ---------------------------------------------------------------------------

#[test]
fn inferred_integer_into_u16() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
    }
    let cfg: Cfg = from_str("port: 8080\n").unwrap();
    assert_eq!(cfg.port, 8080);
}

#[test]
fn inferred_negative_integer_into_i32() {
    #[derive(Deserialize)]
    struct Cfg {
        count: i32,
    }
    let cfg: Cfg = from_str("count: -100\n").unwrap();
    assert_eq!(cfg.count, -100);
}

#[test]
fn integer_overflow_preserved_as_string() {
    // i64 overflow — falls to String under § 5.2 rule 15.
    #[derive(Deserialize)]
    struct Cfg {
        bignum: String,
    }
    let cfg: Cfg = from_str("bignum: 99999999999999999999\n").unwrap();
    assert_eq!(cfg.bignum, "99999999999999999999");
}

#[test]
fn inferred_float_into_f64() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio: 0.5\n").unwrap();
    assert!((cfg.ratio - 0.5).abs() < 1e-12);
}

#[test]
fn inferred_float_scientific_into_f64() {
    #[derive(Deserialize)]
    struct Cfg {
        ratio: f64,
    }
    let cfg: Cfg = from_str("ratio: 1.5e-10\n").unwrap();
    assert!((cfg.ratio - 1.5e-10).abs() < 1e-20);
}

#[test]
fn inferred_float_into_f32() {
    #[derive(Deserialize)]
    struct Cfg {
        x: f32,
    }
    let cfg: Cfg = from_str("x: 3.125\n").unwrap();
    assert_eq!(cfg.x, 3.125_f32);
}

#[test]
fn inferred_integer_into_string_keeps_canonical() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x: 42\n").unwrap();
    assert_eq!(cfg.x, "42");
}

#[test]
fn inferred_float_into_string_keeps_canonical() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    // Under 0.5.0, `3.14` parses as Float with canonical form via ryu.
    let cfg: Cfg = from_str("x: 3.14\n").unwrap();
    // ryu may canonicalize 3.14 to "3.14" (it does for this value)
    assert_eq!(cfg.x, "3.14");
}

// ---------------------------------------------------------------------------
// Backward compat: plain numeric pairs
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
    let cfg: Cfg = from_str("ratio: 2.56\n").unwrap();
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
// Inferred numbers in arrays
// ---------------------------------------------------------------------------

#[test]
fn inferred_integers_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let cfg: Cfg = from_str("ports: [\n    80\n    443\n]\n").unwrap();
    assert_eq!(cfg.ports, vec![80, 443]);
}

#[test]
fn inferred_floats_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ratios: Vec<f64>,
    }
    let cfg: Cfg = from_str("ratios: [\n    0.5\n    1.5\n]\n").unwrap();
    assert_eq!(cfg.ratios, vec![0.5, 1.5]);
}

#[test]
fn mixed_integers_in_array() {
    #[derive(Deserialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let cfg: Cfg = from_str("ports: [\n    80\n    443\n]\n").unwrap();
    assert_eq!(cfg.ports, vec![80, 443]);
}
