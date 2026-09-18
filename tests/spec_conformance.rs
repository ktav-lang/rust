//! Language-agnostic conformance suite from `ktav-lang/spec`.
//!
//! Points at `versions/<SPEC_VERSION>/tests/{valid,invalid}` and checks:
//!   - every `valid/**/*.ktav` (non-canonical) parses and its `Value`
//!     equals the oracle in the sibling `.json` file;
//!   - every `valid/**/*.canonical.ktav` re-parses to the same `Value`;
//!   - `emit_canonical(parse(input.ktav))` matches `name.canonical.ktav`;
//!   - every `invalid/**/*.ktav` is rejected by `ktav::parse` (or, for
//!     byte-level invalid UTF-8 fixtures, by `ktav::from_file` with
//!     `Error::InvalidUtf8` — spec § 6.15);
//!   - every `unrepresentable/**/*.json` Value is rejected by all three
//!     writer surfaces with the exact `ReasonCode` (spec § 5.9.0);
//!   - every `parseable-unrepresentable/*.ktav` parses and matches its
//!     JSON oracle `value`, yet every writer surface rejects it.
//!
//! Spec root resolution (first match wins):
//!   1. env var `KTAV_SPEC_DIR` (absolute path to the spec-repo root);
//!   2. `<CARGO_MANIFEST_DIR>/spec`  — git submodule `ktav-lang/spec`;
//!   3. `<CARGO_MANIFEST_DIR>/../spec` — sibling directory (local dev);
//!   4. if none contains a `versions/` dir, the test logs and returns —
//!      it does not fail, so CI without the spec checkout stays green.

#[path = "spec_conformance/manifest.rs"]
mod manifest;
#[path = "spec_conformance/invalid.rs"]
mod invalid;
#[path = "spec_conformance/oracle.rs"]
mod oracle;
#[path = "spec_conformance/support.rs"]
mod support;
#[path = "spec_conformance/unrepresentable.rs"]
mod unrepresentable;
#[path = "spec_conformance/valid.rs"]
mod valid;
