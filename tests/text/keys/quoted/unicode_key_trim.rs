//! Regression tests for rust#6 — Unicode-whitespace key trimming.
//!
//! Spec 0.7 resolves the § 4 key-trimming question by WIDENING to Unicode:
//! a key segment is trimmed of leading/trailing whitespace from the fixed
//! § 3.3 set (the Unicode `White_Space` property, 25 code points).
//!
//! Pinned here:
//! - bare key segments are trimmed of Unicode whitespace on both edges
//!   (§ 4);
//! - quoted segment content is NEVER trimmed (§ 5.3.3 — the deliberate
//!   contrast);
//! - canonical (§ 5.9.10 rule (a)) and plain writer emission choose the
//!   QUOTED form for a key whose edge character is a § 3.3 whitespace
//!   code point other than LF/CR, so NBSP etc. survive a round-trip
//!   without silent trimming data loss;
//! - bare keys differing only by a trimmed Unicode-whitespace edge
//!   collide (`DuplicateKey`, § 5.5 / Appendix A 0.7 entry), while a bare
//!   key and a quoted NBSP-bearing key stay distinct.

use ktav::render::{emit_canonical, render};
use ktav::{Error, ErrorKind, ObjectMap, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert((*k).into(), v.clone());
    }
    Value::Object(m)
}

/// Parse and assert exactly one pair `(key, "v")`, comparing via a sorted
/// Vec of (key, value) pairs rather than `ObjectMap`'s Debug output.
fn assert_single_pair(case: &str, text: &str, expected_key: &str) {
    let v = ktav::parse(text).unwrap_or_else(|e| panic!("parse({case}, {text:?}): {e}"));
    let pairs = match &v {
        Value::Object(m) => {
            let mut pairs: Vec<(String, String)> = m
                .iter()
                .map(|(k, val)| match val {
                    Value::String(sv) => (k.as_str().to_owned(), sv.as_str().to_owned()),
                    other => panic!("expected String value for {k:?}, got {other:?}"),
                })
                .collect();
            pairs.sort();
            pairs
        }
        other => panic!("expected Object from {case} ({text:?}), got {other:?}"),
    };
    assert_eq!(
        pairs,
        vec![(expected_key.to_owned(), "v".to_owned())],
        "key pairs from {case} ({text:?})"
    );
}

/// Assert the parse fails with `ErrorKind::DuplicateKey`.
fn assert_duplicate_key(text: &str) {
    match ktav::parse(text) {
        Err(Error::Structured(ErrorKind::DuplicateKey { .. })) => {}
        other => panic!("expected DuplicateKey from {text:?}, got {other:?}"),
    }
}

/// Emit side (§ 5.9.10 rule (a)): a key with a Unicode-whitespace edge
/// must be emitted in quoted form and survive a canonical round-trip.
fn assert_quoted_emission(surface: &dyn Fn(&Value) -> Result<String, Error>, edge: &str, cp: &str) {
    let key = format!("{edge}isAdmin");
    let v = obj(&[(key.as_str(), s("v"))]);

    let text = surface(&v).unwrap_or_else(|e| panic!("emit({cp}-edged key): {e}"));
    let quoted = format!("\"{key}\"");
    assert!(
        text.contains(&quoted),
        "emitted text for {cp}-edged key must contain quoted spelling {quoted:?}, got {text:?}"
    );

    let back = ktav::parse(&text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    assert_eq!(back, v, "round-trip of {cp}-edged key must be identity");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn bare_key_unicode_whitespace_edges_are_trimmed() {
    const CASES: &[(&str, char)] = &[
        ("NBSP U+00A0", '\u{00A0}'),
        ("NEL U+0085", '\u{0085}'),
        ("IDEOGRAPHIC SPACE U+3000", '\u{3000}'),
        ("LINE SEPARATOR U+2028", '\u{2028}'),
    ];

    for (name, c) in CASES {
        // leading edge
        assert_single_pair(
            &format!("{name} leading"),
            &format!("{c}isAdmin: v"),
            "isAdmin",
        );
        // trailing edge
        assert_single_pair(
            &format!("{name} trailing"),
            &format!("isAdmin{c}: v"),
            "isAdmin",
        );
        // both edges
        assert_single_pair(
            &format!("{name} both edges"),
            &format!("{c}isAdmin{c}: v"),
            "isAdmin",
        );
    }
}

#[test]
fn quoted_key_unicode_whitespace_edges_are_preserved() {
    // § 5.3.3: quoted segment content is never trimmed.
    assert_single_pair("leading NBSP", "\"\u{00A0}isAdmin\": v", "\u{00A0}isAdmin");
    assert_single_pair("trailing NBSP", "\"isAdmin\u{00A0}\": v", "isAdmin\u{00A0}");
    assert_single_pair(
        "leading U+3000",
        "\"\u{3000}isAdmin\": v",
        "\u{3000}isAdmin",
    );
}

#[test]
fn emit_canonical_quotes_key_with_unicode_whitespace_edge() {
    assert_quoted_emission(&emit_canonical, "\u{00A0}", "NBSP U+00A0");
    assert_quoted_emission(&emit_canonical, "\u{3000}", "U+3000");
    // The plain writer surface shares the key-form helper.
    assert_quoted_emission(&render, "\u{00A0}", "NBSP U+00A0 (render)");
}

#[test]
fn bare_keys_differing_only_by_unicode_ws_edge_collide() {
    // § 5.5 / Appendix A 0.7: bare keys differing only by a trimmed
    // Unicode-whitespace edge collide.
    assert_duplicate_key("isAdmin: v1\n\u{00A0}isAdmin: v2");
    assert_duplicate_key("\u{00A0}isAdmin: v2\nisAdmin: v1");
    assert_duplicate_key("isAdmin: v1\nisAdmin\u{3000}: v2");
}

#[test]
fn quoted_and_bare_keys_with_unicode_ws_edge_stay_distinct() {
    let text = "isAdmin: v1\n\"\u{00A0}isAdmin\": v2";
    let v = ktav::parse(text).unwrap_or_else(|e| panic!("parse({text:?}): {e}"));
    match &v {
        Value::Object(m) => assert_eq!(
            m.iter().count(),
            2,
            "bare and quoted NBSP-edged keys must stay distinct: {m:?}"
        ),
        other => panic!("expected Object from {text:?}, got {other:?}"),
    }
}

#[test]
fn duplicate_quoted_keys_still_collide() {
    assert_duplicate_key("\"\u{00A0}isAdmin\": v1\n\"\u{00A0}isAdmin\": v2");
}
