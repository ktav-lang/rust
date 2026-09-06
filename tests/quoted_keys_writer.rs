//! Spec 0.7 § 5.9.10 writer regression: quoted keys must be emitted
//! byte-for-byte identically to the corpus oracles in
//! `spec/versions/0.7/tests/valid/{key_escaping,quoted_keys}/`, across
//! every writer surface (`emit_canonical`, `render`, and the serde
//! streaming serializer), plus the § 5.9.6/§ 5.9.12 array-root
//! first-item safeguards and the root-first-key guard for a leading
//! U+FEFF (§ 3.1).
//!
//! Every expected string below was copied verbatim (trailing newline
//! included) from the corresponding `.canonical.ktav` fixture; each
//! failure message names the fixture it mirrors.

use std::collections::BTreeMap;

use ktav::render::render;
use ktav::{emit_canonical, from_str, parse, to_string, ObjectMap, Value};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn n(v: i64) -> Value {
    Value::Integer(
        v.to_string()
            .parse()
            .unwrap_or_else(|_| panic!("integer: {v}")),
    )
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// Parse `input`, assert the canonical bytes equal `expected`, and
/// assert the oracle re-parses to the same tree (lossless pin).
fn assert_oracle(fixture: &str, input: &str, expected: &str) {
    let value = parse(input).unwrap_or_else(|e| panic!("{fixture}: parse({input:?}): {e}"));
    let got = emit_canonical(&value).unwrap_or_else(|e| panic!("{fixture}: emit: {e}"));
    assert_eq!(got, expected, "{fixture}: canonical bytes mismatch");
    let reparsed = parse(expected).unwrap_or_else(|e| panic!("{fixture}: parse oracle: {e}"));
    assert_eq!(reparsed, value, "{fixture}: oracle is not lossless");
}

// ---------------------------------------------------------------------------
// STEP 1a — key_escaping/ oracles (0.6 backslash spellings → 0.7 canonical)
// ---------------------------------------------------------------------------

#[test]
fn canonical_oracles_key_escaping() {
    let cases: &[(&str, &str, &str)] = &[
        // fixture, input, expected canonical
        ("canonical_reescape", "a\\.b: v\n", "\"a.b\": v\n"),
        ("literal_dot", "a\\.b: v\n", "\"a.b\": v\n"),
        ("literal_colon", "a\\:b: v\n", "\"a:b\": v\n"),
        ("literal_comma", "a\\,b: v\n", "\"a,b\": v\n"),
        ("literal_open_brace", "a\\{b: v\n", "\"a{b\": v\n"),
        ("literal_close_brace", "a\\}b: v\n", "\"a}b\": v\n"),
        ("literal_open_bracket", "a\\[b: v\n", "\"a[b\": v\n"),
        ("literal_close_bracket", "a\\]b: v\n", "\"a]b\": v\n"),
        (
            "unicode_escape_paren_in_key",
            "a\\u0028b: v\n",
            "\"a(b\": v\n",
        ),
        // backslash named escapes stay BARE in canonical form
        ("literal_backslash", "path\\\\to: v\n", "path\\\\to: v\n"),
        ("literal_newline_in_key", "a\\nb: v\n", "a\\nb: v\n"),
        ("literal_cr_in_key", "a\\rb: v\n", "a\\rb: v\n"),
        // edge_newline_in_key: canonical is the fixture verbatim
        (
            "edge_newline_in_key",
            "\\nlf_start: value_lf_start\nlf_end\\n: value_lf_end\n\
             \\rcr_start: value_cr_start\ncr_end\\r: value_cr_end\n",
            "\\nlf_start: value_lf_start\nlf_end\\n: value_lf_end\n\
             \\rcr_start: value_cr_start\ncr_end\\r: value_cr_end\n",
        ),
        // NUL stays bare via the uppercase \uXXXX form
        (
            "unicode_escape_nul_in_key",
            "a\\u0000b: v\n",
            "a\\u0000b: v\n",
        ),
        (
            "separator_adjacent_escaped_colon",
            "a\\:: v\n",
            "\"a:\": v\n",
        ),
        (
            "unicode_escape_hash_key_comment_collision",
            "\\u0023#tag: v\n",
            "\"##tag\": v\n",
        ),
        (
            "hash_key_with_colon_composite",
            "\\u0023#a\\:b: value\n",
            "\"##a:b\": value\n",
        ),
        (
            "escaped_in_inline_key",
            "obj: {a\\.b: 1}\n",
            "obj: {\n    \"a.b\": 1\n}\n",
        ),
        (
            "mixed_path_and_literal",
            "x.y\\.z: v\n",
            "x: {\n    \"y.z\": v\n}\n",
        ),
    ];
    for (fixture, input, expected) in cases {
        assert_oracle(fixture, input, expected);
    }
}

// ---------------------------------------------------------------------------
// STEP 1b — quoted_keys/ oracles (quoted-segment spellings)
// ---------------------------------------------------------------------------

#[test]
fn canonical_oracles_quoted_keys() {
    let cases: &[(&str, &str, &str)] = &[
        ("dot_no_escape_needed", "\"a.b\": 1\n", "\"a.b\": 1\n"),
        ("colon_no_escape_needed", "\"a:b\": 1\n", "\"a:b\": 1\n"),
        (
            "brackets_and_comma_literal",
            "\"a,b{c}[d]\": 1\n",
            "\"a,b{c}[d]\": 1\n",
        ),
        (
            "comma_inside_quoted_key_in_inline",
            "{ \"a,b\": 1, c: 2 }\n",
            "\"a,b\": 1\nc: 2\n",
        ),
        (
            "inline_pair_quoted_key_with_brace",
            "{ \"a}b\": 1, c: 2 }\n",
            "\"a}b\": 1\nc: 2\n",
        ),
        ("port_example", "\\\"port\\\": 1\n", "\"\\\"port\\\"\": 1\n"),
        (
            "leading_quote_forces_quoted_canonical",
            "\\\"hello: 1\n",
            "\"\\\"hello\": 1\n",
        ),
        (
            "escaped_leading_quote_via_unicode_is_bare",
            "\\u0022hello: 1\n",
            "\"\\\"hello\": 1\n",
        ),
        (
            "structural_escape_with_embedded_double_quote",
            "a\\.b\"c: 1\n",
            "\"a.b\\\"c\": 1\n",
        ),
        // edge whitespace preserved inside quotes
        ("edge_whitespace_preserved", "\" a \": 1\n", "\" a \": 1\n"),
        ("whitespace_only_quoted_key", "\" \": 1\n", "\" \": 1\n"),
        // the full escape table: backslash/comma/braces/brackets/LF/CR/
        // dot/colon/dquote/squote/backtick/é — raw é, two-char \n \r
        (
            "full_escape_table_in_quoted_segment",
            "\"\\\\\\,\\}\\]\\{\\[\\n\\r\\.\\:\\\"\\'\\`\\u00E9\": 1\n",
            "\"\\\\,}]{[\\n\\r.:\\\"'`é\": 1\n",
        ),
        // interior delimiters of the SAME quote stay bare after self-escape
        (
            "self_escape_double_quote_delimiter",
            "\"say \\\"hi\\\"\": 1\n",
            "say \"hi\": 1\n",
        ),
        (
            "self_escape_single_quote_becomes_bare",
            "'don\\'t': 1\n",
            "don't: 1\n",
        ),
        (
            "self_escape_backtick_becomes_bare",
            "`a\\`b`: 1\n",
            "a`b: 1\n",
        ),
        (
            "quote_inside_bare_segment_unaffected",
            "don't: 1\n",
            "don't: 1\n",
        ),
        (
            "trailing_quote_bare_unaffected",
            "port\": 1\n",
            "port\": 1\n",
        ),
        (
            "other_quotes_literal_in_double",
            "\"it's a `test`\": 1\n",
            "it's a `test`: 1\n",
        ),
        // plain keys: quoted input spellings canonicalise bare
        ("backtick_basic", "`a`: 1\n", "a: 1\n"),
        ("double_quote_basic", "\"a\": 1\n", "a: 1\n"),
        ("single_quote_basic", "'a': 1\n", "a: 1\n"),
        (
            "per_segment_dotted_path",
            "a.\"b.c\".d: 1\n",
            "a: {\n    \"b.c\": {\n        d: 1\n    }\n}\n",
        ),
        // § 5.9.6/§ 5.9.12 array-root first-item safeguards
        (
            "array_item_raw_marker_needed",
            ":: \"tis the season\": fa\n",
            ":: \"tis the season\": fa\n",
        ),
        (
            "unterminated_double_quote_first_line_falls_back",
            "\"tis the season: fa\n",
            "\"tis the season: fa\n",
        ),
        (
            "unterminated_leading_quote_falls_back_to_array_item",
            "'tis the season: fa\n",
            "'tis the season: fa\n",
        ),
    ];
    for (fixture, input, expected) in cases {
        assert_oracle(fixture, input, expected);
    }
}

// ---------------------------------------------------------------------------
// STEP 2 — API-constructed Values canonicalise identically to parsed ones
// ---------------------------------------------------------------------------

#[test]
fn api_built_keys_match_parsed_canonical_bytes() {
    let cases: &[(&str, &str)] = &[
        ("a.b", "\"a.b\": 1\n"),
        ("a:b", "\"a:b\": 1\n"),
        ("\"port\"", "\"\\\"port\\\"\": 1\n"),
        ("##x", "\"##x\": 1\n"),
        ("path\\to", "path\\\\to: 1\n"),
        ("a\nb", "a\\nb: 1\n"),
        ("a\u{1}b", "a\\u0001b: 1\n"),
        ("a ", "\"a \": 1\n"),
        ("\u{FEFF}host", "\"\u{FEFF}host\": 1\n"),
    ];
    for (key, expected) in cases {
        let value = obj(&[(key, n(1))]);
        let got = emit_canonical(&value).unwrap_or_else(|e| panic!("emit({key:?}): {e}"));
        assert_eq!(&got, expected, "API-built key {key:?} canonical mismatch");
        let reparsed = parse(expected).unwrap_or_else(|e| panic!("parse oracle {key:?}: {e}"));
        assert_eq!(reparsed, value, "API-built oracle for {key:?} not lossless");
    }
}

// ---------------------------------------------------------------------------
// STEP 3 — `render` and `emit_canonical` agree (shared key emitter)
// ---------------------------------------------------------------------------

#[test]
fn render_agrees_with_canonical_writer_on_quoted_keys() {
    let cases: &[&str] = &["a.b", "a:b", "##x", "\"port\"", "a ", "\u{FEFF}host"];
    for key in cases {
        let value = obj(&[(key, s("v"))]);
        let canonical = emit_canonical(&value).unwrap_or_else(|e| panic!("emit({key:?}): {e}"));
        let rendered = render(&value).unwrap_or_else(|e| panic!("render({key:?}): {e}"));
        assert_eq!(canonical, rendered, "writers disagree on key {key:?}");
    }
}

#[test]
fn render_agrees_on_array_root_raw_marker() {
    // § 5.9.12: a root Array with a compound first item (here: an object)
    // takes the wrapped `[...]` form (first_item_needs_wrap), so this only
    // pins writer agreement on that wrapped form; the pair-candidate `::`
    // raw-marker oracle is asserted separately below.
    let value = Value::Array(vec![obj(&[("tis the season", s("fa"))])]);
    let canonical = emit_canonical(&value).unwrap();
    let rendered = render(&value).unwrap();
    assert_eq!(
        canonical, rendered,
        "writers disagree on array-root raw marker"
    );
    assert_oracle(
        "array_item_raw_marker_needed",
        ":: \"tis the season\": fa\n",
        ":: \"tis the season\": fa\n",
    );
}

// ---------------------------------------------------------------------------
// STEP 4 — serde streaming path (`to_string` / `from_str`)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RenamedField {
    #[serde(rename = "a.b")]
    x: u8,
}

#[test]
fn serde_btreemap_quoted_key_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert("a.b".to_string(), 1_i32);
    let text = to_string(&map).unwrap();
    assert_eq!(text, "\"a.b\": 1\n");
    let back: BTreeMap<String, i32> = from_str(&text).unwrap();
    assert_eq!(back, map);
}

#[test]
fn serde_renamed_field_carries_structural_bytes() {
    let text = to_string(&RenamedField { x: 1 }).unwrap();
    assert_eq!(text, "\"a.b\": 1\n");
    let back: RenamedField = from_str(&text).unwrap();
    assert_eq!(back, RenamedField { x: 1 });
}

#[test]
fn serde_root_first_key_bom_is_quoted() {
    // U+FEFF sorts after ASCII in byte order but before U+FFFD, so it is
    // the FIRST key here — the root-first-key guard must quote it
    // (§ 3.1 / 4a3aff7).
    let mut map = BTreeMap::new();
    map.insert("\u{FEFF}k".to_string(), "v".to_string());
    map.insert("\u{FFFD}z".to_string(), "w".to_string());
    let text = to_string(&map).unwrap();
    assert_eq!(text, "\"\u{FEFF}k\": v\n\u{FFFD}z: w\n");
    // The same key as the SECOND entry stays bare.
    let mut map2 = BTreeMap::new();
    map2.insert("a".to_string(), "v".to_string());
    map2.insert("\u{FEFF}k".to_string(), "w".to_string());
    let text2 = to_string(&map2).unwrap();
    assert_eq!(text2, "a: v\n\u{FEFF}k: w\n");
}

#[test]
fn serde_hash_prefix_key_is_quoted() {
    let mut map = BTreeMap::new();
    map.insert("##weird".to_string(), "v".to_string());
    let text = to_string(&map).unwrap();
    assert_eq!(text, "\"##weird\": v\n");
}
