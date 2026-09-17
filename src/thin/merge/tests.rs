use bumpalo::Bump;

use super::super::event::Event;

use super::*;
use crate::thin::event_parser::parse_events;

/// Post-fix oracle for the key-comparison counter. K distinct
/// Object-valued siblings plus one reopen: K = 8 never builds the
/// index, so its cost is EXACTLY the pre-fix linear-mode cost (29);
/// above `LINEAR_MAX` the index keeps the count linear in K — the
/// pre-fix counts were quadratic (121 / 497 / 2017 for K = 16 / 32 /
/// 64). Only str comparisons count; hashing and probing over
/// occupied-with-different-hash or vacant slots do not.
#[test]
fn key_comparison_counts_stay_linear() {
    use std::fmt::Write as _;

    let mut prev = None;
    for k in [8_usize, 16, 32, 64] {
        let mut text = String::new();
        for i in 0..k {
            let _ = writeln!(text, "a{i}.x: 1");
        }
        text.push_str("a0.y: 2\n");

        let bump = Bump::new();
        let (stream, _) = parse_events(&text, &bump).unwrap();
        perf::reset();
        let _ = merge_reopened(&stream, &bump);
        let count = perf::get();
        eprintln!("K = {k}: {count} key comparisons");
        if k == 8 {
            assert_eq!(count, 29, "K = 8 must keep the exact linear-mode cost");
        } else {
            assert!(
                count <= 4 * k + 16,
                "K = {k}: {count} comparisons exceed the linear bound"
            );
        }
        if let Some(p) = prev {
            assert!(
                count < 3 * p,
                "K = {k}: {count} grows super-linearly vs {p}"
            );
        }
        prev = Some(count);
    }
}

fn json_of(text: &str) -> String {
    let v: serde_json::Value = crate::from_str(text).unwrap();
    serde_json::to_string(&v).unwrap()
}

#[test]
fn value_representative_three_siblings_one_reopen() {
    assert_eq!(
        json_of("a0.x: 1\na1.x: 1\na2.x: 1\na0.y: 2\n"),
        r#"{"a0":{"x":1,"y":2},"a1":{"x":1},"a2":{"x":1}}"#
    );
}

#[test]
fn value_twelve_siblings_reopens_crossing_the_index_threshold() {
    let mut text = String::new();
    let mut want = String::from("{");
    for i in 0..12 {
        let _ = std::fmt::Write::write_fmt(&mut text, format_args!("a{i}.x: 1\n"));
        let _ = std::fmt::Write::write_fmt(&mut want, format_args!("\"a{i}\":{{\"x\":1"));
        if i == 3 {
            want.push_str(",\"y\":2");
        }
        if i == 9 {
            want.push_str(",\"z\":3");
        }
        want.push('}');
        if i < 11 {
            want.push(',');
        }
    }
    text.push_str("a3.y: 2\na9.z: 3\n");
    want.push('}');
    assert_eq!(json_of(&text), want);
}

#[test]
fn value_reopen_inside_nested_object() {
    assert_eq!(
        json_of("o: {\na.x: 1\np: 2\na.y: 3\n}\n"),
        r#"{"o":{"a":{"x":1,"y":3},"p":2}}"#
    );
}

#[test]
fn value_anonymous_array_items_with_reopen() {
    assert_eq!(
        json_of("a.x: 1\nitems: [[2], {b: 3}]\na.y: 4\n"),
        r#"{"a":{"x":1,"y":4},"items":[[2],{"b":3}]}"#
    );
}

#[test]
fn event_indexed_merge_twelve_siblings_two_reopens() {
    let mut text = String::new();
    for i in 0..12 {
        text.push_str(&format!("a{i}.x: 1\n"));
    }
    text.push_str("a3.y: 2\na9.z: 3\n");

    let bump = Bump::new();
    let mut want: Vec<Event> = vec![BeginObject];
    for i in 0..12 {
        let key: &'static str = Box::leak(format!("a{i}").into_boxed_str());
        want.push(Key(key));
        want.push(BeginObject);
        want.push(Key("x"));
        want.push(Integer("1"));
        if i == 3 {
            want.push(Key("y"));
            want.push(Integer("2"));
        }
        if i == 9 {
            want.push(Key("z"));
            want.push(Integer("3"));
        }
        want.push(EndObject);
    }
    want.push(EndObject);
    assert_eq!(merged(&text, &bump), want);
}

fn raw<'a>(text: &'a str, bump: &'a Bump) -> Vec<Event<'a>> {
    let (stream, _) = parse_events(text, bump).unwrap();
    stream.into_iter().collect()
}

fn merged<'a>(text: &'a str, bump: &'a Bump) -> Vec<Event<'a>> {
    let (stream, _) = parse_events(text, bump).unwrap();
    merge_reopened(&stream, bump).into_iter().collect()
}

macro_rules! check {
    ($text:expr, $want:expr) => {{
        let bump = Bump::new();
        let want: Vec<Event> = $want;
        assert_eq!(merged($text, &bump), want);
    }};
}

use Event::{BeginArray, BeginObject, EndArray, EndObject, Integer, Key};

/// Anonymous compound array items survive the merge with the
/// § 5.3.2 reopen (`a.y` after the array) folded into the first
/// `a` block.
#[test]
fn array_in_array_with_reopen() {
    check!(
        "a.x: 1\nitems: [[2]]\na.y: 3\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("items"),
            BeginArray,
            BeginArray,
            Integer("2"),
            EndArray,
            EndArray,
            EndObject,
        ]
    );
}

#[test]
fn object_in_array_with_reopen() {
    check!(
        "a.x: 1\nitems: [{b: 2}]\na.y: 3\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("items"),
            BeginArray,
            BeginObject,
            Key("b"),
            Integer("2"),
            EndObject,
            EndArray,
            EndObject,
        ]
    );
}

/// The corruption site (anonymous compound) lives in a different
/// branch than the reopens.
#[test]
fn compound_item_in_other_branch_than_reopen() {
    check!(
        "a.x: 1\ns: 0\na.y: 3\nitems: [[2]]\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("s"),
            Integer("0"),
            Key("items"),
            BeginArray,
            BeginArray,
            Integer("2"),
            EndArray,
            EndArray,
            EndObject,
        ]
    );
}

/// Reopened dotted-key object inside a root array.
#[test]
fn reopen_inside_root_array() {
    check!(
        "[\n  {\n    a.x: 1\n    s: 2\n    a.y: 3\n  }\n]\n",
        vec![
            BeginArray,
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("s"),
            Integer("2"),
            EndObject,
            EndArray,
        ]
    );
}

/// Deeply nested anonymous compounds mixed with a keyed reopen.
#[test]
fn deeply_nested_anonymous_compounds_with_reopen() {
    check!(
        "a.x: 1\nt: [[{u: [2]}]]\na.y: 3\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("t"),
            BeginArray,
            BeginArray,
            BeginObject,
            Key("u"),
            BeginArray,
            Integer("2"),
            EndArray,
            EndObject,
            EndArray,
            EndArray,
            EndObject,
        ]
    );
}

/// Compound array items without reopens: the zero-copy fast path
/// still returns the raw stream untouched, and the merge pass is
/// the identity on it.
#[test]
fn compound_items_no_reopen_fast_path() {
    let text = "items: [[1], {b: 2}]\nz: 3\n";
    let bump = Bump::new();
    let (stream, reopens) = parse_events(text, &bump).unwrap();
    assert_eq!(reopens, 0);
    assert_eq!(
        stream,
        crate::thin::parse_events_merged(text, &bump).unwrap()
    );
    assert_eq!(stream, merge_reopened(&stream, &bump));
}

#[test]
fn reopen_after_intervening_sibling_merges() {
    check!(
        "a.b: 1
c: 2
a.d: 3
",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("b"),
            Integer("1"),
            Key("d"),
            Integer("3"),
            EndObject,
            Key("c"),
            Integer("2"),
            EndObject,
        ]
    );
}

#[test]
fn reopen_explicit_object_merges() {
    check!(
        "a: {\n    x: 1\n}\na.y: 2\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("2"),
            EndObject,
            EndObject,
        ]
    );
}

#[test]
fn grouped_dotted_keys_unchanged() {
    check!(
        "a.b: 1\na.c: 2\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("b"),
            Integer("1"),
            Key("c"),
            Integer("2"),
            EndObject,
            EndObject,
        ]
    );
}

#[test]
fn merge_is_identity_on_reopen_free_stream() {
    let text = "port: 8080\nhost: \"example.com\"\nlist: [1, 2, 3]\nnested: {\nx: {y: 1}\n}\n";
    let bump = Bump::new();
    let (stream, reopens) = parse_events(text, &bump).unwrap();
    assert_eq!(reopens, 0);
    assert_eq!(stream, merge_reopened(&stream, &bump));
}

#[test]
fn nested_reopen_merges_innermost() {
    check!(
        "a.b.c: 1\nq: 2\na.b.d: 2\n",
        vec![
            BeginObject,
            Key("a"),
            BeginObject,
            Key("b"),
            BeginObject,
            Key("c"),
            Integer("1"),
            Key("d"),
            Integer("2"),
            EndObject,
            EndObject,
            Key("q"),
            Integer("2"),
            EndObject,
        ]
    );
}

#[test]
fn reopen_inside_explicit_frame_merges() {
    check!(
        "o: {\na.x: 1\np: 2\na.y: 3\n}\n",
        vec![
            BeginObject,
            Key("o"),
            BeginObject,
            Key("a"),
            BeginObject,
            Key("x"),
            Integer("1"),
            Key("y"),
            Integer("3"),
            EndObject,
            Key("p"),
            Integer("2"),
            EndObject,
            EndObject,
        ]
    );
}

#[test]
fn arrays_untouched() {
    let text = "a: [1, 2]\nb: 3\n";
    let bump = Bump::new();
    assert_eq!(merged(text, &bump), raw(text, &bump));
}

#[test]
fn inline_roots_pass_through() {
    for text in ["{a: 1}\n", "[1]\n"] {
        let bump = Bump::new();
        assert_eq!(merged(text, &bump), raw(text, &bump));
    }
}
