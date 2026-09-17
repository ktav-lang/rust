//! Differential oracle between the `Value` parser and the formatter's
//! trivia-preserving fork (issue rust#238).
//!
//! `fmt_parser`'s module doc claims "same grammar, same errors, as
//! [`crate::parse`]". Nothing checked that. The fork mirrors
//! `parser::Parser`'s line dispatch across roughly eight parallel
//! methods, and the two files can drift silently: today's § 3.3
//! whitespace fix reached all three parsers only because it happened to
//! live in a shared predicate, which was luck rather than design.
//!
//! The existing corpus test (`tests/format_idempotent.rs`) does not
//! close this. On `Err` it counts the fixture and moves on, so a fork
//! that wrongly REJECTS a valid document leaves every suite green. That
//! is the exact divergence this file is built to catch.
//!
//! Three properties, over every corpus fixture:
//!
//! 1. The two parsers agree on acceptance.
//! 2. When both reject, they report the same error class on the same
//!    line.
//! 3. When both accept, stripping the fork's trivia yields the same
//!    `Value`.
//!
//! This lives in `src/` rather than `tests/` because `parse_with_trivia`
//! is `pub(crate)` — isolating the fork is the whole point, and going
//! through the public `format_str` would mix in writer rejections and
//! blur what failed.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, ErrorEnvelope};
use crate::parser::fmt_parser::{self, PArray, PObject, PValue};
use crate::value::{ObjectMap, Value};

/// The corpus is large; a floor guards against a walk that silently
/// finds nothing — a stale submodule, a wrong path, a truncated
/// checkout. "More than zero" would not have caught the version of this
/// mistake that shipped in three other implementations this week.
const MIN_FIXTURES: usize = 250;

fn strip(value: PValue) -> Value {
    match value {
        PValue::Null => Value::Null,
        PValue::Bool(b) => Value::Bool(b),
        PValue::Integer(s) => Value::Integer(s),
        PValue::Float(s) => Value::Float(s),
        PValue::String(s) => Value::String(s),
        PValue::Array(PArray { items, .. }) => {
            Value::Array(items.into_iter().map(|(_, v)| strip(v)).collect())
        }
        PValue::Object(PObject { pairs, .. }) => {
            let mut out = ObjectMap::default();
            for (key, (_, v)) in pairs {
                out.insert(key, strip(v));
            }
            Value::Object(out)
        }
    }
}

/// Error class and line, the two facets a caller actually branches on.
/// Reuses the envelope rather than matching variants by hand, so a new
/// variant cannot quietly fall through a `_` arm here.
fn classify(err: &Error, src: &str) -> (String, Option<u32>) {
    let env = ErrorEnvelope::from_error(err, src);
    (env.error, env.line)
}

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

fn collect(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
            out.push(path);
        }
    }
}

#[test]
fn fmt_parser_agrees_with_the_value_parser_over_the_corpus() {
    let Some(spec_root) = resolve_spec_root() else {
        eprintln!("fmt_differential: no spec checkout found, skipping");
        return;
    };
    let tests_dir = spec_root.join("versions").join("0.7").join("tests");

    let mut files = Vec::new();
    for bucket in [
        "valid",
        "invalid",
        "unrepresentable",
        "parseable-unrepresentable",
    ] {
        collect(&tests_dir.join(bucket), &mut files);
    }
    files.sort();

    let mut checked = 0usize;
    let mut both_ok = 0usize;
    let mut both_err = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for path in &files {
        // § 6.15 fixtures are deliberately not valid UTF-8 and are
        // outside both parsers' `&str` domain.
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        checked += 1;
        let rel = path.strip_prefix(&tests_dir).unwrap_or(path).display();

        match (crate::parse(&text), fmt_parser::parse_with_trivia(&text)) {
            (Ok(value), Ok(doc)) => {
                both_ok += 1;
                let stripped = strip(doc.root);
                if stripped != value {
                    failures.push(format!(
                        "{rel}: both accepted but the trees differ\n  \
                         parse:      {value:?}\n  fmt_parser: {stripped:?}"
                    ));
                }
            }
            (Err(a), Err(b)) => {
                both_err += 1;
                let (ca, la) = classify(&a, &text);
                let (cb, lb) = classify(&b, &text);
                if (ca.as_str(), la) != (cb.as_str(), lb) {
                    failures.push(format!(
                        "{rel}: both rejected but differently\n  \
                         parse:      {ca} at line {la:?}\n  \
                         fmt_parser: {cb} at line {lb:?}"
                    ));
                }
            }
            (Ok(_), Err(e)) => failures.push(format!(
                "{rel}: parse accepted, fmt_parser REJECTED: {e}\n  \
                 (this is the divergence the formatter's own corpus test \
                 cannot see — it treats any Err as an acceptable outcome)"
            )),
            (Err(e), Ok(_)) => {
                failures.push(format!("{rel}: parse rejected ({e}), fmt_parser accepted"))
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {checked} fixture(s) diverge between parser.rs and \
         fmt_parser.rs:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(
        checked >= MIN_FIXTURES,
        "walked only {checked} fixtures under {} — expected at least \
         {MIN_FIXTURES}. A smaller corpus than intended must fail here, \
         not report success.",
        tests_dir.display()
    );
    eprintln!(
        "fmt_differential: {checked} fixtures agree \
         ({both_ok} accepted by both, {both_err} rejected by both)"
    );
}
