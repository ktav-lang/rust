//! Parser and deserializer error paths.

use ktav::{from_file, from_str, Error};
use serde::Deserialize;

use super::common::fixture;

/// Helper: extract the syntactic message from either the legacy
/// `Error::Syntax(_)` variant or the new `Error::Structured(_)` variant.
/// Display parity (see `tests/error_format.rs`) guarantees this returns
/// the same string for either shape.
fn syntax_msg(err: &Error) -> String {
    match err {
        Error::Syntax(m) => m.clone(),
        Error::Structured(k) => k.to_string(),
        other => panic!("expected Syntax/Structured error, got {:?}", other),
    }
}

#[test]
fn unclosed_object() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: {\n  y: 1\n").unwrap_err();
    assert!(syntax_msg(&err).contains("Unclosed"), "got: {:?}", err);
}

#[test]
fn unclosed_array() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: Vec<String>,
    }
    let err = from_str::<Cfg>("x: [\n  a\n").unwrap_err();
    assert!(syntax_msg(&err).contains("Unclosed"), "got: {:?}", err);
}

#[test]
fn mismatched_bracket() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: {\n]\n").unwrap_err();
    // 0.1.6: bracket mismatches surface as `UnbalancedBracket` with a
    // pinned Display string of `'<closer>' without matching '<opener>'`.
    assert!(
        syntax_msg(&err).contains("UnbalancedBracket"),
        "got: {:?}",
        err
    );
}

#[test]
fn stray_close_bracket() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("}\n").unwrap_err();
    assert!(
        syntax_msg(&err).contains("without matching"),
        "got: {:?}",
        err
    );
}

#[test]
fn duplicate_key() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_str::<Cfg>("port: 80\nport: 443\n").unwrap_err();
    assert!(syntax_msg(&err).contains("duplicate key"), "got: {:?}", err);
}

#[test]
fn duplicate_key_inside_dotted_synthetic() {
    // Inside a synthetic object built from dotted-key prefix `db.`, two
    // hits with the same leaf must be a duplicate-key error — even
    // though the document never wrote a literal `{ ... }` for `db`.
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _db: serde::de::IgnoredAny,
    }
    let err = from_str::<Cfg>("db.host: a\ndb.host: b\n").unwrap_err();
    assert!(syntax_msg(&err).contains("duplicate key"), "got: {:?}", err);
}

#[test]
fn dotted_prefix_after_scalar_at_same_name_conflicts() {
    // Reverse of `dotted_key_conflicts_with_scalar_at_shared_prefix`:
    // first the scalar `a: 1`, then `a.b: 2` tries to descend.
    // The new event-tokenizer must reject this, same as the tree path.
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _a: serde::de::IgnoredAny,
    }
    let err = from_str::<Cfg>("a: 1\na.b: 2\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("conflict"), "got: {}", msg);
}

#[test]
fn scalar_after_dotted_prefix_conflicts() {
    // `a.b: 1` opens a synthetic object at `a`. A subsequent flat
    // `a: 2` would have to overwrite the sub-object — must be a
    // conflict, not a silent drop.
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _a: serde::de::IgnoredAny,
    }
    let err = from_str::<Cfg>("a.b: 1\na: 2\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("conflict"), "got: {}", msg);
}

#[test]
fn interleaved_dotted_prefix_is_rejected_as_conflict() {
    // `a.x: 1` opens synthetic `a`. `b.y: 2` closes `a` and opens `b`.
    // `a.z: 3` would have to *re-open* the closed `a` — the streaming
    // event tokenizer can't do that without buffering the whole
    // document, so it surfaces a clear conflict error (the user can
    // group lines with the same prefix together).
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _a: serde::de::IgnoredAny,
        _b: serde::de::IgnoredAny,
    }
    let err = from_str::<Cfg>("a.x: 1\nb.y: 2\na.z: 3\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("conflict"), "got: {}", msg);
}

#[test]
fn grouped_dotted_keys_with_multiple_prefixes_work() {
    // Sanity-check the canonical pattern: dotted keys grouped per
    // prefix. This is the standard convention; the rejection above
    // only kicks in when prefixes are interleaved.
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        x: u32,
        z: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct B {
        y: u32,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        a: A,
        b: B,
    }
    let cfg: Cfg = from_str("a.x: 1\na.z: 3\nb.y: 2\n").unwrap();
    assert_eq!(cfg.a, A { x: 1, z: 3 });
    assert_eq!(cfg.b, B { y: 2 });
}

#[test]
fn empty_key() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    // Anchor with a known-Object pair (spec § 5.0.1 — first-line
    // `: value` would otherwise be parsed as a top-level Array
    // bare-scalar item).
    let err = from_str::<Cfg>("anchor: ok\n: value\n").unwrap_err();
    assert!(syntax_msg(&err).contains("Empty key"), "got: {:?}", err);
}

#[test]
fn key_with_special_chars() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("[foo]: bar\n").unwrap_err();
    assert!(syntax_msg(&err).contains("Invalid key"), "got: {:?}", err);
}

#[test]
fn line_without_colon_in_object() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    // Anchor with a known-Object pair (spec § 5.0.1).
    let err = from_str::<Cfg>("anchor: ok\njust-some-text\n").unwrap_err();
    // 0.1.6: the bare-word case now surfaces as `MissingSeparator` with
    // the pinned `'key: value' pairs` Display string.
    assert!(
        syntax_msg(&err).contains("MissingSeparator"),
        "got: {:?}",
        err
    );
}

#[test]
fn inline_object_is_valid_under_050() {
    // Under 0.5.0, `{ a: 1 }` is a valid inline object.
    #[derive(Debug, Deserialize)]
    struct Inner {
        a: u32,
    }
    #[derive(Debug, Deserialize)]
    struct Cfg {
        x: Inner,
    }
    let cfg: Cfg = from_str("x: { a: 1 }\n").unwrap();
    assert_eq!(cfg.x.a, 1);
}

#[test]
fn inline_array_is_valid_under_050() {
    // Under 0.5.0, `[a, b, c]` is a valid inline array.
    #[derive(Debug, Deserialize)]
    struct Cfg {
        x: Vec<String>,
    }
    let cfg: Cfg = from_str("x: [a, b, c]\n").unwrap();
    assert_eq!(cfg.x, vec!["a", "b", "c"]);
}

#[test]
fn unterminated_inline_bracket() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: [a-z\n").unwrap_err();
    assert!(
        syntax_msg(&err).contains("Unterminated") || syntax_msg(&err).contains("nline"),
        "got: {:?}",
        err
    );
}

#[test]
fn empty_input_yields_missing_required_field() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_str::<Cfg>("").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("port"), "got: {}", msg);
}

#[test]
fn from_file_missing_path_is_io_error() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _port: u16,
    }
    let err = from_file::<Cfg, _>(fixture("does_not_exist.conf")).unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}
