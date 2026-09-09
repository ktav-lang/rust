//! Pins the two README (en/ru/zh) claims corrected in review round 14
//! (R14-F3), mirroring the `doc_example.rs` convention: the READMEs are
//! not compiled as doctests, so without this the snippets could drift
//! from the writer's actual output unnoticed again.

use ktav::{parse, parse_events, ParseEvent, Value};

/// "### 1. Scalars" example: scalars are typed at the `Value` level from
/// their lexical form, not all strings — only `name` (a bare word) is
/// String; `port` is Integer.
#[test]
fn readme_scalars_example_types_port_as_integer() {
    let v = parse("name: Russia\nport: 20082\n").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("name").unwrap(), &Value::String("Russia".into()));
    assert_eq!(obj.get("port").unwrap(), &Value::Integer("20082".into()));
}

/// The "### Numbers" intro section (R15-F4): `Integer`/`Float` store a
/// *normalized* payload, not the original spelling — `+1_000` becomes
/// `Integer("1000")`, `1.0e+2` becomes `Float("100.0")`.
#[test]
fn readme_numbers_intro_stores_normalized_payload_not_original_spelling() {
    let v = parse("i: +1_000\nf: 1.0e+2\n").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("i").unwrap(), &Value::Integer("1000".into()));
    assert_eq!(obj.get("f").unwrap(), &Value::Float("100.0".into()));
}

/// The `parse_events` doc section: the root's begin/end event pair
/// depends on the document's root kind (§ 5.0.1) — an Array-shaped
/// document emits `BeginArray`/`EndArray`, not an outer Object.
#[test]
fn readme_parse_events_array_root_has_no_outer_object() {
    let mut events = Vec::new();
    parse_events("[1, 2]\n", |ev| {
        events.push(match ev {
            ParseEvent::BeginArray => "BeginArray".to_string(),
            ParseEvent::EndArray => "EndArray".to_string(),
            ParseEvent::Integer(s) => format!("Integer({s})"),
            other => panic!("unexpected event: {other:?}"),
        });
    })
    .unwrap();
    assert_eq!(
        events,
        ["BeginArray", "Integer(1)", "Integer(2)", "EndArray"]
    );
}
