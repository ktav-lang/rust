//! Strings with unusual but legal content: empty, colons, hashes in the
//! middle, unicode, etc.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cfg {
    x: String,
}

#[test]
fn empty_string_round_trips() {
    let cfg = Cfg { x: "".into() };
    let s = to_string(&cfg).unwrap();
    assert_eq!(s, "x: \n");
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn string_with_colons_inside_survives() {
    let cfg = Cfg {
        x: "http://example.com:8080/path".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn string_with_hash_in_middle_is_not_comment() {
    let cfg = Cfg { x: "a#b".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn string_starting_with_hash_after_colon() {
    // Parser splits on FIRST `:`. After that, `#` is ordinary content.
    // Serializer writes `x: #tag`. Parser reads `x`, `:`, ` #tag`.
    // The `#` comes AFTER the `:`, so it's not a comment.
    let cfg = Cfg { x: "#tag".into() };
    let s = to_string(&cfg).unwrap();
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn unicode_values_round_trip() {
    let cfg = Cfg {
        x: "Шалом, мир! 🌍 ❤".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn single_char_values() {
    for c in ['a', 'Z', '0', '/', '-', '_', '.'] {
        let cfg = Cfg { x: c.to_string() };
        let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, back, "char: {:?}", c);
    }
}

#[test]
fn strings_ending_with_bracket_chars() {
    // Trailing `]` in a scalar value: serializer writes it as-is.
    // The PARSER has a special case — a line that starts with `[` and
    // ends with `]` is treated as an empty-array marker (for the `[]`
    // form); but a line whose VALUE (post-colon) ends with `]` but
    // doesn't also start with `[` is just a scalar. Test both.
    for s in ["name]", "a]b]c", "value)"] {
        let cfg = Cfg { x: s.into() };
        let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
        assert_eq!(cfg, back, "value: {:?}", s);
    }
}

#[test]
fn string_with_many_colons() {
    let cfg = Cfg { x: ":::::".into() };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn string_with_whitespace_inside() {
    // Whitespace inside value is preserved. Leading/trailing whitespace
    // is trimmed by the parser when reading `key: value`.
    let cfg = Cfg {
        x: "a  b  c".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn values_with_leading_trailing_whitespace_are_trimmed() {
    // This is EXPECTED behaviour — ser emits `x: value`, parser trims.
    // So if you serialize a string that had leading whitespace, it
    // won't survive. That's documented.
    let cfg = Cfg {
        x: "  padded  ".into(),
    };
    let s = to_string(&cfg).unwrap();
    let back: Cfg = from_str(&s).unwrap();
    // The inner whitespace is gone — this is the trim behaviour.
    assert_eq!(back.x, "padded");
}

#[test]
fn strings_equal_to_empty_compound_inline_get_marker() {
    // "{}" and "[]" would otherwise be read as empty compound values.
    let cfg = Cfg { x: "{}".into() };
    let s = to_string(&cfg).unwrap();
    assert!(s.starts_with("x:: {}"), "got: {}", s);
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);

    let cfg = Cfg { x: "[]".into() };
    let s = to_string(&cfg).unwrap();
    assert!(s.starts_with("x:: []"), "got: {}", s);
    let back: Cfg = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}
