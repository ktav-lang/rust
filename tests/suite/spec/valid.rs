//! Valid-fixture conformance tests (owned API, thin API, roundtrip, canonical emit).

use std::fs;

use serde_json::Value as JsonValue;

use super::oracle::{json_eq_ordered, ktav_to_json};
use super::support::{
    check_known_gap, collect_ktav_files, resolve_spec_root, tests_dir, thin_known_gap,
    THIN_KNOWN_GAPS,
};

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
fn valid_fixtures_match_oracle_via_thin_api() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::valid (thin API): spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "valid");
    if !root.is_dir() {
        panic!(
            "spec 0.8 resolved but the valid fixture dir is missing: {}",
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
            continue;
        }
        // Acceptance alone missed two review-round-2 bugs (F1/F2): the
        // events `parse_events` yields are un-merged, so a merge-pass
        // bug cannot reject a valid fixture, and a silent data bug
        // parses fine while producing the wrong value. Drive the full
        // public thin path — `from_str` runs `parse_events_merged`
        // (conditionally `merge_reopened`) then the `EventDeserializer`
        // — and compare the value against the fixture's own JSON oracle
        // with the same comparator the owned runner uses.
        let json_path = ktav_path.with_extension("json");
        let oracle_src = match fs::read_to_string(&json_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("read {}: {}", json_path.display(), e));
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
        let actual: JsonValue = match ktav::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("from_str {}: {}", rel, e));
                continue;
            }
        };
        if !json_eq_ordered(&actual, &expected) {
            failures.push(format!(
                "thin-API value mismatch in {}:\n  expected: {}\n  actual:   {}",
                rel, expected, actual
            ));
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
        "spec_conformance::valid (thin API): {} fixtures accepted and value-matched vs oracles, {} excluded under known gaps",
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
