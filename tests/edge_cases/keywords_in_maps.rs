//! Keywords (`null` / `true` / `false`) appearing as map keys, map values,
//! and both simultaneously.

use std::collections::BTreeMap;

use ktav::{de::from_value, from_str, parse, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn keyword_strings_as_map_keys() {
    // Map keys are written as-is (identifier validation allows them).
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("true".into(), "yes".into());
    m.insert("false".into(), "no".into());
    m.insert("null".into(), "empty".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn keyword_strings_as_map_values_get_double_colon() {
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("a".into(), "true".into());
    m.insert("b".into(), "null".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("a:: true"));
    assert!(s.contains("b:: null"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn bool_values_in_map_emit_keyword() {
    let mut m: BTreeMap<String, bool> = BTreeMap::new();
    m.insert("on".into(), true);
    m.insert("off".into(), false);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, bool>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("on: true"));
    assert!(s.contains("off: false"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn integer_map_keys_serialize_as_strings_and_round_trip() {
    let mut m: BTreeMap<u32, String> = BTreeMap::new();
    m.insert(1, "one".into());
    m.insert(42, "answer".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<u32, String>,
    }
    let cfg = Cfg { m };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn capitalized_true_in_map_stays_plain_string() {
    // Only lowercase forms trigger `::`. `True` is just a string.
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    m.insert("x".into(), "True".into());

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        m: BTreeMap<String, String>,
    }
    let cfg = Cfg { m };
    let s = to_string(&cfg).unwrap();
    assert!(s.contains("x: True"));
    assert!(!s.contains("x:: True"));
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn typed_numeric_map_keys_read_identically_via_both_apis() {
    // R15-F1: u32 keys must read the same through the thin event path
    // and the owned Value path.
    let text = "1: one\n42: answer\n";
    let thin: BTreeMap<u32, String> = from_str(text).unwrap();
    let owned: BTreeMap<u32, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, owned);
    assert_eq!(thin.get(&42).map(String::as_str), Some("answer"));
}

#[test]
fn typed_i64_map_keys_with_signed_names() {
    let text = "-5: neg\n+7: pos\n0: zero\n";
    let expected: BTreeMap<i64, String> =
        BTreeMap::from([(-5, "neg".into()), (7, "pos".into()), (0, "zero".into())]);
    let thin: BTreeMap<i64, String> = from_str(text).unwrap();
    let owned: BTreeMap<i64, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn typed_u64_map_keys_at_boundaries() {
    let text = "0: zero\n18446744073709551615: max\n";
    let expected: BTreeMap<u64, String> =
        BTreeMap::from([(0, "zero".into()), (u64::MAX, "max".into())]);
    let thin: BTreeMap<u64, String> = from_str(text).unwrap();
    let owned: BTreeMap<u64, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn typed_i128_map_keys_at_boundaries() {
    let text = format!("{}: min\n-1: neg\n{}: max\n", i128::MIN, i128::MAX);
    let expected: BTreeMap<i128, String> = BTreeMap::from([
        (i128::MIN, "min".into()),
        (-1, "neg".into()),
        (i128::MAX, "max".into()),
    ]);
    let thin: BTreeMap<i128, String> = from_str(&text).unwrap();
    let owned: BTreeMap<i128, String> = from_value(parse(&text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn typed_u128_map_keys_at_boundaries() {
    let text = format!("1: one\n{}: max\n", u128::MAX);
    let expected: BTreeMap<u128, String> =
        BTreeMap::from([(1, "one".into()), (u128::MAX, "max".into())]);
    let thin: BTreeMap<u128, String> = from_str(&text).unwrap();
    let owned: BTreeMap<u128, String> = from_value(parse(&text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn typed_bool_map_keys() {
    let text = "true: yes\nfalse: no\n";
    let expected: BTreeMap<bool, String> =
        BTreeMap::from([(false, "no".into()), (true, "yes".into())]);
    let thin: BTreeMap<bool, String> = from_str(text).unwrap();
    let owned: BTreeMap<bool, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn typed_char_map_keys() {
    let text = "a: alpha\nz: zulu\n";
    let expected: BTreeMap<char, String> =
        BTreeMap::from([('a', "alpha".into()), ('z', "zulu".into())]);
    let thin: BTreeMap<char, String> = from_str(text).unwrap();
    let owned: BTreeMap<char, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);

    // Multi-char names stay rejected, identically on both APIs.
    let thin = from_str::<BTreeMap<char, String>>("ab: double\n")
        .unwrap_err()
        .to_string();
    let owned = from_value::<BTreeMap<char, String>>(parse("ab: double\n").unwrap())
        .unwrap_err()
        .to_string();
    assert_eq!(thin, owned);
}

#[test]
fn typed_unit_enum_map_keys() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    enum Tag {
        Alpha,
        Bravo,
    }
    let text = "Alpha: 1\nBravo: 2\n";
    let expected: BTreeMap<Tag, String> =
        BTreeMap::from([(Tag::Alpha, "1".into()), (Tag::Bravo, "2".into())]);
    let thin: BTreeMap<Tag, String> = from_str(text).unwrap();
    let owned: BTreeMap<Tag, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);

    // A bare key name cannot carry variant payload.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
    enum Tagged {
        N(u32),
    }
    let thin = from_str::<BTreeMap<Tagged, String>>("N: x\n")
        .unwrap_err()
        .to_string();
    let owned = from_value::<BTreeMap<Tagged, String>>(parse("N: x\n").unwrap())
        .unwrap_err()
        .to_string();
    assert_eq!(thin, owned);
    assert!(thin.contains("newtype variant"), "got: {thin}");
}

#[test]
fn typed_newtype_struct_map_keys() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    struct Id(u32);
    let text = "1: a\n42: b\n";
    let thin: BTreeMap<Id, String> = from_str(text).unwrap();
    let owned: BTreeMap<Id, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, owned);
    assert!(thin.contains_key(&Id(42)));
}

#[test]
fn numeric_looking_string_keys_stay_text() {
    // `deserialize_any` must NOT sniff numbers out of names: a String-keyed
    // map keeps every name verbatim.
    let text = "1000000000000000000000000000000: big\n0x1F: hex\n1_000: underscores\n";
    let expected: BTreeMap<String, String> = BTreeMap::from([
        ("1000000000000000000000000000000".into(), "big".into()),
        ("0x1F".into(), "hex".into()),
        ("1_000".into(), "underscores".into()),
    ]);
    let thin: BTreeMap<String, String> = from_str(text).unwrap();
    let owned: BTreeMap<String, String> = from_value(parse(text).unwrap()).unwrap();
    assert_eq!(thin, expected);
    assert_eq!(owned, expected);
}

#[test]
fn nested_map_with_typed_keys() {
    let mut m: BTreeMap<u32, BTreeMap<u8, String>> = BTreeMap::new();
    m.insert(1, BTreeMap::from([(2, "x".into()), (3, "y".into())]));
    m.insert(10, BTreeMap::from([(200, "ok".into())]));
    let text = to_string(&m).unwrap();
    let thin: BTreeMap<u32, BTreeMap<u8, String>> = from_str(&text).unwrap();
    let owned: BTreeMap<u32, BTreeMap<u8, String>> = from_value(parse(&text).unwrap()).unwrap();
    assert_eq!(thin, m);
    assert_eq!(owned, m);
}

macro_rules! assert_key_rejected_identically {
    ($text:expr, $ty:ty) => {{
        let thin = from_str::<BTreeMap<$ty, String>>($text)
            .unwrap_err()
            .to_string();
        let owned = from_value::<BTreeMap<$ty, String>>(parse($text).unwrap())
            .unwrap_err()
            .to_string();
        assert_eq!(
            thin,
            owned,
            "error mismatch for {:?} / {}",
            $text,
            stringify!($ty)
        );
    }};
}

#[test]
fn typed_map_key_rejections_identical_across_apis() {
    assert_key_rejected_identically!("300: x\n", u8); // out of range
    assert_key_rejected_identically!("4294967296: x\n", u32); // out of range
    assert_key_rejected_identically!("-1: x\n", u128); // negative, unsigned domain
    assert_key_rejected_identically!("340282366920938463463374607431768211456: x\n", u128); // one past MAX
    assert_key_rejected_identically!("-170141183460469231731687303715884105729: x\n", i128); // one below MIN
    assert_key_rejected_identically!("abc: x\n", u32); // non-numeric
    assert_key_rejected_identically!("1_000: x\n", u32); // underscores are not key grammar
    assert_key_rejected_identically!("0x1F: x\n", u32); // hex is not key grammar
}
