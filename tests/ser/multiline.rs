//! Multi-line string serialization — values containing `\n` are emitted
//! in the verbatim `(( ... ))` form for lossless round-trip.

use ktav::to_string;
use serde::Serialize;

#[test]
fn string_with_newline_uses_verbatim_form() {
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "line1\nline2".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\nline1\nline2\n))\n");
}

#[test]
fn string_with_trailing_newline_emits_extra_blank_line() {
    // The trailing '\n' is preserved via a blank line before `))` — on
    // read-back, the empty line after 'line2' contributes that newline.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "line1\nline2\n".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\nline1\nline2\n\n))\n");
}

#[test]
fn string_without_newline_uses_single_line_form() {
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "single line".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: single line\n");
}

#[test]
fn string_with_newline_inside_array_uses_verbatim_form() {
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg = Cfg {
        items: vec!["one".into(), "multi\nline".into()],
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "items: [\n    one\n    ((\nmulti\nline\n    ))\n]\n");
}

#[test]
fn exactly_one_newline_as_content_emits_blank_line() {
    // Content "\n" is one newline character. Serializer should emit:
    //   body: ((
    //
    //   ))
    // (blank line between `((` and `))`, because content ends with `\n`).
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg { body: "\n".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\n\n\n))\n");
}

#[test]
fn two_trailing_newlines_produce_two_blank_lines() {
    // Content "a\n\n" (final \n + explicit blank). Expect two \n after `a`
    // plus the extra blank-line marker.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "a\n\n".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\na\n\n\n))\n");
}

#[test]
fn no_trailing_newline_emits_no_extra_blank_line() {
    // Content without trailing \n must NOT produce an extra blank line —
    // regression test for a previous bug where both cases were treated the
    // same (double '\n' push).
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "a\nb".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\na\nb\n))\n");
}

#[test]
fn string_preserving_leading_indentation_round_trips_verbatim() {
    // Verbatim serialization: every character preserved exactly.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "   a\n      b".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\n   a\n      b\n))\n");
}
