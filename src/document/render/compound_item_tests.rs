use crate::{parse, ObjectMap, Value};

use super::{render, render_compound_item};

#[test]
fn borrowed_compound_matches_legacy_wrapper_bytes() {
    for source in [
        "{}\n",
        "[]\n",
        "[1, true, {}]\n",
        "payload: first\n",
        "payload: (\n    first\n    second\n)\n",
        "payload: ((\n  leading\ntrailing  \n))\n",
        "payload: [\n    true\n    null\n]\n",
    ] {
        let value = parse(source).unwrap();
        let legacy = render(&Value::Array(vec![value.clone()])).unwrap();
        let borrowed = render_compound_item(&value).unwrap();
        assert_eq!(format!("[\n{borrowed}]\n"), legacy, "{source}");
    }
}

#[test]
fn borrowed_compound_preserves_writer_error_paths() {
    for (key, value) in [
        ("payload", Value::String("has\rcr".into())),
        ("payload", Value::Float("NaN".into())),
        ("", Value::Null),
    ] {
        let mut object = ObjectMap::default();
        object.insert(key.into(), value);
        let value = Value::Object(object);
        let legacy = render(&Value::Array(vec![value.clone()])).unwrap_err();
        let borrowed = render_compound_item(&value).unwrap_err();
        assert_eq!(borrowed.to_string(), legacy.to_string());
    }
}
