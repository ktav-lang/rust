//! Ktav spec 0.7 § 5.3.3 (quoted key segments) and § 6.16 (0.7 change
//! notes) for the owned/tree parser (`ktav::parse`).
//!
//! A key segment that *begins* with `"`, `'`, or `` ` `` is a quoted
//! segment: it holds one flat, verbatim key part (delimiters, escapes
//! and whitespace inside are content), dots/colons/brackets inside do
//! not split, and the segment must be closed on the same line. Quotes
//! that do not *lead* a segment stay ordinary bare-key characters, and
//! quotes in value positions are never key syntax at all.

use ktav::{parse, Error, ErrorKind, Value};

fn expect_key(src: &str, expected: &str) {
    let value = parse(src).unwrap();
    let Value::Object(obj) = value else {
        panic!("{src:?}: expected Object root, got {value:?}");
    };
    assert_eq!(obj.len(), 1, "{src:?}: expected exactly one key");
    let got = obj.keys().next().unwrap().as_str();
    assert_eq!(got, expected, "{src:?}: key mismatch");
}

fn expect_err_kind(src: &str, matcher: impl Fn(&ErrorKind) -> bool, what: &str) {
    match parse(src) {
        Ok(v) => panic!("{src:?}: expected {what}, got Ok({v:?})"),
        Err(Error::Structured(kind)) => {
            assert!(matcher(&kind), "{src:?}: expected {what}, got {kind:?}",)
        }
        Err(other) => panic!("{src:?}: expected Structured error, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Delimiter forms & content-not-trimmed
// ---------------------------------------------------------------------------

#[test]
fn quoted_key_double() {
    expect_key("\"a b\": 1\n", "a b");
}

#[test]
fn quoted_key_single() {
    expect_key("'a b': 1\n", "a b");
}

#[test]
fn quoted_key_backtick() {
    expect_key("`a b`: 1\n", "a b");
}

#[test]
fn quoted_key_interior_whitespace_preserved() {
    let key = " a ";
    expect_key("\" a \": 1\n", key);
    let value = parse("\" a \": 1\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.keys().next().unwrap().chars().count(), 3);
}

#[test]
fn quoted_key_with_colon() {
    let value = parse("\"cache:redis\": enabled\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    let got = obj.get("cache:redis");
    assert_eq!(got, Some(&Value::String("enabled".into())));
}

#[test]
fn quoted_key_with_dots_and_brackets() {
    expect_key("\"a.b[0]{c},d:e\": 1\n", "a.b[0]{c},d:e");
}

#[test]
fn quoted_key_single_delim_holds_other_quotes() {
    let value = parse("'say \"hi\": now': ok\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(
        obj.get("say \"hi\": now"),
        Some(&Value::String("ok".into()))
    );
}

#[test]
fn quoted_key_backtick_holds_both_quotes() {
    expect_key("`it's \"quoted\"`: 1\n", "it's \"quoted\"");
    let value = parse("`it's \"quoted\"`: 1\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.keys().next().unwrap().chars().count(), 13);
}

// ---------------------------------------------------------------------------
// Empty vs whitespace
// ---------------------------------------------------------------------------

#[test]
fn empty_quoted_segments_are_empty_key() {
    for src in ["\"\": 1\n", "'': 1\n", "``: 1\n"] {
        expect_err_kind(src, |k| matches!(k, ErrorKind::EmptyKey { .. }), "EmptyKey");
    }
}

#[test]
fn quoted_single_space_key_is_valid() {
    expect_key("\" \": 1\n", " ");
}

// ---------------------------------------------------------------------------
// Escapes inside quoted segments
// ---------------------------------------------------------------------------

#[test]
fn escaped_own_delimiter_inside_quoted_key() {
    expect_key("\"say \\\"hi\\\"\": 1\n", "say \"hi\"");
}

#[test]
fn escaped_single_quote_inside_single_quoted_key() {
    expect_key("'it\\'s': 1\n", "it's");
}

#[test]
fn escaped_backtick_inside_backtick_key() {
    expect_key(
        r#"`\`a\``: 1
"#,
        "`a`",
    );
}

#[test]
fn quote_escapes_still_work_in_bare_keys() {
    expect_key("a\\\"b: 1\n", "a\"b");
    expect_key("a\\'b: 1\n", "a'b");
}

#[test]
fn unicode_escape_inside_quoted_key() {
    // The source contains the six literal bytes `\u0041`; the § 3.7
    // escape decoding applies to quoted interiors too.
    expect_key("\"a\\u0041b\": 1\n", "aAb");
}

// ---------------------------------------------------------------------------
// Positional rule (non-breaking)
// ---------------------------------------------------------------------------

#[test]
fn mid_segment_quotes_stay_bare() {
    expect_key("don't: 1\n", "don't");
    expect_key("a\"b: 1\n", "a\"b");
    let value = parse("port\": 1\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    let key = obj.keys().next().unwrap().clone();
    assert_eq!(key, "port\"");
    assert_eq!(key.chars().count(), 5);
}

// ---------------------------------------------------------------------------
// Breaking change (intentional)
// ---------------------------------------------------------------------------

#[test]
fn leading_quote_opens_quoted_segment() {
    expect_key("\"port\": 1\n", "port");
}

// ---------------------------------------------------------------------------
// Nothing may follow the closer
// ---------------------------------------------------------------------------

#[test]
fn content_after_closer_is_invalid() {
    expect_err_kind(
        "\"a\"b: 1\n",
        |k| matches!(k, ErrorKind::InvalidKey { .. }),
        "InvalidKey",
    );
    expect_err_kind(
        "\"a\" \"b\": 1\n",
        |k| matches!(k, ErrorKind::InvalidKey { .. }),
        "InvalidKey",
    );
}

#[test]
fn adjacent_quoted_segments_form_path() {
    let value = parse("\"a\".\"b\": 1\n").unwrap();
    let Value::Object(outer) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(inner)) = outer.get("a") else {
        panic!("expected nested Object at `a`, got {:?}", outer.get("a"));
    };
    assert_eq!(inner.get("b"), Some(&Value::Integer("1".into())));
}

// ---------------------------------------------------------------------------
// Mixed dotted keys
// ---------------------------------------------------------------------------

#[test]
fn quoted_segment_inside_dotted_path() {
    let value = parse("a.\"b.c\".d: 1\n").unwrap();
    let Value::Object(a) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(bc)) = a.get("a") else {
        panic!("expected `a`")
    };
    let Some(Value::Object(d)) = bc.get("b.c") else {
        panic!("expected `b.c`, got {:?}", bc.get("b.c"))
    };
    assert_eq!(d.get("d"), Some(&Value::Integer("1".into())));
}

#[test]
fn dotted_whitespace_around_quoted_segment() {
    let value = parse("a. \"b\" : 1\n").unwrap();
    let Value::Object(a) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(b)) = a.get("a") else {
        panic!("expected `a`")
    };
    assert_eq!(b.get("b"), Some(&Value::Integer("1".into())));
}

// ---------------------------------------------------------------------------
// Unterminated quoted key — the three contexts
// ---------------------------------------------------------------------------

#[test]
fn unterminated_quoted_key_in_established_object() {
    let single = "x: 1\n'unterminated: 1\n";
    expect_err_kind(
        single,
        |k| matches!(k, ErrorKind::UnterminatedQuotedKey { .. }),
        "UnterminatedQuotedKey",
    );
    let double = "x: 1\n\"unterminated: 1\n";
    expect_err_kind(
        double,
        |k| matches!(k, ErrorKind::UnterminatedQuotedKey { .. }),
        "UnterminatedQuotedKey",
    );
}

#[test]
fn unterminated_quoted_key_inline_value() {
    match parse("y: {\"a: 1}\n") {
        Err(Error::Structured(ErrorKind::UnterminatedInlineCompound { .. })) => {}
        Err(Error::Structured(ErrorKind::MalformedInlineCompound { .. })) => {
            panic!("expected UnterminatedInlineCompound, got MalformedInlineCompound")
        }
        Err(Error::Structured(ErrorKind::UnterminatedQuotedKey { .. })) => {
            panic!("expected UnterminatedInlineCompound, got UnterminatedQuotedKey")
        }
        other => panic!("expected UnterminatedInlineCompound, got {other:?}"),
    }
}

#[test]
fn unterminated_quoted_key_root_inline() {
    expect_err_kind(
        "{\"a: 1}\n",
        |k| matches!(k, ErrorKind::UnterminatedInlineCompound { .. }),
        "UnterminatedInlineCompound",
    );
}

#[test]
fn unterminated_quote_on_undecided_first_line_is_array_string() {
    let src = "'tis the season: fa\n";
    let value = parse(src).unwrap();
    let Value::Array(items) = value else {
        panic!("expected root Array, got {value:?}");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::String("'tis the season: fa".into()));
}

#[test]
fn raw_marker_still_escapes_colon_ambiguity() {
    let src = "[\n:: 'tis the season: fa\n]\n";
    let value = parse(src).unwrap();
    let Value::Array(items) = value else {
        panic!("expected root Array, got {value:?}");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], Value::String("'tis the season: fa".into()));
}

#[test]
fn unterminated_quoted_key_after_dotted_prefix() {
    expect_err_kind(
        "x: 1\na.\"unterminated: 1\n",
        |k| matches!(k, ErrorKind::UnterminatedQuotedKey { .. }),
        "UnterminatedQuotedKey",
    );
}

// ---------------------------------------------------------------------------
// Control bytes (raw, unescaped)
// ---------------------------------------------------------------------------

#[test]
fn raw_control_byte_in_quoted_segment_is_invalid() {
    let src = format!("\"a{}b\": 1\n", '\u{1}');
    expect_err_kind(
        &src,
        |k| matches!(k, ErrorKind::InvalidKey { .. }),
        "InvalidKey",
    );
}

#[test]
fn raw_del_in_bare_segment_is_invalid() {
    let src = format!("a{}b: 1\n", '\u{7F}');
    expect_err_kind(
        &src,
        |k| matches!(k, ErrorKind::InvalidKey { .. }),
        "InvalidKey",
    );
}

#[test]
fn tab_vt_ff_allowed_in_bare_key() {
    for ch in ['\t', '\u{0B}', '\u{0C}'] {
        let src = format!("a{ch}b: 1\n");
        let expected: String = ['a', ch, 'b'].into_iter().collect();
        expect_key(&src, &expected);
    }
}

// ---------------------------------------------------------------------------
// Duplicate names on decoded content
// ---------------------------------------------------------------------------

#[test]
fn quoted_and_escaped_keys_collide() {
    let src = "\"a.b\": 1\na\\.b: 2\n";
    expect_err_kind(
        src,
        |k| matches!(k, ErrorKind::DuplicateKey { .. }),
        "DuplicateKey",
    );
}

// ---------------------------------------------------------------------------
// Inline pairs
// ---------------------------------------------------------------------------

#[test]
fn quoted_key_brace_inside_inline_object() {
    let value = parse("{\"a}b\": 1, c: 2}\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a}b"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn quoted_key_comma_inside_inline_object() {
    let value = parse("{\"a,b\": 1, c: 2}\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a,b"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("c"), Some(&Value::Integer("2".into())));
}

#[test]
fn root_inline_object_with_quoted_keys() {
    let value = parse("{\"a}b\": 1, c: 2}\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object root")
    };
    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("a}b"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("c"), Some(&Value::Integer("2".into())));
}

// ---------------------------------------------------------------------------
// Values keep quotes opaque ("Keys only" rule)
// ---------------------------------------------------------------------------

#[test]
fn value_position_quotes_are_ordinary_content() {
    let value = parse("a: \"b\"\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.get("a"), Some(&Value::String("\"b\"".into())));
    let Some(Value::String(s)) = obj.get("a") else {
        panic!("expected String")
    };
    assert_eq!(s.chars().count(), 3);
}

#[test]
fn quote_escapes_in_inline_value_force_string() {
    let value = parse("cfg: {v: say \\\"hi\\\"}\nw: \\\"x\\\"\n").unwrap();
    let Value::Object(root) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(cfg)) = root.get("cfg") else {
        panic!("expected `cfg` object")
    };
    assert_eq!(cfg.get("v"), Some(&Value::String("say \"hi\"".into())));
    // Plain pair values are NOT an escape-processing context (§ 3.7's
    // exclusion list: "the body of a pair ... that is the whole content
    // of a line"), so the backslashes stay literal; the body is still
    // classified as String, as written (§ 5.2 rule 15).
    assert_eq!(root.get("w"), Some(&Value::String("\\\"x\\\"".into())));
}

#[test]
fn quote_escape_forces_string_classification() {
    // The recognised `\"` forces String classification (§ 3.7 / § 5.2),
    // and since a plain pair value is not an escape-processing context
    // (§ 3.7 exclusion list), the body is stored as written.
    let value = parse("k: \\\"null\\\"\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object")
    };
    assert_eq!(obj.get("k"), Some(&Value::String("\\\"null\\\"".into())));
}

// ---------------------------------------------------------------------------
// Regression anchors
// ---------------------------------------------------------------------------

#[test]
fn bare_key_and_escapes_unchanged() {
    expect_key("port: 1\n", "port");
    expect_key("a\\.b: 1\n", "a.b");
    expect_key("a\\:b: 1\n", "a:b");
    expect_key("a\\\\b: 1\n", "a\\b");
    let value = parse("x.y\\.z: 1\n").unwrap();
    let Value::Object(x) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(y)) = x.get("x") else {
        panic!("expected `x`")
    };
    assert_eq!(y.get("y.z"), Some(&Value::Integer("1".into())));
}

#[test]
fn array_items_unaffected() {
    let value = parse("[a, \"b,c\"]\n").unwrap();
    let Value::Array(items) = value else {
        panic!("expected root Array, got {value:?}");
    };
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], Value::String("a".into()));
    assert_eq!(items[1], Value::String("\"b".into()));
    assert_eq!(items[2], Value::String("c\"".into()));
}

// ---------------------------------------------------------------------------
// Nested inline compounds with quoted keys
// ---------------------------------------------------------------------------

#[test]
fn quoted_key_brace_triple_nested() {
    let value = parse("v: {a: {b: {\"c}d\": 1}}}\n").unwrap();
    let Value::Object(v) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(a)) = v.get("v") else {
        panic!("expected `v`")
    };
    let Some(Value::Object(b)) = a.get("a") else {
        panic!("expected `a`")
    };
    let Value::Object(inner) = b.get("b").expect("expected `b`") else {
        panic!("expected Object under `b`");
    };
    assert_eq!(inner.get("c}d"), Some(&Value::Integer("1".into())));
    assert_eq!(inner.len(), 1);
}

#[test]
fn quoted_key_brace_two_levels_still_parses() {
    let value = parse("v: {a: {\"b}c\": 1}}\n").unwrap();
    let Value::Object(v) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(a)) = v.get("v") else {
        panic!("expected `v`")
    };
    let Value::Object(inner) = a.get("a").expect("expected `a`") else {
        panic!("expected Object under `a`");
    };
    assert_eq!(inner.get("b}c"), Some(&Value::Integer("1".into())));
}

#[test]
fn quoted_key_brace_deep_with_sibling_and_inner_comma() {
    let value = parse("v: {a: {b: {\"c}d\": 1, x: 2}, e: 3}}\n").unwrap();
    let Value::Object(v) = value else {
        panic!("expected Object")
    };
    let Some(Value::Object(a)) = v.get("v") else {
        panic!("expected `v`")
    };
    let Value::Object(a_body) = a.get("a").expect("expected `a`") else {
        panic!("expected Object under `a`");
    };
    let Value::Object(inner) = a_body.get("b").expect("expected `b`") else {
        panic!("expected Object under `b`");
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(inner.get("c}d"), Some(&Value::Integer("1".into())));
    assert_eq!(inner.get("x"), Some(&Value::Integer("2".into())));
    assert_eq!(a_body.get("e"), Some(&Value::Integer("3".into())));
}
