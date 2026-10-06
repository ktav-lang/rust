//! Wire round-trip over the spec's `valid/` corpus through the `cabi`
//! thin API: every `valid/` corpus fixture must survive `loads` ->
//! `dumps` with its `Value` unchanged, and every fixture that has a
//! `*.canonical.ktav` twin must satisfy
//! `emit_canonical(loads(bytes)) == twin bytes` byte-for-byte.
//!
//! Skips with a log when the spec checkout is absent (tests/ ships in
//! the published package; spec/ does not) — same policy as
//! tests/spec_conformance.rs.
#![cfg(feature = "cabi")]

use std::fs;
use std::path::{Path, PathBuf};

const SPEC_VERSION: &str = "0.8";

fn resolve_spec_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(env) = std::env::var("KTAV_SPEC_DIR") {
        candidates.push(PathBuf::from(env));
    }
    candidates.push(manifest.join("spec"));
    candidates.push(manifest.join("../spec"));
    candidates
        .into_iter()
        .find(|p| p.join("versions").join(SPEC_VERSION).is_dir())
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
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !stem.ends_with(".canonical") {
                out.push(path);
            }
        }
    }
}

#[test]
fn wire_round_trip_over_the_valid_corpus() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("spec corpus not found; skipping the cabi wire round-trip");
        return;
    };
    let mut files = Vec::new();
    collect_ktav_files(
        &spec_root
            .join("versions")
            .join(SPEC_VERSION)
            .join("tests")
            .join("valid"),
        &mut files,
    );
    files.sort();

    let mut n_round_trip = 0;
    let mut n_canonical = 0;

    for path in &files {
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("{:?}: read failed: {e}", path));
        let src = std::str::from_utf8(&bytes)
            .unwrap_or_else(|e| panic!("{:?}: fixture is not UTF-8: {e}", path));
        let expected = ktav::parse(src)
            .unwrap_or_else(|e| panic!("{:?}: corpus fixture should parse: {e}", path));
        let wire = ktav::cabi::loads(&bytes).unwrap_or_else(|e| panic!("{:?}: loads: {e}", path));
        let text_bytes =
            ktav::cabi::dumps(&wire).unwrap_or_else(|e| panic!("{:?}: dumps: {e}", path));
        let text = std::str::from_utf8(&text_bytes)
            .unwrap_or_else(|e| panic!("{:?}: dumps output not UTF-8: {e}", path));
        let actual = ktav::parse(text)
            .unwrap_or_else(|e| panic!("{:?}: dumps output should re-parse: {e}", path));
        assert_eq!(
            actual, expected,
            "{:?}: Value changed across loads -> dumps",
            path
        );
        n_round_trip += 1;

        // Canonical byte oracle, when the spec ships a twin. Build the
        // sibling path with with_file_name — never string concat on
        // full paths (Windows separators).
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let twin = path.with_file_name(format!("{stem}.canonical.ktav"));
        if twin.exists() {
            let twin_bytes =
                fs::read(&twin).unwrap_or_else(|e| panic!("{:?}: read twin failed: {e}", twin));
            let canonical = ktav::cabi::emit_canonical(&wire)
                .unwrap_or_else(|e| panic!("{:?}: emit_canonical: {e}", path));
            assert_eq!(canonical, twin_bytes, "{:?}: canonical bytes diverge", path);
            n_canonical += 1;
        }
    }

    eprintln!(
        "cabi wire round-trip over {n_round_trip} valid corpus fixtures; {n_canonical} canonical byte oracles compared"
    );
    // Measured 221 valid fixtures (221 canonical twins) on the 0.7
    // spec checkout; a smaller number means a truncated corpus checkout.
    const MIN_VALID_FIXTURES: usize = 100;
    assert!(
        n_round_trip >= MIN_VALID_FIXTURES,
        "only {n_round_trip} valid fixtures exercised; corpus checkout looks truncated"
    );
}

/// Independent raw-byte line-terminator count for the § 6 line bound:
/// each LF, lone CR, or CR LF pair is one terminator (§ 3.2).
fn count_terminators(bytes: &[u8]) -> u64 {
    let mut n = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                n += 1;
                i += 1;
            }
            b'\r' => {
                n += 1;
                i += if bytes.get(i + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
            }
            _ => i += 1,
        }
    }
    n
}

/// Every `invalid/` fixture through the C ABI `loads`: an Err envelope
/// whose class matches the oracle, plus the § 6 baseline location
/// invariants — a 1-based integer line bounded by the raw terminator
/// count, and an in-bounds half-open span. raw_bytes fixtures
/// additionally pin the first-invalid offset (computed here from
/// std::str::from_utf8, never from the oracle) and a non-empty span.
#[test]
fn invalid_corpus_through_cabi_loads_carries_baseline_locations() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("spec corpus not found; skipping the cabi invalid-corpus run");
        return;
    };
    let tests = spec_root.join("versions").join(SPEC_VERSION).join("tests");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tests.join("manifest.json"))
            .expect("manifest.json must be readable when the corpus is present"),
    )
    .expect("manifest.json is valid JSON");
    let raw_bytes: std::collections::BTreeSet<String> = manifest["fixture_flags"]
        .as_array()
        .map(|flags| {
            flags
                .iter()
                .filter(|e| {
                    e["flags"]
                        .as_array()
                        .is_some_and(|f| f.iter().any(|f| f == "raw_bytes"))
                })
                .filter_map(|e| e["fixture"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut files = Vec::new();
    collect_ktav_files(&tests.join("invalid"), &mut files);
    files.sort();
    assert!(
        files.len() >= 50,
        "only {} invalid fixtures found; corpus checkout looks truncated",
        files.len()
    );

    let mut checked = 0usize;
    for path in &files {
        let rel = path
            .strip_prefix(tests.join("invalid"))
            .unwrap_or(path)
            .display()
            .to_string()
            .replace('\\', "/");
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{rel}: read failed: {e}"));
        let oracle: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(path.with_extension("json"))
                .unwrap_or_else(|e| panic!("{rel}: unreadable oracle: {e}")),
        )
        .unwrap_or_else(|e| panic!("{rel}: oracle is not valid JSON: {e}"));
        let expected = oracle["expected_error"]
            .as_str()
            .unwrap_or_else(|| panic!("{rel}: oracle has no expected_error string"));

        let payload = match ktav::cabi::loads(&bytes) {
            Err(envelope) => envelope,
            Ok(_) => panic!("{rel}: invalid fixture must be rejected by cabi loads"),
        };
        let v: serde_json::Value = serde_json::from_str(&payload)
            .unwrap_or_else(|e| panic!("{rel}: Err payload is not JSON: {e}: {payload}"));
        assert_eq!(
            v["error"], expected,
            "{rel}: error class diverges from oracle"
        );
        assert!(
            v["message"].as_str().is_some(),
            "{rel}: message must render"
        );

        let max_line = count_terminators(&bytes) + 1;
        let line = v["line"].as_u64().unwrap_or_else(|| {
            panic!(
                "{rel}: § 6 requires a 1-based integer line, got {:?}",
                v["line"]
            )
        });
        assert!(
            line >= 1 && line <= max_line,
            "{rel}: line {line} outside 1..={max_line}"
        );

        let span = v["span"]
            .as_object()
            .unwrap_or_else(|| panic!("{rel}: § 6 requires a span object, got {:?}", v["span"]));
        let start = span["start"].as_u64().unwrap_or_else(|| {
            panic!(
                "{rel}: span.start must be an integer, got {:?}",
                span["start"]
            )
        });
        let end = span["end"]
            .as_u64()
            .unwrap_or_else(|| panic!("{rel}: span.end must be an integer, got {:?}", span["end"]));
        assert!(
            start <= end && end <= bytes.len() as u64,
            "{rel}: span {start}..{end} out of bounds for {} bytes",
            bytes.len()
        );

        // The manifest keys fixtures by stem (no extension); `rel`
        // carries the `.ktav` suffix, so strip it before the lookup.
        if raw_bytes.contains(rel.strip_suffix(".ktav").unwrap_or(&rel)) {
            let bad = match std::str::from_utf8(&bytes) {
                Err(bad) => bad,
                Ok(_) => panic!("{rel}: flagged raw_bytes but the bytes are valid UTF-8"),
            };
            assert_eq!(
                start,
                bad.valid_up_to() as u64,
                "{rel}: span.start must be the first-invalid offset"
            );
            assert!(start < end, "{rel}: span must cover the malformed bytes");
        } else {
            assert!(
                std::str::from_utf8(&bytes).is_ok(),
                "{rel}: bytes are invalid UTF-8 but the manifest does not flag raw_bytes"
            );
        }
        checked += 1;
    }
    eprintln!("cabi invalid corpus: {checked} fixtures checked for baseline locations");
}
