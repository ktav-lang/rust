//! The `message` field: `Display` rendered verbatim, last on the wire.

use crate::helpers::{env, n, obj, ten_keys_present};
use ktav::{emit_canonical, parse_strict, Error};
use serde_json::Value as Json;

// ---------------------------------------------------------------------------
// `message` — the field that stops bindings rendering their own
// ---------------------------------------------------------------------------

/// `message` is `Error`'s `Display`, verbatim, for every error shape —
/// parse-time and writer-time alike. This is the whole point of the
/// field: before it existed the envelope carried structured data but no
/// prose, so five bindings each invented a rendering and all five
/// disagreed with each other and with what Rust itself printed.
#[test]
fn message_is_the_display_rendering_verbatim() {
    // Parse-time, across several kinds.
    // Each of these really is rejected — checked, not assumed. A line
    // like `a:1` on its own is a valid one-element Array under the
    // § 5.0.1 root-kind rule, not a malformed pair, so it needs a
    // preceding pair line to land in an object context first.
    let parse_cases: &[&str] = &[
        "a: 1.10\n",    // LossyScalar (strict only)
        "a: 1\nb:2\n",  // MissingSeparatorSpace
        "a: 1\na: 2\n", // DuplicateKey
        "a: {\n",       // UnclosedCompound
    ];
    for src in parse_cases {
        let err = parse_strict(src).unwrap_err();
        let e = env(&err, src);
        assert_eq!(
            e.message,
            err.to_string(),
            "message must equal Display for {src:?}"
        );
        assert!(!e.message.is_empty(), "message is never empty");
    }

    // Writer-time: no span, but still a rendering.
    let err = emit_canonical(&obj(&[("a", obj(&[("", n(1))]))])).unwrap_err();
    let e = env(&err, "");
    assert_eq!(e.message, err.to_string());
    assert_eq!(e.line, None, "writer-time still emits honest nulls");

    // Top-level Message keeps its text on BOTH `body` and `message`:
    // `body` is the wire payload, `message` is what a host displays.
    let err = Error::Message("boom".to_string());
    let e = env(&err, "");
    assert_eq!(e.body, Some("boom".to_string()));
    assert_eq!(e.message, "boom");
}

/// `message` survives JSON rendering with its escaping intact, and sits
/// last so the nine original fields keep their shipped positions.
#[test]
fn message_round_trips_through_json_and_comes_last() {
    let src = "a: 1.10\n";
    let err = parse_strict(src).unwrap_err();
    let e = env(&err, src);
    let json = e.to_json();
    ten_keys_present(&json);

    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["message"], err.to_string());

    // A control character in the payload must not break the JSON.
    let err = Error::Message("tab\there\nand \"quotes\"".to_string());
    let json = env(&err, "").to_json();
    let v: Json = serde_json::from_str(&json).expect("still valid JSON");
    assert_eq!(v["message"], "tab\there\nand \"quotes\"");
}
