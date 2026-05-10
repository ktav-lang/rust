//! Smoke tests for the public `ktav::thin` event-stream API exposed
//! in 0.1.7. These exercise the callback shape, event ordering, and
//! borrowed-`&str` lifetimes without going through serde.

use ktav::thin::{parse_events, ParseEvent};
use ktav::{Error, ErrorKind};

/// Collect every event for a successful parse into an owned vector
/// (with `Str` / `Key` / `Integer` / `Float` payloads converted to
/// `String`) so tests can assert on the exact sequence.
#[derive(Debug, PartialEq, Eq)]
enum Owned {
    Null,
    Bool(bool),
    Integer(String),
    Float(String),
    Str(String),
    Key(String),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

fn collect(input: &str) -> Vec<Owned> {
    let mut out = Vec::new();
    parse_events(input, |ev| {
        out.push(match ev {
            ParseEvent::Null => Owned::Null,
            ParseEvent::Bool(b) => Owned::Bool(b),
            ParseEvent::Integer(s) => Owned::Integer(s.to_string()),
            ParseEvent::Float(s) => Owned::Float(s.to_string()),
            ParseEvent::Str(s) => Owned::Str(s.to_string()),
            ParseEvent::Key(k) => Owned::Key(k.to_string()),
            ParseEvent::BeginObject => Owned::BeginObject,
            ParseEvent::EndObject => Owned::EndObject,
            ParseEvent::BeginArray => Owned::BeginArray,
            ParseEvent::EndArray => Owned::EndArray,
            _ => unreachable!("ParseEvent is non_exhaustive but all known variants handled"),
        });
    })
    .unwrap();
    out
}

#[test]
fn flat_pairs_emit_expected_sequence() {
    let src = "port: 8080\nhost: example.com\n";
    let events = collect(src);
    assert_eq!(
        events,
        vec![
            Owned::BeginObject,
            Owned::Key("port".into()),
            Owned::Str("8080".into()),
            Owned::Key("host".into()),
            Owned::Str("example.com".into()),
            Owned::EndObject,
        ]
    );
}

#[test]
fn nested_object_brackets_inner_pair() {
    let src = "cfg: {\n  port: 8080\n}\n";
    let events = collect(src);
    assert_eq!(
        events,
        vec![
            Owned::BeginObject,
            Owned::Key("cfg".into()),
            Owned::BeginObject,
            Owned::Key("port".into()),
            Owned::Str("8080".into()),
            Owned::EndObject,
            Owned::EndObject,
        ]
    );
}

#[test]
fn array_with_marker_items() {
    let src = "items: [\n  :: literal\n  :i 42\n  :f 3.14\n]\n";
    let events = collect(src);
    assert_eq!(
        events,
        vec![
            Owned::BeginObject,
            Owned::Key("items".into()),
            Owned::BeginArray,
            Owned::Str("literal".into()),
            Owned::Integer("42".into()),
            Owned::Float("3.14".into()),
            Owned::EndArray,
            Owned::EndObject,
        ]
    );
}

#[test]
fn invalid_input_returns_structured_error_matching_parse() {
    // Same input fed through `ktav::parse` and through `parse_events`
    // must surface the same `ErrorKind` variant.
    // Anchor with a known-Object pair — first-line `port:8080`
    // would otherwise be a top-level Array bare-scalar (spec
    // § 5.0.1) and parse cleanly.
    let src = "anchor: ok\nport:8080\n"; // missing space after the marker on line 2

    let parse_err = ktav::parse(src).expect_err("must error");
    let callback_err = parse_events(src, |_| {
        // Should never be called past the error — but if it is, we
        // still don't panic; the parser has already returned `Err`.
    })
    .expect_err("must error");

    match (&parse_err, &callback_err) {
        (
            Error::Structured(ErrorKind::MissingSeparatorSpace { line: l1, .. }),
            Error::Structured(ErrorKind::MissingSeparatorSpace { line: l2, .. }),
        ) => {
            assert_eq!(l1, l2);
        }
        other => panic!("error kinds did not match: {other:?}"),
    }
}

#[test]
fn dotted_key_resolved_into_synthetic_compound_events() {
    // Dotted keys are resolved at tokenize time → the callback sees a
    // synthetic `Key("a") + BeginObject + Key("b") + Str("v") +
    // EndObject` triple, exactly as if the source had been written
    // out with explicit nesting.
    let src = "a.b: v\n";
    let events = collect(src);
    assert_eq!(
        events,
        vec![
            Owned::BeginObject,
            Owned::Key("a".into()),
            Owned::BeginObject,
            Owned::Key("b".into()),
            Owned::Str("v".into()),
            Owned::EndObject,
            Owned::EndObject,
        ]
    );
}

#[test]
fn event_str_slices_borrow_from_input() {
    // Verify zero-copy: for plain single-line scalars the `&str` event
    // payload must point inside the original input buffer (same byte
    // range), not into a separately-owned arena allocation.
    let src = "port: 8080\nhost: example.com\n";
    let input_start = src.as_ptr() as usize;
    let input_end = input_start + src.len();

    let mut borrowed_count = 0_usize;
    parse_events(src, |ev| match ev {
        ParseEvent::Str(s) | ParseEvent::Key(s) => {
            let p = s.as_ptr() as usize;
            if p >= input_start && p + s.len() <= input_end {
                borrowed_count += 1;
            }
        }
        _ => {}
    })
    .unwrap();

    // 2 keys + 2 string values, all expected to be borrowed.
    assert_eq!(borrowed_count, 4);
}

#[test]
fn re_export_at_crate_root_works() {
    // `ktav::parse_events` and `ktav::ParseEvent` should resolve.
    let mut count = 0_usize;
    ktav::parse_events("k: v\n", |_ev: ktav::ParseEvent<'_>| count += 1).unwrap();
    assert!(count > 0);
}

#[test]
fn keywords_become_typed_events() {
    let src = "a: null\nb: true\nc: false\n";
    let events = collect(src);
    assert_eq!(
        events,
        vec![
            Owned::BeginObject,
            Owned::Key("a".into()),
            Owned::Null,
            Owned::Key("b".into()),
            Owned::Bool(true),
            Owned::Key("c".into()),
            Owned::Bool(false),
            Owned::EndObject,
        ]
    );
}
