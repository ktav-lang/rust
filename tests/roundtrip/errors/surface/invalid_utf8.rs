//! Invalid UTF-8 handling (spec 0.7 § 6.15): a byte-level input that is
//! not valid UTF-8 must fail as [`ktav::Error::InvalidUtf8`] carrying the
//! byte offset of the first invalid sequence, before any line-oriented
//! processing. `from_file` is the crate's only byte-boundary entry
//! point — `parse`, `parse_strict`, `from_str` and `parse_events` all
//! take `&str`, which is valid UTF-8 by the type system's own invariant,
//! so invalid UTF-8 is structurally unreachable there: no safe-Rust test
//! can (or needs to) exercise those paths. That exclusion is
//! intentionally out of scope, not overlooked.

use std::collections::BTreeMap;

use ktav::{Error, Span};

/// Write `bytes` to a process-unique temp file and return its path.
fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ktav_invalid_utf8_{}_{}", std::process::id(), name));
    std::fs::write(&path, bytes).expect("write temp fixture");
    path
}

#[test]
fn from_file_lone_continuation_byte_reports_invalid_utf8() {
    let path = temp_file("lone_continuation", b"key: ok\n\x80");
    let err = ktav::from_file::<BTreeMap<String, String>, _>(&path)
        .expect_err("must reject invalid UTF-8");
    let _ = std::fs::remove_file(&path);

    let Error::InvalidUtf8 { valid_up_to } = &err else {
        panic!("expected Error::InvalidUtf8, got {err:?}");
    };
    assert_eq!(*valid_up_to, 8);
    assert_eq!(err.valid_up_to(), Some(8));
    assert_eq!(err.span(), Some(Span::new(8, 8)));
    assert_eq!(err.line(), None);
    assert!(err.to_string().starts_with("InvalidUtf8"));
}

#[test]
fn from_file_truncated_multibyte_sequence_reports_invalid_utf8() {
    let path = temp_file("truncated_multibyte", b"a: \xE0\x80");
    let err = ktav::from_file::<BTreeMap<String, String>, _>(&path)
        .expect_err("must reject truncated sequence");
    let _ = std::fs::remove_file(&path);

    let Error::InvalidUtf8 { valid_up_to: 3 } = &err else {
        panic!("expected Error::InvalidUtf8 {{ valid_up_to: 3 }}, got {err:?}");
    };
}

#[test]
fn from_file_valid_utf8_with_multibyte_chars_roundtrips() {
    let path = temp_file("valid_multibyte", "café: 😀\n".as_bytes());
    let doc: BTreeMap<String, String> = ktav::from_file(&path).expect("valid UTF-8 must parse");
    let _ = std::fs::remove_file(&path);

    assert_eq!(doc.get("café"), Some(&"😀".to_string()));
}

#[test]
fn from_file_missing_file_is_io_not_invalid_utf8() {
    let mut path = std::env::temp_dir();
    path.push(format!("ktav_invalid_utf8_{}_missing", std::process::id()));
    let err =
        ktav::from_file::<BTreeMap<String, String>, _>(&path).expect_err("missing file must fail");
    assert!(
        matches!(err, Error::Io(_)),
        "expected Error::Io, got {err:?}"
    );
}
