//! Quoted-key tests: key validation and the colon/quote scanners (spec 0.7 § 5.3.3).

// --- validate (spec 0.7 quoted keys, § 5.3.3) --------------------------------

use super::S;
use crate::error::{Error, ErrorKind};
use crate::parser::inline::{decode_key_segment, process_escapes};
use crate::parser::validate::is_valid_key;
use crate::parser::validate::{check_key, KeyValidity};
use crate::value::Value;

#[test]
fn check_key_quoted_segments() {
    use KeyValidity::{Empty, Invalid, Valid};

    // Bare still works.
    assert_eq!(check_key("port"), Valid);
    // Quoted: other quote chars and structural bytes are ordinary content.
    assert_eq!(check_key("\"a b\""), Valid);
    assert_eq!(check_key("`it's \"quoted\"`"), Valid);
    assert_eq!(check_key("\"a,b{c}d[e]:f.g\""), Valid);
    // Quoted content is never trimmed.
    assert_eq!(check_key("\" a \""), Valid);
    assert_eq!(check_key("\" \""), Valid);
    // Empty quoted content is EmptyKey (§ 6.5), for all three delimiters.
    assert_eq!(check_key("\"\""), Empty);
    assert_eq!(check_key("''"), Empty);
    assert_eq!(check_key("``"), Empty);
    // Content after the closer (§ 6.4 "nothing may follow the closer").
    assert_eq!(check_key("\"a\"b"), Invalid);
    assert_eq!(check_key("\"a\" \"b\""), Invalid);
    // Unterminated (normally diagnosed earlier as UnterminatedQuotedKey).
    assert_eq!(check_key("'\"unbalanced"), Invalid);
    // Bare forbidden control bytes / DEL (spec 0.7 § 4 <key-char>).
    assert_eq!(check_key("\u{1}a"), Invalid);
    assert_eq!(check_key("\u{7F}"), Invalid);
    // VT (0x0B) and FF (0x0C) are ALLOWED (new under 0.7).
    assert!(is_valid_key("a\u{B}b"));
    assert!(is_valid_key("a\u{C}b"));
    // Quoted forbidden control byte.
    assert_eq!(check_key("\"\u{1}\""), Invalid);
    // Quoted with escapes — structural bytes via escapes are fine.
    assert_eq!(check_key(r#""a\.b\u{41}""#), Valid);
}

#[test]
fn process_escapes_quote_escapes() {
    assert_eq!(process_escapes(r#"a"b"#, 1, S).unwrap(), "a\"b");
    assert_eq!(process_escapes(r"a\'b", 1, S).unwrap(), "a'b");
    assert_eq!(process_escapes(r"a`b", 1, S).unwrap(), "a`b");
    // Combined with the pre-existing escape set.
    assert_eq!(process_escapes(r"a\:\.b", 1, S).unwrap(), "a:.b");
}

#[test]
fn decode_key_segment_quoted() {
    assert_eq!(decode_key_segment("\"a b\"", 1, S).unwrap(), "a b");
    assert_eq!(
        decode_key_segment("`it's \"quoted\"`", 1, S).unwrap(),
        "it's \"quoted\""
    );
    // Interior is NOT trimmed (spec 0.7 § 5.3.3).
    assert_eq!(decode_key_segment("\" a \"", 1, S).unwrap(), " a ");
    assert_eq!(decode_key_segment(r#""a\:b""#, 1, S).unwrap(), "a:b");
    assert_eq!(decode_key_segment(r#""a\u0041b""#, 1, S).unwrap(), "aAb");
}

#[test]
fn parse_quoted_keys() {
    // Root-level quoted key.
    let v = crate::parse("\"a\": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a"), Some(&Value::Integer("1".into())));

    // A single space as the key — quoted content is never trimmed.
    let v = crate::parse("\" \": 1").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get(" "), Some(&Value::Integer("1".into())));

    // Empty quoted content → EmptyKey (§ 6.5).
    let e = crate::parse("\"\": 1").unwrap_err();
    assert!(matches!(e, Error::Structured(ErrorKind::EmptyKey { .. })));

    // Content after the closer → InvalidKey (§ 6.4).
    let e = crate::parse("\"a\"b: 1").unwrap_err();
    assert!(matches!(e, Error::Structured(ErrorKind::InvalidKey { .. })));

    // Quotes NOT in first position are ordinary key chars (unchanged).
    let v = crate::parse("port\": 1").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("port\""),
        Some(&Value::Integer("1".into()))
    );
    let v = crate::parse("a\"b: 1").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("a\"b"),
        Some(&Value::Integer("1".into()))
    );

    // Value-side: quote escapes now decode inside inline scalar values.
    let v = crate::parse("cfg: {v: say \"hi\"}").unwrap();
    let cfg = v
        .as_object()
        .unwrap()
        .get("cfg")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(cfg.get("v"), Some(&Value::String("say \"hi\"".into())));
}

// --- quoted keys: scanners (spec 0.7 § 5.3.3) -------------------------------

use crate::parser::inline::find_matching_close;
use crate::parser::inline::find_unescaped_colon_inline;
use crate::parser::inline::has_quote_bytes;
use crate::parser::inline::split_top_level;
use crate::parser::inline::ColonScan;
use crate::parser::inline::InlineBody;
use crate::parser::inline::InlineBounds;
use crate::parser::inline::{key_is_single_segment, scan_unescaped_colon, split_key_path};

#[test]
fn quoted_colon_scan_finds_colon_outside_spans() {
    assert_eq!(scan_unescaped_colon("a: 1"), ColonScan::Found(1));
    // Colon inside quotes is skipped.
    assert_eq!(scan_unescaped_colon("\"a: b\": 1"), ColonScan::Found(6));
    assert_eq!(scan_unescaped_colon("'a' : 1"), ColonScan::Found(4));
    assert_eq!(scan_unescaped_colon("`a:b`: 1"), ColonScan::Found(5));
    // Dotted path with a quoted middle segment.
    assert_eq!(scan_unescaped_colon("a.\"b:c\".d: 1"), ColonScan::Found(9));
    // Whitespace after the dot is skipped before the quote test.
    assert_eq!(scan_unescaped_colon("a . \"b\": 1"), ColonScan::Found(7));
}

#[test]
fn quoted_colon_scan_unterminated() {
    assert_eq!(
        scan_unescaped_colon("\"unterm: 1"),
        ColonScan::UnterminatedQuote
    );
    assert_eq!(
        scan_unescaped_colon("a.\"unterm"),
        ColonScan::UnterminatedQuote
    );
}

#[test]
fn quoted_colon_scan_absent_and_escapes() {
    assert_eq!(scan_unescaped_colon("no colon"), ColonScan::Absent);
    // Escaped colon is not a separator.
    assert_eq!(scan_unescaped_colon("a\\:b: 1"), ColonScan::Found(4));
    // Escaped quote inside the span does not close it.
    assert_eq!(scan_unescaped_colon("\"a\\\"b\": 1"), ColonScan::Found(6));
    // Junk after the closer is still scanned normally.
    assert_eq!(scan_unescaped_colon("\"a\"b: 1"), ColonScan::Found(4));
}

#[test]
fn r10f1_colon_scan_value_side_quotes_are_not_key_quotes() {
    // R10-F1: the separator search checks quote-opacity only over the
    // KEY PREFIX up to the candidate colon — the value's own quotes,
    // dots and colons must never turn the pair's own separator into
    // segment content or an unterminated key.
    assert_eq!(scan_unescaped_colon("a: \"b:c\""), ColonScan::Found(1));
    // A value quote that never closes is still VALUE content: Found,
    // not UnterminatedQuote.
    assert_eq!(
        scan_unescaped_colon("a: \"unterminated"),
        ColonScan::Found(1)
    );
    // The positive-control shape: a dot-armed segment start and an
    // unterminated quote INSIDE the value, with a colon after both.
    assert_eq!(
        scan_unescaped_colon("a: b.\"unterm: 2\""),
        ColonScan::Found(1)
    );
    assert_eq!(
        scan_unescaped_colon("a: b.c.\"x:y\".d"),
        ColonScan::Found(1)
    );
    // Truly UNterminated value-side quote after a dot (no closer): the
    // pair separator is still the first colon. This literal is in the
    // golden-corpus input set on purpose: under the pre-fix scope bug
    // (opacity checked over the whole pair) it would flip to
    // UnterminatedQuote, so the corpus distinguishes the value-side
    // sub-class too.
    assert_eq!(
        scan_unescaped_colon("a: b.\"unterm: 2"),
        ColonScan::Found(1)
    );
    // A non-segment-start quote in the prefix is ordinary key content:
    // the colon AFTER it is still found (and `a: b"c: d` shows the pair
    // separator is the FIRST unescaped colon — value quotes cannot move
    // it).
    assert_eq!(scan_unescaped_colon("ab\"c:d: 1"), ColonScan::Found(4));
    assert_eq!(scan_unescaped_colon("a: b\"c: d"), ColonScan::Found(1));
    // Multi-segment keys with quoted middles: candidates inside a span
    // resume after its closer, and the NEXT candidate is the separator.
    assert_eq!(scan_unescaped_colon("a.\"x:y\".b: 1"), ColonScan::Found(9));
    assert_eq!(scan_unescaped_colon("\"a\".\"b\": 1"), ColonScan::Found(7));
    // Escaped quote in the key prefix never opens a segment.
    assert_eq!(scan_unescaped_colon("a\\\"b: 1"), ColonScan::Found(4));
    // Prefix ending mid segment-start whitespace still finds the colon.
    assert_eq!(scan_unescaped_colon("a .  \"b\": 1"), ColonScan::Found(8));
}

#[test]
fn r10f1_colon_scan_no_candidate_corners() {
    // No unescaped colon candidate anywhere: the slow walk still
    // distinguishes Absent from UnterminatedQuote (§ 5.3.3 keeps
    // precedence over MissingSeparator even with no colon at all).
    assert_eq!(scan_unescaped_colon("a b"), ColonScan::Absent);
    assert_eq!(scan_unescaped_colon("a\\:b"), ColonScan::Absent);
    assert_eq!(
        scan_unescaped_colon("\"unterm"),
        ColonScan::UnterminatedQuote
    );
    assert_eq!(
        scan_unescaped_colon("a.\"unterm"),
        ColonScan::UnterminatedQuote
    );
    // A closed quoted key with no colon after it: Absent, not
    // UnterminatedQuote.
    assert_eq!(scan_unescaped_colon("\"a\" b"), ColonScan::Absent);
}

#[test]
fn quoted_split_key_path_keeps_quotes_in_slices() {
    assert_eq!(split_key_path("a.\"b.c\".d"), vec!["a", "\"b.c\"", "d"]);
    assert_eq!(split_key_path("\"a\".\"b\""), vec!["\"a\"", "\"b\""]);
    // Escaped dot is not a separator (no quote bytes → fast path).
    assert_eq!(split_key_path("a\\.b"), vec!["a\\.b"]);
    // Post-dot whitespace stays in the slice — callers trim.
    assert_eq!(split_key_path("a. \"b\""), vec!["a", " \"b\""]);
    assert_eq!(split_key_path("\"a.b\""), vec!["\"a.b\""]);
}

#[test]
fn quoted_key_is_single_segment() {
    assert!(key_is_single_segment("\"a.b\""));
    assert!(!key_is_single_segment("a.\"b\".c"));
    // Unterminated span swallows the rest — one segment (defensive).
    assert!(key_is_single_segment("\"unterm"));
}

#[test]
fn quoted_find_unescaped_colon_inline() {
    // `}` inside a quoted key segment does not affect depth.
    assert_eq!(find_unescaped_colon_inline("\"a}b\": 1"), Some(5));
    // Span never closes — no colon found; caller maps to unterminated.
    assert_eq!(find_unescaped_colon_inline("\"a: 1"), None);
    // Value-side quotes are ignored.
    assert_eq!(find_unescaped_colon_inline("a: \"b}"), Some(1));
}

#[test]
fn quoted_split_top_level_object_mode() {
    // Comma inside a quoted KEY does not split.
    assert_eq!(
        split_top_level(
            "\"a}b\": 1, c: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a}b\": 1, c: 2"),
            has_quote_bytes("\"a}b\": 1, c: 2".as_bytes())
        )
        .unwrap(),
        vec!["\"a}b\": 1", " c: 2"]
    );
    // Comma inside a quoted VALUE does split ("Keys only"): value
    // quotes are ordinary content, so both commas are split points.
    assert_eq!(
        split_top_level(
            "a: \"x,y\", b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: \"x,y\", b: 2"),
            has_quote_bytes("a: \"x,y\", b: 2".as_bytes())
        )
        .unwrap(),
        vec!["a: \"x", "y\"", " b: 2"]
    );
    // Unterminated quoted key segment.
    match split_top_level(
        "\"a: 1",
        1,
        S,
        InlineBody::Object,
        InlineBounds::for_input("\"a: 1"),
        has_quote_bytes("\"a: 1".as_bytes()),
    ) {
        Err(crate::Error::Structured(crate::ErrorKind::UnterminatedInlineCompound { .. })) => {}
        other => panic!(
            "expected UnterminatedInlineCompound, got: {:?}",
            other.err()
        ),
    }
}

#[test]
fn quoted_split_top_level_array_mode_ignores_quotes() {
    // Array bodies never track quotes (value positions — "Keys only"):
    // today's behaviour kept exactly, so the comma inside the quotes
    // splits.
    assert_eq!(
        split_top_level(
            "\"a,b\", c",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("\"a,b\", c"),
            has_quote_bytes("\"a,b\", c".as_bytes())
        )
        .unwrap(),
        vec!["\"a", "b\"", " c"]
    );
}

#[test]
fn quoted_find_matching_close_object_mode() {
    // `}` inside a quoted key segment is opaque to balance counting.
    let input = "{\"a}b\": 1}";
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    // Unterminated span → no matching close.
    assert_eq!(find_matching_close("{\"a: 1}", b'{', b'}'), None);
    let input = "{a: 1, \"b}c\": 2}";
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    // Array-scope positions keep quotes as content (spec "Keys only"):
    // the `]` inside the quotes still closes the compound — no nested
    // object is involved.
    assert_eq!(find_matching_close("[\"a]b\"]", b'[', b']'), Some(3));
}

#[test]
fn quoted_find_matching_close_triple_nested() {
    // A `{` opens a fresh pair list at any nesting level, so quoted-key
    // recognition must be tracked per level (spec 0.7 § 5.3.3).
    let input = r#"{a: {b: {"c}d": 1}}}"#;
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
    let input = r#"{b: {"c}d": 1}}"#;
    assert_eq!(
        find_matching_close(input, b'{', b'}'),
        Some(input.len() - 1)
    );
}

#[test]
fn r3f1_split_top_level_trailing_ws_after_comma_no_phantom_segment() {
    // R3-F1: whitespace after a trailing comma sent `skip_segment_ws`
    // to EOF and the loop indexed out of bounds. The loop must end
    // without emitting a phantom whitespace-only segment.
    for tail in [" ", "\t", "\u{00a0}"] {
        let body = format!("\"a\": 1,{tail}");
        assert_eq!(
            split_top_level(
                &body,
                1,
                S,
                InlineBody::Object,
                InlineBounds::for_input(&body),
                has_quote_bytes(body.as_bytes()),
            )
            .unwrap(),
            vec!["\"a\": 1"]
        );
    }
    // Comma at EOF without whitespace: unchanged — the empty final
    // segment is still emitted and the caller accepts it.
    assert_eq!(
        split_top_level(
            "\"a\": 1,",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a\": 1,"),
            has_quote_bytes("\"a\": 1,".as_bytes())
        )
        .unwrap(),
        vec!["\"a\": 1", ""]
    );
    // Quoted key and quoted VALUE before the trailing comma.
    assert_eq!(
        split_top_level(
            "\"a b\": 1, ",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("\"a b\": 1, "),
            has_quote_bytes("\"a b\": 1, ".as_bytes())
        )
        .unwrap(),
        vec!["\"a b\": 1"]
    );
    assert_eq!(
        split_top_level(
            "a: \"x\", ",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: \"x\", "),
            has_quote_bytes("a: \"x\", ".as_bytes())
        )
        .unwrap(),
        vec!["a: \"x\""]
    );
}

// R3-F1's original catch was the PANIC (`b.` + whitespace to EOF sent
// `skip_segment_ws` past the end); keeping the no-panic behavior was
// correct. The Err(EmptyKey) category asserted here since R3 was not
// (R11-F1) — the review's own words: "the absence of panic was
// correct, the chosen category was not." Reaching `EofAfterWsSkip`
// proves the armed segment holds no unescaped separator, so
// `split_top_level` now pushes the raw remainder exactly like the
// `Exhausted` branch (and hence `SplitFast`) does; the caller's colon
// search then resolves the § 6.12 missing-separator category
// (MalformedInlineCompound), pinned at the full-parse level below.
#[test]
fn r3f1_split_top_level_dotted_key_trailing_ws_raw_last_segment() {
    // Dotted-key `.` + whitespace to EOF: the raw last segment is
    // pushed — byte-identical to what the quote-free fast machine
    // produces for the same shape.
    for tail in [" ", "\t"] {
        let body = format!(" \"a\": 1, b.{tail}");
        let seg2 = format!(" b.{tail}");
        assert_eq!(
            split_top_level(
                &body,
                1,
                S,
                InlineBody::Object,
                InlineBounds::for_input(&body),
                has_quote_bytes(body.as_bytes()),
            )
            .unwrap(),
            vec![" \"a\": 1", seg2.as_str()]
        );
    }
    // Leading-dot form, same shape (quote bytes present: slow path).
    assert_eq!(
        split_top_level(
            " \"a\". ",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input(" \"a\". "),
            has_quote_bytes(" \"a\". ".as_bytes()),
        )
        .unwrap(),
        vec![" \"a\". "]
    );
    // Dot followed by a real segment still works.
    assert_eq!(
        split_top_level(
            "a. b : 1",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a. b : 1"),
            has_quote_bytes("a. b : 1".as_bytes())
        )
        .unwrap(),
        vec!["a. b : 1"]
    );
    // The corrected category, resolved where it belongs: the full
    // parse finds no separator in the pushed raw segment and raises
    // MalformedInlineCompound (§ 6.12), not EmptyKey.
    match crate::parse("{\"a\": 1, b. }") {
        Err(crate::Error::Structured(crate::ErrorKind::MalformedInlineCompound { .. })) => {}
        other => panic!("expected MalformedInlineCompound, got {other:?}"),
    }
}

#[test]
fn r3f1_find_matching_close_eof_after_trailing_ws() {
    // R3-F1 sibling: untrimmed text ending in whitespace after a comma,
    // no closer — None, not an index panic.
    assert_eq!(find_matching_close("{\"a\": 1, ", b'{', b'}'), None);
    // Closer present after the whitespace: unchanged (skip stops at `}`).
    assert_eq!(find_matching_close("{\"a\": 1, }", b'{', b'}'), Some(9));
}

#[test]
fn r3f1_scan_inline_closer_eof_after_trailing_ws() {
    use crate::parser::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F1 sibling: untrimmed text ending in whitespace after a comma,
    // no closer on this text — NotFound, not an index panic.
    assert!(matches!(
        scan_inline_closer("{\"a\": 1, ", b'{', b'}', 1, S),
        InlineCloserScan::NotFound
    ));
    // Closer present after the whitespace: unchanged.
    assert!(matches!(
        scan_inline_closer("{\"a\": 1, }", b'{', b'}', 1, S),
        InlineCloserScan::Found(9)
    ));
}

#[test]
fn r3f1_find_unescaped_colon_inline_eof_after_dotted_ws() {
    // R3-F1 sibling: untrimmed text ending in whitespace after a dot —
    // None, not an index panic.
    assert_eq!(find_unescaped_colon_inline("a. "), None);
    // Dot then colon: unchanged.
    assert_eq!(find_unescaped_colon_inline("a. : 1"), Some(3));
}
