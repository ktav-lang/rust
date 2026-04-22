# Contributing to Ktav

## Core rules

### 1. Every bug fix ships with a regression test

When you find a bug, **before fixing it**, write a test that reproduces
it — the test **must fail on `main`** and pass after the fix. Include
both in the same PR.

The test should live near related tests (e.g. in
`tests/edge_cases/<topic>.rs` or `tests/ser/<topic>.rs`). A short
comment on the test explains the failure mode so a reader ten months
from now understands *why* the case matters.

Why: silent regressions are the deadliest thing that can happen to a
library shipped to many users. A documented trip-wire costs nothing
long-term.

### 2. Performance-sensitive changes include before/after numbers

If a PR touches any of:

- `src/parser/` / `src/thin/parser.rs` — parsing hot path
- `src/ser/text_serializer.rs` / `src/render/` — serialization hot path
- `src/thin/deserializer.rs` / `src/de/` — deserialization hot path
- `src/value/` — the dynamic value type

… include a block in the PR description with **before/after** criterion
numbers for the affected benches:

```
parse_to_struct/100_upstreams_typed
  before: 275 µs
  after:  198 µs
  change: -28%
```

The goal isn't to be fastest at any cost — it's to make changes
*accountable*. A 5 % regression is fine if it buys clarity or
correctness, but it should be visible and justified.

Use the convenience script:

```
./bench.sh                # quick run (warmup 1s, measurement 2s, 20 samples)
./bench.sh full           # criterion defaults — longer, more accurate
./bench.sh parse          # filter: only parse benches
./bench.sh render         # filter: only render benches
./bench.sh "parse|render" # any criterion regex
```

Criterion stores the last run's numbers in `target/criterion/` and
automatically diffs against them on the next run, so you'll see
`change: +/- X %` in the output.

### 3. Public API changes note compatibility

If you touch anything under `pub` in `lib.rs`, in the PR description
say whether it's:

- **semver-compatible** (additions, looser bounds, doc changes); or
- **semver-breaking** (renamed / removed items, changed signatures,
  tightened bounds) — in which case the version bump goes into the
  next `MINOR` while we're pre-1.0.

Update `CHANGELOG.md` in the same PR under `## [Unreleased]`.

### 4. One concept per commit

Commits should be atomic: a bug fix and its test together, a new
feature and its tests together. A rename belongs in its own commit. A
refactor that happens to fix a bug should probably be two commits.

`git log --oneline` reads like a changelog. Write it that way.

## Getting the code

The spec conformance suite lives in the `spec/` git submodule
([`ktav-lang/spec`](https://github.com/ktav-lang/spec)). Clone with
submodules so `cargo test` can run it:

```
git clone --recurse-submodules https://github.com/ktav-lang/rust
```

If you already cloned without `--recurse-submodules`:

```
git submodule update --init
```

## Running tests

```
cargo test                         # all tests (includes spec conformance)
cargo test --test spec_conformance # language-agnostic conformance suite only
cargo test --test edge_cases       # one category
cargo test multiline               # by name filter
cargo test --doc                   # doc-tests only
```

Test categories:

- `src/**/tests.rs` — private unit tests per module.
- `tests/de/*` — deserialization by feature.
- `tests/ser/*` — serialization by feature.
- `tests/roundtrip/*` — round-trip (`T → text → T`).
- `tests/edge_cases/*` — combinatorial edge cases (paren literals,
  keywords in maps, deep nesting, special strings…).
- `tests/fixtures.rs` — end-to-end against real `.conf` files.
- `tests/spec_conformance.rs` — language-agnostic suite from
  `ktav-lang/spec` (valid fixtures match JSON oracle; invalid fixtures
  are rejected; valid fixtures survive lossless round-trip).

## Benchmarks

Source: `benches/parse.rs` (criterion). Scenarios cover:

- `parse_to_value` — raw parse into `Value` (the owned, public tree).
- `parse_to_struct` — parse via the thin path into a typed struct.
- `render` — serialize a typed struct to text.
- `roundtrip` — parse + render.
- `multiline_dedent` — multi-line string parsing under varying line
  counts.

## Code layout guide

Decomposition rule: **one exported item per file**. Private helpers
live with the type that uses them. Folders group closely-related
items (all of `src/thin/` is the zero-copy deserialization path).

```
src/
├── lib.rs                         public entry points
├── value/                         owned Value enum (public)
├── parser/                        Value-building parser
├── render/                        Value → text
├── thin/                          zero-copy de path (ThinValue → T)
├── ser/                           T → Value (public) + T → text (direct)
├── de/                            Value → T (via ValueDeserializer)
└── error/                         Error + serde impls
```

## Philosophy (what not to do)

Ktav's motto is "be the config's friend, not its examiner". Before
proposing a new feature, ask:

- Does this add a new rule the reader must hold in their head?
- Can a line still be understood without its neighbours?

New rules are always costly. Reject everything that doesn't pass those
two checks.
