# Changelog — `ktav` crate

All notable changes to the `ktav` crate are documented here. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.

For the format specification's own history, see the
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) repository.

## [0.1.4] — 2026-04-26

### Changed

- **`Frame::Object` initial capacity 4 → 8** (`src/parser/frame.rs`).
  The parser's per-compound `IndexMap` now pre-sizes for 8 entries
  instead of 4, which eliminates the first growth/rehash for the
  typical 5–8-field config row. This is the **untyped** parse path
  (`ktav::parse → Value`) — the same path every C-ABI binding
  (PHP/JS/Python/Go/Java/C#) walks through `cabi`, so they all see
  the speedup once they pick up 0.1.4.
- Net impact on the `parse_to_value` bench (3-run median): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium in the noise (~−3%).

One-line change; full test suite (334 cases incl. spec conformance)
unaffected.

## [0.1.3] — 2026-04-26

Same content as the yanked 0.1.2 — re-released through the new
automated `Release` workflow (CI verify → `cargo publish`) so future
releases never depend on a manual `cargo publish` from a maintainer's
machine. 0.1.2 was yanked solely to validate the pipeline end-to-end
on a fresh version (crates.io is immutable; we can't re-publish 0.1.2
itself).

## [0.1.2] — 2026-04-26

Re-publish of 0.1.1's contents with the source tree run through
`cargo fmt`. 0.1.1 was yanked because the new files (`benches/vs_json.rs`,
`src/thin/event*.rs`, `src/thin/fast_num.rs`) hadn't been formatted
through rustfmt before publish, which tripped the CI lint check on the
tag push. **Functionally identical to 0.1.1** — only whitespace differs.

## [0.1.1] — 2026-04-26

### Changed

- **Typed-deserialization fast path** — `from_str` and `from_file` no
  longer build a `ThinValue` tree as an intermediate. The parser now
  emits a flat `Vec<Event>` directly into a bump arena, and the serde
  deserializer walks it linearly with a single cursor — one allocation
  per document instead of one per compound, and no per-node enum-
  discriminant load behind a `Box`-style indirection. Net impact on a
  275 KB config: **−18.7%** on `parse → struct` (3.60 ms → 2.93 ms).
- **`fast_num` byte-loop atoi** — the `i8`..`i64` / `u8`..`u64` paths
  in the typed deserializer skip the generic `<T as FromStr>` route
  and call hand-rolled `parse_i64` / `parse_u64` with a width check.
  Floats stay on `f64::from_str`.

### Added

- `Event` token enum and `EventCursor` walker (`thin/event*.rs`),
  internal — not exposed in the public surface.

### Removed

- `ThinValue` enum and its `ThinDeserializer` (replaced by the event
  stream — both were `pub(crate)`, so no breakage at the public API).

### Behavior change

- **Interleaved dotted-key prefixes are now rejected as a conflict**.
  A document like `a.x: 1\nb.y: 2\na.z: 3` (synthetic `a` opened, then
  closed by `b.`, then re-opened by `a.z`) used to silently merge into
  one `a` object via the tree-builder. The event-stream tokenizer
  cannot do that without buffering the whole document, so it now
  surfaces a clear conflict error suggesting the user group lines with
  the same prefix together. Documents with grouped dotted keys (the
  canonical pattern) are unaffected — every spec-conformance fixture
  still passes.

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
