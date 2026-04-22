# Changelog — `ktav` crate

All notable changes to the `ktav` crate are documented here. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.

For the format specification's own history, see the
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) repository.

## [0.1.0] — 2026-04-22

Initial release. Implements [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md).

### Added

- **Parser** — turns Ktav text into a `Value` (owned) or a `ThinValue`
  (zero-copy view over the input buffer). Line-based state machine
  with dotted-key expansion, multi-line strings (stripped and
  verbatim), JSON-style keywords `null` / `true` / `false`, and
  typed-scalar markers `:i` (Integer) and `:f` (Float).
- **Serializer** — two paths:
  - `ktav::to_string` (direct text emission, primary path).
  - `ktav::ser::to_value` / `ktav::render` (two-step for users who
    want to inspect a `Value` between stages).
  Both emit `::` automatically for strings that would otherwise be
  mis-read by the parser, and emit `:i` / `:f` for Rust numeric
  types.
- **Deserializer** — zero-copy path via `ThinValue<'a>` and
  `ThinDeserializer`. Object keys and single-line scalar values are
  borrowed directly from the input; only multi-line strings allocate.
  Accepts both typed-marker and plain-string forms of numbers, so
  documents written without markers deserialize transparently via
  `FromStr`.
- **Serde integration** — `from_str`, `from_file`, `to_string`,
  `to_file` accept any `T: Serialize` / `DeserializeOwned`, including
  `#[derive]`-generated types, nested structs, `Vec`, `Option`,
  `HashMap`, and the usual externally-tagged enum forms. Rust integer
  types (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`) serialize with
  `:i`; floats (`f32`, `f64`) with `:f`; `NaN` and `±Infinity` are
  rejected by the serializer (not representable in Ktav 0.1.0).
- **Raw marker `::`** — forces a value to be a literal String, both
  in pair position (`key:: value`) and as an array-item prefix
  (`:: value`).
- **Typed markers `:i` and `:f`** — explicit Integer / Float in pair
  position (`port:i 8080`, `ratio:f 0.5`) and as array-item prefixes
  (`:i 42`, `:f 3.14`). Values stored as strings at the `Value` layer
  to preserve arbitrary precision.
- **Multi-line strings** — `( ... )` (common-indent stripped) and
  `(( ... ))` (verbatim). Round-trips byte-for-byte via the verbatim
  form.
- **Public `Value` enum** — `Null`, `Bool`, `Integer`, `Float`,
  `String`, `Array`, `Object` (backed by `IndexMap` with
  `rustc_hash::FxBuildHasher`). `Value::as_integer` / `as_float`
  accessors; analogous on `ThinValue`.
- **Error reporting** — every syntax error carries a line number;
  deserialization errors carry a dotted path (`upstreams.[0].port`).
  Typed-scalar violations surface as `InvalidTypedScalar` in the
  message prefix.
- **Spec conformance tests** — `tests/spec_conformance.rs` runs the
  language-agnostic suite from the `ktav-lang/spec` repository
  (resolved via `KTAV_SPEC_DIR` env or `../spec` fallback). Three
  checks: Value-equals-JSON-oracle, invalid-fixtures-rejected, and
  lossless Value-level round-trip through the renderer.

### Performance (criterion, 22 KB typed config, Windows release)

- `parse → struct`: **275 µs** (~80 MB/s)
- `render struct → text`: **46 µs** (~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde` with `derive`
- `indexmap` with the `serde` feature
- `rustc-hash` (FxHash — fast and deterministic; not
  collision-resistant, which a config parser does not need)

### MSRV

`rustc 1.70` or newer.
