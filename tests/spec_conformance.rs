//! Language-agnostic conformance suite from `ktav-lang/spec`.
//!
//! Points at `versions/<SPEC_VERSION>/tests/{valid,invalid}` and checks:
//!   - every `valid/**/*.ktav` parses and its `Value` equals the oracle
//!     in the sibling `.json` file (1:1 mapping, field order significant);
//!   - every `invalid/**/*.ktav` is rejected by `ktav::parse`.
//!
//! Spec root resolution (first match wins):
//!   1. env var `KTAV_SPEC_DIR` (absolute path to the spec-repo root);
//!   2. `<CARGO_MANIFEST_DIR>/spec`  — git submodule `ktav-lang/spec`;
//!   3. `<CARGO_MANIFEST_DIR>/../spec` — sibling directory (local dev);
//!   4. if none contains a `versions/` dir, the test logs and returns —
//!      it does not fail, so CI without the spec checkout stays green.
//!
//! TODO: pull `SPEC_VERSION` from `[package.metadata.ktav] spec-version`
//! instead of hardcoding, once there's a reason to keep both in sync.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::Value;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

// Directory name uses MAJOR.MINOR (per spec/CONTRIBUTING.md — PATCH updates
// edit the same directory in place). The version *string* `0.1.0` lives in
// Cargo.toml's `[package.metadata.ktav] spec-version`; this constant is a
// path segment and stays at `0.1`.
const SPEC_VERSION: &str = "0.1";

fn resolve_spec_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(env) = std::env::var("KTAV_SPEC_DIR") {
        candidates.push(PathBuf::from(env));
    }
    candidates.push(manifest.join("spec"));
    candidates.push(manifest.join("../spec"));
    candidates.into_iter().find(|p| p.join("versions").is_dir())
}

fn tests_dir(spec_root: &Path, bucket: &str) -> PathBuf {
    spec_root
        .join("versions")
        .join(SPEC_VERSION)
        .join("tests")
        .join(bucket)
}

/// Walk `root` recursively and collect every `.ktav` file under it.
fn collect_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ktav_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            out.push(path);
        }
    }
}

/// Convert a `ktav::Value` into a `serde_json::Value` using the 1:1 mapping
/// from the spec. Object iteration uses `IndexMap`'s insertion order;
/// `preserve_order` keeps that order intact. `Integer` / `Float` become
/// native JSON numbers — feasible because `serde_json` is built here with
/// `arbitrary_precision`, which stores the number as its textual form and
/// so survives values outside the i64/u64/f64 range (e.g. the 20-digit
/// integer in `integer_large`).
fn ktav_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::String(s) => JsonValue::String(s.to_string()),
        Value::Integer(s) | Value::Float(s) => {
            // serde_json with `arbitrary_precision` compares Numbers by
            // textual form. Fixture oracles write exponents in lowercase
            // `e` (the JSON-preferred form); Ktav accepts both `e` and
            // `E` in the grammar, so normalize here for the comparison.
            // This does not affect what the Value holds — unit tests in
            // `tests/edge_cases/typed_markers.rs` verify `E` survives
            // round-trip unchanged.
            let normalized = s.replace('E', "e");
            let n = JsonNumber::from_string_unchecked(normalized);
            JsonValue::Number(n)
        }
        Value::Array(items) => JsonValue::Array(items.iter().map(ktav_to_json).collect()),
        Value::Object(obj) => {
            let mut map = JsonMap::new();
            for (k, v) in obj {
                map.insert(k.to_string(), ktav_to_json(v));
            }
            JsonValue::Object(map)
        }
    }
}

/// Ordered recursive comparison. `serde_json` with `preserve_order` already
/// compares objects in order via `PartialEq` (IndexMap's `PartialEq` is
/// order-sensitive), but we walk explicitly so a mismatch message can
/// pinpoint the first differing path — and so we don't silently depend on
/// serde_json's future `PartialEq` semantics.
fn json_eq_ordered(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(x), JsonValue::Bool(y)) => x == y,
        (JsonValue::Number(x), JsonValue::Number(y)) => x == y,
        (JsonValue::String(x), JsonValue::String(y)) => x == y,
        (JsonValue::Array(x), JsonValue::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq_ordered(a, b))
        }
        (JsonValue::Object(x), JsonValue::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .zip(y.iter())
                    .all(|((ka, va), (kb, vb))| ka == kb && json_eq_ordered(va, vb))
        }
        _ => false,
    }
}

#[test]
fn valid_fixtures_match_oracle() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::valid: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "valid");
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();

    let mut failures: Vec<String> = Vec::new();

    for ktav_path in &files {
        let json_path = ktav_path.with_extension("json");
        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();

        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        let oracle_src = match fs::read_to_string(&json_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", json_path.display(), e));
                continue;
            }
        };

        let actual = match ktav::parse(&text) {
            Ok(v) => ktav_to_json(&v),
            Err(e) => {
                failures.push(format!("parse {}: {}", rel, e));
                continue;
            }
        };
        let expected: JsonValue = match serde_json::from_str(&oracle_src) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("oracle {}: {}", json_path.display(), e));
                continue;
            }
        };

        if !json_eq_ordered(&actual, &expected) {
            failures.push(format!(
                "mismatch in {}:\n  expected: {}\n  actual:   {}",
                rel, expected, actual
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixture(s) failed:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn invalid_fixtures_are_rejected() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::invalid: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "invalid");
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();

    let mut failures: Vec<String> = Vec::new();

    for ktav_path in &files {
        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();
        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        // TODO: once `ktav::Error` exposes structured categories, read the
        // sibling `.json` (`{"error": "<CategoryName>"}`) and assert the
        // category matches. For now the Rust impl has a single
        // `Error::Syntax(String)`, so we only check that parsing fails.
        if ktav::parse(&text).is_ok() {
            failures.push(format!("invalid fixture parsed successfully: {}", rel));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} invalid fixture(s) failed:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn valid_fixtures_roundtrip_losslessly() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::roundtrip: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "valid");
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();

    let mut failures: Vec<String> = Vec::new();

    for ktav_path in &files {
        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();

        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        let value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("parse {}: {}", rel, e));
                continue;
            }
        };
        let rendered = match ktav::render::render(&value) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("render {}: {}", rel, e));
                continue;
            }
        };
        let reparsed = match ktav::parse(&rendered) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "reparse {} failed: {}\n  rendered text:\n{}",
                    rel, e, rendered
                ));
                continue;
            }
        };
        if value != reparsed {
            failures.push(format!(
                "roundtrip mismatch in {}:\n  original Value:  {:?}\n  reparsed Value:  {:?}\n  rendered text:\n{}",
                rel, value, reparsed, rendered
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixture(s) failed roundtrip:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}
