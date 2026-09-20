//! Envelope wire-shape tests: field set, order, and writer-time shapes.

use std::collections::BTreeMap;

use super::helpers::{env, f, n, obj, s, ten_keys_present};
use ktav::render::{render, to_string_force_strings};
use ktav::{emit_canonical, parse_strict, to_string, Error, ReasonCode, Span, Value};
use serde_json::Value as Json;

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
    ten_keys_present(&json);
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
    ten_keys_present(&json);
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
// push_json == to_json
// ---------------------------------------------------------------------------

#[test]
fn push_json_matches_to_json() {
    let src = "a: 1.10\n";
    let parse_time = env(&parse_strict(src).unwrap_err(), src);
    let mut out = String::with_capacity(1024);
    parse_time.push_json(&mut out);
    assert_eq!(out, parse_time.to_json());
    ten_keys_present(&out);

    let writer_time = env(
        &emit_canonical(&obj(&[("a", obj(&[("", n(1))]))])).unwrap_err(),
        "",
    );
    let mut out = String::with_capacity(1024);
    writer_time.push_json(&mut out);
    assert_eq!(out, writer_time.to_json());
    ten_keys_present(&out);
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
