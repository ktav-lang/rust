//! The § 8.5 Conformance Runner Contract.
//!
//! § 8.5 makes `versions/<v>/tests/manifest.json` normative and says a
//! runner "MUST load this file before enumerating any fixture". This
//! module is that load, and it is deliberately loud: every failure here
//! aborts before a single fixture executes, because the whole point is to
//! refuse a run whose corpus cannot be vouched for.
//!
//! What it forbids, in the spec's words: "running against a stale
//! checkout, a wrong path, or an earlier specification version's
//! directory". A runner without these checks passes happily on a
//! truncated corpus — every category present, every category non-empty,
//! and half the cases quietly missing. That is exactly the evidence
//! § 8.4 says a conformance claim rests on, so a runner that skips this
//! is not producing evidence at all.
//!
//! Note the asymmetry § 8.5 is careful about: these checks are over the
//! CORPUS CHECKOUT, not over what a given run executes. A parser-only
//! implementation legitimately never runs `unrepresentable/`, but that
//! directory must still be present and complete.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;

/// The newest `schema_version` this runner understands. § 8.5: a runner
/// "MUST reject a manifest whose `schema_version` field names a schema
/// newer than the runner implements rather than guess at its shape".
const SUPPORTED_SCHEMA_VERSION: u64 = 1;

/// A category whose fixtures are counted by `.json` files rather than by
/// `.ktav` inputs, because its fixtures have no `.ktav` side at all.
const JSON_ONLY_CATEGORIES: &[&str] = &["unrepresentable"];

#[derive(Debug)]
pub(crate) struct RunnerManifest {
    /// Fixture stems flagged `raw_bytes`, relative to their category dir
    /// and spelled with `/`, e.g. `invalid_utf8/lone_continuation_byte`.
    pub(crate) raw_bytes: BTreeSet<String>,
}

impl RunnerManifest {
    /// `true` when this fixture must reach the implementation under test
    /// as bytes, never through a lossy text decode.
    pub(crate) fn is_raw_bytes(&self, category: &str, rel: &str) -> bool {
        let _ = category;
        self.raw_bytes.contains(&rel.replace('\\', "/"))
    }
}

/// Count the fixtures § 8.5 counts: one per `<name>` stem shared by a
/// category's sibling files. `valid/` pairs each input with a
/// `.canonical.ktav` twin that is NOT its own fixture, which is why the
/// canonical files are excluded rather than counted.
fn count_fixtures(dir: &Path, json_only: bool) -> usize {
    fn walk(dir: &Path, json_only: bool, n: &mut usize) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, json_only, n);
                continue;
            }
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if json_only {
                if ext == "json" {
                    *n += 1;
                }
            } else if ext == "ktav" && !stem.ends_with(".canonical") {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    walk(dir, json_only, &mut n);
    n
}

/// Load `tests/manifest.json` and enforce § 8.5 against `tests_root`.
/// Panics — before any fixture is enumerated — on every violation.
pub(crate) fn load_and_enforce(tests_root: &Path) -> RunnerManifest {
    let path = tests_root.join("manifest.json");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "§ 8.5: the conformance manifest {} could not be read ({e}); a runner \
             MUST load it before enumerating any fixture",
            path.display()
        )
    });
    let manifest: JsonValue = serde_json::from_str(&src)
        .unwrap_or_else(|e| panic!("§ 8.5: {} is not valid JSON: {e}", path.display()));

    let schema = manifest
        .get("schema_version")
        .and_then(JsonValue::as_u64)
        .unwrap_or_else(|| panic!("§ 8.5: {} has no numeric schema_version", path.display()));
    assert!(
        schema <= SUPPORTED_SCHEMA_VERSION,
        "§ 8.5: manifest schema_version {schema} is newer than this runner implements \
         ({SUPPORTED_SCHEMA_VERSION}); refusing to guess at its shape"
    );

    let declared: BTreeMap<String, u64> = manifest
        .get("categories")
        .and_then(JsonValue::as_object)
        .unwrap_or_else(|| panic!("§ 8.5: {} has no categories map", path.display()))
        .iter()
        .map(|(name, spec)| {
            let count = spec.get("count").and_then(JsonValue::as_u64).unwrap_or_else(|| {
                panic!("§ 8.5: category {name:?} has no numeric count in the manifest")
            });
            (name.clone(), count)
        })
        .collect();

    // The category set is CLOSED in both directions: a declared directory
    // that is missing, and a present directory that is undeclared, are
    // both hard failures. An unknown category "MUST NOT be silently
    // skipped or silently accepted".
    let mut present: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(tests_root)
        .unwrap_or_else(|e| panic!("§ 8.5: cannot read {}: {e}", tests_root.display()))
        .flatten()
    {
        if entry.path().is_dir() {
            present.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    let declared_names: BTreeSet<String> = declared.keys().cloned().collect();
    let missing: Vec<&String> = declared_names.difference(&present).collect();
    let unknown: Vec<&String> = present.difference(&declared_names).collect();
    assert!(
        missing.is_empty(),
        "§ 8.5: category directory/directories {missing:?} are declared in the manifest \
         but absent from {}; the corpus checkout is stale or the path is wrong",
        tests_root.display()
    );
    assert!(
        unknown.is_empty(),
        "§ 8.5: directory/directories {unknown:?} exist under {} but are not declared in \
         the manifest; an unknown category must not be silently accepted",
        tests_root.display()
    );

    for (name, expected) in &declared {
        let json_only = JSON_ONLY_CATEGORIES.contains(&name.as_str());
        let actual = count_fixtures(&tests_root.join(name), json_only) as u64;
        assert_eq!(
            actual, *expected,
            "§ 8.5: category {name:?} holds {actual} fixture(s) but the manifest declares \
             {expected}; this run would have covered a different corpus than the one the \
             specification defines"
        );
    }

    let mut raw_bytes = BTreeSet::new();
    if let Some(flags) = manifest.get("fixture_flags").and_then(JsonValue::as_array) {
        for entry in flags {
            let fixture = entry
                .get("fixture")
                .and_then(JsonValue::as_str)
                .unwrap_or_else(|| panic!("§ 8.5: a fixture_flags entry has no fixture name"));
            let listed: Vec<&str> = entry
                .get("flags")
                .and_then(JsonValue::as_array)
                .map(|a| a.iter().filter_map(JsonValue::as_str).collect())
                .unwrap_or_default();
            for flag in listed {
                assert_eq!(
                    flag, "raw_bytes",
                    "§ 8.5: fixture {fixture:?} carries unknown flag {flag:?}; this runner \
                     implements schema {SUPPORTED_SCHEMA_VERSION}, which defines only raw_bytes"
                );
                raw_bytes.insert(fixture.to_string());
            }
        }
    }

    eprintln!(
        "§ 8.5 runner contract: manifest schema {schema}, {} categor(ies) present and \
         complete, {} raw-bytes fixture(s)",
        declared.len(),
        raw_bytes.len()
    );
    RunnerManifest { raw_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::{resolve_spec_root, runner_manifest};
    use std::fs;

    /// The contract is enforced as a side effect of resolving the spec
    /// root, which is what makes "before enumerating any fixture" true
    /// for every runner. This test makes that enforcement VISIBLE, so a
    /// future refactor that drops the call fails here loudly instead of
    /// quietly returning the suite to a pre-§ 8.5 state.
    #[test]
    fn runner_contract_is_enforced_and_reports_the_raw_bytes_fixture() {
        let Some(_root) = resolve_spec_root() else {
            eprintln!("skipping § 8.5 contract test: spec dir not found");
            return;
        };
        let manifest = runner_manifest().expect("resolving the spec root must enforce § 8.5");
        assert!(
            manifest.is_raw_bytes("invalid", "invalid_utf8/lone_continuation_byte"),
            "§ 8.5: the corpus declares invalid_utf8/lone_continuation_byte as raw_bytes; \
             a runner that loses that flag decodes the fixture as text and destroys the \
             exact condition § 6.15 requires the parser to detect"
        );
    }

    /// Copy the corpus's shape into a temp dir, delete ONE fixture, and
    /// require the loader to refuse it.
    ///
    /// Without this, the check above proves only that a correct corpus
    /// passes — which a function returning unconditionally also does.
    /// § 8.5 exists to catch "a stale checkout, a wrong path, or an
    /// earlier specification version's directory", and the only way to
    /// know it does is to hand it one.
    #[test]
    fn a_corpus_one_fixture_short_is_refused() {
        let Some(root) = resolve_spec_root() else {
            eprintln!("skipping § 8.5 truncation test: spec dir not found");
            return;
        };
        let real = root.join("versions").join("0.7").join("tests");
        let temp = std::env::temp_dir().join(format!("ktav-truncated-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);

        // Mirror only what the loader reads: the manifest and the
        // category directories' fixture files.
        fs::create_dir_all(&temp).unwrap();
        fs::copy(real.join("manifest.json"), temp.join("manifest.json")).unwrap();
        let mut removed = false;
        for category in ["valid", "invalid", "unrepresentable", "parseable-unrepresentable"] {
            copy_tree(&real.join(category), &temp.join(category));
            if category == "valid" && !removed {
                let victim = first_fixture(&temp.join("valid"));
                fs::remove_file(&victim).unwrap();
                removed = true;
            }
        }
        assert!(removed, "the truncation test found no fixture to remove");

        let outcome = std::panic::catch_unwind(|| load_and_enforce(&temp));
        let _ = fs::remove_dir_all(&temp);
        let err = outcome.expect_err("§ 8.5: a corpus one fixture short MUST be refused");
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| err.downcast_ref::<&str>().map(|s| (*s).to_string()).unwrap_or_default());
        assert!(
            message.contains("§ 8.5") && message.contains("valid"),
            "the refusal must name the section and the short category; got {message:?}"
        );
    }

    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let (src, dst) = (entry.path(), to.join(entry.file_name()));
            if src.is_dir() {
                copy_tree(&src, &dst);
            } else {
                fs::copy(&src, &dst).unwrap();
            }
        }
    }

    fn first_fixture(dir: &Path) -> std::path::PathBuf {
        let mut found = Vec::new();
        fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("ktav")
                    && !path.file_stem().and_then(|s| s.to_str()).unwrap_or("").ends_with(".canonical")
                {
                    out.push(path);
                }
            }
        }
        walk(dir, &mut found);
        found.sort();
        found.into_iter().next().expect("no fixture to remove")
    }
}
