//! Verifies the formatter and error-envelope snippets shown in the
//! READMEs (en/ru/zh) actually compile and produce exactly the output
//! the prose claims, mirroring the `readme_strict_snippet.rs`
//! convention of keeping rendered doc snippets under test.
//!
//! These two snippets are the whole evidence a reader has for "comments
//! are preserved" and "the envelope is nine fields in a fixed order", so
//! a silent drift between README and behavior is exactly the failure
//! this file exists to prevent.

use ktav::ErrorEnvelope;

/// `## Formatting` — the inline compound expands to the canonical
/// multi-line form and the comment stays where it was.
#[test]
fn readme_formatting_snippet() {
    let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n").unwrap();
    assert_eq!(
        tidied,
        "## the server\nserver: {\n    host: a\n    port: 80\n}\n"
    );

    // The README also renders that output as a Ktav block; pin the
    // rendered form too, so the two cannot drift apart.
    assert_eq!(
        tidied,
        concat!(
            "## the server\n",
            "server: {\n",
            "    host: a\n",
            "    port: 80\n",
            "}\n",
        )
    );

    // And the fixed-point claim made one paragraph below it.
    assert_eq!(ktav::format_str(&tidied).unwrap(), tidied);
}

/// `### One JSON envelope for every structured error` — the rendered
/// JSON is shown verbatim in all three READMEs.
#[test]
fn readme_envelope_snippet() {
    let src = "a: 1.10\n";
    let err = ktav::parse_strict(src).unwrap_err();
    let json = ErrorEnvelope::from_error(&err, src).to_json();

    assert_eq!(
        json,
        concat!(
            r#"{"error":"LossyScalar","reason":null,"line":1,"#,
            r#""line_text":"a: 1.10","span":{"start":0,"end":7},"#,
            r#""path":null,"body":"1.10","canonical":"1.1","#,
            r#""spec_section":"§3.6/§5.2"}"#
        ),
        "README envelope snippet drifted from ErrorEnvelope::to_json"
    );
}
