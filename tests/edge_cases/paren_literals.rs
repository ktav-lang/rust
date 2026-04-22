//! Regression tests for the `needs_raw_marker` miss: strings equal to
//! `(`, `((`, `()`, `(())` must survive a round-trip.
//!
//! Before the fix:
//! - `"("` was serialized as `x: (\n`, which the parser reads as
//!   "open multi-line" → garbage.
//! - `"()"`/`"(())"` were serialized as `x: ()\n` / `x: (())\n`, which
//!   the parser reads as empty-string form → content `""` on read-back.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cfg {
    x: String,
}

#[test]
fn single_open_paren_emits_marker() {
    let cfg = Cfg { x: "(".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x:: (\n");
}

#[test]
fn double_open_paren_emits_marker() {
    let cfg = Cfg { x: "((".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x:: ((\n");
}

#[test]
fn empty_parens_emits_marker() {
    let cfg = Cfg { x: "()".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x:: ()\n");
}

#[test]
fn empty_double_parens_emits_marker() {
    let cfg = Cfg { x: "(())".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x:: (())\n");
}

#[test]
fn single_open_paren_round_trips() {
    let cfg = Cfg { x: "(".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn double_open_paren_round_trips() {
    let cfg = Cfg { x: "((".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn empty_parens_round_trip() {
    let cfg = Cfg { x: "()".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn empty_double_parens_round_trip() {
    let cfg = Cfg { x: "(())".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn parens_as_array_items_round_trip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ArrCfg {
        items: Vec<String>,
    }
    let cfg = ArrCfg {
        items: vec![
            "(".into(),
            "((".into(),
            "()".into(),
            "(())".into(),
            "(abc)".into(), // no marker needed — not a special token
        ],
    };
    let s = to_string(&cfg).unwrap();
    // Items with special tokens use `::`, but "(abc)" does not.
    assert!(s.contains(":: (\n"));
    assert!(s.contains(":: ((\n"));
    assert!(s.contains(":: ()\n"));
    assert!(s.contains(":: (())\n"));
    assert!(s.contains("    (abc)\n"));
    let back: ArrCfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn partial_parens_do_not_need_marker() {
    // Regression guard: values that only *contain* parens or are
    // unambiguous to the parser must NOT receive a `::` marker.
    let cfg = Cfg {
        x: "(hello)".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x: (hello)\n");
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}
