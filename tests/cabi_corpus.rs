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
    candidates.into_iter().find(|p| p.join("versions").join(SPEC_VERSION).is_dir())
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
