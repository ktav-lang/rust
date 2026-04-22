//! Edge cases for the typed-scalar markers `:i` / `:f`.

use ktav::{parse, Error, Value};

// ---------------------------------------------------------------------------
// Leading-sign normalization
// ---------------------------------------------------------------------------

#[test]
fn leading_plus_on_integer_is_stripped() {
    let v = parse("port:i +5\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("port"),
        Some(&Value::Integer("5".into()))
    );
}

#[test]
fn leading_minus_on_integer_is_preserved() {
    let v = parse("x:i -0\n").unwrap();
    // `-0` passes the integer regex — formally a valid integer literal.
    assert_eq!(
        v.as_object().unwrap().get("x"),
        Some(&Value::Integer("-0".into()))
    );
}

#[test]
fn leading_plus_on_float_mantissa_is_stripped() {
    let v = parse("ratio:f +3.14\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("ratio"),
        Some(&Value::Float("3.14".into()))
    );
}

#[test]
fn leading_minus_on_float_is_preserved() {
    let v = parse("ratio:f -3.14\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("ratio"),
        Some(&Value::Float("-3.14".into()))
    );
}

#[test]
fn exponent_sign_is_preserved_verbatim() {
    let v = parse("ratio:f 3.14E+10\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("ratio"),
        Some(&Value::Float("3.14E+10".into()))
    );
    let v = parse("ratio:f 3.14E-10\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("ratio"),
        Some(&Value::Float("3.14E-10".into()))
    );
    let v = parse("ratio:f 3.14e10\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("ratio"),
        Some(&Value::Float("3.14e10".into()))
    );
}

#[test]
fn large_integer_preserves_every_digit() {
    let v = parse("id:i 99999999999999999999\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("id"),
        Some(&Value::Integer("99999999999999999999".into()))
    );
}

// ---------------------------------------------------------------------------
// Typed markers in arrays
// ---------------------------------------------------------------------------

#[test]
fn typed_integer_in_array() {
    let v = parse("xs: [\n    :i 1\n    :i -2\n    :i +3\n]\n").unwrap();
    let xs = v
        .as_object()
        .unwrap()
        .get("xs")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(xs[0], Value::Integer("1".into()));
    assert_eq!(xs[1], Value::Integer("-2".into()));
    assert_eq!(xs[2], Value::Integer("3".into()));
}

#[test]
fn typed_float_in_array() {
    let v = parse("xs: [\n    :f 0.5\n    :f -1.25\n]\n").unwrap();
    let xs = v
        .as_object()
        .unwrap()
        .get("xs")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(xs[0], Value::Float("0.5".into()));
    assert_eq!(xs[1], Value::Float("-1.25".into()));
}

#[test]
fn mixed_typed_string_keyword_in_object() {
    let src = "\
a:i 42
b:f 3.14
c: hello
d: true
e: null
f:: [literal]
";
    let v = parse(src).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Integer("42".into())));
    assert_eq!(obj.get("b"), Some(&Value::Float("3.14".into())));
    assert_eq!(obj.get("c"), Some(&Value::String("hello".into())));
    assert_eq!(obj.get("d"), Some(&Value::Bool(true)));
    assert_eq!(obj.get("e"), Some(&Value::Null));
    assert_eq!(obj.get("f"), Some(&Value::String("[literal]".into())));
}

// ---------------------------------------------------------------------------
// Invalid typed-scalar bodies — InvalidTypedScalar errors
// ---------------------------------------------------------------------------

fn expect_invalid_typed_scalar(src: &str) -> String {
    let err = parse(src).unwrap_err();
    match err {
        Error::Syntax(m) => {
            assert!(
                m.contains("InvalidTypedScalar"),
                "expected InvalidTypedScalar in error, got: {}",
                m
            );
            m
        }
        _ => panic!("expected Syntax error, got {:?}", err),
    }
}

#[test]
fn integer_marker_with_non_digit_body_is_rejected() {
    expect_invalid_typed_scalar("x:i abc\n");
}

#[test]
fn integer_marker_with_float_body_is_rejected() {
    expect_invalid_typed_scalar("x:i 1.5\n");
}

#[test]
fn integer_marker_with_empty_body_is_rejected() {
    expect_invalid_typed_scalar("x:i \n");
}

#[test]
fn integer_marker_opening_object_is_rejected() {
    expect_invalid_typed_scalar("x:i {\n}\n");
}

#[test]
fn integer_marker_opening_array_is_rejected() {
    expect_invalid_typed_scalar("x:i [\n]\n");
}

#[test]
fn integer_marker_opening_multiline_is_rejected() {
    expect_invalid_typed_scalar("x:i (\n)\n");
}

#[test]
fn float_marker_with_integer_body_is_rejected() {
    // `:f` requires a decimal point in the mantissa — `42` alone is not
    // a valid float literal.
    expect_invalid_typed_scalar("x:f 42\n");
}

#[test]
fn float_marker_with_non_numeric_body_is_rejected() {
    expect_invalid_typed_scalar("x:f abc\n");
}

#[test]
fn float_marker_with_empty_body_is_rejected() {
    expect_invalid_typed_scalar("x:f \n");
}

#[test]
fn float_marker_opening_compound_is_rejected() {
    expect_invalid_typed_scalar("x:f {\n}\n");
}

#[test]
fn float_marker_with_two_dots_is_rejected() {
    expect_invalid_typed_scalar("x:f 1.2.3\n");
}

#[test]
fn float_marker_with_trailing_dot_is_rejected() {
    // `1.` is not allowed — the regex requires at least one digit after.
    expect_invalid_typed_scalar("x:f 1.\n");
}

#[test]
fn float_marker_with_leading_dot_is_rejected() {
    // `.5` is not allowed either — the regex requires at least one digit
    // before.
    expect_invalid_typed_scalar("x:f .5\n");
}

#[test]
fn integer_marker_in_array_invalid_body_is_rejected() {
    expect_invalid_typed_scalar("xs: [\n    :i oops\n]\n");
}

#[test]
fn float_marker_in_array_invalid_body_is_rejected() {
    expect_invalid_typed_scalar("xs: [\n    :f oops\n]\n");
}

// ---------------------------------------------------------------------------
// Marker prefix disambiguation
// ---------------------------------------------------------------------------

#[test]
fn info_colon_is_not_a_typed_integer_marker() {
    // `:info ...` has no whitespace between the `:` and `info` (after the
    // key), so under the mandatory-space rule (spec § 5.3 + § 6.10) this
    // is a MissingSeparatorSpace error. It's neither a typed-integer
    // marker (no `:i` + ws form matches) nor a plain-`:` fallback (the
    // plain `:` separator also requires ws-or-EOL now).
    let err = parse("x:info rest\n").unwrap_err().to_string();
    assert!(
        err.contains("MissingSeparatorSpace"),
        "expected MissingSeparatorSpace, got: {err}"
    );
}

#[test]
fn func_colon_is_not_a_typed_float_marker() {
    // Same reasoning as `info_colon_*` above — `:func` is ambiguous
    // against `:f` + body, the plain `:` also lacks the required ws,
    // so the tightened grammar rejects it with MissingSeparatorSpace.
    let err = parse("x:func rest\n").unwrap_err().to_string();
    assert!(
        err.contains("MissingSeparatorSpace"),
        "expected MissingSeparatorSpace, got: {err}"
    );
}

#[test]
fn raw_marker_still_works_unchanged() {
    let v = parse("x:: [literal]\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("x"),
        Some(&Value::String("[literal]".into()))
    );
}

// ---------------------------------------------------------------------------
// Empty-value forms (EOL satisfies the sep-end rule)
// ---------------------------------------------------------------------------

#[test]
fn plain_separator_followed_by_eol_is_empty_string() {
    // `name:` — no body, line ends right after the separator. The EOL
    // branch of <sep-end> matches, so this is an empty String (not a
    // MissingSeparatorSpace error).
    let v = parse("name:\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("name"),
        Some(&Value::String("".into()))
    );
}

#[test]
fn plain_separator_no_newline_at_eof_is_empty_string() {
    // Same as above but the line has no trailing `\n`. The final line
    // of a document does not need a line separator (§ 3.2).
    let v = parse("name:").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("name"),
        Some(&Value::String("".into()))
    );
}

#[test]
fn raw_separator_followed_by_eol_is_empty_string() {
    // Same as plain but through `::`.
    let v = parse("name::\n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("name"),
        Some(&Value::String("".into()))
    );
}

#[test]
fn plain_separator_followed_by_space_then_eol_is_empty_string() {
    // `name: ` — one space then EOL. The 1*ws branch of <sep-end>
    // matches; the body trims to empty.
    let v = parse("name: \n").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("name"),
        Some(&Value::String("".into()))
    );
}
