//! Keeps the padded-string example rendered in the READMEs (en/ru/zh)
//! under test, mirroring the `doc_example.rs` convention: the READMEs
//! are not compiled as doctests, so without this the snippet could
//! drift from the writer's actual output unnoticed.

use ktav::{parse, render::render, ObjectMap, Value};

#[test]
fn readme_padded_string_example() {
    let mut m = ObjectMap::default();
    m.insert("password".into(), Value::String("hunter2 ".into()));
    let v = Value::Object(m);

    // Exactly the block shown in the READMEs.
    let text = render(&v).unwrap();
    assert_eq!(text, "password: (\n    hunter2 \n)\n");

    // And the claim made right after it: the space survives.
    assert_eq!(parse(&text).unwrap(), v);
}
