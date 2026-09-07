//! Regression tests for inline scalar handling (spec 0.7 § 3.7 / § 5.2).
//!
//! Pinned here:
//! 1. Edge whitespace arriving via decoded escapes is preserved (trimming is
//!    a source-matching concern only — § 4; escapes decode "before further
//!    classification" — § 3.7).
//! 2. § 5.2 rule 5 (`()` / `(())` → empty String) applies on the inline path.
//! 3. Raw-marker bodies and escape-decoded parens bypass/escape rule 5.

use ktav::{parse, ObjectMap, Value};

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

fn arr(items: &[Value]) -> Value {
    Value::Array(items.to_vec())
}

#[test]
fn decoded_edge_whitespace_is_preserved_inline() {
    // Leading decoded space.
    assert_eq!(parse("{s: \\u0020x}").unwrap(), obj(&[("s", s(" x"))]));
    // Trailing decoded space (raw source body is trimmed first, so the
    // trailing space can only come from the escape).
    assert_eq!(parse("[x\\u0020]").unwrap(), arr(&[s("x ")]));
    // Both edges via a single escape... and both edges present.
    let src = format!("{{s: {}x{}}}", "\\u0020", "\\u0020");
    assert_eq!(parse(&src).unwrap(), obj(&[("s", s(" x "))]));
    // Brace escape followed by trailing space.
    assert_eq!(
        parse("{s: \\{abc\\u0020}").unwrap(),
        obj(&[("s", s("{abc "))])
    );
    // Tag-looking item keeps its decoded trailing space.
    assert_eq!(parse("[##tag\\u0020]").unwrap(), arr(&[s("##tag ")]));
}

#[test]
fn empty_paren_inline_items_are_empty_string() {
    assert_eq!(parse("[(), a]").unwrap(), arr(&[s(""), s("a")]));
    assert_eq!(parse("[()]").unwrap(), arr(&[s("")]));
    assert_eq!(parse("{a: ()}").unwrap(), obj(&[("a", s(""))]));
    assert_eq!(parse("{a: (())}").unwrap(), obj(&[("a", s(""))]));
    assert_eq!(
        parse("[x, (), y]").unwrap(),
        arr(&[s("x"), s(""), s("y")])
    );
}

#[test]
fn raw_marker_and_escapes_bypass_empty_paren_rule() {
    // Raw marker bypasses § 5.2 entirely.
    assert_eq!(parse("{s:: ()}").unwrap(), obj(&[("s", s("()"))]));
    // Escape provenance: decoded bytes are never re-classified.
    assert_eq!(parse("[\\u0028\\u0029]").unwrap(), arr(&[s("()")]));
    // Decoded paren as scalar content stays literal.
    assert_eq!(parse("{s: \\u0028}").unwrap(), obj(&[("s", s("("))]));
}

#[test]
fn decoded_edge_whitespace_survives_named_escapes_too() {
    // Named `\n` still decodes to a real LF.
    assert_eq!(parse("{s: a\\nb}").unwrap(), obj(&[("s", s("a\nb"))]));
    // Leading decoded space preserved AND named escape decoded.
    assert_eq!(
        parse("{s: \\u0020a\\nb}").unwrap(),
        obj(&[("s", s(" a\nb"))])
    );
}
