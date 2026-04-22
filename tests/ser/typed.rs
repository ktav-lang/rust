//! Serialization of typed-marker scalars — `:i` for integer types and
//! `:f` for floats.

use ktav::{to_string, Error};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Integer types
// ---------------------------------------------------------------------------

#[test]
fn u8_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: u8,
    }
    assert_eq!(to_string(&Cfg { x: 200 }).unwrap(), "x:i 200\n");
}

#[test]
fn u16_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: u16,
    }
    assert_eq!(to_string(&Cfg { x: 8080 }).unwrap(), "x:i 8080\n");
}

#[test]
fn u32_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: u32,
    }
    assert_eq!(
        to_string(&Cfg { x: 4_000_000_000 }).unwrap(),
        "x:i 4000000000\n"
    );
}

#[test]
fn u64_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: u64,
    }
    assert_eq!(
        to_string(&Cfg { x: u64::MAX }).unwrap(),
        format!("x:i {}\n", u64::MAX)
    );
}

#[test]
fn u128_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: u128,
    }
    assert_eq!(
        to_string(&Cfg { x: u128::MAX }).unwrap(),
        format!("x:i {}\n", u128::MAX)
    );
}

#[test]
fn i8_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: i8,
    }
    assert_eq!(to_string(&Cfg { x: -128 }).unwrap(), "x:i -128\n");
}

#[test]
fn i16_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: i16,
    }
    assert_eq!(to_string(&Cfg { x: -32000 }).unwrap(), "x:i -32000\n");
}

#[test]
fn i32_emits_i_marker_for_negative() {
    #[derive(Serialize)]
    struct Cfg {
        x: i32,
    }
    assert_eq!(to_string(&Cfg { x: -42 }).unwrap(), "x:i -42\n");
}

#[test]
fn i64_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: i64,
    }
    assert_eq!(
        to_string(&Cfg { x: i64::MIN }).unwrap(),
        format!("x:i {}\n", i64::MIN)
    );
}

#[test]
fn i128_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: i128,
    }
    assert_eq!(
        to_string(&Cfg { x: i128::MIN }).unwrap(),
        format!("x:i {}\n", i128::MIN)
    );
}

#[test]
fn usize_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: usize,
    }
    assert_eq!(to_string(&Cfg { x: 42 }).unwrap(), "x:i 42\n");
}

#[test]
fn isize_emits_i_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: isize,
    }
    assert_eq!(to_string(&Cfg { x: -1 }).unwrap(), "x:i -1\n");
}

// ---------------------------------------------------------------------------
// Float types
// ---------------------------------------------------------------------------

#[test]
fn f64_emits_f_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: 0.5 }).unwrap(), "x:f 0.5\n");
}

#[test]
fn f32_emits_f_marker() {
    #[derive(Serialize)]
    struct Cfg {
        x: f32,
    }
    assert_eq!(to_string(&Cfg { x: 0.5 }).unwrap(), "x:f 0.5\n");
}

#[test]
fn f64_whole_number_gets_dot_zero_appended() {
    // f64::Display on 1.0 yields "1" — but `:f` literals require a decimal
    // point. The serializer appends `.0` in that case.
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: 1.0 }).unwrap(), "x:f 1.0\n");
}

#[test]
fn f32_whole_number_gets_dot_zero_appended() {
    #[derive(Serialize)]
    struct Cfg {
        x: f32,
    }
    assert_eq!(to_string(&Cfg { x: 1.0 }).unwrap(), "x:f 1.0\n");
}

#[test]
fn f64_negative() {
    #[derive(Serialize)]
    struct Cfg {
        x: f64,
    }
    assert_eq!(to_string(&Cfg { x: -2.78 }).unwrap(), "x:f -2.78\n");
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
fn array_of_integers_uses_i_marker_per_item() {
    #[derive(Serialize)]
    struct Cfg {
        xs: Vec<u16>,
    }
    let s = to_string(&Cfg {
        xs: vec![80, 443, 8080],
    })
    .unwrap();
    assert_eq!(s, "xs: [\n    :i 80\n    :i 443\n    :i 8080\n]\n");
}

#[test]
fn array_of_floats_uses_f_marker_per_item() {
    #[derive(Serialize)]
    struct Cfg {
        xs: Vec<f64>,
    }
    let s = to_string(&Cfg {
        xs: vec![0.5, 1.5, 2.0],
    })
    .unwrap();
    assert_eq!(s, "xs: [\n    :f 0.5\n    :f 1.5\n    :f 2.0\n]\n");
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
    // u128::MAX = 340282366920938463463374607431768211455
    let s = to_string(&Cfg { x: u128::MAX }).unwrap();
    assert!(
        s.contains("340282366920938463463374607431768211455"),
        "got: {}",
        s
    );
}

#[test]
fn bool_in_struct_stays_keyword_not_typed() {
    // `bool` must NOT pick up `:i` — keywords stay keywords.
    #[derive(Serialize)]
    struct Cfg {
        on: bool,
    }
    assert_eq!(to_string(&Cfg { on: true }).unwrap(), "on: true\n");
}
