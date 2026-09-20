//! rust#7: the thin event parser (`ktav::thin::parse_events`) has its
//! own independent key validation (`is_valid_key` in
//! `src/thin/event_parser.rs`) and must decode the same ten § 3.7 key
//! escapes as the owned parser (`tests/key_escaping.rs`) — `parse()`
//! and `from_str::<T>()` must never disagree on what is a valid key.

use ktav::thin::{parse_events, ParseEvent};

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
fn thin_parser_decodes_all_ten_key_escapes() {
    let mut failures = Vec::new();
    for (escape, decoded) in CASES {
        let src = format!("a{escape}b: 1\n");
        let expected_key = format!("a{decoded}b");
        let mut got_key: Option<String> = None;
        let result = parse_events(&src, |ev| {
            if let ParseEvent::Key(k) = ev {
                got_key = Some(k.to_string());
            }
        });
        match result {
            Ok(()) => {
                if got_key.as_deref() != Some(expected_key.as_str()) {
                    failures.push(format!(
                        "{escape:?}: expected key {expected_key:?}, got {got_key:?} (source: {src:?})"
                    ));
                }
            }
            Err(e) => failures.push(format!(
                "{escape:?}: parse_events failed: {e} (source: {src:?})"
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} key escapes broken in thin parser:\n{}",
        failures.len(),
        CASES.len(),
        failures.join("\n")
    );
}
