//! The unified nine-field error envelope (issue rust#12): one JSON
//! object for every structured error, parse-time and writer-time
//! alike. These tests pin the wire contract — field set and order,
//! path-is-an-array semantics, byte-offset spans, RFC 8259 escaping,
//! spec-section mapping, and the parse-trigger → envelope mapping for
//! every `ErrorKind` and `ReasonCode`.

use std::collections::BTreeMap;

use ktav::render::{render, to_string_force_strings};
use ktav::value::Scalar;
use ktav::{
    emit_canonical, parse, parse_strict, to_string, ConflictKind, Error, ErrorEnvelope, ErrorKind,
    ObjectMap, ReasonCode, Span, Value,
};
use serde_json::Value as Json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn s(v: &str) -> Value {
    Value::String(v.parse().unwrap_or_else(|_| panic!("scalar: {v:?}")))
}

fn f(v: &str) -> Value {
    Value::Float(v.parse().unwrap_or_else(|_| panic!("float scalar: {v:?}")))
}

fn n(v: i64) -> Value {
    Value::Integer(v.to_string().parse().unwrap_or_else(|_| unreachable!()))
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut m = ObjectMap::default();
    for (k, v) in pairs {
        m.insert(Scalar::from(*k), v.clone());
    }
    Value::Object(m)
}

fn env(err: &Error, src: &str) -> ErrorEnvelope {
    ErrorEnvelope::from_error(err, src)
}

/// Cross-check: the envelope JSON must have EXACTLY the nine
/// contractual fields, in order — absent info is an explicit null,
/// never an omitted key.
fn nine_keys_present(json: &str) {
    let v: Json = serde_json::from_str(json).unwrap();
    let object = v.as_object().expect("envelope JSON is an object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "error",
            "reason",
            "line",
            "line_text",
            "span",
            "path",
            "body",
            "canonical",
            "spec_section",
        ]
    );
}

// ---------------------------------------------------------------------------
// Parse-time shape
// ---------------------------------------------------------------------------

#[test]
fn lossy_scalar_envelope_json_shape() {
    let src = "a: 1.10\n";
    let err = parse_strict(src).unwrap_err();
    let kind = match &err {
        Error::Structured(k) => k,
        other => panic!("expected Structured, got {other:?}"),
    };
    assert_eq!(kind.code_name(), "LossyScalar");

    let e = env(&err, src);
    assert_eq!(e.error, "LossyScalar");
    assert_eq!(e.reason, None);
    assert_eq!(e.path, None);
    assert_eq!(e.line, Some(1));
    assert_eq!(e.line_text, Some("a: 1.10".to_string()));
    assert_eq!(e.body, Some("1.10".to_string()));
    assert_eq!(e.canonical, Some("1.1".to_string()));
    assert_eq!(e.spec_section, Some("§3.6/§5.2".to_string()));
    let span = e.span.expect("LossyScalar carries a span");
    // Verified behavior: the LossyScalar span covers the whole pair
    // line, not just the scalar body.
    assert_eq!(span.slice(src), Some("a: 1.10"));
    assert_eq!(span, Span::new(0, 7));

    let json = e.to_json();
    nine_keys_present(&json);
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["error"], "LossyScalar");
    assert_eq!(v["reason"], Json::Null);
    assert_eq!(v["line"], 1);
    assert_eq!(v["line_text"], "a: 1.10");
    assert_eq!(v["span"]["start"], span.start);
    assert_eq!(v["span"]["end"], span.end);
    assert_eq!(v["path"], Json::Null);
    assert_eq!(v["body"], "1.10");
    assert_eq!(v["canonical"], "1.1");
    assert_eq!(v["spec_section"], "§3.6/§5.2");
}

// ---------------------------------------------------------------------------
// Writer-time shape
// ---------------------------------------------------------------------------

#[test]
fn writer_empty_key_name_envelope_shape() {
    let root = obj(&[("a", obj(&[("", n(1))]))]);
    let err = emit_canonical(&root).unwrap_err();
    let e = env(&err, "irrelevant source");
    assert_eq!(e.error, "UnrepresentableAt");
    assert_eq!(e.reason, Some("EmptyKeyName".to_string()));
    assert_eq!(e.path, Some(vec!["a".to_string(), "".to_string()]));
    assert_eq!(e.line, None);
    assert_eq!(e.line_text, None);
    assert_eq!(e.span, None);
    assert_eq!(e.body, None);
    assert_eq!(e.canonical, None);
    assert_eq!(e.spec_section, Some("§5.9.0".to_string()));

    let json = e.to_json();
    nine_keys_present(&json);
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["error"], "UnrepresentableAt");
    assert_eq!(v["reason"], "EmptyKeyName");
    assert_eq!(v["path"], serde_json::json!(["a", ""]));
    assert_eq!(v["line"], Json::Null);
    assert_eq!(v["line_text"], Json::Null);
    assert_eq!(v["span"], Json::Null);
    assert_eq!(v["body"], Json::Null);
    assert_eq!(v["canonical"], Json::Null);
    assert_eq!(v["spec_section"], "§5.9.0");
}

#[test]
fn writer_paths_per_reason_code() {
    let nan = || obj(&[("a", Value::Array(vec![obj(&[("b", f("NaN"))])]))]);
    let cases: &[(Value, ReasonCode, Vec<&str>, &str)] = &[
        (Value::Null, ReasonCode::ScalarRoot, vec![], "§5.9.0"),
        (
            obj(&[("a", obj(&[("", n(1))]))]),
            ReasonCode::EmptyKeyName,
            vec!["a", ""],
            "§5.9.0",
        ),
        (nan(), ReasonCode::NonFiniteFloat, vec!["a", "b"], "§5.9.0"),
        (
            obj(&[("k", s("a\rb"))]),
            ReasonCode::CRByte,
            vec!["k"],
            "§5.9.7",
        ),
        (
            obj(&[("k", s("))\n)"))]),
            ReasonCode::BothFormsRequired,
            vec!["k"],
            "§5.9.7",
        ),
        (
            obj(&[("k", s("))\nx "))]),
            ReasonCode::TrailingWhitespaceCollision,
            vec!["k"],
            "§5.9.7",
        ),
        (
            obj(&[("k", s(" ))\n x"))]),
            ReasonCode::LeadingWhitespaceCollision,
            vec!["k"],
            "§5.9.7",
        ),
    ];
    for (root, code, path, section) in cases {
        let err = emit_canonical(root).unwrap_err();
        match &err {
            Error::UnrepresentableAt {
                code: got_code,
                path: got_path,
            } => {
                assert_eq!(got_code, code);
                assert_eq!(
                    got_path,
                    &path.iter().map(|s| s.to_string()).collect::<Vec<_>>()
                );
            }
            other => panic!("expected UnrepresentableAt for {code:?}, got {other:?}"),
        }
        let e = env(&err, "");
        assert_eq!(e.error, "UnrepresentableAt");
        assert_eq!(e.reason, Some(code.code_name().to_string()));
        assert_eq!(e.line, None);
        assert_eq!(e.line_text, None);
        assert_eq!(e.span, None);
        assert_eq!(e.spec_section, Some(section.to_string()));
    }

    // The other Value-walking surfaces carry the same path.
    let nan_path = || obj(&[("a", Value::Array(vec![obj(&[("b", f("NaN"))])]))]);
    // `render` rejects the same NaN shape with the same path.
    // `to_string_force_strings` cannot fire NonFiniteFloat there by
    // construction — Floats are coerced to Strings BEFORE the § 5.9.0
    // check — so it is pinned with a CRByte shape instead.
    let cr_path = || obj(&[("a", Value::Array(vec![obj(&[("b", s("x\ry"))])]))]);
    let render_err = render(&nan_path()).unwrap_err();
    match &render_err {
        Error::UnrepresentableAt { code, path } => {
            assert_eq!(*code, ReasonCode::NonFiniteFloat);
            assert_eq!(path, &vec!["a".to_string(), "b".to_string()]);
        }
        other => panic!("expected UnrepresentableAt, got {other:?}"),
    }
    let force_err = to_string_force_strings(&cr_path()).unwrap_err();
    match &force_err {
        Error::UnrepresentableAt { code, path } => {
            assert_eq!(*code, ReasonCode::CRByte);
            assert_eq!(path, &vec!["a".to_string(), "b".to_string()]);
        }
        other => panic!("expected UnrepresentableAt, got {other:?}"),
    }
}

#[test]
fn streaming_serde_writer_has_null_path() {
    // The streaming surface takes any serde type, not `Value`
    // (which does not implement `Serialize` itself).
    let streaming: BTreeMap<String, BTreeMap<String, i32>> =
        [("a".to_string(), [("".to_string(), 1)].into_iter().collect())]
            .into_iter()
            .collect();
    let err = to_string(&streaming).unwrap_err();
    assert_eq!(err.reason_code(), Some(ReasonCode::EmptyKeyName));
    let e = env(&err, "");
    assert_eq!(e.error, "Unrepresentable");
    assert_eq!(e.reason, Some("EmptyKeyName".to_string()));
    assert_eq!(e.path, None);
}

// ---------------------------------------------------------------------------
// path is an array of exact decoded segments — never a joined string
// ---------------------------------------------------------------------------

#[test]
fn path_is_array_not_joined_string() {
    // Writer side: the root holds a literal "a -> b" key next to a
    // dotted "a.b" pair, so a joined-string path would be ambiguous.
    let root = obj(&[
        ("a -> b", obj(&[("x", s("1"))])),
        ("a", obj(&[("b", f("NaN"))])),
    ]);
    assert!(
        matches!(&root, Value::Object(m) if m.contains_key(&Scalar::from("a -> b"))),
        "the root must hold the literal key \"a -> b\""
    );
    let err = emit_canonical(&root).unwrap_err();
    let e = env(&err, "");
    assert_eq!(e.reason, Some("NonFiniteFloat".to_string()));
    assert_eq!(e.path, Some(vec!["a".to_string(), "b".to_string()]));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(
        json.contains("\"path\":[\"a\",\"b\"]"),
        "path must be a JSON array, got: {json}"
    );
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a", "b"]));

    // Parse side: the same ambiguity, resolved the same way.
    let src = "a -> b: 1\na.b: 1\na.b: 2\n";
    let err = parse(src).unwrap_err();
    let kind = match &err {
        Error::Structured(k @ ErrorKind::DuplicateKey { .. }) => k,
        other => panic!("expected DuplicateKey, got {other:?}"),
    };
    assert_eq!(kind.code_name(), "DuplicateKey");
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a".to_string(), "b".to_string()]));
}

#[test]
fn decoded_dot_stays_one_segment() {
    let src = "a\\.b: 1\na\\.b: 2\n";
    let err = parse(src).unwrap_err();
    match &err {
        Error::Structured(ErrorKind::DuplicateKey { key, .. }) => {
            // The `key` field carries the RAW dotted-path text; the
            // envelope decodes it (see the path assertion below).
            assert_eq!(key, "a\\.b");
        }
        other => panic!("expected DuplicateKey, got {other:?}"),
    }
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a.b".to_string()]));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(json.contains("\"path\":[\"a.b\"]"), "got: {json}");
}

// ---------------------------------------------------------------------------
// JSON string escaping
// ---------------------------------------------------------------------------

#[test]
fn json_escaping_control_quote_backslash_nonbmp() {
    // a. raw control byte → lowercase \u00xx
    let src = "a\u{1}b: v\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a\u{1}b".to_string()]));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(json.contains("\\u0001"), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a\u{1}b"]));

    // b. quote inside a quoted key → \"
    let src = "\"a\\\"b\": 1\n\"a\\\"b\": 2\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["a\"b".to_string()]));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(json.contains("\\\""), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["a\"b"]));

    // c. non-BMP astral char → raw UTF-8, no surrogate escaping
    let src = "\"🎉\": 1\n\"🎉\": 2\n";
    let err = parse(src).unwrap_err();
    let e = env(&err, src);
    assert_eq!(e.path, Some(vec!["🎉".to_string()]));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(
        json.as_bytes().windows(4).any(|w| w == "🎉".as_bytes()),
        "raw 4-byte UTF-8 🎉 must survive, got: {json}"
    );
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["path"], serde_json::json!(["🎉"]));

    // d. line_text carrying quotes → \" escapes, serde_json round-trip
    let src = "\"a\"b: v\n";
    let err = parse(src).unwrap_err();
    assert!(matches!(
        &err,
        Error::Structured(ErrorKind::InvalidKey { .. })
    ));
    let e = env(&err, src);
    assert_eq!(e.line_text, Some("\"a\"b: v".to_string()));
    let json = e.to_json();
    nine_keys_present(&json);
    assert!(json.contains("\\\"a\\\"b: v"), "got: {json}");
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["line_text"], "\"a\"b: v");
}

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
// Q1 — parens have an escape path
// ---------------------------------------------------------------------------

#[test]
fn q1_parens_have_an_escape_path() {
    let doc = parse("a\\u0028b: v\n").unwrap();
    assert_eq!(
        doc,
        obj(&[("a(b", s("v"))]),
        "\\u0028 must decode to ( in a key"
    );
    let doc = parse("a\\u0029b: v\n").unwrap();
    assert_eq!(doc, obj(&[("a)b", s("v"))]));

    let err = parse("a\\(b: v\n").unwrap_err();
    match &err {
        Error::Structured(ErrorKind::BadEscapeSequence { sequence, .. }) => {
            assert_eq!(sequence, "\\(");
        }
        other => panic!("expected BadEscapeSequence, got {other:?}"),
    }

    let text = emit_canonical(&obj(&[("a(b", s("v"))])).unwrap();
    assert!(
        text.starts_with("\"a(b\": v"),
        "canonical must quote the paren key, got: {text:?}"
    );
    assert_eq!(parse(&text).unwrap(), obj(&[("a(b", s("v"))]));

    let doc = parse("a\\u0020: v\n").unwrap();
    assert_eq!(
        doc,
        obj(&[("a ", s("v"))]),
        "the escaped space must survive key trimming (§ 6.13 escape path)"
    );

    let doc = parse("\\uD83D\\uDE00: v\n").unwrap();
    assert_eq!(doc, obj(&[("😀", s("v"))]));

    assert!(parse_strict("a\\u0028b: v\n").is_ok());
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
    nine_keys_present(&json);
    let v: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(v["span"]["start"], 13);
}

// ---------------------------------------------------------------------------
// Q3 — named escapes win over \uXXXX in the canonical writer
// ---------------------------------------------------------------------------

#[test]
fn q3_named_escape_wins_over_unicode_escape() {
    // a. literal backslash → \\ (named form), never \u005C
    let text = emit_canonical(&obj(&[("path\\to", s("v"))])).unwrap();
    assert!(text.contains("path\\\\to"), "got: {text:?}");
    assert!(!text.contains("u005C"));

    // b. leading LF → \n (named form), never \u000A
    let text = emit_canonical(&obj(&[("\nlf", s("v"))])).unwrap();
    assert!(text.starts_with("\\nlf: v"), "got: {text:?}");
    assert!(!text.contains("u000A"));
    // g. and it round-trips
    assert_eq!(parse(&text).unwrap(), obj(&[("\nlf", s("v"))]));

    // c. DEL → \u007F with uppercase hex
    let text = emit_canonical(&obj(&[("a\u{7F}b", s("v"))])).unwrap();
    assert!(text.contains("\\u007F"), "got: {text:?}");
    assert!(!text.contains("\\u007f"));

    // d. NUL → \u0000
    let text = emit_canonical(&obj(&[("a\u{0}b", s("v"))])).unwrap();
    assert!(text.contains("\\u0000"), "got: {text:?}");

    // e. a literal dot forces quoting but the dot stays raw —
    // § 5.9.10 prefers quoting over the \. escape
    let text = emit_canonical(&obj(&[("a.b", s("v"))])).unwrap();
    assert!(text.contains("\"a.b\""), "got: {text:?}");
    assert!(!text.contains("\\."));

    // f. an interior tab stays raw inside the emitted key
    let text = emit_canonical(&obj(&[("a\tb", s("v"))])).unwrap();
    assert!(text.contains("a\tb"), "got: {text:?}");
}

// ---------------------------------------------------------------------------
// push_json == to_json
// ---------------------------------------------------------------------------

#[test]
fn push_json_matches_to_json() {
    let src = "a: 1.10\n";
    let parse_time = env(&parse_strict(src).unwrap_err(), src);
    let mut out = String::with_capacity(1024);
    parse_time.push_json(&mut out);
    assert_eq!(out, parse_time.to_json());
    nine_keys_present(&out);

    let writer_time = env(
        &emit_canonical(&obj(&[("a", obj(&[("", n(1))]))])).unwrap_err(),
        "",
    );
    let mut out = String::with_capacity(1024);
    writer_time.push_json(&mut out);
    assert_eq!(out, writer_time.to_json());
    nine_keys_present(&out);
}

// ---------------------------------------------------------------------------
// The two writer rejections are named apart
// ---------------------------------------------------------------------------

/// Both writer surfaces refuse the same document for the same reason, but
/// only the Value-walking one knows which key is at fault. The envelope
/// names them apart — `"Unrepresentable"` versus `"UnrepresentableAt"` —
/// mirroring [`Error::Unrepresentable`] and [`Error::UnrepresentableAt`],
/// so a consumer can tell "refused" from "refused, and here is where"
/// without having to probe whether `path` happens to be null.
///
/// Pinned in one test on purpose: the two names were identical before the
/// envelope shipped, and collapsing them back would be a silent wire-format
/// regression that no other assertion here would catch.
#[test]
fn writer_rejections_are_named_apart() {
    let doc = obj(&[("a", obj(&[("", n(1))]))]);

    // Value-walking writer: knows the offending key path.
    let walking = env(&emit_canonical(&doc).unwrap_err(), "");
    assert_eq!(walking.error, "UnrepresentableAt");
    assert_eq!(walking.path, Some(vec!["a".to_string(), String::new()]));

    // Streaming serde writer: same refusal, no position to report.
    let streaming_value: BTreeMap<String, BTreeMap<String, i32>> =
        [("a".to_string(), [(String::new(), 1)].into_iter().collect())]
            .into_iter()
            .collect();
    let streaming = env(&to_string(&streaming_value).unwrap_err(), "");
    assert_eq!(streaming.error, "Unrepresentable");
    assert_eq!(streaming.path, None);

    // The distinction is in `error` and `path` only — the reason a consumer
    // switches on to decide "was this refused?" is identical.
    assert_ne!(walking.error, streaming.error);
    assert_eq!(walking.reason, streaming.reason);
    assert_eq!(walking.reason, Some("EmptyKeyName".to_string()));
}
