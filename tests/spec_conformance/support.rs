//! Shared fixtures/spec-root helpers for the spec conformance suite.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::ReasonCode;
use serde_json::Value as JsonValue;

const SPEC_VERSION: &str = "0.7";

pub(crate) fn resolve_spec_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(env) = std::env::var("KTAV_SPEC_DIR") {
        candidates.push(PathBuf::from(env));
    }
    candidates.push(manifest.join("spec"));
    candidates.push(manifest.join("../spec"));
    candidates.into_iter().find(|p| p.join("versions").is_dir())
}

pub(crate) fn tests_dir(spec_root: &Path, bucket: &str) -> PathBuf {
    spec_root
        .join("versions")
        .join(SPEC_VERSION)
        .join("tests")
        .join(bucket)
}

/// Walk `root` recursively and collect every `.ktav` file that is NOT
/// a `*.canonical.ktav` file.
pub(crate) fn collect_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
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

/// Walk `root` recursively and collect every `.json` file.
pub(crate) fn collect_json_files(root: &Path, out: &mut Vec<PathBuf>) {
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
pub(crate) fn validate_reason_fixture(
    fixture: &JsonValue,
    rel: impl std::fmt::Display,
) -> ReasonCode {
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
pub(crate) struct InvalidFixture {
    /// Path relative to the `invalid/` dir, for messages.
    pub(crate) rel: String,
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
    /// Validated `expected_error` category name.
    pub(crate) expected: String,
}

/// Collect every invalid fixture with its validated oracle. Panics if
/// the dir is missing, empty, or any oracle is missing/unreadable or
/// fails schema validation.
pub(crate) fn collect_invalid_fixtures(spec_root: &Path) -> Vec<InvalidFixture> {
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

/// Thin-parser gaps tracked under SEPARATE review findings, excluded from
/// the two thin-API runners below. Each entry pins the exact category the
/// thin parser produces today, so an entry rots LOUDLY: if a listed
/// fixture starts passing, or fails with a different category, the runner
/// panics with removal instructions instead of silently passing.
///
/// Currently empty: F3 (dotted-key re-opens rejected as `KeyPathConflict`)
/// is fixed.
pub(crate) const THIN_KNOWN_GAPS: &[(&str, &str, &str)] = &[];

/// Look up a fixture in [`THIN_KNOWN_GAPS`] by its path relative to the
/// bucket dir (`invalid/` rels in the thin invalid runner, `valid/` rels
/// in the thin valid runner). The two tables never collide: no `valid/`
/// fixture shares a name with a listed `invalid/` fixture or vice versa.
pub(crate) fn thin_known_gap(rel: &str) -> Option<&'static str> {
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
pub(crate) fn check_known_gap(
    rel: &str,
    finding: &str,
    pinned: &str,
    result: &Result<(), ktav::Error>,
) {
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
