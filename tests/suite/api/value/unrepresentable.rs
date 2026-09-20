//! Writer-side representability (spec 0.7 § 5.9.0): a `Value` that no
//! conforming ktav document can hold must be rejected by every writer
//! surface with `Error::Unrepresentable` carrying the exact
//! [`ktav::ReasonCode`] — before any byte is emitted.
//!
//! Covered surfaces:
//! - `emit_canonical` (canonical writer, spec § 5.9)
//! - `render` (plain `Value` renderer)
//! - `to_string_force_strings` (renders the coerced value; `NonFiniteFloat`
//!   cannot fire there by construction, the other codes still can)
//! - `to_string` (serde text serializer)
//!
//! Also pinned: the originally-cited rust#5 problem keys round-trip
//! losslessly, the root check precedes the node check, and the error
//! API surface (`code_name`, `Display`, `reason_code`).

use std::collections::BTreeMap;

use ktav::render::render;
use ktav::ser::to_value;
use ktav::{
    emit_canonical, from_str, parse, to_string, to_string_force_strings, Error, ObjectMap,
    ReasonCode, Value,
};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn f(v: &str) -> Value {
    Value::Float(v.parse().unwrap_or_else(|_| panic!("float scalar: {v:?}")))
}

fn n(v: i64) -> Value {
    Value::Integer(v.to_string().parse().unwrap_or_else(|_| unreachable!()))
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// The exact `ReasonCode` the given canonical-emission failure carries.
fn code_of(r: Result<String, Error>) -> ReasonCode {
    let e = r.unwrap_err();
    e.reason_code()
        .unwrap_or_else(|| panic!("expected a reason code, got: {e}"))
}

/// `emit_canonical` → `parse` must reproduce the exact `Value`.
fn assert_canonical_roundtrip(v: &Value) {
    let text = emit_canonical(v).unwrap_or_else(|e| panic!("emit_canonical({v:?}): {e}"));
    let back = parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(&back, v, "canonical text was {text:?}");
}

/// `render` → `parse` must reproduce the exact `Value`.
fn assert_render_roundtrip(v: &Value) {
    let text = render(v).unwrap_or_else(|e| panic!("render({v:?}): {e}"));
    let back = parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(&back, v, "rendered text was {text:?}");
}

// ---------------------------------------------------------------------------
// Section 1 — one rejection test per reason code, per surface
// ---------------------------------------------------------------------------

// --- ScalarRoot ------------------------------------------------------------

#[test]
fn scalar_root_canonical() {
    for v in [Value::Null, Value::Bool(true), n(1), f("1.5"), s("x")] {
        assert_eq!(code_of(emit_canonical(&v)), ReasonCode::ScalarRoot);
    }
}

#[test]
fn scalar_root_render() {
    for v in [Value::Null, Value::Bool(true), n(1), f("1.5"), s("x")] {
        assert_eq!(code_of(render(&v)), ReasonCode::ScalarRoot);
    }
}

#[test]
fn scalar_root_force_strings() {
    assert_eq!(
        code_of(to_string_force_strings(&Value::Null)),
        ReasonCode::ScalarRoot
    );
}

#[test]
fn scalar_root_serde_scalars() {
    assert_eq!(code_of(to_string(&42_i32)), ReasonCode::ScalarRoot);
    assert_eq!(code_of(to_string(&"x")), ReasonCode::ScalarRoot);
    assert_eq!(code_of(to_string(&None::<i32>)), ReasonCode::ScalarRoot);
    assert_eq!(code_of(to_string(&())), ReasonCode::ScalarRoot);
}

#[derive(Serialize)]
enum Color {
    Red,
}

#[test]
fn scalar_root_serde_unit_variant() {
    // A unit-variant root serializes as a bare string scalar.
    assert_eq!(code_of(to_string(&Color::Red)), ReasonCode::ScalarRoot);
}

#[test]
fn array_root_is_representable() {
    // § 5.0.1 permits an Array root; the serde surface accepts it (the
    // old generic "top-level value must be an object" rejection for
    // seq-like roots is gone). Scalar roots stay ScalarRoot.
    let text = to_string(&vec![1_i32, 2]).unwrap();
    assert_eq!(text, "1\n2\n");
}

// --- EmptyKeyName ----------------------------------------------------------

#[test]
fn empty_key_name_canonical() {
    // The primary rust#5 case: this used to emit `: v`, re-parsing as
    // Array-root ["v"].
    let v = obj(&[("", s("v"))]);
    assert_eq!(code_of(emit_canonical(&v)), ReasonCode::EmptyKeyName);
}

#[test]
fn empty_key_name_render() {
    let v = obj(&[("", s("v"))]);
    assert_eq!(code_of(render(&v)), ReasonCode::EmptyKeyName);
}

#[test]
fn empty_key_name_force_strings() {
    let v = obj(&[("", s("v"))]);
    assert_eq!(
        code_of(to_string_force_strings(&v)),
        ReasonCode::EmptyKeyName
    );
}

#[test]
fn empty_key_name_nested() {
    let v = obj(&[("ok", s("v")), ("bad", obj(&[("", s("x"))]))]);
    assert_eq!(code_of(emit_canonical(&v)), ReasonCode::EmptyKeyName);
}

#[test]
fn empty_key_name_serde_map() {
    let m: BTreeMap<String, String> = [("".to_string(), "v".to_string())].into_iter().collect();
    assert_eq!(code_of(to_string(&m)), ReasonCode::EmptyKeyName);
}

/// Struct with an empty field name, via a tiny manual `Serialize` impl.
struct EmptyField;

impl Serialize for EmptyField {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = ser.serialize_struct("EmptyField", 1)?;
        st.serialize_field("", &1_i32)?;
        st.end()
    }
}

#[test]
fn empty_key_name_serde_struct_field() {
    assert_eq!(code_of(to_string(&EmptyField)), ReasonCode::EmptyKeyName);
}

#[derive(Serialize)]
enum EmptyTupleVariantName {
    #[serde(rename = "")]
    Empty(i64, i64),
}

#[test]
fn empty_key_name_serde_tuple_variant() {
    // A root tuple variant's name IS the root Object's first key
    // (§ 8.2 single-pair Object); an empty name would emit the
    // rust#5-shaped `: [...]` document, so § 5.9.0 rejects it before
    // any byte is written.
    assert_eq!(
        code_of(to_string(&EmptyTupleVariantName::Empty(1, 2))),
        ReasonCode::EmptyKeyName
    );
}

#[test]
fn empty_key_name_serde_tuple_variant_writer_agreement() {
    // Both writer surfaces must agree on the empty variant name: the
    // owned to_value + render path already rejected it; the streaming
    // to_string path must return the same Unrepresentable(EmptyKeyName).
    let v = to_value(&EmptyTupleVariantName::Empty(1, 2)).unwrap();
    assert_eq!(code_of(render(&v)), ReasonCode::EmptyKeyName);
    assert_eq!(
        code_of(to_string(&EmptyTupleVariantName::Empty(1, 2))),
        ReasonCode::EmptyKeyName
    );
}

// --- NonFiniteFloat --------------------------------------------------------

#[test]
fn non_finite_float_canonical() {
    assert_eq!(
        code_of(emit_canonical(&obj(&[("k", f("NaN"))]))),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(emit_canonical(&obj(&[("k", f("Infinity"))]))),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(emit_canonical(&obj(&[("k", f("-Infinity"))]))),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(emit_canonical(&Value::Array(vec![f("NaN")]))),
        ReasonCode::NonFiniteFloat
    );
    let deep = obj(&[("a", Value::Array(vec![obj(&[("deep", f("inf"))])]))]);
    assert_eq!(code_of(emit_canonical(&deep)), ReasonCode::NonFiniteFloat);
}

#[test]
fn non_finite_float_render() {
    assert_eq!(
        code_of(render(&obj(&[("k", f("NaN"))]).clone())),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(render(&Value::Array(vec![f("Infinity")]))),
        ReasonCode::NonFiniteFloat
    );
}

#[derive(Serialize)]
struct CfgF64 {
    x: f64,
}

#[derive(Serialize)]
struct CfgF32 {
    x: f32,
}

#[derive(Serialize)]
struct WithVec {
    xs: Vec<f64>,
}

#[test]
fn non_finite_float_serde() {
    assert_eq!(
        code_of(to_string(&CfgF64 { x: f64::NAN })),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(to_string(&CfgF64 { x: f64::INFINITY })),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(to_string(&CfgF64 {
            x: f64::NEG_INFINITY
        })),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(to_string(&CfgF32 { x: f32::NAN })),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(
        code_of(to_string(&WithVec {
            xs: vec![1.0, f64::NAN]
        })),
        ReasonCode::NonFiniteFloat
    );
}

#[test]
fn finite_floats_not_rejected() {
    // No over-rejection: finite floats emit and round-trip exactly.
    assert_canonical_roundtrip(&obj(&[("k", f("1.5"))]));
    assert_render_roundtrip(&obj(&[("k", f("1.5"))]));
    assert_canonical_roundtrip(&obj(&[("k", f("-0.0"))]));
    assert_render_roundtrip(&obj(&[("k", f("-0.0"))]));
    // `1e100` canonicalises scientifically; compare parsed-back Values.
    assert_canonical_roundtrip(&obj(&[("k", f("1e100"))]));
    assert_render_roundtrip(&obj(&[("k", f("1e100"))]));
}

// --- CRByte ----------------------------------------------------------------

#[test]
fn cr_byte_canonical() {
    assert_eq!(
        code_of(emit_canonical(&obj(&[("k", s("a\rb"))]))),
        ReasonCode::CRByte
    );
    assert_eq!(
        code_of(emit_canonical(&Value::Array(vec![s("a\rb")]))),
        ReasonCode::CRByte
    );
}

#[test]
fn cr_byte_render() {
    assert_eq!(
        code_of(render(&obj(&[("k", s("a\rb"))]).clone())),
        ReasonCode::CRByte
    );
    assert_eq!(
        code_of(render(&Value::Array(vec![s("a\rb")]))),
        ReasonCode::CRByte
    );
}

#[test]
fn cr_byte_force_strings() {
    assert_eq!(
        code_of(to_string_force_strings(&obj(&[("k", s("a\rb"))]))),
        ReasonCode::CRByte
    );
}

#[test]
fn cr_byte_serde() {
    #[derive(Serialize)]
    struct Msg {
        text: String,
    }
    assert_eq!(
        code_of(to_string(&Msg {
            text: "a\rb".to_string()
        })),
        ReasonCode::CRByte
    );
    let m: BTreeMap<String, String> = [("k".to_string(), "a\rb".to_string())]
        .into_iter()
        .collect();
    assert_eq!(code_of(to_string(&m)), ReasonCode::CRByte);
}

// --- Multi-line collision codes (§ 5.9.7) ----------------------------------

#[test]
fn both_forms_required_canonical() {
    let v = obj(&[("k", s("))\n)"))]);
    assert_eq!(code_of(emit_canonical(&v)), ReasonCode::BothFormsRequired);
}

#[test]
fn both_forms_required_render() {
    let v = obj(&[("k", s("))\n)"))]);
    assert_eq!(code_of(render(&v.clone())), ReasonCode::BothFormsRequired);
}

#[test]
fn both_forms_required_serde() {
    #[derive(Serialize)]
    struct Msg {
        text: String,
    }
    assert_eq!(
        code_of(to_string(&Msg {
            text: "))\n)".to_string()
        })),
        ReasonCode::BothFormsRequired
    );
    let m: BTreeMap<String, String> = [("k".to_string(), "))\n)".to_string())]
        .into_iter()
        .collect();
    assert_eq!(code_of(to_string(&m)), ReasonCode::BothFormsRequired);
}

#[test]
fn trailing_whitespace_collision_canonical() {
    let v = obj(&[("k", s("))\nx "))]);
    assert_eq!(
        code_of(emit_canonical(&v)),
        ReasonCode::TrailingWhitespaceCollision
    );
}

#[test]
fn trailing_whitespace_collision_render() {
    let v = obj(&[("k", s("))\nx "))]);
    assert_eq!(
        code_of(render(&v.clone())),
        ReasonCode::TrailingWhitespaceCollision
    );
}

#[test]
fn trailing_whitespace_collision_serde() {
    #[derive(Serialize)]
    struct Msg {
        text: String,
    }
    assert_eq!(
        code_of(to_string(&Msg {
            text: "))\nx ".to_string()
        })),
        ReasonCode::TrailingWhitespaceCollision
    );
    let m: BTreeMap<String, String> = [("k".to_string(), "))\nx ".to_string())]
        .into_iter()
        .collect();
    assert_eq!(
        code_of(to_string(&m)),
        ReasonCode::TrailingWhitespaceCollision
    );
}

#[test]
fn leading_whitespace_collision_canonical() {
    // Leading space before `))` plus a shared leading space on the
    // second line — mirrors the spec fixture.
    let v = obj(&[("k", s(" ))\n x"))]);
    assert_eq!(
        code_of(emit_canonical(&v)),
        ReasonCode::LeadingWhitespaceCollision
    );
}

#[test]
fn leading_whitespace_collision_render() {
    let v = obj(&[("k", s(" ))\n x"))]);
    assert_eq!(
        code_of(render(&v.clone())),
        ReasonCode::LeadingWhitespaceCollision
    );
}

#[test]
fn leading_whitespace_collision_serde() {
    #[derive(Serialize)]
    struct Msg {
        text: String,
    }
    assert_eq!(
        code_of(to_string(&Msg {
            text: " ))\n x".to_string()
        })),
        ReasonCode::LeadingWhitespaceCollision
    );
    let m: BTreeMap<String, String> = [("k".to_string(), " ))\n x".to_string())]
        .into_iter()
        .collect();
    assert_eq!(
        code_of(to_string(&m)),
        ReasonCode::LeadingWhitespaceCollision
    );
}

#[test]
fn collision_edge_single_indented_segment() {
    // Single segment, indented, trimming to `))` → leading collision only.
    let v = obj(&[("k", s("  ))"))]);
    assert_eq!(
        code_of(emit_canonical(&v)),
        ReasonCode::LeadingWhitespaceCollision
    );
    assert_eq!(
        code_of(render(&v.clone())),
        ReasonCode::LeadingWhitespaceCollision
    );
}

#[test]
fn collision_edge_trailing_wins_over_leading() {
    // Both applicable → trailing wins (our pinned priority; both
    // collision shapes are conforming outcomes).
    let v = obj(&[("k", s("  ))  "))]);
    assert_eq!(
        code_of(emit_canonical(&v)),
        ReasonCode::TrailingWhitespaceCollision
    );
    assert_eq!(
        code_of(render(&v.clone())),
        ReasonCode::TrailingWhitespaceCollision
    );
}

// --- R8-F5: differing indents are representable (§ 5.6 common prefix) ------

#[test]
fn r8f5_differing_tab_space_indent_roundtrip() {
    // TAB at position 0 on line 1, SPACE on line 2: § 5.6 — "a line
    // starting with a tab and a line starting with a space share no
    // common prefix at all". The `))` segment forces the stripped
    // fallback; nothing is dedented, so the body is representable and
    // must round-trip to the identical Value.
    let v = obj(&[("value", s("\talpha\n ))"))]);
    assert_canonical_roundtrip(&v);
    assert_render_roundtrip(&v);
}

#[test]
fn r8f5_differing_unicode_indent_roundtrip() {
    // U+2000 vs U+2001 share their first two UTF-8 bytes but are
    // different code points; § 5.6 measures the common prefix
    // code-point-for-code-point, so again no shared prefix exists.
    let v = obj(&[("value", s("\u{2000}alpha\n\u{2001}))"))]);
    assert_canonical_roundtrip(&v);
    assert_render_roundtrip(&v);
}

#[test]
fn r8f5_review_lossless_candidate_parses_to_same_value() {
    // The review's verified lossless document for the rejected body:
    // the four writer-added spaces dedent away, the differing
    // TAB/SPACE survive.
    let v = obj(&[("value", s("\talpha\n ))"))]);
    let back = parse("value: (\n    \talpha\n     ))\n)\n")
        .unwrap_or_else(|e| panic!("parse of the review candidate: {e}"));
    assert_eq!(back, v);
}

#[test]
fn r8f5_differing_indent_serde_roundtrip() {
    let m: BTreeMap<String, String> = [("value".to_string(), "\talpha\n ))".to_string())]
        .into_iter()
        .collect();
    let out = to_string(&m).unwrap_or_else(|e| panic!("to_string: {e}"));
    let back: BTreeMap<String, String> = from_str(&out).unwrap();
    assert_eq!(back, m);
}

#[test]
fn r8f5_differing_indent_array_item_roundtrip() {
    let v = Value::Array(vec![s("\talpha\n ))")]);
    assert_canonical_roundtrip(&v);
    assert_render_roundtrip(&v);
}

#[test]
fn r8f5_shared_unicode_prefix_still_collides() {
    // The negative survives for a REAL common prefix: U+2000 at the
    // same position on every non-blank line is one identical leading
    // code point, which § 5.6's dedent strips on re-parse.
    let v = obj(&[("k", s("\u{2000}))\n\u{2000}x"))]);
    assert_eq!(
        code_of(emit_canonical(&v.clone())),
        ReasonCode::LeadingWhitespaceCollision
    );
    assert_eq!(
        code_of(render(&v.clone())),
        ReasonCode::LeadingWhitespaceCollision
    );
}

// ---------------------------------------------------------------------------
// Section 2 — rust#5 regression (the originally-cited keys)
// ---------------------------------------------------------------------------

fn rust5_keys() -> Vec<&'static str> {
    vec!["##note", " k", "k ", "k\nx", "a,b", "a{b"]
}

#[test]
fn rust5_problem_keys_roundtrip_canonical() {
    for key in rust5_keys() {
        let v = obj(&[(key, s("v"))]);
        let out = emit_canonical(&v).unwrap_or_else(|e| panic!("emit_canonical({key:?}): {e}"));
        let back = parse(&out).unwrap_or_else(|e| panic!("parse({out:?}): {e}"));
        // Full lossless round-trip including the exact key (the quoted
        // `" k": v` form must re-parse WITHOUT trimming).
        assert_eq!(back, v, "canonical text was {out:?}");
    }
    // Spot-assert the exact emitted form for two of them.
    let out = emit_canonical(&obj(&[("##note", s("v"))])).unwrap();
    assert_eq!(out, "\"##note\": v\n");
    let out = emit_canonical(&obj(&[("a,b", s("v"))])).unwrap();
    assert_eq!(out, "\"a,b\": v\n");
}

#[test]
fn rust5_problem_keys_roundtrip_render() {
    for key in rust5_keys() {
        let v = obj(&[(key, s("v"))]);
        let out = render(&v.clone()).unwrap_or_else(|e| panic!("render({key:?}): {e}"));
        let back = parse(&out).unwrap_or_else(|e| panic!("parse({out:?}): {e}"));
        assert_eq!(back, v, "rendered text was {out:?}");
    }
}

#[test]
fn rust5_problem_keys_roundtrip_serde() {
    let original: BTreeMap<String, String> = rust5_keys()
        .into_iter()
        .map(|k| (k.to_string(), "v".to_string()))
        .collect();
    let out = to_string(&original).unwrap_or_else(|e| panic!("to_string: {e}"));
    let back: BTreeMap<String, String> =
        from_str(&out).unwrap_or_else(|e| panic!("from_str({out:?}): {e}"));
    assert_eq!(back, original);
}

#[test]
fn rust5_empty_key_rejected_everywhere() {
    let v = obj(&[("", s("v"))]);
    assert_eq!(
        code_of(emit_canonical(&v.clone())),
        ReasonCode::EmptyKeyName
    );
    assert_eq!(code_of(render(&v.clone())), ReasonCode::EmptyKeyName);
    let m: BTreeMap<String, String> = [("".to_string(), "v".to_string())].into_iter().collect();
    assert_eq!(code_of(to_string(&m)), ReasonCode::EmptyKeyName);
}

// ---------------------------------------------------------------------------
// Section 3 — no-partial-output + precedence
// ---------------------------------------------------------------------------

#[test]
fn rejection_yields_no_output() {
    // A LATER violation (second pair) still rejects. The API returns
    // `Result<String>`, so no bytes can escape on the error path —
    // structural no-partial-output; the pre-pass runs before the
    // buffer is even allocated.
    let v = obj(&[("ok", s("v")), ("bad", f("NaN"))]);
    assert!(emit_canonical(&v).is_err());
    assert_eq!(
        code_of(emit_canonical(&v.clone())),
        ReasonCode::NonFiniteFloat
    );
}

#[test]
fn root_check_precedes_node_check() {
    // Root is a valid Object, node holds a NaN: the node check ran —
    // never ScalarRoot.
    let v = obj(&[("k", f("NaN"))]);
    assert_eq!(
        code_of(emit_canonical(&v.clone())),
        ReasonCode::NonFiniteFloat
    );
    assert_eq!(code_of(render(&v.clone())), ReasonCode::NonFiniteFloat);
}

// ---------------------------------------------------------------------------
// Section 4 — error API surface pins
// ---------------------------------------------------------------------------

#[test]
fn code_names_match_spec_spelling() {
    assert_eq!(ReasonCode::ScalarRoot.code_name(), "ScalarRoot");
    assert_eq!(ReasonCode::EmptyKeyName.code_name(), "EmptyKeyName");
    assert_eq!(ReasonCode::NonFiniteFloat.code_name(), "NonFiniteFloat");
    assert_eq!(ReasonCode::CRByte.code_name(), "CRByte");
    assert_eq!(
        ReasonCode::BothFormsRequired.code_name(),
        "BothFormsRequired"
    );
    assert_eq!(
        ReasonCode::TrailingWhitespaceCollision.code_name(),
        "TrailingWhitespaceCollision"
    );
    assert_eq!(
        ReasonCode::LeadingWhitespaceCollision.code_name(),
        "LeadingWhitespaceCollision"
    );
}

#[test]
fn display_names_reason_and_section() {
    for code in [
        ReasonCode::ScalarRoot,
        ReasonCode::EmptyKeyName,
        ReasonCode::NonFiniteFloat,
        ReasonCode::CRByte,
        ReasonCode::BothFormsRequired,
        ReasonCode::TrailingWhitespaceCollision,
        ReasonCode::LeadingWhitespaceCollision,
    ] {
        let text = code.to_string();
        assert!(text.contains(code.code_name()), "{text}");
        assert!(text.contains("spec §"), "{text}");
    }
}

#[test]
fn parse_error_has_no_reason_code() {
    let e = parse("}").unwrap_err();
    assert!(e.reason_code().is_none(), "{e}");
}
