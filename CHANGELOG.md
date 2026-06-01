# Changelog — `ktav` crate

All notable changes to the `ktav` crate are documented here. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.

For the format specification's own history, see the
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) repository.


## [0.6.0] — 2026-06-01

Implements Ktav specification 0.6.0. Adds **key escaping**: keys now
process the §3.7 escape set, and two new escapes — `\.` and `\:` —
allow literal dot/colon characters inside a key segment.

### Breaking

- A literal backslash in a key now requires `\\`. Previously the
  parser treated `\` in a key as an opaque content byte (no escape
  processing). Source files that contain a single `\` in a key must
  double it to keep the same key bytes. Backslashes in values are
  unchanged.

### Added

- Escape table grows from 8 to 10 sequences: the existing eight
  (`\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`) plus the two new
  key-oriented ones — `\.` (literal dot in a key segment, does NOT
  split the dotted path) and `\:` (literal colon, does NOT act as the
  pair separator). The two new escapes are valid (redundant) in value
  contexts too.
- The escape-aware key scanner now treats the **first unescaped** `:`
  (or `::`) as the pair separator and splits dotted paths on
  **unescaped** `.` only. Inline-compound keys (`{a\.b: 1}`) follow
  the same rules.
- Render path re-escapes `\`, `.`, `:` in every emitted key segment,
  guaranteeing parse → render → parse identity for keys containing
  literal dots or colons.
- Zero-copy event path keeps borrowing the source slice for a key
  segment that has no `\`; segments with escapes are decoded into the
  bump arena so the event's `&'a str` lifetime is preserved.

### Errors

- `\X` in a key with `X` not in the ten-escape set is now a
  `BadEscapeSequence`. `\.` and `\:` are no longer `BadEscapeSequence`
  in any context.


## [0.5.0] — 2026-05-28

Implements Ktav specification 0.5.0. This is a breaking release:
the parser, serializer, and Value model are rewritten for the new
language semantics.

### Breaking

- Typed markers `:i` and `:f` removed. Numbers, booleans, and `null`
  are inferred from the lexical form (spec §§ 3.6, 5.2).
- Comments use `##` (line-start only). A single `#` byte is content.
- Bare `port: 8080` is now `Integer(8080)`, not `String("8080")`.
  Write `port:: 8080` to keep a String.
- Lone `{` / `[` on the first content line opens a multi-line root
  Object / Array (spec § 5.0.1 rules 4–5). The 0.1.1 JSONL-style
  semantic is removed.
- `Float` Values no longer carry the textual form; canonicalised via
  `ryu`.
- Key segments are trimmed of leading/trailing whitespace (spec § 4).
- Line terminators are `LF`, `CR`, or `CR LF`; `CR` is never a
  content byte.
- `ErrorKind::InlineNonEmptyCompound` and `InvalidTypedScalar` are
  deprecated (`#[doc(hidden)]`); the parser no longer emits them.

### Added

- **Inline compounds** `{k: v, …}` / `[i, …]` (spec § 5.8) with
  trailing comma, mid-value brace literal (§ 5.8.5), and nesting
  depth limit of 128.
- **Eight escape sequences** in inline scalars: `\\`, `\,`, `\}`,
  `\]`, `\{`, `\[`, `\n`, `\r` (spec § 3.7).
- **Number literal grammar** — `0x`, `0o`, `0b`, decimal, underscore
  separators; i64 overflow falls back to String (spec § 3.6).
- **`emit_canonical()`** — spec § 5.9 normative writer output,
  byte-deterministic across implementations.
- **`src/parser/inline.rs`** — inline-compound parser.
- **`src/render/canonical.rs`** — canonical writer.
- New error variants: `UnterminatedInlineCompound`,
  `MalformedInlineCompound`, `BadEscapeSequence`,
  `OrphanLineAfterTopLevelInline`.
- Triple-test conformance harness (`tests/spec_conformance.rs`):
  93 valid + 31 invalid fixtures from `spec/versions/0.5/tests/`.
- Parser fast-paths: plain-decimal integers, no-underscore floats,
  ryu-reuse, first-byte fast-reject, LF-only line splitting,
  pre-sized Bump arena.

### Changed

- License: `MIT` → `MIT OR Apache-2.0`.
- Spec submodule pinned to `v0.5.0` (`4d0a8aa`).
- Doctests disabled (`[lib] doctest = false`); examples remain as
  `text` blocks in doc comments.

## [0.3.1] — 2026-05-10

Backward-compatible feature release tracking spec 0.1.1.

### Added

- **Top-level Array support** (spec § 5.0.1, added in spec 0.1.1) —
  a document whose first content line has an array-item shape
  (bare scalar, `:: text`, `:i 42`, `:f 3.14`, lone `{` / `[`, or a
  multi-line opener `(` / `((`) is now parsed as a root-level
  `Value::Array`. Previously the root was always `Value::Object`,
  so a bare scalar at line 1 errored as `MissingSeparator`.
  Empty / comments-only documents still default to an empty Object
  (preserves 0.3.0 behaviour).
- **`ktav::to_string_force_strings(value)`** — render any `Value`
  with every scalar coerced to a String. Typed integers (`:i`),
  typed floats (`:f`), booleans, and null are flattened to their
  textual form; compounds preserve their structure. The output
  round-trips back through the parser as the same set of String
  scalars. Useful for "everything is a string" dumps for downstream
  consumers that don't understand typed markers, or for diff-
  friendly canonical text.
- New `Render` exit point: `render` accepts both Object and Array
  top-level values (top-level Arrays render as bare item-per-line,
  no `[...]` brackets).

### Compatibility

Strictly additive. Every document valid under 0.3.0 stays valid
under 0.3.1 and produces the same `Value`. Only inputs 0.3.0
rejected as `MissingSeparator` (bare-scalar first lines) are now
accepted as Arrays. The error variants and their spans are
unchanged for the Object path.

The parser, render, thin event-parser, and thin event-deserializer
all honour spec § 5.0.1 consistently — `parse`, `parse_events`, and
`from_str` agree on the root kind for any given input.


## [0.3.0] — 2026-05-08

Minor release with one breaking parser strictness change, a
diagnostic-range fix, and hot-path micro-optimisations on the
typed-deserialize path.

### Fixed

- `ErrorKind::DuplicateKey` and `ErrorKind::KeyPathConflict` now carry
  the span of the **offending key**, not the closing `}` / `]` of the
  compound that would have been assigned to it. Previously, when the
  conflict was detected on `attach_child_value` (e.g. `value: { ... }`
  duplicating an earlier `value: ...`), the saved span pointed at the
  closing brace because that's the position the parser had at hand at
  the moment of detection.

  The parser now stores the key's own span (`pending_key_span`) on the
  parent frame when a compound is opened, and reuses it when the
  compound closes and the value is attached. Editors / IDEs that draw
  diagnostic underlines from `Span` now point at the key.

  This is a fix for a span value, not an API change — `ErrorKind`
  shape is unchanged.

### Changed (breaking — parser strictness)

- `key: (value)` and `key: ((value))` now error with
  `ErrorKind::InlineNonEmptyCompound { body: "paren-string" }`.
  These shapes used to be accepted as plain string scalars `(value)`,
  but they are visually indistinguishable from multi-line openers and
  would confuse readers. The raw-marker form `key:: (value)` remains
  valid and is the canonical way to encode such literals. The
  ktav-lsp formatter auto-rewrites the legacy form on save.

### Optimised (no API change)

- `render::render` pre-sizes the output `String` with a recursive
  `estimate_size(value)` to skip the doubling reallocations that
  `push_str` chains would otherwise trigger on multi-KiB outputs.
- `EventCursor::peek` / `next` use `unsafe get_unchecked` for
  bounds-elision on the hot path; the parser's well-formed-stream
  invariant guarantees `pos < len()` on every call. Falls back to
  `None` when the invariant is violated, so malformed inputs remain
  safe — the unsafe path is a pure branch elision win.
- `MapAccess::next_key_seed` folds redundant `peek + next` into a
  single `next` (both branches consume the cursor anyway).
- Event `BumpVec` capacity hint raised from `text.len() / 8 + 16`
  to `text.len() / 4 + 64`. The previous hint underestimated the
  ~1-event-per-5-bytes density on synth fixtures and triggered
  8–10 realloc-copy steps inside the bump arena on a 500 KiB doc.

### Experiment (reverted)

- A streaming-deserializer refactor (parse on demand, no whole-doc
  `Vec<Event>`) was implemented and tested — all 404 tests passed,
  but parse_to_struct regressed 15–60 % vs. the existing cursor on
  this hardware. Cause: the cursor walks a contiguous slice with
  one monotonic branch the predictor nails 100 %, while streaming
  interleaves parser state-machine work with deserializer work and
  blows the predictor. The streaming code was removed; the
  `EventSink<'a>` trait introduced for the experiment survives in
  `event.rs` as harmless generic infrastructure (zero cost when
  used only with `BumpVec`).


## [0.2.0] — 2026-05-07

Minor release with two breaking output / validation changes:

### Changed (breaking)

- **Multi-line strings emit indented stripped form `( ... )` by default**,
  not verbatim `(( ... ))`. Verbatim is still produced as a fallback when
  the content has its own leading whitespace (which the parser-side
  dedent would clobber) or contains a sole-`)` line that would close the
  stripped form prematurely. Code that compares `to_string` /
  `render` output byte-for-byte to a baked-in `((...))` literal needs to
  be updated. Round-tripping (`parse(to_string(v)) == v`) is unchanged.

      // Before (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // After (0.2.0):
      // body: (
      //     line1
      //     line2
      // )

  Both `Value` → `render::render(&value)` and `T: Serialize` →
  `ser::to_string(&t)` paths are updated consistently.

- **Typed-float marker `:f` now accepts integer literals.** The mantissa's
  decimal point is **optional**: `:f 42` is valid (parsed as `42.0`),
  matching the JSON / TOML / YAML convention that integer literals
  coerce to floats. `:f 1.` (no fractional digits) and `:f .5` (no
  integer part) remain invalid. Code that depends on `:f 42` raising
  `InvalidTypedScalar` needs to be updated.

### Spec

- `spec/versions/0.1/tests` fixture `typed_float_without_decimal` moved
  from `invalid/` to `valid/typed_float_integer_body` to reflect the new
  semantics. Spec submodule synced.


## [0.1.5] — 2026-05-01

Major release: structured errors with byte-offset spans, public
event-based parser API, `#[non_exhaustive]` retroactively applied to
the error enums for forward-compatibility.

### Added

- `ErrorKind` enum (10 spec-defined variants + `Other`) with byte-offset
  `span: Span` on every variant, exposing `(line, column, kind)` directly
  to downstream consumers without regex-parsing the formatted message.

      pub enum ErrorKind {
          MissingSeparatorSpace { line, column, marker, span },
          InvalidTypedScalar    { line, marker, body, span },
          DuplicateKey          { line, key, span },
          KeyPathConflict       { line, path, kind: ConflictKind, span },
          EmptyKey              { line, span },
          InvalidKey            { line, key, span },
          UnclosedCompound      { kind: CompoundKind, span },
          UnbalancedBracket     { line, expected: CompoundKind, found: char, span },
          InlineNonEmptyCompound{ line, body, span },
          MissingSeparator      { line, span },
          Other                 { line: Option<u32>, message, span },
      }

- `Error::Structured(ErrorKind)` variant on the existing `Error` enum.
- `pub struct Span { start: u32, end: u32 }` with `Span::new`,
  `Span::EMPTY`, `slice(input)`, and `line_col(input)` (1-based line,
  0-based byte column — multi-byte UTF-8 aware via tests pinning
  Cyrillic and 🦀).
- `Error::line() -> Option<u32>` and `Error::span() -> Option<Span>`
  convenience accessors covering every variant.
- `pub mod thin` — public event-based parser API:
  `ktav::parse_events(input, callback)` invoking the supplied
  `FnMut(ParseEvent<'_>)` for each event borrowed from the input.
  `ParseEvent` is a `#[non_exhaustive]` enum with 10 variants
  (`Null`, `Bool`, `Integer`, `Float`, `Str`, `Key`, `BeginObject`,
  `EndObject`, `BeginArray`, `EndArray`). The internal bumpalo arena
  stays private — the public API does not leak the arena type.
- Crate-level runnable doctest in `src/lib.rs` demonstrating both
  `Error::Structured` matching with `Span::slice` and the
  `parse_events` callback shape.
- Three new top-level test files:
  `tests/error_format.rs` — Display-string regression net (canonical
  pinning for the 7 categories that LSP / bindings rely on);
  `tests/structured_errors.rs` — variant identity + (line, span) byte
  ranges per spec invalid fixture;
  `tests/error_spans.rs` — span byte-range semantics + `Span::slice`
  and `Span::line_col` edge cases (UTF-8 multi-byte, char-boundary
  rounding);
  `tests/error_accessors.rs` — every `Error` variant tested for
  `line()` / `span()` returning `Some` / `None` as documented;
  `tests/non_exhaustive.rs` — wildcard-arm reachability proof for
  `Error` and `ErrorKind`;
  `tests/thin_public.rs` — event sequencing, nested compounds, marker
  items, error propagation, borrow contract.
- Synthetic Criterion benchmarks under `benches/` covering parse
  perf at small_1k / medium_50k / large_500k workloads on both
  success and error paths. Baseline numbers in `bench-baseline.md`.

### Changed

- `#[non_exhaustive]` retroactively applied to `Error`, `ErrorKind`,
  `ConflictKind`, and `CompoundKind`. Future variant additions are
  no longer breaking changes for downstream `match`-ers, who must
  now include a `_ =>` arm.
- The parser no longer constructs `Error::Syntax(format!(...))` at any
  internal call site (~37 sites refactored to `Error::Structured`).
  A regression guard test
  (`parser_no_longer_emits_legacy_syntax_variant`) runs 12 invalid
  inputs and fails CI loudly if anyone reintroduces the legacy
  variant inside `src/`.
- `parser/parse_str.rs` replaces `str::lines()` with a manual
  byte-walking loop maintaining a cumulative `line_start` counter
  so byte-offset spans can be computed at every error site without
  rescanning. `thin/event_parser.rs` mirrors the same plumbing on
  the zero-copy path.
- `Display for ErrorKind` is byte-identical to the strings the parser
  previously formatted into `Error::Syntax(...)` for the seven
  pre-existing categories — the contract that lets every existing
  string-based caller keep working unmodified during the
  ecosystem-wide migration tracked in
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md).
- Three formerly-`Other` shapes promoted to named `ErrorKind` variants:
  `UnbalancedBracket` (stray closer / shape mismatch),
  `InlineNonEmptyCompound` (`x: {foo}` — spec § 6.7),
  `MissingSeparator` (line with no `:`). After this promotion, `Other`
  contains only parser-internal invariants that no spec invalid
  fixture can trigger.

### Performance

`cargo bench --bench parse -- --quick` against the 0.1.4 baseline:

|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| `parse_synth/small_1k`         | 16.1 µs        | 16.0 µs| −0.6 % |
| `parse_synth/medium_50k`       | 896 µs         | 663 µs | −26 %  |
| `parse_synth/large_500k`       | 9.49 ms        | 9.27 ms| −2.3 % |
| `parse_synth_error/small_1k`   | 7.5 µs         | 7.2 µs | −4.0 % |
| `parse_synth_error/medium_50k` | 340 µs         | 346 µs | +1.8 % |
| `parse_synth_error/large_500k` | 4.47 ms        | 4.50 ms| +0.7 % |

Net: zero success-path regression. Error-path slightly faster — the
new `Display` impl constructs the formatted string lazily at
`.to_string()` time, whereas the prior `format!(...)` allocated a
`String` at every error site eagerly. The cumulative-byte counter
that powers spans is statistically free.

### Notes

- `Error::Syntax(String)` is preserved for backward compatibility —
  the public API stays deny-no-old-callers. Removal is deferred to
  ktav 1.0.
- Test count: 332 (0.1.4) → 391 (+59) plus 1 new doctest.
- The cabi/binding migration to consume `ErrorKind` over the FFI
  boundary is tracked separately in
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) and ships as a
  coordinated ecosystem 0.2.0.

### SemVer note

Adding `#[non_exhaustive]` to a previously-unmarked enum (`Error`,
`ConflictKind`, `CompoundKind`) is, per the
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
a breaking change that would normally require a major bump (0.2.0).
This release ships as **0.1.5** intentionally:

1. Pre-1.0 Cargo convention permits breaking changes on any bump,
   including patches.
2. All known downstream consumers of `ktav::Error` (the six language
   bindings under `ktav-lang/`) call `Err(e) => e.to_string()` only.
   No exhaustive `match err { Error::Io(_) => …, Error::Syntax(_)
   => …, Error::Message(_) => … }` patterns exist in the ecosystem
   that this change would silently break.
3. The seven canonical-category Display strings remain byte-identical
   to 0.1.4, so any hypothetical out-of-tree consumer doing string
   matching keeps working unmodified.

If your code does keep an exhaustive match over `ktav::Error` and
this release breaks it, add an `_ => …` arm. That arm is now
required forever and will not need to change again as future
variants are added.

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
