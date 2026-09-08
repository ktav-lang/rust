//! Conformance tests for review finding R7-F1 — spec § 5.8.5, § 5.3.3,
//! § 5.4, § 5.2 rules 6–9, § 6.11–§ 6.13.
//!
//! Every boundary consumer of an inline compound must give a byte the
//! SAME meaning: a `{`/`[` that is not the first non-whitespace code
//! point of an inline value is literal content (§ 5.8.5 — the value
//! already began as a scalar); a `::` raw scalar consumes to the first
//! unescaped `,` / `}` / `]` with a leading `{`/`[` literal (§ 5.4,
//! § 5.8.5); a quoted key segment (`"`, `'`, `` ` `` — § 5.3.3) is
//! opaque to `,`/`}`/`]` counting; and the depth-0 closer must be the
//! body's own kind (§ 5.2 rules 6–9). Before R7-F1 the nested-path
//! closer search (`find_matching_close`) kept naive opener semantics,
//! pushed a phantom scope for a literal mid-scalar `[`, and rejected
//! valid documents with `MalformedInlineCompound`.
//!
//! The oracles below are derived from the spec text and assert
//! concrete `Value`s / JSON. Owned-vs-thin parity proves nothing here
//! (both APIs shared the defect), so parity is not the oracle.

use ktav::error::ErrorKind;
use ktav::thin::{parse_events, ParseEvent};
use ktav::value::ObjectMap;
use ktav::{from_str, parse, parse_strict, Error, Value};

fn s(v: &str) -> Value {
    Value::String(v.into())
}

fn i(v: i64) -> Value {
    Value::Integer(v.to_string().into())
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

fn arr(items: Vec<Value>) -> Value {
    Value::Array(items)
}

fn expect_ok(src: &str, want: Value) {
    match parse(src) {
        Ok(v) => assert_eq!(v, want, "parse() on {src:?}"),
        Err(e) => panic!("parse({src:?}) expected Ok, got Err {e:?}"),
    }
    match parse_strict(src) {
        Ok(v) => assert_eq!(v, want, "parse_strict() on {src:?}"),
        Err(e) => panic!("parse_strict({src:?}) expected Ok, got Err {e:?}"),
    }
}

fn expect_json(src: &str, want: serde_json::Value) {
    match from_str::<serde_json::Value>(src) {
        Ok(v) => assert_eq!(v, want, "from_str on {src:?}"),
        Err(e) => panic!("from_str({src:?}) expected {want}, got Err {e:?}"),
    }
}

#[derive(Debug)]
enum Want {
    /// § 6.11 UnterminatedInlineCompound
    Unterminated,
    /// § 6.12 MalformedInlineCompound
    Malformed,
    /// § 6.13 BadEscapeSequence with the exact offending sequence
    BadEscape(&'static str),
}

fn expect_err(src: &str, want: &Want) {
    let results = [
        ("parse", parse(src).err()),
        ("from_str", from_str::<serde_json::Value>(src).err()),
    ];
    for (api, res) in results {
        let err = res.unwrap_or_else(|| panic!("{api}({src:?}) expected Err, got Ok"));
        match (&err, want) {
            (
                Error::Structured(ErrorKind::UnterminatedInlineCompound { .. }),
                Want::Unterminated,
            ) => {}
            (Error::Structured(ErrorKind::MalformedInlineCompound { .. }), Want::Malformed) => {}
            (
                Error::Structured(ErrorKind::BadEscapeSequence { sequence, .. }),
                Want::BadEscape(seq),
            ) => assert_eq!(sequence, *seq, "{api}({src:?}) bad-escape payload"),
            _ => panic!("{api}({src:?}) expected {want:?}, got {err:?}"),
        }
    }
}

// --- R7-F1 repro family (§ 5.8.5 mid-scalar opener + § 5.3.3 quoted key) ---

#[test]
fn repro_root_array_wrapped() {
    // The value of `a` begins as the scalar `x`, so the `[` is literal
    // content and the comma after it terminates the scalar (§ 5.8.5).
    // The `}` inside the quoted key `"b}c"` is key content (§ 5.3.3).
    expect_ok(
        "[{a: x[, \"b}c\": 2}]",
        arr(vec![obj(&[("a", s("x[")), ("b}c", i(2))])]),
    );
    expect_json(
        "[{a: x[, \"b}c\": 2}]",
        serde_json::json!([{"a": "x[", "b}c": 2}]),
    );
}

#[test]
fn repro_nested_object_wrapped() {
    expect_ok(
        "{outer: {a: x[, \"b}c\": 2}}",
        obj(&[("outer", obj(&[("a", s("x[")), ("b}c", i(2))]))]),
    );
}

#[test]
fn repro_control_root_still_ok() {
    // The unwrapped control parsed correctly even before the fix.
    expect_ok("{a: x[, \"b}c\": 2}", obj(&[("a", s("x[")), ("b}c", i(2))]));
}

#[test]
fn repro_raw_scalar_variant() {
    // § 5.4 / § 5.8.5: after `::` the raw scalar consumes to the first
    // unescaped `,`; a leading `[` is literal. The quoted key still
    // starts a fresh pair after that comma.
    expect_ok(
        "[{a:: [x, \"b}c\": 2}]",
        arr(vec![obj(&[("a", s("[x")), ("b}c", i(2))])]),
    );
}

#[test]
fn raw_scalar_terminated_by_unescaped_closer() {
    // § 5.4 / § 5.8.5: a raw scalar is the § 4 inline-scalar production
    // — it consumes to the first UNESCAPED `,` / `}` / `]`. The `]`
    // after `x[1` therefore terminates the raw value AND is structural
    // (crossed, inside the inner object), so the inner body never
    // balances and the outer compound closes mid-body → § 6.12.
    expect_err("{o: {a:: x[1], c: 2}}", &Want::Malformed);
}

#[test]
fn raw_scalar_escaped_closers_are_literal() {
    // Same shape, but the closers are escaped (§ 3.7): the raw scalar
    // now consumes to the comma and the brackets stay literal content.
    expect_ok(
        "{o: {a:: x\\[1\\], c: 2}}",
        obj(&[("o", obj(&[("a", s("x[1]")), ("c", i(2))]))]),
    );
}

// --- all three quoted-key delimiters (§ 5.3.3) ------------------------------

#[test]
fn delimiters_dq_sq_backtick() {
    let want = arr(vec![obj(&[("a", s("x[")), ("b}c", i(2))])]);
    expect_ok("[{a: x[, \"b}c\": 2}]", want.clone());
    expect_ok("[{a: x[, 'b}c': 2}]", want.clone());
    expect_ok("[{a: x[, `b}c`: 2}]", want);
}

// --- literal `{` mid-scalar followed by a quoted key (§ 5.8.5) ---------------

#[test]
fn literal_brace_mid_scalar_quoted_key() {
    expect_ok(
        "[{a: x{, \"b}c\": 2}]",
        arr(vec![obj(&[("a", s("x{")), ("b}c", i(2))])]),
    );
}

#[test]
fn spec_5_8_5_example_nested() {
    // The § 5.8.5 example, additionally wrapped in an array so the
    // nested find path decides the item's boundary.
    expect_ok(
        "[{a: hello{world, b: x}]",
        arr(vec![obj(&[("a", s("hello{world")), ("b", s("x"))])]),
    );
}

// --- quoted-key opacity with a comma inside the key (§ 5.3.3) ----------------

#[test]
fn comma_inside_quoted_key_after_mid_scalar() {
    expect_ok(
        "[{a: x[, \"b,c\": 2, 'd}e': 3}]",
        arr(vec![obj(&[("a", s("x[")), ("b,c", i(2)), ("d}e", i(3))])]),
    );
}

#[test]
fn multi_item_array_with_quoted_keys() {
    expect_ok(
        "[{a: x[, \"b}c\": 2}, {d: 3}]",
        arr(vec![
            obj(&[("a", s("x[")), ("b}c", i(2))]),
            obj(&[("d", i(3))]),
        ]),
    );
}

// --- § 6.11/§ 6.12 tri-state through the nested path -------------------------

#[test]
fn crossed_closer_at_outer_level_is_unterminated() {
    // `[{a: x{y}}]`: the depth-0 closer of the inner object is
    // `}`@7; the following `}` then has no matching open scope at the
    // array's own level — a crossed closer is never a matching `]`
    // (§ 5.2 rules 6–9) → the array is unterminated → § 6.11.
    expect_err("[{a: x{y}}]", &Want::Unterminated);
}

#[test]
fn found_mid_body_is_malformed() {
    // § 6.12 (§ 5.2 rule 8): the matching closer sits mid-body and
    // non-whitespace content follows it. Decided by the outermost
    // scan — with find and scan sharing identical byte semantics the
    // outermost scan is always the gatekeeper, so the nested
    // find-first dispatch inherits this verdict unchanged.
    expect_err("{a: x{y}, z: 1}", &Want::Malformed);
    expect_err("[{a: x{y}}, 2]", &Want::Unterminated);
}

#[test]
fn comma_after_midvalue_closer_separates_array_items() {
    // In an ARRAY body the depth-0 `}` ends the first item (`{a: x{y}`),
    // so the comma after it is a real item separator. The second item
    // `z` then runs into the crossed `}` — the wrapping array never
    // receives its own `]` → § 6.11, not § 6.12.
    expect_err("[{a: x{y}, z}]", &Want::Unterminated);
}

#[test]
fn tri_state_not_found_is_unterminated() {
    // The object closes but the wrapping array never does → § 6.11.
    expect_err("[{a: x[, \"b}c\": 2}", &Want::Unterminated);
}

// --- crossed closer through the nested path (§ 5.2 rules 6–9) ----------------

#[test]
fn crossed_closer_nested_rejected() {
    // The `]` inside the object body is structural and crossed: the
    // depth-0 closer is not the body's own kind → no matching closer
    // → § 6.11. (Before R7-F1 the naive find accepted `1]b` as the
    // value scalar.)
    expect_err("[{a: 1]b}]", &Want::Unterminated);
    expect_err("{a: 1]b}", &Want::Unterminated);
}

// --- BadEscapeSequence precedence (§ 6.13 / rules 6–9 preamble) --------------

#[test]
fn bad_escape_precedence_via_nested_find() {
    expect_err("[{a: \\q, \"b}c\": 2}]", &Want::BadEscape("\\q"));
    // Bad escape AND crossed closer: the escape is reported first.
    expect_err("[{a: \\q]x}]", &Want::BadEscape("\\q"));
}

// --- round-6 repro stays green (regression pin) ------------------------------

#[test]
fn round6_repro_still_ok() {
    // Item 2 of the inner array is the scalar `"x` (quotes are content
    // in array value positions, § 5.3.3 "Keys only"); the quoted key
    // `"b}c"` then starts a fresh pair.
    expect_ok(
        "[{a: [1, \"x], \"b}c\": 2}]",
        arr(vec![obj(&[
            ("a", arr(vec![i(1), s("\"x")])),
            ("b}c", i(2)),
        ])]),
    );
}

// --- thin direct-scan path (same boundary consumer, end-product oracle) ------

#[test]
fn thin_path_repro_events() {
    // `ParseEvent<'_>` borrows the input, so events must be collected
    // into an owned mirror (the same pattern as tests/thin_public.rs).
    #[derive(Debug, PartialEq, Eq)]
    enum Owned {
        BeginArray,
        BeginObject,
        Key(&'static str),
        Str(&'static str),
        Integer(&'static str),
        EndObject,
        EndArray,
    }
    fn collect(input: &str) -> Vec<Owned> {
        let mut evs = Vec::new();
        parse_events(input, |e| {
            evs.push(match e {
                ParseEvent::BeginArray => Owned::BeginArray,
                ParseEvent::BeginObject => Owned::BeginObject,
                ParseEvent::Key(k) => Owned::Key(Box::leak(k.to_string().into_boxed_str())),
                ParseEvent::Str(v) => Owned::Str(Box::leak(v.to_string().into_boxed_str())),
                ParseEvent::Integer(v) => Owned::Integer(Box::leak(v.to_string().into_boxed_str())),
                ParseEvent::EndObject => Owned::EndObject,
                ParseEvent::EndArray => Owned::EndArray,
                other => panic!("thin path unexpected event: {other:?}"),
            })
        })
        .unwrap_or_else(|e| panic!("thin path errored: {e:?}"));
        evs
    }
    let evs = collect("[{a: x[, \"b}c\": 2}]");
    assert_eq!(
        evs,
        vec![
            Owned::BeginArray,
            Owned::BeginObject,
            Owned::Key("a"),
            Owned::Str("x["),
            Owned::Key("b}c"),
            Owned::Integer("2"),
            Owned::EndObject,
            Owned::EndArray,
        ]
    );
}
