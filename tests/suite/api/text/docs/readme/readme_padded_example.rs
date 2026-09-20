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
/// depends on which side the whitespace is on. Both force the verbatim
/// form as of 0.7; both are pinned here,
/// byte for byte, along with the round-trip claim that follows them.
#[test]
fn readme_trailing_space_uses_the_verbatim_form() {
    // As of 0.7 the stripped form strips trailing whitespace from every
    // content line (§ 5.6), so a trailing space forces the verbatim form —
    // which preserves it byte-for-byte.
    let v = obj("password", "hunter2 ");
    let text = render(&v).unwrap();
    assert_eq!(text, "password: ((\nhunter2 \n))\n");
    assert_eq!(parse(&text).unwrap(), v);
}

#[test]
fn readme_leading_space_forces_the_verbatim_form() {
    // Both leading and trailing whitespace force the verbatim form as
    // of 0.7: stripping would eat the leading spaces, and 0.7 § 5.6
    // strips trailing whitespace from every content line. Pinning both
    // examples keeps the READMEs honest about the writer's actual output.
    let v = obj("indent", "  padded");
    let text = render(&v).unwrap();
    assert_eq!(text, "indent: ((\n  padded\n))\n");
    assert_eq!(parse(&text).unwrap(), v);
}
