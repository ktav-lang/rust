//! Idempotence of `ktav::format_str` over the whole spec conformance
//! corpus (issue rust#13's "Definition of done": `fmt(fmt(x)) ==
//! fmt(x)` across the full corpus, not a handful of examples).
//!
//! Also checks, wherever `format_str` succeeds, that re-parsing its
//! output preserves the document's structure (`Value`) exactly — the
//! formatter must never change what a document means, only how it is
//! spelled.
//!
//! Spec root resolution mirrors `tests/spec_conformance.rs` exactly:
//! env var `KTAV_SPEC_DIR`, then `<manifest>/spec`, then
//! `<manifest>/../spec`; if none has a `versions/` dir the test logs
//! and returns without failing, so CI without the spec checkout stays
//! green.

use std::fs;
use std::path::{Path, PathBuf};

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

fn tests_root(spec_root: &Path) -> PathBuf {
    spec_root.join("versions").join(SPEC_VERSION).join("tests")
}

/// Every `.ktav` file anywhere under `root` (canonical fixtures
/// included — they are documents in their own right, already in § 5.9
/// canonical structure, and are a natural idempotence fixture: a
/// canonical file is comment-free and blank-free by construction, so
/// `format_str` on it must reproduce it byte-for-byte, spot-checked
/// separately in `format_canonical_equivalence.rs`).
fn collect_all_ktav_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_all_ktav_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            out.push(path);
        }
    }
}

#[test]
fn format_is_idempotent_and_structure_preserving_over_the_corpus() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("format_idempotent: no spec checkout found, skipping");
        return;
    };
    let root = tests_root(&spec_root);

    let mut files = Vec::new();
    for bucket in [
        "valid",
        "invalid",
        "unrepresentable",
        "parseable-unrepresentable",
    ] {
        collect_all_ktav_files(&root.join(bucket), &mut files);
    }
    assert!(
        !files.is_empty(),
        "expected to find .ktav fixtures under {}",
        root.display()
    );

    let mut checked_ok = 0usize;
    let mut checked_err = 0usize;

    for path in &files {
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // A handful of `invalid/invalid_utf8/*.ktav` fixtures are
        // deliberately not valid UTF-8 at all (spec § 6.15) — out of
        // `format_str`'s `&str`-only domain, same as `parse_events`'s
        // stance in `tests/spec_conformance.rs`.
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };

        match ktav::format_str(text) {
            Err(_) => {
                // Invalid / unrepresentable input — format_str is
                // expected to reject it the same way emit_canonical
                // would. Not a failure of this property test.
                checked_err += 1;
            }
            Ok(once) => {
                checked_ok += 1;

                let twice = ktav::format_str(&once).unwrap_or_else(|e| {
                    panic!(
                        "format_str failed on its OWN output for {}: {e}\nfirst pass:\n{once}",
                        path.display()
                    )
                });
                assert_eq!(
                    once,
                    twice,
                    "fmt(fmt(x)) != fmt(x) for {}\nfirst pass:\n{once}\nsecond pass:\n{twice}",
                    path.display()
                );

                // Structure must be preserved: re-parsing the formatted
                // output yields the same Value as the original source
                // (when the original itself parses — some
                // `unrepresentable`/`parseable-unrepresentable` bucket
                // files are valid Ktav that only the WRITER rejects, so
                // `format_str` erroring on those is separately covered
                // above and `parse` on them still succeeds here).
                if let Ok(original_value) = ktav::parse(text) {
                    let reparsed = ktav::parse(&once).unwrap_or_else(|e| {
                        panic!(
                            "re-parsing formatted output failed for {}: {e}\n{once}",
                            path.display()
                        )
                    });
                    assert_eq!(
                        original_value,
                        reparsed,
                        "format_str changed document structure for {}",
                        path.display()
                    );
                }
            }
        }
    }

    // Sanity: the corpus is large and mostly formattable — make sure
    // this test is actually exercising the formatter, not silently
    // erroring on everything.
    assert!(
        checked_ok > 100,
        "expected most of the corpus to format successfully, only {checked_ok} did ({checked_err} errored)"
    );
}
