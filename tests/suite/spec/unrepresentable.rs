//! Unrepresentable-fixture conformance tests (writer-surface rejection).

use std::fs;

use serde_json::Value as JsonValue;

use super::oracle::{json_eq_ordered, json_to_ktav, ktav_to_json, SerValue};
use super::support::{
    collect_json_files, collect_ktav_files, resolve_spec_root, tests_dir, validate_reason_fixture,
};

#[test]
fn unrepresentable_fixtures_are_rejected_with_reason_codes() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::unrepresentable: spec dir not found");
        return;
    };
    let root = tests_dir(&spec_root, "unrepresentable");
    if !root.is_dir() {
        panic!(
            "spec 0.8 resolved but the unrepresentable fixture dir is missing: {}",
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
            "spec 0.8 resolved but the parseable-unrepresentable fixture dir is missing: {}",
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
