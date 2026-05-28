//! Language-agnostic conformance suite from `ktav-lang/spec`.
//!
//! Points at `versions/<SPEC_VERSION>/tests/{valid,invalid}` and checks:
//!   - every `valid/**/*.ktav` (non-canonical) parses and its `Value`
//!     equals the oracle in the sibling `.json` file;
//!   - every `valid/**/*.canonical.ktav` re-parses to the same `Value`;
//!   - `emit_canonical(parse(input.ktav))` matches `name.canonical.ktav`;
//!   - every `invalid/**/*.ktav` is rejected by `ktav::parse`.
//!
//! Spec root resolution (first match wins):
//!   1. env var `KTAV_SPEC_DIR` (absolute path to the spec-repo root);
//!   2. `<CARGO_MANIFEST_DIR>/spec`  — git submodule `ktav-lang/spec`;
//!   3. `<CARGO_MANIFEST_DIR>/../spec` — sibling directory (local dev);
//!   4. if none contains a `versions/` dir, the test logs and returns —
//!      it does not fail, so CI without the spec checkout stays green.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::Value;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

const SPEC_VERSION: &str = "0.5";

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

/// Walk `root` recursively and collect every `.ktav` file that is NOT
/// a `*.canonical.ktav` file.
fn collect_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ktav_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            // Skip canonical files — they are tested alongside their input
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !stem.ends_with(".canonical") {
                out.push(path);
            }
        }
    }
}

/// Convert a `ktav::Value` into a `serde_json::Value` using the 1:1 mapping
/// from the spec.
fn ktav_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::String(s) => JsonValue::String(s.to_string()),
        Value::Integer(s) => {
            // Under 0.5.0, Integer stores canonical base-10 decimal.
            let n = JsonNumber::from_string_unchecked(s.to_string());
            JsonValue::Number(n)
        }
        Value::Float(s) => {
            // Under 0.5.0, Float stores canonical form via ryu.
            // ryu may output forms like "0.5" which are valid JSON numbers.
            let text = s.to_string();
            let n = JsonNumber::from_string_unchecked(text);
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

/// Ordered recursive comparison. Numbers are compared by parsing to f64
/// to handle different textual representations of the same value (e.g.
/// `1e9` vs `1000000000.0`).
fn json_eq_ordered(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(x), JsonValue::Bool(y)) => x == y,
        (JsonValue::Number(x), JsonValue::Number(y)) => {
            // Try numeric comparison via f64 first (handles e.g. 1e9 vs 1000000000.0)
            let x_str = x.to_string();
            let y_str = y.to_string();
            if x_str == y_str {
                return true;
            }
            // Fall back to f64 comparison
            match (x_str.parse::<f64>(), y_str.parse::<f64>()) {
                (Ok(xf), Ok(yf)) => xf == yf,
                _ => false,
            }
        }
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
    eprintln!("spec_conformance::valid: {} fixtures passed", files.len());
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
    eprintln!(
        "spec_conformance::invalid: {} fixtures rejected",
        files.len()
    );
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
        let rendered = match ktav::render::emit_canonical(&value) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("emit_canonical {}: {}", rel, e));
                continue;
            }
        };
        let reparsed = match ktav::parse(&rendered) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "reparse {} failed: {}\n  canonical text:\n{}",
                    rel, e, rendered
                ));
                continue;
            }
        };
        if value != reparsed {
            failures.push(format!(
                "roundtrip mismatch in {}:\n  original Value:  {:?}\n  reparsed Value:  {:?}\n  canonical text:\n{}",
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

// ---------------------------------------------------------------------------
// Triple-test runner (§ 5.9 conformance)
// ---------------------------------------------------------------------------

#[test]
fn valid_fixtures_canonical_emit() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::canonical: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "valid");
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();

    let mut failures: Vec<String> = Vec::new();
    let mut tested = 0;

    for ktav_path in &files {
        let canonical_path = ktav_path.with_extension("canonical.ktav");
        // Not all fixtures have canonical files yet — skip if missing
        if !canonical_path.exists() {
            // Try the other naming convention
            let stem = ktav_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let alt = ktav_path
                .parent()
                .unwrap()
                .join(format!("{}.canonical.ktav", stem));
            if !alt.exists() {
                continue;
            }
        }

        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();
        let stem = ktav_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let canonical_path = ktav_path
            .parent()
            .unwrap()
            .join(format!("{}.canonical.ktav", stem));
        if !canonical_path.exists() {
            continue;
        }

        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        let expected_canonical = match fs::read_to_string(&canonical_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!(
                    "read canonical {}: {}",
                    canonical_path.display(),
                    e
                ));
                continue;
            }
        };

        // Test 1: parse input
        let value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("parse {}: {}", rel, e));
                continue;
            }
        };

        // Test 2: emit_canonical matches expected
        let actual_canonical = match ktav::render::emit_canonical(&value) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("emit_canonical {}: {}", rel, e));
                continue;
            }
        };
        if actual_canonical != expected_canonical {
            failures.push(format!(
                "canonical mismatch in {}:\n  expected:\n{}\n  actual:\n{}",
                rel, expected_canonical, actual_canonical
            ));
            continue;
        }

        // Test 3: idempotency — re-parse canonical and re-emit
        let reparsed = match ktav::parse(&actual_canonical) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("reparse canonical {}: {}", rel, e));
                continue;
            }
        };
        if value != reparsed {
            failures.push(format!(
                "idempotency mismatch in {}: parse(input) != parse(canonical)",
                rel
            ));
            continue;
        }

        tested += 1;
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} canonical fixture(s) failed:\n{}",
            failures.len(),
            tested + failures.len(),
            failures.join("\n")
        );
    }
    eprintln!("spec_conformance::canonical: {} fixtures passed", tested);
}
