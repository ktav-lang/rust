//! Invalid-fixture conformance tests (owned API and thin API).

use super::support::{
    check_known_gap, collect_invalid_fixtures, resolve_spec_root, thin_known_gap, THIN_KNOWN_GAPS,
};

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
