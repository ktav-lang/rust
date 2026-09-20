//! Regression test for issue rust#14 — spec 0.7 § 5.0.1 rule 6 / § 3.3 /
//! § 4 `<sep-end>`.
//!
//! Root-kind detection for a first content line whose first unescaped
//! `:` is a plain (non-`::`) separator must accept it as satisfying
//! `<sep-end> ::= 1*ws | &line-end` for the FULL § 3.3 whitespace set
//! (25 code points), not just SPACE/TAB. § 3.3 states this is "the
//! single definition of whitespace used throughout this specification
//! ... There is no separate, narrower 'structural' whitespace concept",
//! and structural root-kind detection (§ 5.0.1 rule 6) is exactly such a
//! use.
//!
//! `is_pair_shape` in `src/parser/classify.rs` tested
//! `after.starts_with([' ', '\t'])`; fixed to
//! `after.starts_with(is_ktav_whitespace)`. That single function is
//! shared by the main parser (`classify_root_kind_050` in
//! `src/parser/parser.rs`), the thin event parser
//! (`src/thin/event_parser.rs`), and — since `src/parser/fmt_parser.rs`
//! imports `classify_root_kind_050` from `parser.rs` rather than forking
//! it — the formatter parser too, so one fix reaches all three.

use ktav::thin::{parse_events, ParseEvent};
use ktav::{format_str, parse, ObjectMap, Value};

/// All 23 line-bounded § 3.3 members that can actually appear after a
/// colon on a single pre-split line (the full 25-point set minus LF/CR,
/// which § 3.2 line splitting already consumes before this code ever
/// sees a line). Named per the issue's repro table plus the rest of the
/// § 3.3 set for full coverage, not NBSP alone.
const SEP_END_MEMBERS: [(&str, &str); 23] = [
    ("\t", "TAB U+0009"),
    ("\u{0B}", "VT U+000B"),
    ("\u{0C}", "FF U+000C"),
    (" ", "SPACE U+0020"),
    ("\u{85}", "NEL U+0085"),
    ("\u{A0}", "NBSP U+00A0"),
    ("\u{1680}", "OGHAM SPACE MARK U+1680"),
    ("\u{2000}", "EN QUAD U+2000"),
    ("\u{2001}", "EM QUAD U+2001"),
    ("\u{2002}", "EN SPACE U+2002"),
    ("\u{2003}", "EM SPACE U+2003"),
    ("\u{2004}", "THREE-PER-EM SPACE U+2004"),
    ("\u{2005}", "FOUR-PER-EM SPACE U+2005"),
    ("\u{2006}", "SIX-PER-EM SPACE U+2006"),
    ("\u{2007}", "FIGURE SPACE U+2007"),
    ("\u{2008}", "PUNCTUATION SPACE U+2008"),
    ("\u{2009}", "THIN SPACE U+2009"),
    ("\u{200A}", "HAIR SPACE U+200A"),
    ("\u{2028}", "LINE SEPARATOR U+2028"),
    ("\u{2029}", "PARAGRAPH SEPARATOR U+2029"),
    ("\u{202F}", "NARROW NBSP U+202F"),
    ("\u{205F}", "MMSP U+205F"),
    ("\u{3000}", "IDEOGRAPHIC SPACE U+3000"),
];

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// `a:<ws>1` as the FIRST content line must be an Object `{a: 1}`, for
/// every § 3.3 code point — this is the exact shape from issue
/// rust#14's repro table.
#[test]
fn first_line_plain_colon_root_is_object_for_every_sep_end_member() {
    for (ws, name) in SEP_END_MEMBERS {
        let src = format!("a:{ws}1\n");
        assert_eq!(
            parse(&src).unwrap(),
            obj(&[("a", Value::Integer("1".into()))]),
            "root misclassified for {name} ({ws:?}) in {src:?}"
        );
    }
}

/// Same root-kind bug, reached via comments/blanks first — § 5.0.1's
/// "first CONTENT line" skips comments and blank lines before deciding,
/// so they must not mask the bug (issue rust#14's corpus-gap note).
#[test]
fn comments_and_blanks_before_first_content_line_do_not_mask_the_bug() {
    for (ws, name) in SEP_END_MEMBERS {
        let src = format!("## c\na:{ws}1\n");
        assert_eq!(
            parse(&src).unwrap(),
            obj(&[("a", Value::Integer("1".into()))]),
            "root misclassified (after comment) for {name} ({ws:?}) in {src:?}"
        );
        let src = format!("\na:{ws}1\n");
        assert_eq!(
            parse(&src).unwrap(),
            obj(&[("a", Value::Integer("1".into()))]),
            "root misclassified (after blank) for {name} ({ws:?}) in {src:?}"
        );
    }
}

/// The thin event parser (`src/thin/event_parser.rs`) shares the same
/// `is_pair_shape` call — must agree with the owned parser.
#[test]
fn thin_event_parser_root_is_object_for_every_sep_end_member() {
    for (ws, name) in SEP_END_MEMBERS {
        let src = format!("a:{ws}1\n");
        let mut events = Vec::new();
        parse_events(&src, |e| {
            events.push(match e {
                ParseEvent::BeginObject => "BeginObject",
                ParseEvent::BeginArray => "BeginArray",
                _ => "other",
            });
        })
        .unwrap_or_else(|e| panic!("thin parse failed for {name} ({ws:?}): {e}"));
        assert_eq!(
            events.first(),
            Some(&"BeginObject"),
            "thin parser root misclassified for {name} ({ws:?}) in {src:?}"
        );
    }
}

/// The formatter's parser fork (`src/parser/fmt_parser.rs`) reuses
/// `classify_root_kind_050` from `parser.rs` rather than duplicating
/// root-kind detection, so the fix reaches it too: re-parsing
/// `format_str`'s output must still yield an Object, not an Array of one
/// String.
#[test]
fn formatter_root_is_object_for_every_sep_end_member() {
    for (ws, name) in SEP_END_MEMBERS {
        let src = format!("a:{ws}1\n");
        let formatted =
            format_str(&src).unwrap_or_else(|e| panic!("format_str failed for {name}: {e}"));
        assert_eq!(
            parse(&formatted).unwrap(),
            obj(&[("a", Value::Integer("1".into()))]),
            "formatter round-trip misclassified root for {name} ({ws:?}); formatted:\n{formatted}"
        );
    }
}

/// Bounding cases from the issue that already worked and must keep
/// working unchanged — the bug was narrow to the plain-`:` root-kind
/// test only, not to whitespace handling in general.
#[test]
fn already_correct_neighbors_are_unaffected() {
    // Once the root is already Object (from an earlier pair), the pair
    // parser handles the full § 3.3 set fine — this never went through
    // root-kind detection at all.
    assert_eq!(
        parse("x: 0\na:\u{a0}1\n").unwrap(),
        obj(&[
            ("x", Value::Integer("0".into())),
            ("a", Value::Integer("1".into())),
        ])
    );
    // A leading SPACE satisfies `is_pair_shape`'s test; the maximal
    // whitespace run (including a following NBSP) is then consumed by
    // `<sep-end>`.
    assert_eq!(
        parse("a: \u{a0}1\n").unwrap(),
        obj(&[("a", Value::Integer("1".into()))])
    );
    // The `::` raw-marker branch never used the narrow predicate.
    assert_eq!(
        parse("a::\u{a0}body\n").unwrap(),
        obj(&[("a", Value::String("body".into()))])
    );
    // Inline compounds (§ 5.8) already accept the full set.
    assert_eq!(
        parse("k: {a:\u{a0}1}\n").unwrap(),
        obj(&[("k", obj(&[("a", Value::Integer("1".into()))]))])
    );
}
