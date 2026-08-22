//! Render losslessness: every writer surface must either produce a
//! document that parses back to the exact same data, or return an
//! error — never silently emit a document that parses to something
//! else (spec § 8.2 / § 8.3; the CR rule of § 5.9.7 sets the
//! precedent for the error path).
//!
//! Covered surfaces:
//! - `emit_canonical` (spec § 5.9 canonical writer)
//! - `to_string` (serde text serializer)
//! - `to_string_force_strings` / `render` (plain `Value` renderer)

use std::collections::BTreeMap;

use ktav::{emit_canonical, from_str, parse, to_string, to_string_force_strings, ObjectMap, Value};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// `emit_canonical` → `parse` must reproduce the exact `Value`.
fn assert_canonical_roundtrip(v: &Value) {
    let text = emit_canonical(v).unwrap_or_else(|e| panic!("emit_canonical({v:?}): {e}"));
    let back = parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(&back, v, "canonical text was {text:?}");
}

/// `render` (via `to_string_force_strings`, all leaves already Strings)
/// → `parse` must reproduce the exact `Value`.
fn assert_force_strings_roundtrip(v: &Value) {
    let text =
        to_string_force_strings(v).unwrap_or_else(|e| panic!("to_string_force_strings: {e}"));
    let back = parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(&back, v, "rendered text was {text:?}");
}

// ---------------------------------------------------------------------------
// Canonical writer — padded strings (§ 5.9.7 mandates verbatim form)
// ---------------------------------------------------------------------------

#[test]
fn canonical_keeps_leading_whitespace_in_string() {
    assert_canonical_roundtrip(&obj(&[("k", s("  padded"))]));
}

#[test]
fn canonical_keeps_trailing_whitespace_in_string() {
    assert_canonical_roundtrip(&obj(&[("password", s("hunter2 "))]));
}

#[test]
fn canonical_keeps_edge_whitespace_on_both_sides() {
    assert_canonical_roundtrip(&obj(&[("k", s("  padded  "))]));
}

#[test]
fn canonical_keeps_whitespace_only_string() {
    assert_canonical_roundtrip(&obj(&[("k", s("   "))]));
}

#[test]
fn canonical_keeps_tab_padded_string() {
    assert_canonical_roundtrip(&obj(&[("k", s("\tpadded\t"))]));
}

#[test]
fn canonical_keeps_padded_array_items() {
    assert_canonical_roundtrip(&obj(&[(
        "arr",
        Value::Array(vec![s("  pad"), s("pad  "), s(", ")]),
    )]));
}

#[test]
fn canonical_keeps_control_byte_string() {
    assert_canonical_roundtrip(&obj(&[("k", s("a\x01b"))]));
}

// ---------------------------------------------------------------------------
// Canonical writer — bodies the parser would re-classify (§ 5.9.5)
// ---------------------------------------------------------------------------

#[test]
fn canonical_paren_prefixed_string_reparses() {
    assert_canonical_roundtrip(&obj(&[("mode", s("(disabled)"))]));
    assert_canonical_roundtrip(&obj(&[("k", s("(x"))]));
}

#[test]
fn canonical_paren_prefixed_array_item_reparses() {
    assert_canonical_roundtrip(&obj(&[("arr", Value::Array(vec![s("(x"), s("(none)")]))]));
}

// ---------------------------------------------------------------------------
// Canonical writer — unrepresentable values must error, not corrupt
// ---------------------------------------------------------------------------

#[test]
fn canonical_rejects_padded_sole_double_paren_line() {
    // "  ))  " trims to `))`, so verbatim form is impossible, and
    // stripped form cannot keep the padding — like CR, the value has
    // no canonical representation.
    assert!(emit_canonical(&obj(&[("k", s("  ))  "))])).is_err());
}

// ---------------------------------------------------------------------------
// Canonical writer — tricky keys that ARE representable keep working
// ---------------------------------------------------------------------------

#[test]
fn canonical_keeps_representable_tricky_keys() {
    for key in ["a.b", "a:b", "#note", "x##y", "first name", "a\tb", "путь"] {
        assert_canonical_roundtrip(&obj(&[(key, s("x"))]));
    }
}

// ---------------------------------------------------------------------------
// Serde text serializer (`to_string`) — same guarantees
// ---------------------------------------------------------------------------

#[test]
fn serde_keeps_trailing_whitespace_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        password: String,
    }
    let cfg = Cfg {
        password: "hunter2 ".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn serde_keeps_leading_whitespace_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        sep: String,
    }
    let cfg = Cfg { sep: "  x".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn serde_keeps_whitespace_only_string() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        pad: String,
    }
    let cfg = Cfg { pad: "   ".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn serde_keeps_padded_strings_in_seq() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        parts: Vec<String>,
    }
    let cfg = Cfg {
        parts: vec!["  pad".into(), "pad  ".into(), ", ".into()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn serde_keeps_padded_string_in_nested_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Inner {
        v: String,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<Inner>,
    }
    let cfg = Cfg {
        items: vec![Inner { v: "pad  ".into() }, Inner { v: "  pad".into() }],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn serde_rejects_cr_string() {
    #[derive(Debug, Serialize)]
    struct Cfg {
        v: String,
    }
    let cfg = Cfg { v: "a\rb".into() };
    assert!(to_string(&cfg).is_err());
}

#[test]
fn serde_escapes_dotted_map_keys() {
    let mut m = BTreeMap::new();
    m.insert("a.b".to_string(), 1_i32);
    let back: BTreeMap<String, i32> = from_str(&to_string(&m).unwrap()).unwrap();
    assert_eq!(m, back);
}

// ---------------------------------------------------------------------------
// Plain `Value` renderer (`to_string_force_strings` → `render`)
// ---------------------------------------------------------------------------

#[test]
fn force_strings_keeps_padded_strings() {
    assert_force_strings_roundtrip(&obj(&[("k", s("  padded  "))]));
    assert_force_strings_roundtrip(&obj(&[("k", s("   "))]));
    assert_force_strings_roundtrip(&obj(&[("arr", Value::Array(vec![s("  pad"), s("pad  ")]))]));
}

#[test]
fn force_strings_rejects_cr_string() {
    assert!(to_string_force_strings(&obj(&[("k", s("a\rb"))])).is_err());
}

// ---------------------------------------------------------------------------
// Unicode whitespace — the parser trims with `str::trim()` (the full
// Unicode `White_Space` set), so the writer checks must match it, not
// just ASCII space/tab.
// ---------------------------------------------------------------------------

#[test]
fn canonical_keeps_unicode_whitespace_edges() {
    for v in ["\u{A0}val\u{A0}", "\u{85}val", "val\u{3000}", "\u{2028}a"] {
        assert_canonical_roundtrip(&obj(&[("k", s(v))]));
        assert_canonical_roundtrip(&obj(&[("arr", Value::Array(vec![s(v)]))]));
    }
}

#[test]
fn serde_keeps_unicode_whitespace_edges() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        v: String,
        parts: Vec<String>,
    }
    let cfg = Cfg {
        v: "\u{A0}val\u{A0}".into(),
        parts: vec!["\u{85}val".into(), "val\u{3000}".into()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn force_strings_keeps_unicode_whitespace_edges() {
    assert_force_strings_roundtrip(&obj(&[("k", s("\u{A0}val\u{A0}"))]));
}

// ---------------------------------------------------------------------------
// Comment-marker collision: a bare array item starting with `##` would
// be read back as a comment line and silently dropped.
// ---------------------------------------------------------------------------

#[test]
fn canonical_keeps_comment_marker_array_item() {
    assert_canonical_roundtrip(&obj(&[("arr", Value::Array(vec![s("## x"), s("##")]))]));
}

#[test]
fn serde_keeps_comment_marker_array_item() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        notes: Vec<String>,
    }
    let cfg = Cfg {
        notes: vec!["## x".into()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn force_strings_keeps_comment_marker_array_item() {
    assert_force_strings_roundtrip(&obj(&[("arr", Value::Array(vec![s("## x")]))]));
}

// ---------------------------------------------------------------------------
// Bare array items the parser reads as markers or closers: a leading
// `::` is eaten as the raw marker, and a sole `]` / `}` line closes
// the enclosing compound.
// ---------------------------------------------------------------------------

#[test]
fn canonical_keeps_marker_shaped_array_items() {
    assert_canonical_roundtrip(&obj(&[(
        "arr",
        Value::Array(vec![s(":: x"), s("::x"), s("]"), s("}")]),
    )]));
}

#[test]
fn serde_keeps_marker_shaped_array_items() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg = Cfg {
        items: vec![":: x".into(), "]".into(), "}".into()],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

// ---------------------------------------------------------------------------
// Multi-line form selection: the sole-`)` / sole-`))` collisions must
// yield an error or a lossless fallback, never a corrupt document.
// ---------------------------------------------------------------------------

#[test]
fn canonical_rejects_both_terminator_lines() {
    // Verbatim breaks on the `))` line, stripped breaks on the `)`
    // line — no form holds this body.
    assert!(emit_canonical(&obj(&[("k", s("a\n)\n))"))])).is_err());
    assert!(emit_canonical(&obj(&[("arr", Value::Array(vec![s("a\n)\n))")]))])).is_err());
}

#[test]
fn canonical_keeps_partially_indented_body_with_double_paren_line() {
    // `))` forces the stripped fallback; the unindented `a` pins the
    // common indent to zero, so `  b` must survive the dedent.
    assert_canonical_roundtrip(&obj(&[("k", s("a\n  b\n))"))]));
}

#[test]
fn serde_keeps_double_paren_body_with_indented_line() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        v: String,
    }
    let cfg = Cfg {
        v: "))\n  x\ny".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn force_strings_keeps_double_paren_body_with_indented_line() {
    assert_force_strings_roundtrip(&obj(&[("k", s("))\n  x\ny"))]));
}
