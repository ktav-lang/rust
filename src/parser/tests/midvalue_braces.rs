//! R3-F4 tests: mid-scalar braces have no structural meaning (spec 0.7 § 5.8.5).

use super::S;
use crate::parser::inline::{find_unescaped_colon_inline, has_quote_bytes, InlineBounds};
// --- R3-F4: mid-scalar braces have no structural meaning (spec 0.7 § 5.8.5) --

#[test]
fn r3f4_split_top_level_midvalue_balanced_brace_splits_at_inner_comma() {
    use crate::parser::inline::{split_top_level, InlineBody};
    // R3-F4: a balanced `{...}` mid-scalar must NOT be skipped wholesale;
    // the comma inside it is a real top-level separator (§ 5.8.5), and
    // the comma after the mid-scalar `}` also splits (no comma-shielding).
    assert_eq!(
        split_top_level(
            "a: x{y,z}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: x{y,z}, b: 2"),
            has_quote_bytes("a: x{y,z}, b: 2".as_bytes())
        )
        .unwrap(),
        vec!["a: x{y", "z}", " b: 2"]
    );
}

#[test]
fn r3f4_split_top_level_midvalue_brace_quote_aware_slow_path() {
    use crate::parser::inline::{split_top_level, InlineBody};
    // R3-F4: same rule on the quote-aware slow path.
    assert_eq!(
        split_top_level(
            "k: \"v\", a: x{y,z}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("k: \"v\", a: x{y,z}, b: 2"),
            has_quote_bytes("k: \"v\", a: x{y,z}, b: 2".as_bytes())
        )
        .unwrap(),
        vec!["k: \"v\"", " a: x{y", "z}", " b: 2"]
    );
}

#[test]
fn r3f4_split_top_level_array_body_midvalue_brace_splits_at_inner_comma() {
    use crate::parser::inline::{split_top_level, InlineBody};
    // R3-F4: array bodies share the fast splitter and the rule.
    assert_eq!(
        split_top_level(
            "x{y,z}, 2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("x{y,z}, 2"),
            has_quote_bytes("x{y,z}, 2".as_bytes())
        )
        .unwrap(),
        vec!["x{y", "z}", " 2"]
    );
}

#[test]
fn r3f4_split_top_level_genuine_value_start_compounds_guard() {
    use crate::parser::inline::{split_top_level, InlineBody};
    // R3-F4 guards: a compound that IS the first code point of a value
    // keeps its structural meaning — no split inside it.
    assert_eq!(
        split_top_level(
            "{a: 1}, 2",
            1,
            S,
            InlineBody::Array,
            InlineBounds::for_input("{a: 1}, 2"),
            has_quote_bytes("{a: 1}, 2".as_bytes())
        )
        .unwrap(),
        vec!["{a: 1}", " 2"]
    );
    assert_eq!(
        split_top_level(
            "a: {y: 1}, b: 2",
            1,
            S,
            InlineBody::Object,
            InlineBounds::for_input("a: {y: 1}, b: 2"),
            has_quote_bytes("a: {y: 1}, b: 2".as_bytes())
        )
        .unwrap(),
        vec!["a: {y: 1}", " b: 2"]
    );
}

#[test]
fn r3f4_scan_inline_closer_midvalue_balanced_brace_is_body_closer() {
    use crate::parser::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4: the `}` after `z` (the mid-scalar close) is the body's
    // closer — the scan must not treat the balanced span as opaque.
    assert!(matches!(
        scan_inline_closer("{a: x{y,z}, b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::Found(9)
    ));
}

#[test]
fn r3f4_scan_inline_closer_crossed_bracket_not_found() {
    use crate::parser::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4: the crossed `]` mid-scalar is not a matching `}` closer.
    assert!(matches!(
        scan_inline_closer("{a: x[y,z], b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::NotFound
    ));
}

#[test]
fn r3f4_scan_inline_closer_genuine_compounds_guard() {
    use crate::parser::inline::{scan_inline_closer, InlineCloserScan};
    // R3-F4 guards: value-start compounds still find their real closer.
    assert!(matches!(
        scan_inline_closer("{a: {y: 1}, b: 2}", b'{', b'}', 1, S),
        InlineCloserScan::Found(16)
    ));
    assert!(matches!(
        scan_inline_closer("[{a: 1}, 2]", b'[', b']', 1, S),
        InlineCloserScan::Found(10)
    ));
}

// R8 regression: the InlineBounds memo must be a PURE memo — every
// recorded (opener, closer) pair must be exactly what the live
// dispatches compute over the same span. The recording walk once
// popped a frame at a later kind-matched closer even though a crossed
// closer inside the span had already returned the span's own depth to
// zero (the byte where the standalone scan stops with `NotFound`), so
// the memo said `Found` where the live scan said `NotFound` and split
// segmented differently (observable: different
// MalformedInlineCompound detail payloads, fuzz2-confirmed). This test
// cross-checks every recorded pair against BOTH live dispatches over
// exactly the shapes that fired, plus an exhaustive sweep of short
// structural bodies.
#[test]
fn memo_bounds_are_a_pure_memo_of_the_live_dispatches() {
    use crate::parser::inline::{
        find_matching_close, scan_inline_closer, scan_inline_closer_with_bounds, InlineCloserScan,
    };

    // Shapes that fired during the audit / differential fuzz (the four
    // MEMO_MISMATCH audit slices, embedded in a value position so the
    // gate walk actually records spans around them, and the four
    // fuzz2-diverging documents).
    let hostile = [
        "{a: {]}{, ,x{x,}",
        "{a: {[][x]}]}n\"}",
        "{a: {x   ]}, 2}",
        "{a: {]a{ :x}}",
        "{a: [a],{:[,:,}]}",
        "{a: [ a:,:[],{:[}, ]}",
        "{a: [],{[:[,}]}",
        "{a: [],[:[},a[{ {,a ]}",
        // R8-F2: the five review inputs. Pre-fix the quote-free
        // members let the fast gate walk phantom-open an Array
        // after the closed `[]` (value_start residue) and record /
        // verdict differently from their quote-bearing twins.
        "[[][text]",
        "[[]['text]",
        "[{}[text]",
        "{a: [[][text]}",
        "{a: [[][text], q: '}",
    ];

    let check = |body: &str| {
        let bytes = body.as_bytes();
        let (open, close) = if bytes[0] == b'[' {
            (b'[', b']')
        } else {
            (b'{', b'}')
        };
        let mut pairs = Vec::new();
        let verdict = scan_inline_closer_with_bounds(body, open, close, 0, S, &mut pairs);
        // Recording only happens on a Found gate; nothing to check otherwise.
        if !matches!(verdict, InlineCloserScan::Found(_)) {
            assert!(
                pairs.is_empty(),
                "bounds recorded on a non-Found gate: {body:?} {pairs:?}"
            );
            return;
        }
        for &(o, c) in &pairs {
            let ob = bytes[o];
            let (so, sc) = if ob == b'[' {
                (b'[', b']')
            } else {
                (b'{', b'}')
            };
            // BOTH consumer shapes must agree with the memo:
            // - the SUFFIX `&body[o..]` is what split's opener jump
            //   sub-scans (`scan_inline_closer(&input[i..], ...)`), and
            // - the BOUNDED VALUE `&body[o..=c]` is what the find-first
            //   dispatch hands to `known_closer` / `find_matching_close`
            //   / `scan_inline_closer`. R8-F2: checking the suffix alone
            //   let a quote-aware memo agree with a quote-aware re-scan
            //   while the actual consumer re-scanned the bounded value
            //   in quote-free fast mode. For the bounded shape the
            //   recorded closer is the last byte, so the live dispatch
            //   must find it exactly there.
            for (shape_name, span) in [("suffix", &body[o..]), ("bounded", &body[o..=c])] {
                // `c - o` is the recorded closer's index in both shapes;
                // for the bounded shape it is also the last byte, which
                // is exactly the `idx == len - 1` closed-compound read
                // the find-first dispatch gives a memo hit.
                assert!(
                    matches!(
                        scan_inline_closer(span, so, sc, 0, S),
                        InlineCloserScan::Found(f) if f == c - o
                    ),
                    "scan ({shape_name}) disagrees with the memo at {o}..{c} in {body:?}"
                );
                assert_eq!(
                    find_matching_close(span, so, sc),
                    Some(c - o),
                    "find ({shape_name}) disagrees with the memo at {o}..{c} in {body:?}"
                );
            }
        }
    };

    for b in hostile {
        check(b);
    }

    // Exhaustive sweep over short structural bodies (depth, crossed
    // closers, mid-scalar openers, raw markers, top-level commas).
    // R8-F2: the alphabet MUST carry quote bytes (all three § 5.3.3
    // delimiters) and § 3.3 whitespace — without them every body is
    // quote-free, the gate always picks the fast walk, and the whole
    // quote-aware memo surface (ScanQ recordings consumed by a
    // quote-free consumer slice) is unreachable. That blind spot is
    // why the R8-F2 family survived this test.
    let alpha: &[u8] = b"{[]}a:,.'\"` ";
    let mut buf = [0u8; 5];
    for len in 1..=5usize {
        let total = alpha.len().pow(len as u32);
        for mut idx in 0..total {
            for d in (0..len).rev() {
                buf[d] = alpha[idx % alpha.len()];
                idx /= alpha.len();
            }
            let body = std::str::from_utf8(&buf[..len]).unwrap();
            // Only bodies the gate scan accepts as inline compounds build a map.
            if body.starts_with('{') || body.starts_with('[') {
                check(body);
            }
        }
    }
}

// R8 regression pin: the fuzz2-diverging documents must keep the
// pre-R8 segmentation (ground truth probed from main @ 4477ae2). The
// memo bug changed only the `detail` payload (which segment the
// diagnostic quoted), so the pins cover the payload exactly. The
// fourth former member of this list is pinned separately below: R8-F2
// legitimately moved its whole category, not just its detail.
// R9-F1 reclassification: all three inputs moved a second time, for an
// independent spec reason. The old MalformedInlineCompound verdict
// depended on former quirk 4 — find/scan commas set `value_start = true`
// even in Object scopes — so the `{` after the Object comma
// phantom-opened a compound (§ 5.8.5) that consumed a `}` and let the
// walk reach the body's own closer; the pair split then ran and quoted
// a missing-separator detail. § 4 excludes brackets from `<key-char>`
// and `<inline-pair>` begins with `<key>`, so the position after an
// Object comma is a KEY position and the `{` there is NOT a
// value-position opener. Without the phantom scope a `]` returns the
// shared depth to zero first; § 5.2's matching-closer rule requires a
// depth-0 closer to be the body's own kind, so there is no same-line
// matching closer and § 6.11 diagnoses UnterminatedInlineCompound —
// the identical reading the quote-aware sibling below pins for
// `k: {a: [],[:[},a[{ {,a ]}` since R8-F2.
#[test]
fn r9f1_crossed_closer_fuzz_inputs_reclassified_by_key_position() {
    let cases = [
        "k: {a: [a],{:[,:,}]}",
        "k: {a: [ a:,:[],{:[}, ]}",
        "k: {a: [],{[:[,}]}",
    ];
    for input in cases {
        let expect_unterminated = |res: Result<(), crate::Error>| {
            assert!(
                matches!(
                    res,
                    Err(crate::Error::Structured(
                        crate::ErrorKind::UnterminatedInlineCompound { .. }
                    ))
                ),
                "input {input:?}: expected UnterminatedInlineCompound, got {res:?}"
            );
        };
        expect_unterminated(crate::parse(input).map(|_| ()));
        expect_unterminated(crate::parse_strict(input).map(|_| ()));
        expect_unterminated(crate::from_str::<serde_json::Value>(input).map(|_| ()));
        expect_unterminated(crate::parse_events(input, |_| {}).map(|_| ()));
    }
}

// R9-F1: an unescaped `[`/`{` in an inline-KEY position is a forbidden
// `<key-char>` (§ 4), so once the separator is located the pair is
// diagnosed as InvalidKey (§ 6.4 via § 5.3.1's bare-segment rule) —
// never as a compound-shape error. Two mechanisms carried the old
// verdicts: (a) find/scan commas set `value_start = true` even in
// Object scopes, so the opener gate admitted the bracket as a
// value-position compound opener whose closer then swallowed the
// body's own (UnterminatedInlineCompound) — § 4 puts the `<key>`
// production first in `<inline-pair>`, so a comma's successor position
// in an Object is a key position; (b) the inline pair's colon scan
// counted `{`/`[` as depth and hid the separator that is actually
// present (MalformedInlineCompound "missing ':'") — § 5.3.1: "Validation
// operates on the raw prefix up to the first unescaped separator,
// however malformed the separator's surrounding whitespace is".
#[test]
fn r9f1_key_bracket_is_invalid_key_not_phantom_compound() {
    use crate::ErrorKind;
    let cases = [
        // First pair (mechanism (b) only).
        "{[a: 1}",
        "{{a: 1}",
        // Later pair after an Object comma (mechanism (a) in the shape
        // scan and the split, then (b) in the pair split). A quote in a
        // neighbouring value must not change the diagnosis (§ 5.3.3:
        // quotes in value positions are ordinary content).
        "{x: 0, [a: 1}",
        "{x: 0, {a: 1}",
        "{x: 0, [a: 1, q: '}",
    ];
    for input in cases {
        let expect_invalid_key = |res: Result<(), crate::Error>| match res {
            Err(crate::Error::Structured(ErrorKind::InvalidKey { key, .. })) => {
                assert!(
                    key.starts_with('[') || key.starts_with('{'),
                    "input {input:?}: unexpected offending key {key:?}"
                );
            }
            other => panic!("input {input:?}: expected InvalidKey, got {other:?}"),
        };
        expect_invalid_key(crate::parse(input).map(|_| ()));
        expect_invalid_key(crate::parse_strict(input).map(|_| ()));
        expect_invalid_key(crate::from_str::<serde_json::Value>(input).map(|_| ()));
        expect_invalid_key(crate::parse_events(input, |_| {}).map(|_| ()));
    }
}

// R9-F1 positive controls: the fix must not touch legitimate brackets.
// - `\[` is a § 3.7 escape form: the decoded key is the ordinary
//   single-segment key `[a` (decode-time semantics unchanged).
// - `[1]` after `a:` IS a value position (§ 5.8.5): the value is a
//   real nested Array; likewise a nested Object value.
// - Quoted-key opacity (§ 5.3.3) stays exactly as it is: the `}` inside
//   the quoted segment is content, and a quoted key may quote brackets.
#[test]
fn r9f1_key_bracket_positive_controls_unchanged() {
    use crate::Value;

    let v = crate::parse(r"{x: 0, \[a: 1}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("x"), Some(&Value::Integer("0".into())));
    assert_eq!(obj.get("[a"), Some(&Value::Integer("1".into())));

    let v = crate::parse(r"{\[a: 1}").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("[a"),
        Some(&Value::Integer("1".into()))
    );

    let v = crate::parse("{x: 0, a: [1]}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("x"), Some(&Value::Integer("0".into())));
    assert_eq!(
        obj.get("a"),
        Some(&Value::Array(vec![Value::Integer("1".into())]))
    );

    let v = crate::parse("{x: 0, a: {b: 2}}").unwrap();
    let inner = v
        .as_object()
        .unwrap()
        .get("a")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(inner.get("b"), Some(&Value::Integer("2".into())));

    // § 5.3.3's own example, verbatim: the quoted key's `}` is content.
    let v = crate::parse("{\"a}b\": 1, c: 2}").unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("a}b"), Some(&Value::Integer("1".into())));
    assert_eq!(obj.get("c"), Some(&Value::Integer("2".into())));

    // A quoted key may quote brackets; the quoted span stays opaque.
    let v = crate::parse("{\"[x]\": 1}").unwrap();
    assert_eq!(
        v.as_object().unwrap().get("[x]"),
        Some(&Value::Integer("1".into()))
    );

    // Same values via the event entry point.
    assert!(crate::parse_events(r"{x: 0, \[a: 1}", |_| {}).is_ok());
    assert!(crate::parse_events("{x: 0, a: [1]}", |_| {}).is_ok());
    assert!(crate::parse_events("{\"a}b\": 1, c: 2}", |_| {}).is_ok());
}

// R9-F1 precedence: a forbidden bracket AND a malformed escape in the
// same inline compound. § 5.2's rules-6–9 preamble (spec: "the result
// is BadEscapeSequence (§ 6.13), which takes precedence over a missing
// closer") fires while the compound triage scans for the closer —
// before any pair is split — so the escape wins inside `{...}`. The
// multiline pair path runs no compound triage; § 5.3.1's bare-segment
// listing orders InvalidKey (raw forbidden `<key-char>`) before
// BadEscapeSequence (malformed `\X`) within one segment, and the shared
// validator checks forbidden raw bytes before decoding, so `a\q[` is
// InvalidKey.
#[test]
fn r9f1_forbidden_bracket_and_bad_escape_precedence() {
    let expect_bad_escape = |res: Result<(), crate::Error>| {
        assert!(
            matches!(
                res,
                Err(crate::Error::Structured(
                    crate::ErrorKind::BadEscapeSequence { .. }
                ))
            ),
            "expected BadEscapeSequence, got {res:?}"
        );
    };
    expect_bad_escape(crate::parse(r"{[a\q: 1}").map(|_| ()));
    expect_bad_escape(crate::parse_strict(r"{[a\q: 1}").map(|_| ()));
    expect_bad_escape(crate::from_str::<serde_json::Value>(r"{[a\q: 1}").map(|_| ()));
    expect_bad_escape(crate::parse_events(r"{[a\q: 1}", |_| {}).map(|_| ()));

    let err = crate::parse(r"a\q[ : 1").expect_err("must be InvalidKey");
    assert!(
        matches!(
            err,
            crate::Error::Structured(crate::ErrorKind::InvalidKey { .. })
        ),
        "expected InvalidKey, got {err:?}"
    );
}

// R9-F1 unit pin: the inline pair's separator scan shares the § 4/§ 5.3
// key-separator scanner — NO compound depth in the key. A bracket in
// the key prefix must not hide the separator that is actually present.
#[test]
fn r9f1_find_unescaped_colon_inline_ignores_key_depth() {
    assert_eq!(find_unescaped_colon_inline("[a: 1"), Some(2));
    assert_eq!(find_unescaped_colon_inline("{a: 1"), Some(2));
    // Quoted bracket content is skipped wholesale; the span's end is
    // followed by the separator.
    assert_eq!(find_unescaped_colon_inline("\"[a\": 1"), Some(4));
    // A colon inside a nested VALUE compound never wins: the separator
    // precedes the value, which is why dropping key-side depth is safe.
    assert_eq!(find_unescaped_colon_inline("a: {b: 1}"), Some(1));
    assert_eq!(find_unescaped_colon_inline("a: [1, {c: 2}]"), Some(1));
}

// R8-F2 recategorization of the fourth formerly-pinned document:
// `k: {a: [],[:[},a[{ {,a ]}` reached the pair-split path only via the
// FAST gate walk, which carried `in_key = true` residue through the
// gated `[` after the top-level comma (former quirk 2) and
// phantom-opened the `[:[` Array as a nested compound, shifting the
// depth accounting so the body's own `}` seemed to close it. The
// quote-aware machine sets `in_key = false` at every opener, so the
// `}` after `[:[` matches no scope kind and the `]` before the final
// `}` is a CROSSED closer at depth 0 — § 5.2's matching-closer rule
// yields no same-line matching closer, which § 6.11 diagnoses as
// UnterminatedInlineCompound. The fast machine now agrees (R8-F2), so
// every entry point reports UnterminatedInlineCompound.
#[test]
fn r8f2_fast_in_key_residue_no_longer_recategorizes_crossed_closer() {
    let input = "k: {a: [],[:[},a[{ {,a ]}";
    let expect_unterminated = |res: Result<(), crate::Error>| {
        assert!(
            matches!(
                res,
                Err(crate::Error::Structured(
                    crate::ErrorKind::UnterminatedInlineCompound { .. }
                ))
            ),
            "input {input:?}: expected UnterminatedInlineCompound"
        );
    };
    expect_unterminated(crate::parse(input).map(|_| ()));
    expect_unterminated(crate::parse_strict(input).map(|_| ()));
    expect_unterminated(crate::parse_events(input, |_| {}).map(|_| ()));
}

// R8-F2 regression: the five review-round-8 inputs. Every one is an
// INVALID document, and the quote-aware reading gives the same
// category for all five. The value position is already consumed by the
// closed inner compound (the empty `[]` / `{}`), so the trailing
// `[text` is content after a closed value, not an unterminated one:
//
// - § 5.8.5: "The decision is made once, when the parser begins
//   reading an inline value: if the first non-whitespace code point is
//   `{` or `[`, the value is a nested compound; otherwise the value is
//   an inline scalar that runs to the next unescaped `,` / `}` / `]`"
//   — after the inner compound closes there is no open value for a
//   following `[` to open.
// - § 6.12: "Non-whitespace content after the same-line matching
//   closer of a value-position compound ... The closer makes the
//   compound closed; the trailing bytes are therefore malformed
//   content, not an unterminated compound." The enclosing compound
//   itself still closes on the same line, so the defect is
//   MalformedInlineCompound, not UnterminatedInlineCompound.
// - § 5.3.3 ("Keys only"): a quote character in a value position
//   "is ordinary content with no special meaning" — the later
//   unrelated `'` in the last input must not change any earlier
//   byte's role, so the quote-free and quote-bearing twins MUST get
//   the same verdict.
//
// Pre-R8-F2 the fast walks left `value_start` set after the empty
// Array closed (former quirk 1) and carried `in_key` residue through
// `[` openers (former quirk 2), phantom-opened a nested compound at
// the next `[`, swallowed the enclosing body's own closer, and
// reported UnterminatedInlineCompound for the quote-free twins
// (`[[][text]`, `{a: [[][text]}`) while the quote-bearing twins
// (`[[]['text]`, `{a: [[][text], q: '}`) already reported
// MalformedInlineCompound — the same bytes, two verdicts, decided by
// a later unrelated quote.
#[test]
fn r8f2_closed_empty_array_consumes_value_start_across_modes() {
    let cases = [
        "[[][text]",
        "[[]['text]",
        "[{}[text]",
        "{a: [[][text]}",
        "{a: [[][text], q: '}",
    ];
    for input in cases {
        let expect_malformed = |res: Result<(), crate::Error>| {
            assert!(
                matches!(
                    res,
                    Err(crate::Error::Structured(
                        crate::ErrorKind::MalformedInlineCompound { .. }
                    ))
                ),
                "input {input:?}: expected MalformedInlineCompound"
            );
        };
        expect_malformed(crate::parse(input).map(|_| ()));
        expect_malformed(crate::parse_strict(input).map(|_| ()));
        expect_malformed(crate::parse_events(input, |_| {}).map(|_| ()));
    }
}
