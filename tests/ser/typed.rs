//! Serialization of typed scalars — under spec 0.5.0 integers use plain
//! `: ` (no `:i` marker) and floats use plain `: ` (no `:f` marker).
//! Number literals are inferred from their lexical form by the parser.

use ktav::{to_string, Error};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Integer types
// ---------------------------------------------------------------------------

#[test]
fn u8_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: u8,
    }
    assert_eq!(to_string(&Cfg { x: 200 }).unwrap(), "x: 200\n");
}

#[test]
fn u16_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: u16,
    }
    assert_eq!(to_string(&Cfg { x: 8080 }).unwrap(), "x: 8080\n");
}

#[test]
fn u32_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: u32,
    }
    assert_eq!(
        to_string(&Cfg { x: 4_000_000_000 }).unwrap(),
        "x: 4000000000\n"
    );
}

#[test]
fn u64_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: u64,
    }
    assert_eq!(
        to_string(&Cfg { x: u64::MAX }).unwrap(),
        format!("x: {}\n", u64::MAX)
    );
}

#[test]
fn u128_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: u128,
    }
    assert_eq!(
        to_string(&Cfg { x: u128::MAX }).unwrap(),
        format!("x: {}\n", u128::MAX)
    );
}

#[test]
fn i8_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: i8,
    }
    assert_eq!(to_string(&Cfg { x: -128 }).unwrap(), "x: -128\n");
}

#[test]
fn i16_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: i16,
    }
    assert_eq!(to_string(&Cfg { x: -32000 }).unwrap(), "x: -32000\n");
}

#[test]
fn i32_emits_plain_separator_for_negative() {
    #[derive(Serialize)]
    struct Cfg {
        x: i32,
    }
    assert_eq!(to_string(&Cfg { x: -42 }).unwrap(), "x: -42\n");
}

#[test]
fn i64_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: i64,
    }
    assert_eq!(
        to_string(&Cfg { x: i64::MIN }).unwrap(),
        format!("x: {}\n", i64::MIN)
    );
}

#[test]
fn i128_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: i128,
    }
    assert_eq!(
        to_string(&Cfg { x: i128::MIN }).unwrap(),
        format!("x: {}\n", i128::MIN)
    );
}

#[test]
fn usize_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: usize,
    }
    assert_eq!(to_string(&Cfg { x: 42 }).unwrap(), "x: 42\n");
}

#[test]
fn isize_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: isize,
    }
    assert_eq!(to_string(&Cfg { x: -1 }).unwrap(), "x: -1\n");
}

// ---------------------------------------------------------------------------
// Float types
// ---------------------------------------------------------------------------

#[test]
fn f64_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: 0.5 }).unwrap(), "x: 0.5\n");
}

#[test]
fn f32_emits_plain_separator() {
    #[derive(Serialize)]
    struct Cfg {
        x: f32,
    }
    assert_eq!(to_string(&Cfg { x: 0.5 }).unwrap(), "x: 0.5\n");
}

#[test]
fn f64_whole_number_gets_dot_zero_appended() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: 1.0 }).unwrap(), "x: 1.0\n");
}

#[test]
fn f32_whole_number_gets_dot_zero_appended() {
    #[derive(Serialize)]
    struct Cfg {
        x: f32,
    }
    assert_eq!(to_string(&Cfg { x: 1.0 }).unwrap(), "x: 1.0\n");
}

#[test]
fn f64_negative() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: -2.78 }).unwrap(), "x: -2.78\n");
}

#[test]
fn f64_nan_errors() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    let err = to_string(&Cfg { x: f64::NAN }).unwrap_err();
    assert!(
        matches!(err, Error::Message(ref m) if m.contains("NaN")),
        "got: {:?}",
        err
    );
}

#[test]
fn f64_infinity_errors() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    let err = to_string(&Cfg { x: f64::INFINITY }).unwrap_err();
    assert!(
        matches!(err, Error::Message(ref m) if m.contains("Infinity")),
        "got: {:?}",
        err
    );
}

#[test]
fn f64_neg_infinity_errors() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    let err = to_string(&Cfg {
        x: f64::NEG_INFINITY,
    })
    .unwrap_err();
    assert!(
        matches!(err, Error::Message(ref m) if m.contains("Infinity")),
        "got: {:?}",
        err
    );
}

#[test]
fn f32_nan_errors() {
    #[derive(Serialize)]
    struct Cfg {
        x: f32,
    }
    let err = to_string(&Cfg { x: f32::NAN }).unwrap_err();
    assert!(
        matches!(err, Error::Message(ref m) if m.contains("NaN")),
        "got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arrays of typed values
// ---------------------------------------------------------------------------

#[test]
fn array_of_integers_uses_plain_per_item() {
    #[derive(Serialize)]
    struct Cfg {
        xs: Vec<u16>,
    }
    let s = to_string(&Cfg {
        xs: vec![80, 443, 8080],
    })
    .unwrap();
    assert_eq!(s, "xs: [\n    80\n    443\n    8080\n]\n");
}

#[test]
fn array_of_floats_uses_plain_per_item() {
    #[derive(Serialize)]
    struct Cfg {
        xs: Vec<f64>,
    }
    let s = to_string(&Cfg {
        xs: vec![0.5, 1.5, 2.0],
    })
    .unwrap();
    assert_eq!(s, "xs: [\n    0.5\n    1.5\n    2.0\n]\n");
}

// ---------------------------------------------------------------------------
// Large integers retain every digit
// ---------------------------------------------------------------------------

#[test]
fn u128_max_preserves_all_digits() {
    #[derive(Serialize)]
    struct Cfg {
        x: u128,
    }
    let s = to_string(&Cfg { x: u128::MAX }).unwrap();
    assert!(
        s.contains("340282366920938463463374607431768211455"),
        "got: {}",
        s
    );
}

#[test]
fn bool_in_struct_stays_keyword_not_typed() {
    #[derive(Serialize)]
    struct Cfg {
        on: bool,
    }
    assert_eq!(to_string(&Cfg { on: true }).unwrap(), "on: true\n");
}
