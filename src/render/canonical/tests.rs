use super::num::needs_raw_marker;
use super::walk::emit_canonical;
use crate::value::Value;

use super::*;
use crate::value::ObjectMap;
use compact_str::CompactString;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

fn obj(pairs: Vec<(&str, Value)>) -> Value {
    let mut map: ObjectMap = IndexMap::with_capacity_and_hasher(pairs.len(), FxBuildHasher);
    for (k, v) in pairs {
        map.insert(CompactString::new(k), v);
    }
    Value::Object(map)
}

fn arr(items: Vec<Value>) -> Value {
    Value::Array(items)
}

fn int(n: i64) -> Value {
    let mut buf = itoa::Buffer::new();
    Value::Integer(CompactString::new(buf.format(n)))
}

fn float(f: f64) -> Value {
    let mut buf = ryu::Buffer::new();
    Value::Float(CompactString::new(buf.format(f)))
}

fn s(text: &str) -> Value {
    Value::String(CompactString::new(text))
}

#[test]
fn empty_object_root_produces_zero_bytes() {
    let v = obj(vec![]);
    assert_eq!(emit_canonical(&v).unwrap(), "");
}

#[test]
fn empty_array_root_produces_brackets() {
    let v = arr(vec![]);
    assert_eq!(emit_canonical(&v).unwrap(), "[]\n");
}

#[test]
fn simple_pairs() {
    let v = obj(vec![
        ("host", s("localhost")),
        ("port", int(8080)),
        ("debug", Value::Bool(true)),
    ]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "host: localhost\nport: 8080\ndebug: true\n");
}

#[test]
fn null_and_false_keywords() {
    let v = obj(vec![
        ("maintenance", Value::Null),
        ("enabled", Value::Bool(false)),
    ]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "maintenance: null\nenabled: false\n");
}

#[test]
fn float_values() {
    let v = obj(vec![("ratio", float(0.5)), ("sci", float(1.5e-3))]);
    let out = emit_canonical(&v).unwrap();
    // ryu shortest: 0.5 → "0.5", 1.5e-3 → "0.0015"
    assert!(out.contains("ratio: 0.5\n"), "got: {out}");
    // ryu may produce "0.0015" or "1.5e-3" — accept both canonical forms
    assert!(
        out.contains("sci: 1.5e-3\n") || out.contains("sci: 0.0015\n"),
        "got: {out}"
    );
}

// --- 0.7 § 5.2 rule 14 / § 5.9.8: zero canonicalisation and
// domain-floor scale magnitudes ---------------------------------------

#[test]
fn canonical_float_zero_forms_pass_through() {
    assert_eq!(canonical_float("0.0"), "0.0");
    assert_eq!(canonical_float("-0.0"), "-0.0");
}

#[test]
fn canonical_zero_emits_decimal_with_sign() {
    let v = obj(vec![("z", float(0.0)), ("nz", float(-0.0))]);
    assert_eq!(emit_canonical(&v).unwrap(), "z: 0.0\nnz: -0.0\n");
}

#[test]
fn canonical_min_positive_scale_magnitudes() {
    let v = obj(vec![
        ("k", float(f64::MIN_POSITIVE)),
        ("mn", float(f64::from_bits(1))),
        ("mnn", float(-f64::from_bits(1))),
    ]);
    assert_eq!(
        emit_canonical(&v).unwrap(),
        "k: 2.2250738585072014e-308\nmn: 5e-324\nmnn: -5e-324\n"
    );
}

#[test]
fn empty_string_pair() {
    let v = obj(vec![("note", s(""))]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "note:\n");
}

#[test]
fn raw_marker_for_keywords() {
    let v = obj(vec![("a", s("true")), ("b", s("null")), ("c", s("false"))]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "a:: true\nb:: null\nc:: false\n");
}

#[test]
fn raw_marker_for_numbers() {
    let v = obj(vec![("a", s("42")), ("b", s("0.5")), ("c", s("0xFF"))]);
    let out = emit_canonical(&v).unwrap();
    assert!(out.contains("a:: 42\n"));
    assert!(out.contains("b:: 0.5\n"));
    assert!(out.contains("c:: 0xFF\n"));
}

#[test]
fn raw_marker_for_inline_opener() {
    let v = obj(vec![("a", s("{hello}"))]);
    let out = emit_canonical(&v).unwrap();
    assert!(out.contains("a:: {hello}\n"));
}

#[test]
fn nested_object() {
    let v = obj(vec![(
        "server",
        obj(vec![("host", s("localhost")), ("port", int(8080))]),
    )]);
    let out = emit_canonical(&v).unwrap();
    let expected = "server: {\n    host: localhost\n    port: 8080\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn nested_array() {
    let v = obj(vec![("tags", arr(vec![s("a"), s("b")]))]);
    let out = emit_canonical(&v).unwrap();
    let expected = "tags: [\n    a\n    b\n]\n";
    assert_eq!(out, expected);
}

#[test]
fn array_root_bare_items() {
    let v = arr(vec![s("foo"), s("bar"), s("baz")]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "foo\nbar\nbaz\n");
}

#[test]
fn array_root_wraps_when_first_item_is_compound() {
    let v = arr(vec![arr(vec![s("a"), s("b")]), arr(vec![s("c"), s("d")])]);
    let out = emit_canonical(&v).unwrap();
    let expected = "[\n    [\n        a\n        b\n    ]\n    [\n        c\n        d\n    ]\n]\n";
    assert_eq!(out, expected);
}

#[test]
fn array_root_does_not_wrap_for_scalars() {
    let v = arr(vec![int(1), int(2), int(3)]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "1\n2\n3\n");
}

#[test]
fn cr_in_string_is_error() {
    let v = obj(vec![("x", s("hello\rworld"))]);
    assert!(emit_canonical(&v).is_err());
}

#[test]
fn verbatim_multiline_string() {
    let v = obj(vec![("msg", s("line one\nline two"))]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(
        out,
        "msg: ((\n\
         line one\n\
         line two\n\
         ))\n"
    );
}

#[test]
fn verbatim_multiline_in_array_item() {
    let v = arr(vec![s("line one\nline two"), s("end")]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(
        out,
        "((\n\
         line one\n\
         line two\n\
         ))\n\
         end\n"
    );
}

#[test]
fn empty_string_array_item() {
    let v = arr(vec![s(""), s("ok")]);
    let out = emit_canonical(&v).unwrap();
    assert_eq!(out, "::\nok\n");
}

#[test]
fn raw_marker_for_paren_tokens() {
    let v = obj(vec![
        ("a", s("(")),
        ("b", s("((")),
        ("c", s("()")),
        ("d", s("(())")),
    ]);
    let out = emit_canonical(&v).unwrap();
    // Spec 0.7: bare `(` / `((` (multi-line openers) and `()` / `(())`
    // (rule 5, → empty String) need `::`; other `(`-prefixed strings
    // round-trip plain (fixture `inline/paren_scalar_is_string`).
    assert!(out.contains("a:: (\n"));
    assert!(out.contains("b:: ((\n"));
    assert!(out.contains("c:: ()\n"));
    assert!(out.contains("d:: (())\n"));
}

#[test]
fn mixed_heterogeneous_array() {
    let v = obj(vec![(
        "mixed",
        arr(vec![
            s("plain_string"),
            int(42),
            Value::Bool(true),
            Value::Null,
            s("true"), // keyword collision → raw marker
            obj(vec![("nested_obj", s("inside"))]),
            arr(vec![s("nested_array")]),
        ]),
    )]);
    let out = emit_canonical(&v).unwrap();
    let expected = "\
mixed: [
    plain_string
    42
    true
    null
    :: true
    {
        nested_obj: inside
    }
    [
        nested_array
    ]
]
";
    assert_eq!(out, expected);
}

#[test]
fn integer_canonical_negative() {
    let v = obj(vec![("x", int(-1)), ("y", int(-42))]);
    let out = emit_canonical(&v).unwrap();
    assert!(out.contains("x: -1\n"));
    assert!(out.contains("y: -42\n"));
}

#[test]
fn integer_canonical_zero() {
    // `-0` should normalise to `0` — but since we use itoa, -0i64
    // would be `0` anyway (no negative zero in i64). The canonical
    // form from the parser would store "0".
    let v = obj(vec![("z", int(0))]);
    let out = emit_canonical(&v).unwrap();
    assert!(out.contains("z: 0\n"));
}

#[test]
fn needs_raw_marker_integer_forms() {
    assert!(needs_raw_marker("42"));
    assert!(needs_raw_marker("-1"));
    assert!(needs_raw_marker("+7"));
    assert!(needs_raw_marker("0xFF"));
    assert!(needs_raw_marker("0o755"));
    assert!(needs_raw_marker("0b1111_0000"));
    assert!(needs_raw_marker("1_000_000"));
    assert!(!needs_raw_marker("hello"));
    assert!(!needs_raw_marker("42abc"));
}

#[test]
fn needs_raw_marker_float_forms() {
    assert!(needs_raw_marker("0.5"));
    assert!(needs_raw_marker("1.5e-3"));
    assert!(needs_raw_marker("1e9"));
    assert!(!needs_raw_marker("1."));
    assert!(!needs_raw_marker(".5"));
}

// --- § 5.9.10: key form selection + re-escape ------------------------
// Exact-oracle expectations verified against the corpus
// fixtures (spec/versions/0.8/tests/valid/{key_escaping,quoted_keys}).

/// Structural bytes (`.` `:` `,` `{` `}` `[` `]` `(` `)`) route the
/// key to quoted form (§ 5.9.10 rule (a)) — bare `\.` is the old
/// 0.6 spelling, never the 0.7 canonical output.
#[test]
fn structural_bytes_force_quoted() {
    for key in ["a.b", "a:b", "a,b", "a{b", "a}b", "a[b", "a]b", "a(b"] {
        let v = obj(vec![(key, s("v"))]);
        let expected = format!("\"{}\": v\n", key);
        assert_eq!(emit_canonical(&v).unwrap(), expected, "key: {key}");
    }
}

/// A literal backslash alone stays BARE — quoted form needs the
/// identical `\\` escape, so quoting buys nothing (§ 5.9.10).
#[test]
fn literal_backslash_stays_bare() {
    let v = obj(vec![("path\\to", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "path\\\\to: v\n");
}

/// A leading `"` forces quoted form (rule (b)); the interior
/// delimiter occurrences need only `\"` (§ 5.9.10 `port` example).
#[test]
fn leading_quote_forces_quoted() {
    let v = obj(vec![("\"port\"", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"\\\"port\\\"\": v\n");
}

/// A leading `##` forces quoted form (rule (d)) — no bare escape
/// changes the raw first two bytes § 5.1 rule 2 inspects.
#[test]
fn leading_double_hash_forces_quoted() {
    let v = obj(vec![("##a:b", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"##a:b\": v\n");
    let v = obj(vec![("##tag", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"##tag\": v\n");
}

/// LF/CR inside a key stay BARE with the named escapes `\n`/`\r` —
/// a quoted segment never admits them raw, so quoting buys nothing.
#[test]
fn interior_newline_and_cr_stay_bare() {
    let v = obj(vec![("a\nb", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\\nb: v\n");
    let v = obj(vec![("a\rb", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\\rb: v\n");
}

/// Edge LF/CR stay BARE too (edge-whitespace exemption, same
/// quoted-excludes-them-anyway argument) — corpus
/// `key_escaping/edge_newline_in_key`.
#[test]
fn edge_newline_and_cr_stay_bare() {
    for key in ["\nlf", "lf\n", "\rcr", "cr\r"] {
        let v = obj(vec![(key, s("v"))]);
        let expected = format!("{}: v\n", key.replace('\n', "\\n").replace('\r', "\\r"));
        assert_eq!(emit_canonical(&v).unwrap(), expected, "key: {key:?}");
    }
}

/// Interior tab (and any interior § 3.3 whitespace) stays raw.
#[test]
fn interior_tab_stays_bare_raw() {
    let v = obj(vec![("a\tb", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\tb: v\n");
}

/// Control bytes and DEL use `\uXXXX` with four UPPERCASE hex
/// digits, bare form (corpus `unicode_escape_nul_in_key`).
#[test]
fn control_bytes_use_uppercase_unicode_escape() {
    let v = obj(vec![("a\u{1}b", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\\u0001b: v\n");
    let v = obj(vec![("a\u{0}b", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\\u0000b: v\n");
    let v = obj(vec![("a\u{7F}b", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\\u007Fb: v\n");
}

/// Edge § 3.3 whitespace (other than LF/CR) forces quoted form —
/// quoted content is never trimmed (§ 5.3.3), and the whitespace
/// is emitted raw inside the quotes (corpus
/// `quoted_keys/edge_whitespace_preserved`).
#[test]
fn edge_whitespace_forces_quoted() {
    for key in ["a ", " a", " "] {
        let v = obj(vec![(key, s("v"))]);
        let expected = format!("\"{}\": v\n", key);
        assert_eq!(emit_canonical(&v).unwrap(), expected, "key: {key:?}");
    }
    let v = obj(vec![("\tx", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"\tx\": v\n");
}

/// U+FEFF at the start of the ROOT Object's FIRST key forces
/// quoted form (rule (c) — byte-offset-0 BOM collision, § 5.9.12),
/// with the U+FEFF raw inside the quotes. The same key at any
/// other pair position is emitted bare.
#[test]
fn bom_root_first_key_quoted_elsewhere_bare() {
    // Root, first pair → quoted.
    let v = obj(vec![("\u{FEFF}host", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"\u{FEFF}host\": v\n");
    // Root, SECOND pair → bare.
    let v = obj(vec![("ok", s("v")), ("\u{FEFF}host", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "ok: v\n\u{FEFF}host: v\n");
    // First pair of a NESTED object → bare.
    let v = obj(vec![("outer", obj(vec![("\u{FEFF}host", s("v"))]))]);
    assert_eq!(
        emit_canonical(&v).unwrap(),
        "outer: {\n    \u{FEFF}host: v\n}\n"
    );
    // U+FEFF not the first code point → bare always.
    let v = obj(vec![("a\u{FEFF}host", s("v"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "a\u{FEFF}host: v\n");
}

/// The root_first flag is keyed on PAIR POSITION, not content: a
/// root object's first key still takes form selection (quoting for
/// a structural byte) even when followed by more pairs.
#[test]
fn root_first_key_with_structural_byte_still_quoted_among_pairs() {
    let v = obj(vec![("a.b", s("v")), ("c", s("w"))]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"a.b\": v\nc: w\n");
}

// -----------------------------------------------------------------------
// § 5.9.6 / § 5.9.12 — Array root, first item safeguards
// -----------------------------------------------------------------------

/// A first item whose body is a pair candidate is read by § 5.0.1
/// rule 6 as the root Object's first pair, so the raw-marker form
/// must be used instead. Fixture oracle:
/// `quoted_keys/array_item_raw_marker_needed.canonical.ktav`.
#[test]
fn array_root_first_item_pair_candidate_takes_raw_marker() {
    let v = arr(vec![s("\"tis the season\": fa")]);
    let text = emit_canonical(&v).unwrap();
    assert_eq!(text, ":: \"tis the season\": fa\n");
    let back = crate::parse(&text).unwrap();
    assert_eq!(back, v);
}

/// An unterminated leading quote swallows the colon
/// (`find_unescaped_colon` returns no separator), so the body is
/// NOT a pair candidate and stays bare. Fixture oracles:
/// `quoted_keys/unterminated_double_quote_first_line_falls_back` /
/// `unterminated_leading_quote_falls_back_to_array_item`.
#[test]
fn array_root_first_item_unterminated_quote_stays_bare() {
    let v = arr(vec![s("\"tis the season: fa")]);
    assert_eq!(emit_canonical(&v).unwrap(), "\"tis the season: fa\n");
    let v = arr(vec![s("'tis the season: fa")]);
    assert_eq!(emit_canonical(&v).unwrap(), "'tis the season: fa\n");
}

/// Plain glued `:` fails `<sep-end>` — not a pair candidate. Glued
/// `::` (raw marker) and `: ` (whitespace-terminated) ARE.
#[test]
fn array_root_first_item_glued_colon_stays_bare() {
    assert_eq!(emit_canonical(&arr(vec![s("a:b")])).unwrap(), "a:b\n");
    assert_eq!(emit_canonical(&arr(vec![s("a::b")])).unwrap(), ":: a::b\n");
    assert_eq!(emit_canonical(&arr(vec![s("a: b")])).unwrap(), ":: a: b\n");
}

/// Only the FIRST item of an Array root is exposed to root-kind
/// detection — later items are dispatched directly as array-item
/// lines (§ 5.0.1 rules 7–8).
#[test]
fn array_root_second_item_not_guarded() {
    let v = arr(vec![s("head"), s("a: b")]);
    assert_eq!(emit_canonical(&v).unwrap(), "head\na: b\n");
}

/// Nested array items are never root-detected.
#[test]
fn nested_array_first_item_not_guarded() {
    let v = obj(vec![("arr", arr(vec![s("a: b")]))]);
    assert_eq!(emit_canonical(&v).unwrap(), "arr: [\n    a: b\n]\n");
}

/// § 5.9.12: a first item beginning with U+FEFF takes the
/// raw-marker form independently of the pair-candidate test —
/// bare form would place the BOM at byte offset 0, where readers
/// strip it per § 3.1. Every other position stays bare.
#[test]
fn array_root_first_item_bom_takes_raw_marker() {
    assert_eq!(
        emit_canonical(&arr(vec![s("\u{FEFF}host")])).unwrap(),
        ":: \u{FEFF}host\n"
    );
    // Second position → bare.
    let v = arr(vec![s("x"), s("\u{FEFF}host")]);
    assert_eq!(emit_canonical(&v).unwrap(), "x\n\u{FEFF}host\n");
    // Nested array → bare.
    let v = obj(vec![("arr", arr(vec![s("\u{FEFF}host")]))]);
    assert_eq!(emit_canonical(&v).unwrap(), "arr: [\n    \u{FEFF}host\n]\n");
}

/// A plain scalar first item is unaffected by the safeguards.
#[test]
fn array_root_first_item_plain_scalar_unaffected() {
    let v = arr(vec![s("plain"), s("a: b")]);
    assert_eq!(emit_canonical(&v).unwrap(), "plain\na: b\n");
}

/// The wrapped form is untouched: when the first item is a
/// compound, the root wraps in `[...]` and the first content line
/// is `[` itself — no item is root-detected, so no string guard
/// applies (§ 5.9.3 / § 5.9.6).
#[test]
fn array_root_wrapped_form_untouched_by_first_item_guard() {
    let v = arr(vec![obj(vec![("k", s("v"))])]);
    assert_eq!(
        emit_canonical(&v).unwrap(),
        "[\n    {\n        k: v\n    }\n]\n"
    );
}
