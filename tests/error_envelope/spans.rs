//! Spans, line text, and spec-section mapping onto the envelope.

use crate::helpers::{env, ten_keys_present};
use ktav::{emit_canonical, parse, ConflictKind, Error, ErrorKind, Span, Value};
use serde_json::Value as Json;

// ---------------------------------------------------------------------------
// line_text edges
// ---------------------------------------------------------------------------

#[test]
fn line_text_edges() {
    // EOF span: last line is still reported
    let src = "a: {\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert!(err.reason_code().is_none());
    assert!(matches!(
        &err,
        Error::Structured(ErrorKind::UnclosedCompound { .. })
    ));
    assert_eq!(e.line_text, Some("a: {".to_string()));

    // CRLF: the trailing \r is stripped
    let src = "a: 1\r\nb,x: 2\r\n";
    let err = parse(src).unwrap_err();
    assert!(matches!(
        &err,
        Error::Structured(ErrorKind::InvalidKey { .. })
    ));
    let e = env(&err, src);
    assert_eq!(e.line_text, Some("b,x: 2".to_string()));

    // Empty source: no line text even though the kind has a span
    let err = Error::Structured(ErrorKind::EmptyKey {
        line: 1,
        span: Span::EMPTY,
    });
    let e = env(&err, "");
    assert_eq!(e.line, Some(1));
    assert_eq!(e.line_text, None);
    assert_eq!(e.span, Some(Span::EMPTY));

    // Writer-time errors emit honest nulls — the source is ignored
    let err = emit_canonical(&Value::Null).unwrap_err();
    let e = env(&err, "irrelevant");
    assert_eq!(e.line, None);
    assert_eq!(e.line_text, None);
    assert_eq!(e.span, None);

    // InvalidUtf8: insertion-point span at valid_up_to, no line
    let err = Error::InvalidUtf8 { valid_up_to: 7 };
    let e = env(&err, "0123456789");
    assert_eq!(e.error, "InvalidUtf8");
    assert_eq!(e.span, Some(Span::new(7, 7)));
    assert_eq!(e.line_text, Some("0123456789".to_string()));
    assert_eq!(e.line, None);
    assert_eq!(e.spec_section, Some("§6.15".to_string()));
}

// ---------------------------------------------------------------------------
// Every ErrorKind maps onto the envelope
// ---------------------------------------------------------------------------

#[test]
fn every_error_kind_maps_onto_envelope() {
    let span = Span::new(0, 1);
    let kinds: Vec<ErrorKind> = vec![
        ErrorKind::MissingSeparatorSpace {
            line: 1,
            column: 0,
            marker: ':',
            span,
        },
        ErrorKind::InvalidTypedScalar {
            line: 1,
            marker: 'i',
            body: "x".to_string(),
            span,
        },
        ErrorKind::LossyScalar {
            line: 1,
            body: "1.10".to_string(),
            canonical: "1.1".to_string(),
            span,
        },
        ErrorKind::DuplicateKey {
            line: 1,
            key: "a".to_string(),
            span,
        },
        ErrorKind::KeyPathConflict {
            line: 1,
            path: "a.b".to_string(),
            kind: ConflictKind::BlockedByValue,
            span,
        },
        ErrorKind::EmptyKey { line: 1, span },
        ErrorKind::InvalidKey {
            line: 1,
            key: "a,b".to_string(),
            span,
        },
        ErrorKind::UnclosedCompound {
            kind: ktav::CompoundKind::Object,
            span,
        },
        ErrorKind::UnbalancedBracket {
            line: 1,
            span,
            expected: ktav::CompoundKind::Object,
            found: '}',
        },
        ErrorKind::InlineNonEmptyCompound {
            line: 1,
            span,
            body: "object".to_string(),
        },
        ErrorKind::MissingSeparator { line: 1, span },
        ErrorKind::UnterminatedInlineCompound { line: 1, span },
        ErrorKind::UnterminatedQuotedKey { line: 1, span },
        ErrorKind::MalformedInlineCompound {
            line: 1,
            span,
            detail: "leading comma".to_string(),
        },
        ErrorKind::BadEscapeSequence {
            line: 1,
            span,
            sequence: "\\q".to_string(),
        },
        ErrorKind::OrphanLineAfterTopLevelInline { line: 1, span },
        ErrorKind::Other {
            line: Some(2),
            message: "internal".to_string(),
            span: Span::EMPTY,
        },
    ];
    let sections = [
        Some("§6.10"),
        Some("§6.9"),
        Some("§3.6/§5.2"),
        Some("§6.2"),
        Some("§6.3"),
        Some("§6.5"),
        Some("§6.4"),
        Some("§6.1"),
        Some("§6.1"),
        Some("§6.7"),
        Some("§6.6"),
        Some("§6.11"),
        Some("§6.16"),
        Some("§6.12"),
        Some("§6.13"),
        Some("§6.14"),
        None,
    ];
    assert_eq!(kinds.len(), 17);
    for (kind, section) in kinds.iter().zip(sections) {
        let err = Error::Structured(kind.clone());
        let e = env(&err, "");
        assert_eq!(e.error, kind.code_name());
        assert_eq!(e.spec_section.as_deref(), section, "kind: {kind:?}");
        assert_eq!(e.reason, None);
    }
    // The legacy doc(hidden) variants' body contract:
    let legacy = env(
        &Error::Structured(ErrorKind::InvalidTypedScalar {
            line: 1,
            marker: 'i',
            body: "x".to_string(),
            span,
        }),
        "",
    );
    assert_eq!(legacy.body, Some("x".to_string()));
    let legacy = env(
        &Error::Structured(ErrorKind::InlineNonEmptyCompound {
            line: 1,
            span,
            body: "object".to_string(),
        }),
        "",
    );
    // Its `body` is a kind label, not source text — not mapped.
    assert_eq!(legacy.body, None);
}
// ---------------------------------------------------------------------------
// Q2 — span units are byte offsets
// ---------------------------------------------------------------------------

#[test]
fn q2_span_unit_is_byte_offsets() {
    let src = "🎉: v\nport:8080\n";
    let err = parse(src).unwrap_err();
    let kind = match &err {
        Error::Structured(k) => k,
        other => panic!("expected Structured, got {other:?}"),
    };
    assert_eq!(kind.code_name(), "MissingSeparatorSpace");
    assert_eq!(err.line(), Some(2));
    let span = err.span().unwrap();
    assert_eq!(span.slice(src), Some("8080"));
    // 🎉 is 4 UTF-8 bytes → line 1 is 8 bytes, "port:" is 5 more → 13.
    // A UTF-16-unit count would be 11.
    assert_eq!(span.start, 13);

    let e = env(&err, src);
    let json = e.to_json();
    ten_keys_present(&json);
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["span"]["start"], 13);
}
