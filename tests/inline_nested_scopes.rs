//! Regression tests for review round-4 findings R4-F1 and R4-F2 —
//! spec 0.7 § 5.8.2 / § 5.8.4 / § 5.8.5 / § 5.3.3.
//!
//! R4-F1 (per-scope raw openers): a `::` raw scalar nested inside an inner
//! compound makes only LEADING OPENERS literal (§ 5.8.5); R5-F3 corrects the
//! closer side: the scalar terminates on the FIRST unescaped `,`, `}`, or
//! `]`, regardless of which scope the closer byte belongs to.
//!
//! R4-F2 (scope restoration after nested closers): closing a nested
//! Array/Object must restore the enclosing Object's key-context.
//!
//! Most cases fail until the fix lands; "controls" and "error pins"
//! must pass before AND after the fix.

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

// --- Group A: R4-F1, nested raw scalars ---------------------------------------

#[test]
fn nested_raw_object_in_array() {
    // 1. primary repro
    let src = r"[{a:: x}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("a", Value::String("x".into()))])]),
        "raw-scalar nested repro failed for {src:?}"
    );

    // 3. quoted key forces the quote-aware slow path
    let src = r#"[{"a":: x}]
"#;
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("a", Value::String("x".into()))])]),
        "quoted-key raw-scalar nested repro failed for {src:?}"
    );

    // 5. sibling item after the raw-containing compound closes
    let src = r"[{a:: x}, 2]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[
            obj(&[("a", Value::String("x".into()))]),
            Value::Integer("2".into()),
        ]),
        "sibling item after nested raw compound failed for {src:?}"
    );
}

#[test]
fn nested_double_colon_scalar_array_in_object() {
    // 2. The array item `:: x` is NOT a raw marker — an item position
    //    derives only `<inline-value>` (§ 4, <inline-item-list>), so the
    //    item is the <inline-scalar> `:: x` → String (§ 5.2). The raw
    //    `::` branch is scoped to inline-PAIR positions (§ 4 notes;
    //    § 5.8.5: "The raw `::` branch of an inline pair is not an
    //    inline value").
    let src = r"{a: [:: x]}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", arr(&[Value::String(":: x".into())]))]),
        "double-colon scalar in nested array failed for {src:?}"
    );

    // 4. quoted key forces the slow path
    let src = r#"{"a": [:: x]}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", arr(&[Value::String(":: x".into())]))]),
        "quoted-key double-colon scalar in nested array failed for {src:?}"
    );

    // 6. sibling pair after the compound closes
    let src = r"{a: [:: x], b: 2}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::String(":: x".into())])),
            ("b", Value::Integer("2".into())),
        ]),
        "sibling pair after nested compound failed for {src:?}"
    );
}

#[test]
fn raw_content_openers_stay_literal() {
    // 8. a `[` inside a raw value must NOT start nesting (§ 5.8.5). The
    //    scalar `x[` is terminated by the object's OWN closer `}` — the
    //    first unescaped closer it meets is a matching one, so the R5-F3
    //    fix changes nothing here and this stays a positive test.
    let src = r"[{a:: x[}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("a", Value::String("x[".into()))])]),
        "literal opener in nested raw value failed for {src:?}"
    );

    // 33. escaped-`[` variant: `\[` decodes to a literal `[` in the
    //     scalar; the unescaped `}` still closes the object.
    let src = r"[{a:: x\[}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("a", Value::String("x[".into()))])]),
        "escaped opener in nested raw value failed for {src:?}"
    );
}

#[test]
fn raw_terminated_by_any_unescaped_closer() {
    // R5-F3: `<inline-raw-scalar>` terminates on the FIRST unescaped
    // `,`, `}`, or `]`, regardless of which scope the closer belongs to.
    // A crossed closer that returns depth to zero without being the
    // body's own closer leaves the compound unterminated. (The pre-fix
    // code accepted all of these with the closer smuggled into the
    // scalar.)

    // 34. primary repro — `]` terminates the scalar but does not match
    //     the object's `}` (fast path).
    let src = r"{a:: x]}
";
    match parse(src) {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!("case 34: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 35. quoted key forces the slow path — same rejection.
    let src = r#"{"a":: x]}
"#;
    match parse(src) {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!("case 35: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 7. (was: accepted as `x{y]`) a literal `{` stays content, but the
    //    unescaped `]` still terminates the scalar; the crossed `]`
    //    returns depth to zero without being the body's `}`.
    let src = r"{a:: x{y]}
";
    match parse(src) {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!("case 7: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 9. (was: accepted as `x]`) a crossed `]` inside a nested raw
    //    scalar leaves the object without its matching closer.
    let src = r"[{a:: x]}]
";
    match parse(src) {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!("case 9: expected UnterminatedInlineCompound, got {other:?}"),
    }
}

#[test]
fn raw_escaped_closer_stays_content() {
    // Controls: escaping the closer keeps it in the scalar (§ 3.7);
    // these must pass before AND after the fix.

    // 36. the R5-F3 control.
    let src = r"{a:: x\]}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("x]".into()))]),
        "escaped closer control failed for {src:?}"
    );

    // 37. slow-path (quoted key) variant.
    let src = r#"{"a":: x\]}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("x]".into()))]),
        "slow-path escaped closer control failed for {src:?}"
    );

    // 38. escaped variant of old case 7: `\]` keeps the closer in the
    //     scalar, the literal `{` stays content, `}` closes the object.
    let src = r"{a:: x{y\]}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("a", Value::String("x{y]".into()))]),
        "escaped closer after literal opener failed for {src:?}"
    );

    // 39. escaped variant of old case 9: `\]` inside the nested scalar;
    //     the object's `}` and the array's `]` close normally.
    let src = r"[{a:: x\]}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("a", Value::String("x]".into()))])]),
        "escaped crossed closer failed for {src:?}"
    );
}

#[test]
fn raw_terminated_by_comma_mid_nesting() {
    // 40. an unescaped `,` terminates a raw scalar inside a nested
    //     compound: `x` ends at the comma and `b` is a sibling pair.
    let src = r"[{a:: x, b: 2}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[
            ("a", Value::String("x".into())),
            ("b", Value::Integer("2".into())),
        ])]),
        "comma-terminated raw scalar mid-nesting failed for {src:?}"
    );
}

#[test]
fn raw_same_kind_nesting_and_deeper() {
    // 10. same-kind nesting: the inner item `:: x` is a plain
    //     <inline-scalar> → String (§ 4: an item derives only
    //     <inline-value>; a first byte of `:` is "any other first byte")
    let src = r"[[:: x]]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[arr(&[Value::String(":: x".into())])]),
        "same-kind double-colon nesting failed for {src:?}"
    );

    // 11. deeper nesting
    let src = r"{k: [{a:: x}]}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[("k", arr(&[obj(&[("a", Value::String("x".into()))])]))]),
        "deeper raw nesting failed for {src:?}"
    );
}

// --- Group B: R4-F2, scope restoration ----------------------------------------

#[test]
fn scope_restore_after_array_close() {
    // 12. primary repro
    let src = r#"{a: [1], "x}": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("x}", Value::Integer("2".into())),
        ]),
        "scope restore after array close failed for {src:?}"
    );

    // 14. single-quote key delimiter
    let src = r"{a: [1], 'x}': 2}
";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("x}", Value::Integer("2".into())),
        ]),
        "scope restore, single-quote key failed for {src:?}"
    );

    // 15. backtick key delimiter
    let src = "{a: [1], `x}`: 2}\n";
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("x}", Value::Integer("2".into())),
        ]),
        "scope restore, backtick key failed for {src:?}"
    );

    // 16.
    let src = r#"{a: [1], "x[": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("x[", Value::Integer("2".into())),
        ]),
        "scope restore, bracket-containing key failed for {src:?}"
    );

    // 17.
    let src = r#"{a: [1], "x]": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("x]", Value::Integer("2".into())),
        ]),
        "scope restore, close-bracket key failed for {src:?}"
    );
}

#[test]
fn scope_restore_mixed_nesting() {
    // 18. Array-then-Object nesting
    let src = r#"{a: [1], b: {c: 2}, "x]": 3}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", arr(&[Value::Integer("1".into())])),
            ("b", obj(&[("c", Value::Integer("2".into()))])),
            ("x]", Value::Integer("3".into())),
        ]),
        "mixed Array-then-Object restore failed for {src:?}"
    );

    // 19. Object-then-Array nesting
    let src = r#"{a: {b: 1}, c: [2], "x}": 3}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", obj(&[("b", Value::Integer("1".into()))])),
            ("c", arr(&[Value::Integer("2".into())])),
            ("x}", Value::Integer("3".into())),
        ]),
        "mixed Object-then-Array restore failed for {src:?}"
    );

    // 20. quoted key in the SECOND element
    let src = r#"[{a: 1}, {"b}": 2}]
"#;
    assert_eq!(
        parse(src).unwrap(),
        arr(&[
            obj(&[("a", Value::Integer("1".into()))]),
            obj(&[("b}", Value::Integer("2".into()))]),
        ]),
        "quoted key in second array element failed for {src:?}"
    );

    // 21. deepest stack-restore case
    let src = r#"{a: [1, {b: 2}], "x}": 3}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            (
                "a",
                arr(&[
                    Value::Integer("1".into()),
                    obj(&[("b", Value::Integer("2".into()))]),
                ])
            ),
            ("x}", Value::Integer("3".into())),
        ]),
        "deepest stack restore failed for {src:?}"
    );
}

// --- Group C: controls (must pass today AND after the fix) ---------------------

#[test]
fn controls_unchanged() {
    // 13. R4-F2 control
    let src = r#"{a: {b: 1}, "x}": 2}
"#;
    assert_eq!(
        parse(src).unwrap(),
        obj(&[
            ("a", obj(&[("b", Value::Integer("1".into()))])),
            ("x}", Value::Integer("2".into())),
        ]),
        "object-nesting control failed for {src:?}"
    );

    // 22.
    let src = r"[{b: 1}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[obj(&[("b", Value::Integer("1".into()))])]),
        "control 22 failed for {src:?}"
    );

    // 23.
    let src = r"[1, [2], {a: 3}]
";
    assert_eq!(
        parse(src).unwrap(),
        arr(&[
            Value::Integer("1".into()),
            arr(&[Value::Integer("2".into())]),
            obj(&[("a", Value::Integer("3".into()))]),
        ]),
        "control 23 failed for {src:?}"
    );
}

// --- Group D: error-category pins (must pass today) ----------------------------

#[test]
fn error_pins_unchanged() {
    // 24. `}` closes the object, the array never closes
    let err = parse("[{a:: x}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 24: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 25.
    let err = parse("{a: [:: x\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 25: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 26. crossed closer: `}` does not close the `[` scope
    let err = parse("{a: [1}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 26: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 27. crossed: final `}` returns depth to zero but is not the body's `]`
    let err = parse("[{a: 1]}\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 27: expected UnterminatedInlineCompound, got {other:?}"),
    }
}

// --- Group E: thin API event streams -------------------------------------------

#[test]
fn thin_api() {
    // 28.
    let src = r"[{a:: x}]
";
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginArray,
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("x".into()),
            Ev::EndObject,
            Ev::EndArray,
        ],
        "thin events for raw nested repro failed for {src:?}"
    );

    // 29.
    let src = r#"{a: [1], "x}": 2}
"#;
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::EndArray,
            Ev::Key("x}".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
        ],
        "thin events for scope-restore repro failed for {src:?}"
    );

    // 30.
    let src = r#"{a: [1, {b: 2}], "x}": 3}
"#;
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::BeginArray,
            Ev::Integer("1".into()),
            Ev::BeginObject,
            Ev::Key("b".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndArray,
            Ev::Key("x}".into()),
            Ev::Integer("3".into()),
            Ev::EndObject,
        ],
        "thin events for deepest restore failed for {src:?}"
    );
}

#[test]
fn thin_api_raw_closer_termination() {
    // 41. escaped closer decodes inside the event stream.
    let src = r"{a:: x\]}
";
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("x]".into()),
            Ev::EndObject,
        ],
        "thin events for escaped-closer control failed for {src:?}"
    );

    // 42. unescaped crossed closer → UnterminatedInlineCompound.
    let src = r"{a:: x]}
";
    let err = parse_events(src, |_| {}).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 42: expected UnterminatedInlineCompound, got {other:?}"),
    }

    // 43. comma terminates the raw scalar mid-nesting (event view).
    let src = r"[{a:: x, b: 2}]
";
    assert_eq!(
        collect(src),
        vec![
            Ev::BeginArray,
            Ev::BeginObject,
            Ev::Key("a".into()),
            Ev::Str("x".into()),
            Ev::Key("b".into()),
            Ev::Integer("2".into()),
            Ev::EndObject,
            Ev::EndArray,
        ],
        "thin events for comma-terminated raw failed for {src:?}"
    );
}

// --- Group F: typed API ---------------------------------------------------------

#[test]
fn typed_api() {
    use serde_json::json;

    // 31.
    let src = r"[{a:: x}]
";
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, json!([{"a": "x"}]), "typed raw nested repro failed");

    // 32.
    let src = r#"{a: [1], "x}": 2}
"#;
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(
        v,
        json!({"a": [1], "x}": 2}),
        "typed scope-restore repro failed for {src:?}"
    );
}

#[test]
fn typed_api_raw_closer_termination() {
    use serde_json::json;

    // 44. escaped-closer control via the typed API.
    let src = r"{a:: x\]}
";
    let v: serde_json::Value = from_str(src).unwrap();
    assert_eq!(v, json!({"a": "x]"}), "typed escaped-closer control failed");

    // 45. rejection via the typed API.
    let src = r"{a:: x]}
";
    let err = from_str::<serde_json::Value>(src).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }) => {}
        other => panic!("case 45: expected UnterminatedInlineCompound, got {other:?}"),
    }
}
