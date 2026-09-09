# R11-F1 fix: SplitQ's last segment after a dotted key is raw data, not EmptyKey

Finding: R11-F1 (P2) in `2026-09-09-0903-spec-0.7-rust-review-round-11.md` —
R10-F1 (commit `acbc34a`) threads a single root-body `has_quotes: bool`
down to every `split_top_level` call, so a quote-free descendant can run
the quote-aware `SplitQ` machine. For one shape the two machines were NOT
equivalent: a `.`-armed key segment whose § 3.3 whitespace skip runs to
EOF (`{a: {b. }, q: '}`). `SplitFast` always reaches plain `Exhausted` and
pushes the raw segment; the caller's colon search then finds none and
raises the § 6.12 missing-separator `MalformedInlineCompound`.
`SplitQ`'s `ScanStop::EofAfterWsSkip` handler instead raised `EmptyKey`
for the non-blank remainder — a category that belongs exclusively to a
dotted path whose final segment is empty AFTER a separator was found
(`a.: 1`, § 6.5 via `insert_value`), which never reaches this stop at all.

Fixed on branch `r11-splitq` from base `b01a43a`; commits `433dd32` (fix
+ soundness comments) and `6d0af7b` (test corrections + matrix).
`Cargo.toml` untouched at `version = "0.6.4"` / `spec-version = "0.6.4"`.

## The one-branch fix

`src/parser/inline.rs`, `split_top_level`, `ScanStop::EofAfterWsSkip`
arm: the non-blank-remainder branch no longer returns
`Err(ErrorKind::EmptyKey)`; it pushes `&input[sc.seg_at..]` and returns
`Ok(sc.segments)` — byte-for-byte what the `Exhausted` arm (and hence
`split_top_level_fast`/`SplitFast`) does. The blank-remainder branch is
untouched: only-whitespace after the last comma stays a valid trailing
comma with no final segment emitted (the callers treat the two forms
identically).

Why this is the whole fix: reaching `EofAfterWsSkip` requires the
seg-start block guard `TRACK_QUOTES && in_key && seg_start`
(`src/parser/inline.rs`, `Scanner::run`). In the split configs
`in_key` goes true→false ONLY at a key-position `:` and false→true ONLY
at a body-depth comma (split never saves/restores scopes:
`USE_STACK = false`, `CLOSER_LITERAL = true`, nested openers are jumped
via `OPENER_JUMP`, so inner commas are never visited). A comma also
advances `seg_at` past itself. Therefore, at the stop, `input[seg_at..]`
cannot contain an unescaped `:` — one would have flipped `in_key` and
only a later comma could re-arm the block, moving `seg_at` beyond it.
(Escaped `\:` and quoted-segment colons never flip `in_key` and are
equally invisible to the callers' `find_unescaped_colon_inline`.) The
remainder is thus always a separator-less last segment, and pushing it
makes SplitQ's verdict flow through exactly SplitFast's: the caller
(`parse_inline_object_inner` / thin `scan_inline_object`) trims, finds
no unescaped colon, and raises MalformedInlineCompound per § 6.12, with
§ 5.8.2 routing to § 5.3's separator-finding-before-key-validation rule.

Not done, per the review: no per-level `has_quote_bytes` re-scan was
reintroduced (R10-F1's prescan removal stays), no scanner configs were
re-forked, no new machine added.

## Soundness comment correction (R10-F1 record)

The threaded-flag comments (`parse_inline_object`, `split_top_level` in
`src/parser/inline.rs`; `scan_inline_events` in
`src/thin/inline_emit.rs`) and the R10-F1 report
(`2026-09-09-r10f1-quote-prescan-cost.md`) claimed root-level quote
PRESENCE "implies" descendant presence. False: a descendant can be
quote-free while the root carries a quote elsewhere. The real argument
is asymmetric — root-level ABSENCE guarantees descendant absence (every
slice is a substring), so `SplitFast` is always safe for a quote-free
root; root-level PRESENCE only means a quote-free descendant MAY run
the quote-aware machine, safe only because the machines agree on
quote-free input — exactly the property this fix restores (it was the
one false shape). All four texts now state this.

## Test corrections

- `r3f1_split_top_level_dotted_key_trailing_ws_is_empty_key`
  (`src/parser/tests.rs`) pinned the wrong category since R3 — it
  asserted `Err(EmptyKey)` for `" \"a\": 1, b. "` via a direct
  `split_top_level` call with a hand-set quote flag. Rewritten as
  `r3f1_split_top_level_dotted_key_trailing_ws_raw_last_segment`: the
  call now returns `Ok([" \"a\": 1", " b. "])` — exactly what
  `SplitFast` produces for the same shape — and the full-parse
  category assertion (`{"a": 1, b. }` → MalformedInlineCompound) moved
  to the public API where it belongs. The review's own words stand in
  the test comment: the absence of panic was correct, the chosen
  category was not.
- `dotted_key_trailing_ws_is_empty_key` (`tests/inline_trailing_whitespace.rs`)
  pinned the same wrong category through the public API (`{ "a": 1,
  b. }` → EmptyKey on parse/from_str/parse_events). Renamed to
  `dotted_key_trailing_ws_missing_separator`, asserts
  MalformedInlineCompound on all three APIs (plus the nested
  `outer: { ... }` form), and adds the genuine-dotted-key positive
  control `{ "a": 1, b.: 2 }` → EmptyKey.

## Required matrix (new test, all four entry points, debug AND release)

`r11f1_splitq_last_segment_missing_separator_matches_fast_machine`
(`src/parser/tests.rs`) asserts `MalformedInlineCompound` through
`crate::parse`, `crate::parse_strict`, `crate::from_str::<serde_json::Value>`
and `crate::parse_events` for:

- quote at an ANCESTOR KEY position: `{"a": {b. }}`;
- quote in a SIBLING VALUE: `{a: {b. }, q: '}` and `{q: ', a: {b. }}`;
- quote in a SIBLING ARRAY ITEM: `[{b. }, ']`;
- SPACE, TAB and U+2000 after the dot — identical treatment;
- sibling quote species `'`, `"` and backtick.

Positive controls, unchanged: `{a: {b.}, q: '}` (no trailing whitespace;
plain `Exhausted` path) and quote-free `{a: {b.}}` → MalformedInlineCompound;
`{a: {b: 1, }, q: '}` → Ok everywhere (tree spot-checked);
`{a.: 1}` and `{"a": 1, b.: 2}` → EmptyKey (the genuine dotted
empty-key, which HAS a separator, on both fast and quote-aware paths).

## Golden corpus: 7 inputs × 4 parser legs, ERR→ERR only

Both `corpus_dump.rs` harvest roots point at this worktree, so the input
sets are identical by construction (post harness builds this branch,
base harness `worktrees/r11-base`, detached `b01a43a`). Both sides
regenerated: `inputs=7671 harvested=2218 generated=5453 panics=0` each,
six legs (P/S/J/E/R/C). Diff: 56 lines = 7 inputs × 4 parser legs; no
OK/ERR flips, no R/C legs (error paths emit none), everything else
byte-identical.

| input | base → post (all of P/S/J/E) | why |
|---|---|---|
| `{ "a": 1, b. }` + NL | EmptyKey → MalformedInlineCompound | pre-existing integration-test literal; separator-less last segment, § 6.12 |
| `outer: { "a": 1, b. }` + NL | EmptyKey → MalformedInlineCompound | same shape nested under a key |
| `{"a": 1, b. }` | EmptyKey → MalformedInlineCompound | R3 unit test's new full-parse assertion |
| `{a: {b. }, q: '}` | EmptyKey → MalformedInlineCompound | repro doc: quote in sibling value after |
| `{q: ', a: {b. }}` | EmptyKey → MalformedInlineCompound | repro doc: quote in sibling value before |
| `{"a": {b. }}` | EmptyKey → MalformedInlineCompound | repro doc: quote at ancestor key |
| `[{b. }, ']` | EmptyKey → MalformedInlineCompound | repro doc: quote in sibling array item |

Every changed line is the same verdict flip: the segment `b.` has no
unescaped separator, so § 6.12 diagnoses MalformedInlineCompound
("inline object pair missing ':' separator in 'b.'") — previously the
bug mis-categorized it as § 6.5 EmptyKey. Repro doc #1 `{a: {b. }}`
(quote-free root) does NOT appear: the root flag is false there, the
fast machine always handled it, and it was already correct. All new
matrix literals that do not trigger the bug parse identically on both
sides, hence no diff lines.

## Verification

| check | outcome |
|---|---|
| `cargo build` | PASS |
| `cargo test` (no skip flags, `--no-fail-fast`) | PASS — 1079 passed / 0 failed / 3 ignored / 52 suites (baseline `b01a43a`: 1078/3/52 + 1 new test) |
| matrix in release (`cargo test --release`, matrix + R3 + integration targets) | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo fmt --check` | PASS |
| `cargo test --test spec_conformance` with `KTAV_SPEC_DIR=D:/dev/ktav-lang/rust/spec` | 9 passed — 202 valid, 61/60 invalid category-matched, canonical 202 |
| `Cargo.toml` | untouched: `version = "0.6.4"`, `spec-version = "0.6.4"` |
| golden corpus, both sides regenerated | 7 inputs × 4 legs flip ERR→ERR as tabled above; 0 panics |

The three ignored tests are the pre-existing `arena_probe` and two ix
timing probes. The six-leg corpus run executed all four parser entry
points under a RELEASE build; the matrix test itself additionally ran
under both debug and release `cargo test`.

## Files touched

- `src/parser/inline.rs` — the `EofAfterWsSkip` last-segment branch +
  corrected threading comments.
- `src/thin/inline_emit.rs` — corrected threading comment.
- `src/parser/tests.rs` — R3 unit test corrected; R11-F1 matrix added.
- `tests/inline_trailing_whitespace.rs` — integration category
  corrected + positive control.
- `docs/reviews/2026-09-09-r10f1-quote-prescan-cost.md` — soundness
  paragraph restated.
- `docs/reviews/` — this report.

No push, no version bumps.
