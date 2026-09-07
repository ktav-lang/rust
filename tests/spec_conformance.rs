//! Language-agnostic conformance suite from `ktav-lang/spec`.
//!
//! Points at `versions/<SPEC_VERSION>/tests/{valid,invalid}` and checks:
//!   - every `valid/**/*.ktav` (non-canonical) parses and its `Value`
//!     equals the oracle in the sibling `.json` file;
//!   - every `valid/**/*.canonical.ktav` re-parses to the same `Value`;
//!   - `emit_canonical(parse(input.ktav))` matches `name.canonical.ktav`;
//!   - every `invalid/**/*.ktav` is rejected by `ktav::parse` (or, for
//!     byte-level invalid UTF-8 fixtures, by `ktav::from_file` with
//!     `Error::InvalidUtf8` — spec § 6.15);
//!   - every `unrepresentable/**/*.json` Value is rejected by all three
//!     writer surfaces with the exact `ReasonCode` (spec § 5.9.0);
//!   - every `parseable-unrepresentable/*.ktav` parses and matches its
//!     JSON oracle `value`, yet every writer surface rejects it.
//!
//! Spec root resolution (first match wins):
//!   1. env var `KTAV_SPEC_DIR` (absolute path to the spec-repo root);
//!   2. `<CARGO_MANIFEST_DIR>/spec`  — git submodule `ktav-lang/spec`;
//!   3. `<CARGO_MANIFEST_DIR>/../spec` — sibling directory (local dev);
//!   4. if none contains a `versions/` dir, the test logs and returns —
//!      it does not fail, so CI without the spec checkout stays green.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::{ObjectMap, ReasonCode, Value};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

const SPEC_VERSION: &str = "0.7";

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

/// Ordered recursive comparison. Numbers are kind-aware and exact:
/// an integer-spelled token only matches an integer-spelled token and
/// a float-spelled token only a float-spelled one (`1` != `1.0`);
/// integers compare exactly at any magnitude (no f64 round-trip);
/// floats compare as binary64 values with the sign of zero preserved
/// (`0.0` != `-0.0`, spec § 5.9.8), so different spellings of the same
/// value (`1e9` vs `1000000000.0`) still compare equal.
fn json_eq_ordered(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(x), JsonValue::Bool(y)) => x == y,
        (JsonValue::Number(x), JsonValue::Number(y)) => {
            let x_str = x.to_string();
            let y_str = y.to_string();
            match (
                json_number_is_float_spelled(&x_str),
                json_number_is_float_spelled(&y_str),
            ) {
                (false, false) => json_integers_equal(&x_str, &y_str),
                (true, true) => json_floats_equal(&x_str, &y_str),
                // Kind mismatch: a ktav Integer must never equal a Float
                // regardless of numeric value, in either direction.
                (false, true) | (true, false) => false,
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

/// Classify a JSON number token by its lexical spelling, the same rule
/// `json_to_ktav` applies in the oracle→Value direction: `.`/`e`/`E`
/// makes it float-spelled, otherwise integer-spelled. This recovers the
/// original `ktav::Value` kind on the ktav side because both scalar
/// kinds store canonical spellings (§ 5.9.8: Integer is base-10
/// decimal; Float is ryu decimal/scientific and always carries `.` or
/// `e`), and `serde_json` is compiled with `arbitrary_precision`, so
/// `Number::to_string()` returns the exact token.
fn json_number_is_float_spelled(token: &str) -> bool {
    token.contains(['.', 'e', 'E'])
}

/// Exact integer comparison. JSON integers carry no exponent or
/// leading zeros, so after normalising `-0`, string equality is exact
/// numeric equality at any magnitude — no f64 round-trip, which cannot
/// represent all i64 values (e.g. `9223372036854775807` parses to the
/// same f64 as `9223372036854775806`).
fn json_integers_equal(x: &str, y: &str) -> bool {
    let norm = if x == "-0" { "0" } else { x };
    let normy = if y == "-0" { "0" } else { y };
    norm == normy
}

/// Float comparison within the crate's binary64 domain. Values compare
/// as `f64` (so `1e9` == `1000000000.0`), except that the sign of zero
/// is significant: § 5.9.8 keeps `0.0` and `-0.0` as distinct canonical
/// spellings, so the comparator must not collapse them through IEEE
/// 754 `0.0 == -0.0`. Spellings outside the finite binary64 domain
/// (e.g. `1e400`, which parses to infinity) have no value to compare —
/// they match only textually.
fn json_floats_equal(x: &str, y: &str) -> bool {
    match (x.parse::<f64>(), y.parse::<f64>()) {
        (Ok(xf), Ok(yf)) if xf.is_finite() && yf.is_finite() => {
            if xf == 0.0 && yf == 0.0 {
                xf.is_sign_negative() == yf.is_sign_negative()
            } else {
                xf == yf
            }
        }
        _ => x == y,
    }
}

#[test]
fn json_eq_ordered_number_semantics() {
    let n = |s: &str| JsonValue::Number(JsonNumber::from_string_unchecked(s.to_string()));

    // Kind is respected in both directions (review finding F5).
    assert!(json_eq_ordered(&n("1"), &n("1")));
    assert!(!json_eq_ordered(&n("1"), &n("1.0")));
    assert!(!json_eq_ordered(&n("1.0"), &n("1")));
    assert!(!json_eq_ordered(&n("0"), &n("0.0")));

    // Integers compare exactly — no f64 round-trip: both spellings
    // parse to the same f64 but are different integers.
    assert!(json_eq_ordered(
        &n("9223372036854775806"),
        &n("9223372036854775806")
    ));
    assert!(!json_eq_ordered(
        &n("9223372036854775806"),
        &n("9223372036854775807")
    ));
    assert!(json_eq_ordered(
        &n("-9223372036854775808"),
        &n("-9223372036854775808")
    ));

    // Floats compare as binary64 values — different spellings of the
    // same value are equal...
    assert!(json_eq_ordered(&n("1e9"), &n("1000000000.0")));
    assert!(json_eq_ordered(
        &n("9007199254740992.0"),
        &n("9.007199254740992e15")
    ));
    // ...but the sign of zero is significant (spec § 5.9.8).
    assert!(json_eq_ordered(&n("0.0"), &n("0.0")));
    assert!(json_eq_ordered(&n("-0.0"), &n("-0.0")));
    assert!(!json_eq_ordered(&n("0.0"), &n("-0.0")));
    assert!(!json_eq_ordered(&n("-0.0"), &n("0.0")));

    // Non-number leaves are unaffected.
    assert!(json_eq_ordered(
        &JsonValue::Array(vec![n("1"), n("2")]),
        &JsonValue::Array(vec![n("1"), n("2")])
    ));
    assert!(!json_eq_ordered(
        &JsonValue::Array(vec![n("1"), n("2")]),
        &JsonValue::Array(vec![n("1"), n("2.0")])
    ));
}

/// Walk `root` recursively and collect every `.json` file.
fn collect_json_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(path);
        }
    }
}

/// Parse a fixture's `unrepresentable_reason` string to the exact
/// `ReasonCode`. An unknown name panics — that IS the schema validation.
fn reason_code_from_name(name: &str) -> ReasonCode {
    match name {
        "ScalarRoot" => ReasonCode::ScalarRoot,
        "EmptyKeyName" => ReasonCode::EmptyKeyName,
        "NonFiniteFloat" => ReasonCode::NonFiniteFloat,
        "CRByte" => ReasonCode::CRByte,
        "BothFormsRequired" => ReasonCode::BothFormsRequired,
        "TrailingWhitespaceCollision" => ReasonCode::TrailingWhitespaceCollision,
        "LeadingWhitespaceCollision" => ReasonCode::LeadingWhitespaceCollision,
        other => panic!("fixture `unrepresentable_reason` {other:?} is not a known ReasonCode"),
    }
}

/// Validate the shared 3-field fixture schema (`value`,
/// `unrepresentable_reason`, `note`) and extract the expected
/// `ReasonCode`. Panics loudly on any schema violation.
fn validate_reason_fixture(fixture: &JsonValue, rel: impl std::fmt::Display) -> ReasonCode {
    let map = fixture
        .as_object()
        .unwrap_or_else(|| panic!("fixture {rel}: top-level JSON is not an object"));
    if map.len() != 3 {
        panic!(
            "fixture {rel}: expected exactly the fields value/unrepresentable_reason/note, \
             found {} field(s)",
            map.len()
        );
    }
    for required in ["value", "unrepresentable_reason", "note"] {
        if !map.contains_key(required) {
            panic!("fixture {rel}: missing required field {required:?}");
        }
    }
    let reason = map["unrepresentable_reason"]
        .as_str()
        .unwrap_or_else(|| panic!("fixture {rel}: unrepresentable_reason is not a string"));
    let note = map["note"]
        .as_str()
        .unwrap_or_else(|| panic!("fixture {rel}: note is not a string"));
    if note.is_empty() {
        panic!("fixture {rel}: note is empty");
    }
    reason_code_from_name(reason)
}

/// Membership test over every category name a `invalid/` oracle may
/// spell: the `ErrorKind` variant names (see `ErrorKind::code_name`)
/// plus `InvalidUtf8` — a top-level `Error` variant, not an
/// `ErrorKind`, because § 6.15 byte-level rejection happens before any
/// line-oriented parsing. An oracle naming anything else is a corpus
/// schema violation and must fail loudly.
fn is_known_error_category(name: &str) -> bool {
    matches!(
        name,
        "MissingSeparatorSpace"
            | "InvalidTypedScalar"
            | "LossyScalar"
            | "DuplicateKey"
            | "KeyPathConflict"
            | "EmptyKey"
            | "InvalidKey"
            | "UnclosedCompound"
            | "UnbalancedBracket"
            | "InlineNonEmptyCompound"
            | "MissingSeparator"
            | "UnterminatedInlineCompound"
            | "UnterminatedQuotedKey"
            | "MalformedInlineCompound"
            | "BadEscapeSequence"
            | "OrphanLineAfterTopLevelInline"
            | "Other"
            | "InvalidUtf8"
    )
}

/// Validate the shared 2-field invalid-fixture schema (`expected_error`,
/// `note`) and return the expected category name. Panics loudly on any
/// schema violation — including an `expected_error` that names no known
/// category; that IS the schema validation (mirrors
/// `validate_reason_fixture`).
fn validate_error_fixture(fixture: &JsonValue, rel: impl std::fmt::Display) -> &str {
    let map = fixture
        .as_object()
        .unwrap_or_else(|| panic!("fixture {rel}: top-level JSON is not an object"));
    if map.len() != 2 {
        panic!(
            "fixture {rel}: expected exactly the fields expected_error/note, \
             found {} field(s)",
            map.len()
        );
    }
    for required in ["expected_error", "note"] {
        if !map.contains_key(required) {
            panic!("fixture {rel}: missing required field {required:?}");
        }
    }
    let expected = map["expected_error"]
        .as_str()
        .unwrap_or_else(|| panic!("fixture {rel}: expected_error is not a string"));
    let note = map["note"]
        .as_str()
        .unwrap_or_else(|| panic!("fixture {rel}: note is not a string"));
    if note.is_empty() {
        panic!("fixture {rel}: note is empty");
    }
    if !is_known_error_category(expected) {
        panic!("fixture {rel}: expected_error {expected:?} is not a known error category");
    }
    expected
}

/// An invalid fixture with its validated oracle. `bytes` are read once
/// here so both the owned-API and thin-API passes walk the identical
/// corpus.
struct InvalidFixture {
    /// Path relative to the `invalid/` dir, for messages.
    rel: String,
    path: PathBuf,
    bytes: Vec<u8>,
    /// Validated `expected_error` category name.
    expected: String,
}

/// Collect every invalid fixture with its validated oracle. Panics if
/// the dir is missing, empty, or any oracle is missing/unreadable or
/// fails schema validation.
fn collect_invalid_fixtures(spec_root: &Path) -> Vec<InvalidFixture> {
    let root = tests_dir(spec_root, "invalid");
    if !root.is_dir() {
        panic!(
            "spec 0.7 resolved but the invalid fixture dir is missing: {}",
            root.display()
        );
    }
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        panic!("no `.ktav` fixtures found under {}", root.display());
    }

    let mut fixtures = Vec::new();
    for ktav_path in &files {
        let rel = ktav_path
            .strip_prefix(&root)
            .unwrap_or(ktav_path)
            .display()
            .to_string();
        let json_path = ktav_path.with_extension("json");
        let oracle_src = match fs::read_to_string(&json_path) {
            Ok(t) => t,
            Err(e) => panic!("fixture {}: missing/unreadable `.json` sibling: {}", rel, e),
        };
        let fixture: JsonValue = match serde_json::from_str(&oracle_src) {
            Ok(v) => v,
            Err(e) => panic!("fixture {}: oracle json parse error: {}", rel, e),
        };
        let expected = validate_error_fixture(&fixture, &rel).to_string();
        let bytes = match fs::read(ktav_path) {
            Ok(b) => b,
            Err(e) => panic!("fixture {}: unreadable `.ktav` file: {}", rel, e),
        };
        fixtures.push(InvalidFixture {
            rel,
            path: ktav_path.clone(),
            bytes,
            expected,
        });
    }
    fixtures
}

/// Convert a `serde_json::Value` (a fixture oracle) into a `ktav::Value`.
fn json_to_ktav(v: &JsonValue) -> Value {
    match v {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::String(s) => {
            Value::String(s.parse().unwrap_or_else(|_| panic!("string scalar: {s:?}")))
        }
        JsonValue::Number(n) => {
            // serde_json is compiled with `arbitrary_precision`, so
            // `to_string()` returns the exact lexical token.
            let token = n.to_string();
            if !token.contains(['.', 'e', 'E']) {
                Value::Integer(
                    token
                        .parse()
                        .unwrap_or_else(|_| panic!("integer scalar: {token:?}")),
                )
            } else {
                Value::Float(
                    token
                        .parse()
                        .unwrap_or_else(|_| panic!("float scalar: {token:?}")),
                )
            }
        }
        JsonValue::Array(items) => Value::Array(items.iter().map(json_to_ktav).collect()),
        JsonValue::Object(map) => {
            // `$float` sentinel: only meaningful in the `unrepresentable/`
            // category — records NaN / ±Infinity as a one-field object.
            if map.len() == 1 {
                if let Some(JsonValue::String(f)) = map.get("$float") {
                    return Value::Float(
                        f.parse()
                            .unwrap_or_else(|_| panic!("$float sentinel: {f:?}")),
                    );
                }
            }
            let mut m = ObjectMap::default();
            for (k, val) in map {
                m.insert(k.as_str().into(), json_to_ktav(val));
            }
            Value::Object(m)
        }
    }
}

/// Thin-parser gaps tracked under SEPARATE review findings, excluded from
/// the two thin-API runners below. Each entry pins the exact category the
/// thin parser produces today, so an entry rots LOUDLY: if a listed
/// fixture starts passing, or fails with a different category, the runner
/// panics with removal instructions instead of silently passing.
///
/// F3 (review finding F3, P2): the linear event model closes a
/// dotted-key object before later lines can extend it, rejecting two
/// spec-valid documents with `KeyPathConflict`.
const THIN_KNOWN_GAPS: &[(&str, &str, &str)] = &[
    // (path relative to the bucket dir, finding, category the thin parser produces today)
    // --- F3: valid fixtures rejected by the thin API ---
    (
        "dotted_keys/extend_explicit_object.ktav",
        "F3",
        "KeyPathConflict",
    ),
    (
        "dotted_keys/reopen_after_sibling.ktav",
        "F3",
        "KeyPathConflict",
    ),
];

/// Look up a fixture in [`THIN_KNOWN_GAPS`] by its path relative to the
/// bucket dir (`invalid/` rels in the thin invalid runner, `valid/` rels
/// in the thin valid runner). The two tables never collide: no `valid/`
/// fixture shares a name with a listed `invalid/` fixture or vice versa.
fn thin_known_gap(rel: &str) -> Option<&'static str> {
    // `rel` comes from `Path::display`, so separators are OS-native;
    // the table spells `/`.
    let normalized = rel.replace('\\', "/");
    THIN_KNOWN_GAPS
        .iter()
        .find(|(path, _, _)| *path == normalized.as_str())
        .map(|(_, finding, _)| *finding)
}

/// Assert that a known-gap fixture still fails with EXACTLY its pinned
/// category. `Ok(())` or a changed category both panic loudly so the
/// allowlist cannot rot silently.
fn check_known_gap(rel: &str, finding: &str, pinned: &str, result: &Result<(), ktav::Error>) {
    let actual = match result {
        Ok(()) => panic!(
            "thin parser now accepts {rel}: finding {finding} appears fixed — \
             remove the entry from THIN_KNOWN_GAPS and re-run"
        ),
        Err(ktav::Error::Structured(kind)) => kind.code_name(),
        Err(other) => panic!(
            "thin parser's category for known-gap fixture {rel} changed from \
             {pinned} to a non-structured error: {other}; update \
             THIN_KNOWN_GAPS (and the linked finding) consciously"
        ),
    };
    if actual != pinned {
        panic!(
            "thin parser's category for known-gap fixture {rel} changed from \
             {pinned} to {actual}: update THIN_KNOWN_GAPS (and the linked \
             finding) consciously"
        );
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
    let fixtures = collect_invalid_fixtures(&spec_root);

    let mut failures: Vec<String> = Vec::new();

    for fixture in &fixtures {
        match std::str::from_utf8(&fixture.bytes) {
            Ok(text) => match ktav::parse(text) {
                Ok(_) => failures.push(format!(
                    "invalid fixture {} parsed successfully, expected {} rejection",
                    fixture.rel, fixture.expected
                )),
                Err(e) => match &e {
                    ktav::Error::Structured(kind) => {
                        let actual = kind.code_name();
                        if actual != fixture.expected {
                            failures.push(format!(
                                "invalid fixture {}: expected category {}, got {} ({})",
                                fixture.rel, fixture.expected, actual, e
                            ));
                        }
                    }
                    other => failures.push(format!(
                        "invalid fixture {}: expected Structured error with category {}, got: {}",
                        fixture.rel, fixture.expected, other
                    )),
                },
            },
            Err(_) => {
                // Byte-level invalid UTF-8 (spec § 6.15): must be rejected
                // by the file entry point with `Error::InvalidUtf8`.
                // NOTE: `ktav::Value` does not implement DeserializeOwned,
                // so the target type is `String`; the UTF-8 validation in
                // `from_file` happens before any deserialization, so the
                // error variant is identical for any `T`.
                if fixture.expected != "InvalidUtf8" {
                    panic!(
                        "fixture {}: bytes are invalid UTF-8 but the oracle expects {}",
                        fixture.rel, fixture.expected
                    );
                }
                match ktav::from_file::<String, _>(&fixture.path) {
                    Ok(_) => failures.push(format!(
                        "invalid UTF-8 fixture accepted by from_file: {}",
                        fixture.rel
                    )),
                    Err(e) => match &e {
                        ktav::Error::InvalidUtf8 { .. } => {}
                        other => failures.push(format!(
                            "invalid UTF-8 fixture {}: expected Error::InvalidUtf8, got: {}",
                            fixture.rel, other
                        )),
                    },
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} invalid fixture(s) failed:\n{}",
            failures.len(),
            fixtures.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::invalid: {} fixtures rejected with matching categories (owned API)",
        fixtures.len()
    );
}

#[test]
fn invalid_fixtures_categories_match_oracles_via_thin_api() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::invalid (thin API): spec dir not found");
        return;
    };
    let fixtures = collect_invalid_fixtures(&spec_root);

    let mut failures: Vec<String> = Vec::new();
    let mut invalid_utf8 = 0;
    let mut known_gaps = 0;

    for fixture in &fixtures {
        let Ok(text) = std::str::from_utf8(&fixture.bytes) else {
            // The thin API takes `&str`, so byte-level rejection
            // (spec § 6.15) is out of its domain — covered by the
            // `from_file` check in `invalid_fixtures_are_rejected`.
            invalid_utf8 += 1;
            continue;
        };
        let result = ktav::parse_events(text, |_ev: ktav::ParseEvent<'_>| ());
        if let Some(finding) = thin_known_gap(&fixture.rel) {
            // Tracked under a separate review finding: pin the exact
            // category the thin parser produces today so the entry rots
            // loudly when the finding is fixed.
            let pinned = THIN_KNOWN_GAPS
                .iter()
                .find(|(path, _, _)| *path == fixture.rel.replace('\\', "/").as_str())
                .map(|(_, _, cat)| *cat)
                .unwrap();
            known_gaps += 1;
            check_known_gap(&fixture.rel, finding, pinned, &result);
            continue;
        }
        match result {
            Ok(()) => failures.push(format!(
                "thin API accepted invalid fixture {}, expected {}",
                fixture.rel, fixture.expected
            )),
            Err(e) => match &e {
                ktav::Error::Structured(kind) => {
                    let actual = kind.code_name();
                    if actual != fixture.expected {
                        failures.push(format!(
                            "invalid fixture {}: expected category {}, got {} ({})",
                            fixture.rel, fixture.expected, actual, e
                        ));
                    }
                }
                other => failures.push(format!(
                    "invalid fixture {}: expected Structured error with category {}, got: {}",
                    fixture.rel, fixture.expected, other
                )),
            },
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} invalid fixture(s) failed (thin API):\n{}",
            failures.len(),
            fixtures.len() - invalid_utf8,
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::invalid (thin API): {} fixtures rejected with matching categories, {} excluded under known finding F3 ({} invalid-UTF-8 fixtures are byte-entry-only)",
        fixtures.len() - invalid_utf8 - known_gaps,
        known_gaps,
        invalid_utf8
    );
}

#[test]
fn valid_fixtures_parse_via_thin_api() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::valid (thin API): spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "valid");
    if !root.is_dir() {
        panic!(
            "spec 0.7 resolved but the valid fixture dir is missing: {}",
            root.display()
        );
    }
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        panic!("no `.ktav` fixtures found under {}", root.display());
    }

    let mut failures: Vec<String> = Vec::new();
    let mut known_gaps = 0;

    for ktav_path in &files {
        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();
        let rel = rel.to_string();
        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        let result = ktav::parse_events(&text, |_| {});
        if let Some(finding) = thin_known_gap(&rel) {
            // Tracked under a separate review finding: pin the exact
            // category the thin parser produces today so the entry rots
            // loudly when the finding is fixed.
            let pinned = THIN_KNOWN_GAPS
                .iter()
                .find(|(path, _, _)| *path == rel.replace('\\', "/").as_str())
                .map(|(_, _, cat)| *cat)
                .unwrap();
            known_gaps += 1;
            check_known_gap(&rel, finding, pinned, &result);
            continue;
        }
        if let Err(e) = result {
            failures.push(format!("thin API rejected valid fixture {}: {}", rel, e));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixture(s) failed (thin API):\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::valid (thin API): {} fixtures accepted, {} excluded under known finding F3",
        files.len() - known_gaps,
        known_gaps
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
        let stem = ktav_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let canonical_path = ktav_path
            .parent()
            .unwrap()
            .join(format!("{}.canonical.ktav", stem));
        // Not all fixtures have canonical files yet — skip if missing.
        if !canonical_path.exists() {
            continue;
        }

        let rel_path = ktav_path.strip_prefix(&root).unwrap_or(ktav_path);
        let rel = rel_path.display();
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

        // Test 2: emit_canonical matches expected.
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

/// Serde mirror of a `ktav::Value`: lets the serde text serializer
/// (`ktav::to_string`) be exercised on Values (which deliberately do not
/// implement `Serialize`). Scalars go through their canonical text form;
/// non-finite floats surface as raw `f64`s so the serializer's own
/// rejection (`NonFiniteFloat`) fires exactly as it would for a real
/// `Serialize` type holding the same value.
struct SerValue<'a>(&'a Value);

impl Serialize for SerValue<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Null => ser.serialize_none(),
            Value::Bool(b) => ser.serialize_bool(*b),
            Value::Integer(i) => {
                let n: i64 = i.to_string().parse().map_err(serde::ser::Error::custom)?;
                ser.serialize_i64(n)
            }
            Value::Float(f) => {
                ser.serialize_f64(f.to_string().parse().map_err(serde::ser::Error::custom)?)
            }
            Value::String(s) => ser.serialize_str(s.as_ref()),
            Value::Array(items) => ser.collect_seq(items.iter().map(SerValue)),
            Value::Object(obj) => {
                let mut map = ser.serialize_map(Some(obj.len()))?;
                for (k, v) in obj {
                    map.serialize_entry(&k.to_string(), &SerValue(v))?;
                }
                map.end()
            }
        }
    }
}

#[test]
fn unrepresentable_fixtures_are_rejected_with_reason_codes() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::unrepresentable: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "unrepresentable");
    if !root.is_dir() {
        panic!(
            "spec 0.7 resolved but the unrepresentable fixture dir is missing: {}",
            root.display()
        );
    }
    let mut files = Vec::new();
    collect_json_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        panic!("no `.json` fixtures found under {}", root.display());
    }

    let mut failures: Vec<String> = Vec::new();

    for fixture_path in &files {
        let rel = fixture_path
            .strip_prefix(&root)
            .unwrap_or(fixture_path)
            .display();
        let src = match fs::read_to_string(fixture_path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };
        let fixture: JsonValue = match serde_json::from_str(&src) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("json parse {}: {}", rel, e));
                continue;
            }
        };
        let expected = validate_reason_fixture(&fixture, &rel);
        let value = json_to_ktav(&fixture["value"]);

        for (surface, err) in [
            ("emit_canonical", ktav::render::emit_canonical(&value).err()),
            ("render", Some(ktav::render::render(&value).unwrap_err())),
            (
                "to_string",
                Some(ktav::to_string(&SerValue(&value)).unwrap_err()),
            ),
        ] {
            let Some(err) = err else {
                failures.push(format!(
                    "{}: expected rejection with {:?} but {} succeeded",
                    rel, expected, surface
                ));
                continue;
            };
            match err.reason_code() {
                Some(actual) if actual == expected => {}
                Some(actual) => failures.push(format!(
                    "{}: {}: expected reason code {:?}, got {:?} ({})",
                    rel, surface, expected, actual, err
                )),
                None => failures.push(format!(
                    "{}: {}: expected rejection with {:?}, got non-representability error: {}",
                    rel, surface, expected, err
                )),
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} unrepresentable fixture(s) failed:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::unrepresentable: {} fixtures rejected",
        files.len()
    );
}

#[test]
fn parseable_unrepresentable_fixtures_reject_canonical_emit() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::parseable_unrepresentable: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "parseable-unrepresentable");
    if !root.is_dir() {
        panic!(
            "spec 0.7 resolved but the parseable-unrepresentable fixture dir is missing: {}",
            root.display()
        );
    }
    let mut files = Vec::new();
    collect_ktav_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        panic!("no `.ktav` fixtures found under {}", root.display());
    }

    let mut failures: Vec<String> = Vec::new();

    for ktav_path in &files {
        let rel = ktav_path.strip_prefix(&root).unwrap_or(ktav_path).display();
        let stem = ktav_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // The spec forbids a canonical spelling for these fixtures.
        let canonical = ktav_path
            .parent()
            .unwrap()
            .join(format!("{}.canonical.ktav", stem));
        if canonical.exists() {
            panic!(
                "fixture {}: a `.canonical.ktav` file exists but the spec \
                 forbids canonical output for parseable-unrepresentable values",
                rel
            );
        }

        let json_path = ktav_path.with_extension("json");
        let oracle_src = match fs::read_to_string(&json_path) {
            Ok(t) => t,
            Err(e) => panic!("fixture {}: missing/unreadable `.json` sibling: {}", rel, e),
        };
        let fixture: JsonValue = match serde_json::from_str(&oracle_src) {
            Ok(v) => v,
            Err(e) => panic!("fixture {}: oracle json parse error: {}", rel, e),
        };
        let expected = validate_reason_fixture(&fixture, &rel);

        let text = match fs::read_to_string(ktav_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", rel, e));
                continue;
            }
        };

        // These must PARSE — they are parser-producible.
        let value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("parse {}: expected success, got: {}", rel, e));
                continue;
            }
        };

        // The parsed Value must match the oracle.
        let actual = ktav_to_json(&value);
        if !json_eq_ordered(&actual, &fixture["value"]) {
            failures.push(format!(
                "oracle mismatch in {}:\n  expected: {}\n  actual:   {}",
                rel, fixture["value"], actual
            ));
        }

        // ...but every writer surface must reject it.
        for (surface, err) in [
            ("emit_canonical", ktav::render::emit_canonical(&value).err()),
            ("render", Some(ktav::render::render(&value).unwrap_err())),
            (
                "to_string",
                Some(ktav::to_string(&SerValue(&value)).unwrap_err()),
            ),
        ] {
            let Some(err) = err else {
                failures.push(format!(
                    "{}: expected rejection with {:?} but {} succeeded",
                    rel, expected, surface
                ));
                continue;
            };
            match err.reason_code() {
                Some(actual_code) if actual_code == expected => {}
                Some(actual_code) => failures.push(format!(
                    "{}: {}: expected reason code {:?}, got {:?} ({})",
                    rel, surface, expected, actual_code, err
                )),
                None => failures.push(format!(
                    "{}: {}: expected rejection with {:?}, got non-representability error: {}",
                    rel, surface, expected, err
                )),
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} parseable-unrepresentable fixture(s) failed:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::parseable-unrepresentable: {} fixtures rejected",
        files.len()
    );
}
