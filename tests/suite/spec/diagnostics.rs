use super::support::{collect_invalid_fixtures, resolve_spec_root, InvalidFixture};
use ktav::{Error, ErrorEnvelope, Span};
use serde_json::Value as Json;

fn check_location(fixture: &InvalidFixture, line: Option<u32>, span: Option<Span>) {
    let expected = fixture.location.as_ref().expect("location oracle");
    assert_eq!(line, Some(expected.line), "{}: source line", fixture.rel);
    let actual = span.unwrap_or_else(|| panic!("{}: missing span", fixture.rel));
    assert_eq!(actual.start, expected.start, "{}: span start", fixture.rel);
    assert!(
        actual.start < actual.end && actual.end as usize <= fixture.bytes.len(),
        "{}: invalid span {actual:?}",
        fixture.rel
    );
    if let Some(end) = expected.end {
        assert_eq!(actual.end, end, "{}: span end", fixture.rel);
    }
    if let Some(minimum_end) = expected.minimum_end {
        assert!(
            actual.end >= minimum_end,
            "{}: incomplete covering span",
            fixture.rel
        );
    }
}

fn check_error(fixture: &InvalidFixture, error: &Error, source: Option<&str>) {
    let category = match error {
        Error::Structured(kind) => kind.code_name(),
        Error::InvalidUtf8 {
            valid_up_to, span, ..
        } => {
            assert_eq!(
                *valid_up_to, span.start as usize,
                "{}: UTF-8 offset",
                fixture.rel
            );
            "InvalidUtf8"
        }
        other => panic!("{}: unexpected error {other:?}", fixture.rel),
    };
    assert_eq!(category, fixture.expected, "{}: category", fixture.rel);
    check_location(fixture, error.line(), error.span());
    let envelope = ErrorEnvelope::from_error(error, source.unwrap_or(""));
    assert_eq!(envelope.error, fixture.expected);
    assert_eq!(envelope.line, error.line());
    assert_eq!(envelope.span, error.span());
    if let Some(source) = source {
        assert_eq!(
            error.span().unwrap().line_col(source).0,
            fixture.location.as_ref().unwrap().line,
            "{}: Span::line_col",
            fixture.rel
        );
        let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
        let line = fixture.location.as_ref().unwrap().line as usize;
        assert_eq!(
            envelope.line_text.as_deref(),
            normalized.split('\n').nth(line - 1),
            "{}: line_text",
            fixture.rel
        );
    } else {
        assert!(envelope.line_text.is_none());
    }
    let wire: Json = serde_json::from_str(&envelope.to_json()).unwrap();
    assert_eq!(wire["line"].as_u64(), error.line().map(u64::from));
    assert_eq!(
        wire["span"]["start"].as_u64(),
        error.span().map(|span| u64::from(span.start))
    );
    assert_eq!(
        wire["span"]["end"].as_u64(),
        error.span().map(|span| u64::from(span.end))
    );
}

#[test]
fn source_diagnostics_match_versioned_fixture_oracles() {
    let Some(root) = resolve_spec_root() else {
        eprintln!("skipping diagnostics: spec corpus unavailable outside required CI");
        return;
    };
    let fixtures = collect_invalid_fixtures(&root);
    let mut checked = 0;
    for fixture in fixtures.iter().filter(|fixture| fixture.location.is_some()) {
        checked += 1;
        let source = std::str::from_utf8(&fixture.bytes).ok();
        if let Some(text) = source {
            check_error(fixture, &ktav::parse(text).unwrap_err(), source);
            check_error(fixture, &ktav::parse_strict(text).unwrap_err(), source);
            check_error(fixture, &ktav::format_str(text).unwrap_err(), source);
            let error = ktav::parse_events(text, |_event: ktav::ParseEvent<'_>| ()).unwrap_err();
            check_error(fixture, &error, source);
            check_error(
                fixture,
                &ktav::from_str::<String>(text).unwrap_err(),
                source,
            );
        }
        let error = ktav::from_file::<String, _>(&fixture.path).unwrap_err();
        check_error(fixture, &error, source);
    }
    assert!(
        checked > 0,
        "versioned diagnostic fixtures must not be skipped"
    );
    eprintln!("spec diagnostics: {checked} fixture locations verified across Rust entry points");
}

#[cfg(feature = "cabi")]
#[test]
fn source_cabi_diagnostics_match_same_fixture_oracles() {
    let Some(root) = resolve_spec_root() else {
        eprintln!("skipping C ABI diagnostics: spec corpus unavailable outside required CI");
        return;
    };
    let operations = [
        ktav::cabi::loads as fn(&[u8]) -> Result<Vec<u8>, String>,
        ktav::cabi::loads_strict,
        ktav::cabi::format,
        ktav::cabi::canonical_from_source,
    ];
    let fixtures = collect_invalid_fixtures(&root);
    let mut checked = 0;
    for fixture in fixtures.iter().filter(|fixture| fixture.location.is_some()) {
        checked += 1;
        for operation in operations {
            let error = operation(&fixture.bytes).unwrap_err();
            let wire: Json = serde_json::from_str(&error).expect("C ABI error envelope");
            assert_eq!(wire["error"].as_str(), Some(fixture.expected.as_str()));
            let line = wire["line"]
                .as_u64()
                .map(|value| u32::try_from(value).unwrap());
            let span = Span::new(
                u32::try_from(wire["span"]["start"].as_u64().expect("span start")).unwrap(),
                u32::try_from(wire["span"]["end"].as_u64().expect("span end")).unwrap(),
            );
            check_location(fixture, line, Some(span));
        }
    }
    assert!(checked > 0, "C ABI diagnostic fixtures must not be skipped");
    eprintln!(
        "spec diagnostics: {checked} fixture locations verified across C ABI source operations"
    );
}
