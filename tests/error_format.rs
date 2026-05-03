//! Canonical regression net for `ErrorKind`'s `Display` impl.
//!
//! Mirrors the seven-category contract pinned by
//! `editor/lsp/tests/error_format_pinning.rs`. Display output is the
//! contract that lets `0.1.x` string-matching callers keep working
//! after the structured-errors refactor in `0.2.0`. Any wording change
//! breaks the LSP's range-tightening and every `Error::Syntax`-string
//! consumer downstream.
//!
//! Each test asserts both that `ktav::parse` returns the new
//! `Error::Structured(...)` variant **and** that its `Display` output
//! is byte-identical to the legacy `Error::Syntax(format!(...))` text.

use ktav::{CompoundKind, ConflictKind, Error, ErrorKind};

fn err(text: &str) -> Error {
    match ktav::parse(text) {
        Err(e) => e,
        Ok(_) => panic!("expected parse to fail for: {:?}", text),
    }
}

fn structured_string(e: &Error) -> String {
    match e {
        Error::Structured(k) => k.to_string(),
        Error::Syntax(m) => m.clone(),
        other => panic!("expected Syntax/Structured error, got {:?}", other),
    }
}

#[test]
fn pin_missing_separator_space() {
    let e = err("key:value\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::MissingSeparatorSpace { line: 1, .. })
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 1: MissingSeparatorSpace: separator must be followed by whitespace or end of line"
    );
}

#[test]
fn pin_invalid_typed_scalar() {
    let e = err("port:i abc\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::InvalidTypedScalar { line: 1, .. })
        ),
        "got: {e:?}"
    );
    let s = structured_string(&e);
    assert!(s.starts_with("Line 1: InvalidTypedScalar: "), "got: {s}");
}

#[test]
fn pin_duplicate_key() {
    let e = err("port: 80\nport: 443\n");
    assert!(
        matches!(
            &e,
            Error::Structured(ErrorKind::DuplicateKey { line: 2, key, .. }) if key == "port"
        ),
        "got: {e:?}"
    );
    assert_eq!(structured_string(&e), "Line 2: duplicate key 'port'");
}

#[test]
fn pin_key_path_conflict() {
    let e = err("db: 1\ndb.x: 2\n");
    assert!(
        matches!(
            &e,
            Error::Structured(ErrorKind::KeyPathConflict {
                line: 2,
                kind: ConflictKind::BlockedByValue,
                path,
                ..
            }) if path == "db.x"
        ),
        "got: {e:?}"
    );
    let s = structured_string(&e);
    assert!(
        s.starts_with("Line 2: conflict at 'db.x'")
            && s.contains("an existing value blocks the path"),
        "got: {s}"
    );
}

#[test]
fn pin_empty_key() {
    let e = err(": value\n");
    assert!(
        matches!(e, Error::Structured(ErrorKind::EmptyKey { line: 1, .. })),
        "got: {e:?}"
    );
    assert_eq!(structured_string(&e), "Empty key at line 1");
}

#[test]
fn pin_invalid_key() {
    let e = err("a.: 1\n");
    assert!(
        matches!(e, Error::Structured(ErrorKind::InvalidKey { line: 1, .. })),
        "got: {e:?}"
    );
    let s = structured_string(&e);
    assert!(s.starts_with("Invalid key at line 1: '"), "got: {s}");
}

#[test]
fn pin_unclosed_object() {
    let e = err("obj: {\n  a: 1\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::UnclosedCompound {
                kind: CompoundKind::Object,
                ..
            })
        ),
        "got: {e:?}"
    );
    assert_eq!(structured_string(&e), "Unclosed object at end of input");
}

#[test]
fn pin_unclosed_array() {
    let e = err("arr: [\n  1\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::UnclosedCompound {
                kind: CompoundKind::Array,
                ..
            })
        ),
        "got: {e:?}"
    );
    assert_eq!(structured_string(&e), "Unclosed array at end of input");
}

#[test]
fn pin_unclosed_multiline() {
    let e = err("text: (\n  hello\n");
    assert!(
        matches!(e, Error::Structured(ErrorKind::UnclosedCompound { .. })),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Unclosed multi-line string at end of input"
    );
}

#[test]
fn pin_unbalanced_bracket_stray() {
    // `}` at top level — closer without matching opener.
    let e = err("}\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::UnbalancedBracket {
                line: 1,
                found: '}',
                ..
            })
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 1: UnbalancedBracket: '}' without matching '{'"
    );
}

#[test]
fn pin_unbalanced_bracket_mismatch() {
    // `obj: {` then `]` — wrong-shape closer relative to the open.
    let e = err("obj: {\n]\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::UnbalancedBracket {
                line: 2,
                found: ']',
                ..
            })
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 2: UnbalancedBracket: ']' without matching '{'"
    );
}

#[test]
fn pin_inline_nonempty_object() {
    let e = err("server: { host: 127.0.0.1 }\n");
    assert!(
        matches!(
            &e,
            Error::Structured(ErrorKind::InlineNonEmptyCompound { line: 1, body, .. }) if body == "object"
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 1: InlineNonEmptyCompound: inline non-empty object is not supported; put entries on separate lines"
    );
}

#[test]
fn pin_inline_nonempty_array() {
    let e = err("items: [ a b c ]\n");
    assert!(
        matches!(
            &e,
            Error::Structured(ErrorKind::InlineNonEmptyCompound { line: 1, body, .. }) if body == "array"
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 1: InlineNonEmptyCompound: inline non-empty array is not supported; put entries on separate lines"
    );
}

#[test]
fn pin_missing_separator() {
    let e = err("just-some-text\n");
    assert!(
        matches!(
            e,
            Error::Structured(ErrorKind::MissingSeparator { line: 1, .. })
        ),
        "got: {e:?}"
    );
    assert_eq!(
        structured_string(&e),
        "Line 1: MissingSeparator: object entries must be 'key: value' pairs"
    );
}

#[test]
fn parser_no_longer_emits_legacy_syntax_variant() {
    // Regression guard: every parser-emitted error MUST be Structured(_)
    // (or the unrelated Io/Message). If anything routes back through
    // `Error::Syntax(_)`, this test catches it.
    let cases = [
        "key:value\n",
        "port:i abc\n",
        "port: 80\nport: 443\n",
        "db: 1\ndb.x: 2\n",
        ": value\n",
        "a.: 1\n",
        "obj: {\n  a: 1\n",
        "arr: [\n  1\n",
        "text: (\n  hello\n",
        "}\n",
        "obj: {\n}\n]\n",
        "key { x: 1 }\n",
    ];
    for src in cases {
        match ktav::parse(src) {
            Err(Error::Structured(_)) => {}
            Err(Error::Syntax(m)) => panic!("legacy Syntax variant from parser for {src:?}: {m}"),
            Err(other) => panic!("unexpected non-Structured error for {src:?}: {other:?}"),
            Ok(_) => panic!("expected parse failure for {src:?}"),
        }
    }
}
