//! `strict-lossy/` conformance tests (spec § 8.1): for each fixture, lax
//! `parse` must accept the input and produce `lax_value`, and strict
//! `parse_strict` must reject it with `LossyScalar` naming `body` and
//! `canonical`.

use std::fs;
use std::path::PathBuf;

use serde_json::Value as JsonValue;

use crate::oracle::{json_eq_ordered, ktav_to_json};
use crate::support::{collect_ktav_files, resolve_spec_root, tests_dir};
use ktav::{Error, ErrorKind};

struct StrictLossyFixture {
    rel: String,
    source: String,
    lax_value: JsonValue,
    expected_body: String,
    expected_canonical: String,
}

fn collect_strict_lossy_fixtures(spec_root: &std::path::Path) -> Vec<StrictLossyFixture> {
    let root = tests_dir(spec_root, "strict-lossy");
    let mut files: Vec<PathBuf> = Vec::new();
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
        let oracle: JsonValue = match serde_json::from_str(&oracle_src) {
            Ok(v) => v,
            Err(e) => panic!("fixture {}: oracle json parse error: {}", rel, e),
        };
        let map = oracle
            .as_object()
            .unwrap_or_else(|| panic!("fixture {rel}: top-level oracle JSON is not an object"));
        let allowed: std::collections::BTreeSet<&str> =
            ["lax_value", "expected_error", "body", "canonical", "note"]
                .into_iter()
                .collect();
        for key in map.keys() {
            if !allowed.contains(key.as_str()) {
                panic!("fixture {rel}: unexpected oracle field {key:?}");
            }
        }
        let lax_value = map
            .get("lax_value")
            .unwrap_or_else(|| panic!("fixture {rel}: oracle missing `lax_value`"))
            .clone();
        let expected_error = map
            .get("expected_error")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("fixture {rel}: oracle missing string `expected_error`"));
        if expected_error != "LossyScalar" {
            panic!("fixture {rel}: expected_error must be \"LossyScalar\", got {expected_error:?}");
        }
        let expected_body = map
            .get("body")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("fixture {rel}: oracle missing string `body`"))
            .to_string();
        let expected_canonical = map
            .get("canonical")
            .and_then(JsonValue::as_str)
            .unwrap_or_else(|| panic!("fixture {rel}: oracle missing string `canonical`"))
            .to_string();
        let note = map.get("note").and_then(JsonValue::as_str).unwrap_or("");
        if note.is_empty() {
            panic!("fixture {rel}: note is empty");
        }
        let source = match fs::read_to_string(ktav_path) {
            Ok(s) => s,
            Err(e) => panic!("fixture {}: unreadable `.ktav` file: {}", rel, e),
        };

        fixtures.push(StrictLossyFixture {
            rel,
            source,
            lax_value,
            expected_body,
            expected_canonical,
        });
    }
    fixtures
}

#[test]
fn strict_lossy_fixtures_round_trip_lax_and_reject_strict() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("skipping spec_conformance::strict_lossy: spec dir not found");
        return;
    };
    // strict-lossy/ was introduced in spec 0.8.0. The committed `spec`
    // submodule is pinned at that corpus, and `resolve_spec_root`
    // refuses to resolve any checkout that lacks
    // `versions/<SPEC_VERSION>` (currently 0.8), so in a normal checkout
    // this branch is dead. It is defensive insurance against a future
    // SPEC_VERSION whose corpus predates this category — it only fires
    // when the directory is absent outright, never when it exists but is
    // empty or incomplete.
    if !tests_dir(&spec_root, "strict-lossy").is_dir() {
        eprintln!(
            "skipping spec_conformance::strict_lossy: this spec checkout predates the \
             strict-lossy/ category (spec § 8.1)"
        );
        return;
    }
    let fixtures = collect_strict_lossy_fixtures(&spec_root);

    let mut failures: Vec<String> = Vec::new();

    for fixture in &fixtures {
        match ktav::parse(&fixture.source) {
            Ok(value) => {
                let actual_json = ktav_to_json(&value);
                if !json_eq_ordered(&actual_json, &fixture.lax_value) {
                    failures.push(format!(
                        "fixture {}: lax parse value {actual_json} does not match oracle lax_value {}",
                        fixture.rel, fixture.lax_value
                    ));
                }
            }
            Err(e) => failures.push(format!(
                "fixture {}: lax parse unexpectedly failed: {e}",
                fixture.rel
            )),
        }

        match ktav::parse_strict(&fixture.source) {
            Ok(_) => failures.push(format!(
                "fixture {}: strict parse unexpectedly succeeded, expected LossyScalar",
                fixture.rel
            )),
            Err(Error::Structured(ErrorKind::LossyScalar {
                body, canonical, ..
            })) => {
                if body != fixture.expected_body {
                    failures.push(format!(
                        "fixture {}: strict body {body:?} != oracle body {:?}",
                        fixture.rel, fixture.expected_body
                    ));
                }
                if canonical != fixture.expected_canonical {
                    failures.push(format!(
                        "fixture {}: strict canonical {canonical:?} != oracle canonical {:?}",
                        fixture.rel, fixture.expected_canonical
                    ));
                }
            }
            Err(other) => failures.push(format!(
                "fixture {}: strict parse failed with a non-LossyScalar error: {other}",
                fixture.rel
            )),
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} of {} strict-lossy fixture(s) failed:\n{}",
            failures.len(),
            fixtures.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "spec_conformance::strict_lossy: {} fixtures verified (lax value + strict LossyScalar)",
        fixtures.len()
    );
}
