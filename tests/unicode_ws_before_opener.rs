//! Regression tests for review findings R4-F3 and R5-F5 — spec 0.7
//! § 3.3 / § 5.8.1 / § 5.8.5: a `{`/`[` opens a nested inline compound
//! iff it is the FIRST NON-WHITESPACE code point of the inline value,
//! where § 3.3 whitespace is the closed 25-code-point Unicode
//! White_Space set (23 members are reachable in inline scanning; LF/CR
//! cannot occur — lines are pre-split). Every member before a nested
//! opener must not consume the value-start position; adjacent
//! non-members must clear it like any ordinary content.

use std::collections::BTreeMap;

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::{from_str, parse, Error, ObjectMap, Value};

#[derive(Debug, PartialEq)]
enum Ev {
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

fn collect(src: &str) -> Vec<Ev> {
    let mut out = Vec::new();
    parse_events(src, |e| {
        out.push(match e {
            ParseEvent::Null => Ev::Null,
            ParseEvent::Bool(b) => Ev::Bool(b),
            ParseEvent::Integer(s) => Ev::Integer(s.to_string()),
            ParseEvent::Float(s) => Ev::Float(s.to_string()),
            ParseEvent::Str(s) => Ev::Str(s.to_string()),
            ParseEvent::Key(s) => Ev::Key(s.to_string()),
            ParseEvent::BeginObject => Ev::BeginObject,
            ParseEvent::EndObject => Ev::EndObject,
            ParseEvent::BeginArray => Ev::BeginArray,
            ParseEvent::EndArray => Ev::EndArray,
            _ => unreachable!(),
        });
    })
    .unwrap();
    out
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

// --- 1. whole § 3.3 inline set, both nesting directions ----------------------

#[test]
fn whole_whitespace_set_before_nested_opener() {
    // All 23 inline-scanning § 3.3 members (the 25-code-point list
    // minus LF U+000A and CR U+000D, which never reach inline
    // scanning), enumerated individually.
    let members = [
        "\t", "\u{0B}", "\u{0C}", " ", "\u{85}", "\u{A0}", "\u{1680}", "\u{2000}", "\u{2001}",
        "\u{2002}", "\u{2003}", "\u{2004}", "\u{2005}", "\u{2006}", "\u{2007}", "\u{2008}",
        "\u{2009}", "\u{200A}", "\u{2028}", "\u{2029}", "\u{202F}", "\u{205F}", "\u{3000}",
    ];
    for ws in members {
        let src = format!("{{a:{ws}[1]}}\n");
        assert_eq!(
            parse(&src).unwrap(),
            obj(&[("a", arr(&[Value::Integer("1".into())]))]),
            "object->array failed for ws {ws:?} in {src:?}"
        );
        let src = format!("[{ws}{{a: 1}}]\n");
        assert_eq!(
            parse(&src).unwrap(),
            arr(&[obj(&[("a", Value::Integer("1".into()))])]),
            "array->object failed for ws {ws:?} in {src:?}"
        );
    }
}

// --- 1b. adjacent non-members are ordinary literal content -------------------

#[test]
fn adjacent_nonmembers_do_not_open_nested() {
    // U+1FFF (just below U+2000), U+2010 (just above U+200A) and
    // U+FEFF (BOM; § 3.1 strips it only at document start) are NOT
    // § 3.3 members, so each must clear `value_start` like any ordinary
    // content: a `{`/`[` after one of them (no comma to re-arm it) is a
    // mid-scalar literal, not a nested opener.
    for ch in ["\u{1FFF}", "\u{2010}", "\u{FEFF}"] {
        let err = parse(&format!("{{a: {ch}[1]}}\n")).unwrap_err();
        match &err {
            Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
            other => panic!("expected UnterminatedInlineCompound for {ch:?}, got {other:?}"),
        }
        let err = parse(&format!("[{ch}{{a: 1}}]\n")).unwrap_err();
        match &err {
            Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
            other => panic!("expected UnterminatedInlineCompound for {ch:?}, got {other:?}"),
        }
    }
}

// --- 2. after-comma cases (value_start re-armed by the comma arm) ------------

#[test]
fn whitespace_after_comma_still_arms_value_start() {
    assert_eq!(
        parse("{a: 1, b:\u{a0}{c: 2}}\n").unwrap(),
        obj(&[
            ("a", Value::Integer("1".into())),
            ("b", obj(&[("c", Value::Integer("2".into()))])),
        ])
    );
    assert_eq!(
        parse("[1,\u{a0}[2]]\n").unwrap(),
        arr(&[
            Value::Integer("1".into()),
            arr(&[Value::Integer("2".into())])
        ])
    );
}

// --- 3. nested at depth ------------------------------------------------------

#[test]
fn whitespace_before_opener_at_depth() {
    assert_eq!(
        parse("{a: {b:\u{a0}[1]}}\n").unwrap(),
        obj(&[("a", obj(&[("b", arr(&[Value::Integer("1".into())]))]))])
    );
}

// --- 4. quoted keys (slow quote-tracking paths) ------------------------------

#[test]
fn quoted_key_paths() {
    assert_eq!(
        parse("{\"a\":\u{a0}[1]}\n").unwrap(),
        obj(&[("a", arr(&[Value::Integer("1".into())]))])
    );
    assert_eq!(
        parse("[\u{a0}{\"a\": 1}]\n").unwrap(),
        arr(&[obj(&[("a", Value::Integer("1".into()))])])
    );
    assert_eq!(
        parse("{a: 1, \"b\":\u{a0}{c: 2}}\n").unwrap(),
        obj(&[
            ("a", Value::Integer("1".into())),
            ("b", obj(&[("c", Value::Integer("2".into()))])),
        ])
    );
}

// --- 5. thin API event sequences ---------------------------------------------

#[test]
fn thin_events_for_repros() {
    assert_eq!(
        collect("{a:\u{a0}[1]}\n"),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::EndArray,
            Ev::EndObject
        ]
    );
    assert_eq!(
        collect("[\u{a0}{a: 1}]\n"),
        vec![
            Ev::BeginArray,
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Integer("1".into()),
            Ev::EndObject,
            Ev::EndArray
        ]
    );
    assert_eq!(
        collect("{\"a\":\u{a0}[1]}\n"),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::EndArray,
            Ev::EndObject
        ]
    );
}

// --- 6. typed API -------------------------------------------------------------

#[test]
fn typed_api() {
    let m: BTreeMap<String, Vec<i32>> = from_str("{a:\u{a0}[1]}\n").unwrap();
    assert_eq!(m, BTreeMap::from([("a".to_string(), vec![1])]));

    let v: Vec<BTreeMap<String, i32>> = from_str("[\u{a0}{a: 1}]\n").unwrap();
    assert_eq!(v, vec![BTreeMap::from([("a".to_string(), 1)])]);
}

// --- 7. behavior-preservation pins --------------------------------------------

#[test]
fn pins_unchanged() {
    // Whitespace between the two colons is NOT a `::` raw marker.
    assert_eq!(
        parse("{a:\u{a0}: 1}\n").unwrap(),
        obj(&[("a", Value::String(": 1".into()))])
    );
    // Raw marker still makes the LEADING bracket literal (§ 5.8.5), but
    // the scalar's terminator grammar (R5-F3) still applies: the
    // unescaped `]` terminates the scalar and crosses the object scope,
    // so the document is rejected.
    let err = parse("{a::\u{a0}[1]}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
    // Same literal-opener intent in valid form: an escaped `]` stays in
    // the scalar; the unescaped `}` closes the object.
    assert_eq!(
        parse("{a::\u{a0}[\\]1}\n").unwrap(),
        obj(&[("a", Value::String("[]1".into()))])
    );
    // Interior NBSP is content.
    assert_eq!(
        parse("{a: x\u{a0}y}\n").unwrap(),
        obj(&[("a", Value::String("x\u{a0}y".into()))])
    );
    // Edge NBSP still trimmed.
    assert_eq!(
        parse("{a:\u{a0}x}\n").unwrap(),
        obj(&[("a", Value::String("x".into()))])
    );
    // Whitespace with no opener after it.
    assert_eq!(
        parse("[[\u{a0}1]]\n").unwrap(),
        arr(&[arr(&[Value::Integer("1".into())])])
    );
    // Non-whitespace multi-byte still consumes value-start: é is NOT
    // § 3.3 whitespace, so a `[` after it (no comma to re-arm
    // value_start) is a mid-scalar literal and the scan reports
    // UnterminatedInlineCompound — the same result as `{a: z[1]}`.
    // (Verified identical on HEAD 08be197; guards against over-applying
    // the fix to non-whitespace multi-byte lead bytes like é's 0xC3.)
    let err = parse("{a: é[1]}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
}
