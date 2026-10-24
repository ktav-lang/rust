use ktav::{Error, ErrorEnvelope, ErrorKind, Span};

#[test]
fn line_text_and_columns_preserve_terminator_boundaries() {
    let source = "a\r\nb\rc\nd";
    let cases = [
        (0, 1, 0, "a"),
        (1, 1, 1, "a"),
        (2, 1, 2, "a"),
        (3, 2, 0, "b"),
        (4, 2, 1, "b"),
        (5, 3, 0, "c"),
        (6, 3, 1, "c"),
        (7, 4, 0, "d"),
        (8, 4, 1, "d"),
    ];
    for (offset, line, column, text) in cases {
        let span = Span::new(offset, offset);
        let error = Error::Structured(ErrorKind::EmptyKey { line, span });
        let envelope = ErrorEnvelope::from_error(&error, source);
        assert_eq!(envelope.line_text.as_deref(), Some(text), "offset {offset}");
        assert_eq!(span.line_col(source), (line, column), "offset {offset}");
    }
}

#[test]
fn line_text_preserves_bom_utf8_and_defensive_span_handling() {
    let source = "\u{FEFF}שלום\r\nx";
    let error = Error::Structured(ErrorKind::EmptyKey {
        line: 2,
        span: Span::new(13, 14),
    });
    let envelope = ErrorEnvelope::from_error(&error, source);
    assert_eq!(envelope.line_text.as_deref(), Some("x"));
    assert_eq!(Span::new(13, 14).line_col(source), (2, 0));

    let inside_char = Error::Structured(ErrorKind::EmptyKey {
        line: 1,
        span: Span::new(4, 4),
    });
    assert!(ErrorEnvelope::from_error(&inside_char, source)
        .line_text
        .is_none());
    let beyond_eof = Error::Structured(ErrorKind::EmptyKey {
        line: 2,
        span: Span::new(u32::MAX, u32::MAX),
    });
    assert_eq!(
        ErrorEnvelope::from_error(&beyond_eof, source)
            .line_text
            .as_deref(),
        Some("x")
    );
    assert!(ErrorEnvelope::from_error(&error, "").line_text.is_none());
}

#[test]
fn json_encoding_matches_independent_escaping_oracle() {
    let controls: String = (0u8..=31).map(char::from).collect();
    let ascii: String = (0u8..=127).map(char::from).collect();
    let payloads = [
        String::new(),
        ascii,
        "ordinary text 123".repeat(2048),
        "café שלום 中文 😀".repeat(256),
        format!("{controls}\"\\"),
        "safe\0שלום\"\\😀".repeat(256),
        format!("{}\"שלום\\\u{1}{}", "a".repeat(4096), "z".repeat(4096)),
    ];
    for payload in payloads {
        let quoted = serde_json::to_string(&payload).unwrap();
        let expected = format!(
            "{{\"error\":\"Message\",\"reason\":null,\"line\":null,\"line_text\":null,\"span\":null,\"path\":null,\"body\":{quoted},\"canonical\":null,\"spec_section\":null,\"message\":{quoted}}}"
        );
        let actual = ErrorEnvelope::from_error(&Error::Message(payload), "").to_json();
        assert_eq!(actual, expected);
    }
}
