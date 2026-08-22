//! Keeps the padded-string example rendered in the READMEs (en/ru/zh)
//! under test, mirroring the `doc_example.rs` convention: the READMEs
//! are not compiled as doctests, so without this the snippet could
//! drift from the writer's actual output unnoticed.

use ktav::{parse, render::render, ObjectMap, Value};

fn obj(key: &str, val: &str) -> Value {
    let mut m = ObjectMap::default();
    m.insert(key.into(), Value::String(val.into()));
    Value::Object(m)
}

/// The READMEs show *two* blocks to make the point that the form
/// depends on which side the whitespace is on. Both are pinned here,
/// byte for byte, along with the round-trip claim that follows them.
#[test]
fn readme_trailing_space_uses_the_stripped_form() {
    // Stripping removes only the common leading indent, so a trailing
    // space survives it — this is why the example is not verbatim.
    let v = obj("password", "hunter2 ");
    let text = render(&v).unwrap();
    assert_eq!(text, "password: (\n    hunter2 \n)\n");
    assert_eq!(parse(&text).unwrap(), v);
}

#[test]
fn readme_leading_space_forces_the_verbatim_form() {
    // Stripping would eat the leading spaces, so the writer must reach
    // for `(( ))` here. Pinning this stops the two README examples from
    // silently collapsing into the same form.
    let v = obj("indent", "  padded");
    let text = render(&v).unwrap();
    assert_eq!(text, "indent: ((\n  padded\n))\n");
    assert_eq!(parse(&text).unwrap(), v);
}
