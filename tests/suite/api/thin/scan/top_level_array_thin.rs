//! Smoke tests for top-level Array support through the thin event
//! path — spec § 5.0.1 must be honoured by `parse_events` and
//! `from_str` the same way it is by `parse`.

use ktav::thin::{parse_events, ParseEvent};

fn collect_kinds(src: &str) -> Result<Vec<&'static str>, ktav::Error> {
    let mut kinds = Vec::new();
    parse_events(src, |e| {
        let kind = match e {
            ParseEvent::Null => "Null",
            ParseEvent::Bool(_) => "Bool",
            ParseEvent::Integer(_) => "Integer",
            ParseEvent::Float(_) => "Float",
            ParseEvent::Str(_) => "Str",
            ParseEvent::Key(_) => "Key",
            ParseEvent::BeginObject => "BeginObject",
            ParseEvent::EndObject => "EndObject",
            ParseEvent::BeginArray => "BeginArray",
            ParseEvent::EndArray => "EndArray",
            _ => "Unknown",
        };
        kinds.push(kind);
    })?;
    Ok(kinds)
}

#[test]
fn thin_parse_events_top_array_emits_array_brackets() {
    let kinds = collect_kinds("foo\nbar\n").unwrap();
    assert_eq!(kinds, vec!["BeginArray", "Str", "Str", "EndArray"]);
}

#[test]
fn thin_from_str_top_array_into_vec() {
    let v: Vec<String> = ktav::from_str("foo\nbar\nbaz\n").unwrap();
    assert_eq!(v, vec!["foo", "bar", "baz"]);
}

#[test]
fn thin_parse_events_top_object_still_object() {
    let kinds = collect_kinds("a: 1\nb: 2\n").unwrap();
    assert_eq!(kinds[0], "BeginObject");
    assert_eq!(*kinds.last().unwrap(), "EndObject");
}
