//! rust#7: § 3.7 / § 4 define ten key escape sequences. All ten must
//! decode to their literal byte in the owned parser (`ktav::parse`).
//! Before the fix, `is_valid_key` re-validated the *decoded* key and
//! rejected `\,` `\{` `\}` `\[` `\]` `\n` `\r` even though they arrived
//! through a legitimate escape — only `\\`, `\.`, `\:` worked.

use ktav::{parse, Value};

const CASES: &[(&str, &str)] = &[
    (r"\\", "\\"),
    (r"\,", ","),
    (r"\}", "}"),
    (r"\]", "]"),
    (r"\{", "{"),
    (r"\[", "["),
    (r"\n", "\n"),
    (r"\r", "\r"),
    (r"\.", "."),
    (r"\:", ":"),
];

#[test]
fn owned_parser_decodes_all_ten_key_escapes() {
    let mut failures = Vec::new();
    for (escape, decoded) in CASES {
        let src = format!("a{escape}b: 1\n");
        let expected_key = format!("a{decoded}b");
        match parse(&src) {
            Ok(Value::Object(obj)) => {
                let got = obj.keys().next().map(|k| k.as_str());
                if got != Some(expected_key.as_str()) {
                    failures.push(format!(
                        "{escape:?}: expected key {expected_key:?}, got {got:?} (source: {src:?})"
                    ));
                }
            }
            Ok(other) => failures.push(format!("{escape:?}: expected Object, got {other:?}")),
            Err(e) => failures.push(format!("{escape:?}: parse failed: {e} (source: {src:?})")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} key escapes broken:\n{}",
        failures.len(),
        CASES.len(),
        failures.join("\n")
    );
}

/// The dotted-path splitter must treat an escaped `\.` as literal, not
/// as a path separator — this already worked before the fix (`\.` was
/// one of the three that decoded correctly) but is pinned here
/// alongside the other nine so a future regression in the split logic
/// shows up in the same place.
#[test]
fn escaped_dot_is_not_a_path_separator() {
    let value = parse("a\\.b: 1\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object root");
    };
    assert_eq!(obj.len(), 1);
    assert_eq!(obj.keys().next().unwrap().as_str(), "a.b");
}

/// A genuinely dotted key alongside an escaped-dot segment must still
/// nest correctly: `a.b\.c: 1` is path `a` -> `b.c` (one literal dot
/// inside the second segment), not `a` -> `b` -> `c`.
#[test]
fn dotted_path_and_escaped_dot_combine_correctly() {
    let value = parse("a.b\\.c: 1\n").unwrap();
    let Value::Object(obj) = value else {
        panic!("expected Object root");
    };
    let Some(Value::Object(inner)) = obj.get("a") else {
        panic!("expected nested Object at `a`");
    };
    assert_eq!(inner.keys().next().unwrap().as_str(), "b.c");
}
