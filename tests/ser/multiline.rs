//! Multi-line string serialization — values containing `\n` are emitted
//! in the indented stripped `( ... )` form for readability. The verbatim
//! `(( ... ))` form is used as a fallback only when the content has its
//! own leading whitespace (which stripped's dedent would clobber) or
//! contains a sole-`)` line that would close stripped prematurely.

use ktav::to_string;
use serde::Serialize;

#[test]
fn plain_multiline_uses_stripped_form_with_indent() {
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "line1\nline2".into(),
    };
    let s = to_string(&cfg).unwrap();
    // Each content line is indented by one level (4 spaces); dedent on
    // parse strips it back off for byte-exact round-trip.
    assert_eq!(s, "body: (\n    line1\n    line2\n)\n");
}

#[test]
fn string_with_trailing_newline_emits_extra_blank_line() {
    // The trailing '\n' is preserved via a blank line before `)` — on
    // read-back, the empty line after 'line2' contributes that newline.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "line1\nline2\n".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: (\n    line1\n    line2\n\n)\n");
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
fn string_with_newline_inside_array_uses_stripped_form() {
    #[derive(Serialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg = Cfg {
        items: vec!["one".into(), "multi\nline".into()],
    };
    let s = to_string(&cfg).unwrap();
    // Array item indent = 1 → content indent = 2 (8 spaces).
    assert_eq!(s, "items: [\n    one\n    (\n        multi\n        line\n    )\n]\n");
}

#[test]
fn string_with_leading_whitespace_falls_back_to_verbatim() {
    // Stripped's dedent would lose the original leading spaces, so we
    // fall back to verbatim which copies bytes literally.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "    indented\nplain".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\n    indented\nplain\n))\n");
}

#[test]
fn string_with_sole_closing_paren_line_falls_back_to_verbatim() {
    // A line containing only `)` would close the stripped form early.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "x\n)\ny".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: ((\nx\n)\ny\n))\n");
}

#[test]
fn exactly_one_newline_as_content_emits_blank_line() {
    // Content "\n" — one newline. In stripped form, blank lines stay blank
    // (no indent prefix), and the trailing \n is preserved by the empty
    // line before `)`.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg { body: "\n".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: (\n\n\n)\n");
}

#[test]
fn two_trailing_newlines_produce_two_blank_lines() {
    // Content "a\n\n" — `a`, then two trailing \n. In stripped form: indented
    // `a`, then two blank lines (preserve original \n count) before `)`.
    #[derive(Serialize)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "a\n\n".into(),
    };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "body: (\n    a\n\n\n)\n");
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
    assert_eq!(s, "body: (\n    a\n    b\n)\n");
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
