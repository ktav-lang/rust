//! Pins the `## Corners worth knowing` snippets rendered in the three
//! READMEs. Each claim there is a behavioural claim, so each one is
//! executed here rather than asserted in prose.
//!
//! These four corners — quoted keys, where escapes apply, the leading
//! BOM, and invalid UTF-8 — were undocumented until 2026-09-17 despite
//! being headline additions of the 0.7 specification. Writing them up
//! immediately corrected a wrong assumption: escapes do NOT decode in a
//! bare block-level value, only inside inline compounds and quoted
//! keys. Without running the examples the README would have said the
//! opposite.

use ktav::{parse, Value};

fn obj_keys(v: &Value) -> Vec<String> {
    match v {
        Value::Object(m) => m.keys().map(|k| k.as_str().to_string()).collect(),
        other => panic!("expected an Object, got {other:?}"),
    }
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    match v {
        Value::Object(m) => match m.get(key) {
            Some(Value::String(s)) => s.as_str(),
            other => panic!("{key}: expected a String, got {other:?}"),
        },
        other => panic!("expected an Object, got {other:?}"),
    }
}

/// A dot in a bare key nests; a quoted key keeps the dot in the name.
#[test]
fn quoted_keys_snippet() {
    let doc = parse("db.host: primary\n\"db.host\": literal\n\"a b\": spaces are fine too\n")
        .expect("snippet must parse");

    let keys = obj_keys(&doc);
    // `db` from the nesting form, then the two literal keys.
    assert_eq!(keys, vec!["db", "db.host", "a b"]);

    // The bare form really nested, rather than making a literal key.
    match &doc {
        Value::Object(m) => {
            let db = m.get("db").expect("db must exist");
            assert_eq!(str_at(db, "host"), "primary");
        }
        other => panic!("expected an Object, got {other:?}"),
    }

    assert_eq!(str_at(&doc, "db.host"), "literal");
    assert_eq!(str_at(&doc, "a b"), "spaces are fine too");
}

/// Escapes decode inside inline compounds and quoted keys, and NOT in a
/// bare block-level value. This is the corner the README would have got
/// wrong if the example had not been run.
#[test]
fn escapes_snippet() {
    let doc = parse("inline: {greek: \\u03b1, csv: a\\,b}\n\"\\u00e9\": 1\nliteral: \\u0041\n")
        .expect("snippet must parse");

    match &doc {
        Value::Object(m) => {
            let inline = m.get("inline").expect("inline must exist");
            assert_eq!(str_at(inline, "greek"), "α");
            // The escaped comma is content, not a separator: one entry,
            // not two.
            assert_eq!(str_at(inline, "csv"), "a,b");
            assert_eq!(obj_keys(inline), vec!["greek", "csv"]);
        }
        other => panic!("expected an Object, got {other:?}"),
    }

    // The quoted key decoded to a single é.
    assert!(obj_keys(&doc).contains(&"é".to_string()));

    // The bare value did NOT decode.
    assert_eq!(str_at(&doc, "literal"), "\\u0041");
}

/// Exactly one leading U+FEFF is skipped; anywhere else it is content.
#[test]
fn leading_bom_snippet() {
    let skipped = parse("\u{feff}a: 1\n").expect("leading BOM must be skipped");
    assert_eq!(obj_keys(&skipped), vec!["a"]);

    let kept = parse("a: 1\n\u{feff}b: 2\n").expect("a later BOM is ordinary content");
    assert_eq!(obj_keys(&kept), vec!["a", "\u{feff}b"]);
}
