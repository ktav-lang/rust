# R10-F1 fix: quote-presence prescans no longer re-walk each descendant subtree

Finding: R10-F1 (P2) in `2026-09-09-0010-spec-0.7-rust-review-round-10.md` —
the InlineBounds boundary memo removed the `O(N·D)` closer re-scan, but two
SEPARATE repeated quote-presence prescans (`has_quote_bytes`, three
`bytes.contains` passes) kept a depth multiplier alive: `split_top_level`
prescanned the full inner text of every non-empty inline body per nesting
level, and `scan_unescaped_colon` prescanned the whole pair text INCLUDING
the value when only the key prefix up to the separator needs the check.
Pure cost defect; no document mis-parsed.

Fixed on branch `r10-quote` from base `acf62a4`; commits `5032c3f` (fix),
`5e0df13` (proof), `924e9cc` (corpus literal pin). `Cargo.toml` untouched
at `version = "0.6.4"` / `spec-version = "0.6.4"`.

## What state is threaded, and where

`has_quotes: bool` — "the ROOT inline body contains at least one quote
byte" — is computed ONCE per parse entry and threaded down:

1. `src/parser/inline.rs`: `parse_inline_object` / `parse_inline_array`
   compute it over the full body (`has_quote_bytes(input.as_bytes())`) and
   pass it through `parse_inline_object_inner`, `parse_inline_array_inner`,
   `parse_inline_value`, `parse_inline_value_raw` to
   `split_top_level(input, line_num, span, body, bounds, has_quotes)`,
   whose fast/quote-aware dispatch now consumes the flag instead of calling
   `has_quote_bytes` per level.
2. `src/thin/inline_emit.rs`: `scan_inline_events` computes it once over
   the root body and threads it through `scan_inline_object`,
   `scan_inline_array_into`, `scan_inline_value`,
   `scan_inline_value_trimmed` to the same `split_top_level` calls.

Soundness: every nested slice is a substring of the root body, so
root-level quote presence implies descendant presence (monotone under
substring). The converse is allowed: a quote-free level whose root carries
quotes runs the quote-aware machine — harmless because R8-F2 made the fast
and quote-aware machines byte-identical on quote-free slices (R9-F1 closed
the last divergence). So the flag can over-approximate per level but never
under-approximate; only speed is affected by false positives, measured
invisible below. No new scanning strategy was introduced: SplitFast/SplitQ
and every other config pair are untouched; only the dispatch input changed.

`has_quote_bytes` is now `pub(crate)` so both engines share the
instrumented helper.

## Separator search: key prefix only

`scan_unescaped_colon` (`src/parser/inline.rs`) is candidate-driven:

1. `find_unescaped_colon_fast` (memchr2 `\` / `:`, escape-aware) proposes
   the first unescaped `:` candidate;
2. `key_prefix_quote_state(s, from, cand)` decides quote-opacity over the
   KEY PREFIX `[from, cand)` ONLY — never the value: a SIMD
   `has_quote_bytes` early-out over the prefix, then a byte walk mirroring
   the § 5.3.3 machine (segment-start rule, `skip_segment_ws`,
   `quoted_span_end`, `\` consumes the next byte);
3. a candidate outside every segment is the separator (`Found`); a
   candidate inside a segment that closes past it resumes after the
   closer (never at a segment start); a segment that never closes is
   `UnterminatedQuote`;
4. the pre-fix full quote-aware walk survives verbatim as
   `scan_unescaped_colon_slow`, used only when NO unescaped candidate
   exists anywhere but quote bytes are present — the `Absent` vs
   `UnterminatedQuote` distinction there (§ 6.16 precedence over
   `MissingSeparator`) does not depend on finding a colon.

`find_unescaped_colon` and `find_unescaped_colon_inline` are unchanged
delegates, so the out-of-scope callers (`classify.rs`, `parser.rs`,
`event_parser.rs`) keep their exact Found/Absent/Unterminated semantics.
The value's own quotes/colons/dots cannot move or hide the pair separator:
the candidate colon's prefix is all that is ever inspected.

## Why positional opacity and escape-awareness are provably unaffected

Mechanism: the prefix walk is the old slow machine restricted to
`[from, cand)` with the identical resume state (`from == 0` seeds
`seg_start = true`; after a closed span `resume = closer + 1` seeds
`seg_start = false`, exactly like the slow walk's `i = end + 1;
seg_start = false`). Escapes are excluded from candidate positions by the
memchr scan itself (a colon preceded by `\` is never proposed), and the
walk skips escaped bytes with the same `i += 2` rule. Unterminated spans
are detected by the same `quoted_span_end` over the same byte range.

Tests — all re-run green in the full suite (1078 passed / 3 ignored /
52 suites, no skip flags):

- Existing pins, unchanged in intent:
  `quoted_colon_scan_finds_colon_outside_spans`, `quoted_colon_scan_unterminated`,
  `quoted_colon_scan_absent_and_escapes`, `quoted_find_unescaped_colon_inline`,
  `quoted_split_top_level_object_mode`, `quoted_split_top_level_array_mode_ignores_quotes`,
  `quoted_find_matching_close_object_mode`, `quoted_find_matching_close_triple_nested`,
  `r3f1_split_top_level_*` (5), `r3f1_find_unescaped_colon_inline_eof_after_dotted_ws`,
  `r3f2_split_top_level_array_body_quoted_key_object`, `r3f4_split_top_level_*` (5),
  `r9f1_find_unescaped_colon_inline_ignores_key_depth`,
  `memo_bounds_are_a_pure_memo_of_the_live_dispatches` (memo purity,
  quote-bearing alphabet sweep).
- New pins (this fix): `r10f1_colon_scan_value_side_quotes_are_not_key_quotes`
  (value-side quotes/dots never move or hide the separator, including the
  truly-unterminated `a: b."unterm: 2`), `r10f1_colon_scan_no_candidate_corners`
  (Absent vs UnterminatedQuote with no candidate colon),
  `r10f1_root_quotes_threaded_over_quotefree_descendants` (threaded flag
  over quote-free descendants: quoted-key opacity, segmentation and
  engine parity through `parse` / `parse_strict` / `from_str` /
  `parse_events`).

## Deterministic counter: before/after

Instrument: `ix_probe` (cfg(test), thread-local) gains `hq_calls` /
`hq_bytes` / `hq_max`; `has_quote_bytes` records the slice length handed
to each check (one call = up to three `contains` passes, so true byte
views are 1x–3x `hq_bytes`; the factor is identical on both sides, so
comparisons hold). It also records the entry-point scans and the per-pair
prefix checks — i.e. ALL remaining prescan work, not just the removed one.

Caveat (per the review): the existing `body_bytes` counter measures the
RECORDING pass of the gate scan, not these separate `contains` calls, so
it never proved linearity of the whole inline parse; `hq_bytes` is the
instrument that does.

Probe: `ix_probe_r10f1_quote_prescan_counters` over
`ix_fixtures::deep_chain_leaf(depth, leaf_bytes)` (`k: {a: {a: … leaf …}}`,
`root:` line shape), paths P (`parse`) and E (`parse_events`) — identical
numbers on both paths because both engines share the helpers. AFTER
numbers come from this branch; BEFORE numbers were measured on a throwaway
instrumented copy of base `acf62a4`
(`D:/system_artefact/Temp/ktav-a2-harness/r10f1-measure-base`) carrying
the same counter, fixture and probe code — nothing else differs.

`hq_bytes` (bytes handed to quote-presence checks per parse):

| shape (leaf, depth) | doc_bytes | before | after | ratio |
|---|---|---|---|---|
| M=512, D=1  | 520    | 2 589    | 1 039   | 2.5x  |
| M=512, D=4  | 535    | 5 787    | 1 075   | 5.4x  |
| M=512, D=8  | 555    | 10 191   | 1 123   | 9.1x  |
| M=512, D=32 | 675    | 39 975   | 1 411   | 28.3x |
| M=512, D=64 | 835    | 88 647   | 1 795   | 49.4x |
| M=8192, D=8 | 8 235  | 156 111  | 16 483  | 9.5x  |
| M=65536, D=8| 65 579 | 1 245 647| 131 171 | 9.5x  |

- Depth family (fixed M=512): before grew ≈ 1 364 bytes/level
  ((88 647 − 2 589) / 63 — the M-byte leaf re-scanned at every level by
  both sites); after grows ≈ 12 bytes/level (per-level key-prefix checks
  on the one-byte key `a`). The D-multiplier is gone: cost is
  `Theta(M + D)`.
- Leaf family (fixed D=8): before ≈ 19.0x `doc_bytes` (the M-byte leaf
  re-checked at 8 levels × 2 sites); after ≈ 2.0x (the two root-body
  scans: the § 5.2 gate + the parse entry). Linear in M with a factor-2
  constant.
- `hq_max` stays the root body on both sides — what collapsed is the
  MULTIPLICITY of near-root-size checks, not their size.
- Scaling pins for future rounds:
  `r10f1_quote_prescan_cost_no_depth_multiplier`
  (`hq_bytes(D=32) < 2·hq_bytes(D=1) + 32·64`, plus `≥ M` so a broken
  prescan cannot hide) and `r10f1_quote_prescan_cost_linear_in_leaf`
  (`M ≤ hq_bytes < 8·M`, growth ratio in M bounded), both failing on the
  pre-fix code by construction of the family.

## Golden corpus: empty diff, positive control

Procedure: both `corpus_dump.rs` harvest roots point at this worktree, so
the input sets are identical by construction; the post harness builds
against this branch, the base harness against `worktrees/r10-base-quote`
(detached `acf62a4`). Both sides regenerated with the FINAL input set:
`inputs=7648 harvested=2194 generated=5454 panics=0` each, six legs
(P/S/J/E/R/C), **`diff` EMPTY**.

An empty diff is not evidence by itself (this series has hit that three
times), so a positive control was run: the scope bug was deliberately
reintroduced by changing the separator scan's opacity check from the key
prefix to the whole pair (`key_prefix_quote_state(s, 0, s.len())`), the
post harness rebuilt, and the dump re-generated:

- Control run 1 (input set before the literal below): the corpus
  DISTINGUISHES the bug — 19 inputs flip on all four parser legs
  (P/S/J/E; R/C follow the parse), e.g. `"a: b": 1` → `InvalidKey`,
  `a."b:c".d: 1` and `` `a:b`: 1 `` mis-segmented. All 19 are
  pre-existing corpus literals, quoted-KEY-side shapes.
- The pure VALUE-side sub-class was NOT visible: `a: b."unterm: 2` (dot-
  armed segment start, unterminated quote inside a value, colon after
  both) was absent from the input set. The literal is now pinned in
  `r10f1_colon_scan_value_side_quotes_are_not_key_quotes` (commit
  `924e9cc`), and control run 2 (final input set) DISTINGUISHES it too —
  190 diff lines including that input, which flips from a pair to an
  unterminated-key error class.
- After reverting the control and rebuilding: the diff against base is
  EMPTY again (verified byte-identical). As in R9-F4, the corpus cannot
  observe COST at all — the deterministic counters and scaling pins above
  are the actual proof of the cost fix; the corpus + controls prove the
  behavior surface is unchanged.

## Verification

| check | outcome |
|---|---|
| `cargo build` | PASS |
| `cargo test` (no skip flags) | PASS — 1078 passed / 0 failed / 3 ignored / 52 suites (baseline 1071/3/52 + 7 new tests) |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo fmt --check` | PASS |
| `Cargo.toml` | untouched: `version = "0.6.4"`, `spec-version = "0.6.4"` |
| golden corpus, both sides regenerated | EMPTY diff (7648 inputs, six legs, 0 panics) |
| positive control (scope bug reintroduced) | corpus DISTINGUISHES (19 inputs; + the pinned value-side literal after `924e9cc`) |
| memo purity tests (`memo_bounds_are_a_pure_memo_of_the_live_dispatches` et al.) | PASS, unchanged |

The three ignored tests are the pre-existing `arena_probe` and two ix
timing probes. The worktree's `spec/` submodule is uninitialized, so the
9 `spec_conformance` tests need the orchestrator's `KTAV_SPEC_DIR` run
before merging. Release build of the lib happened transitively via the
release harness binaries.

## Files touched

- `src/parser/inline.rs` — threading, candidate-driven separator scan,
  `has_quote_bytes` pub(crate) + instrumentation, ix_probe `hq_*` counters.
- `src/thin/inline_emit.rs` — threading through the thin engine.
- `src/parser/tests.rs` — 7 new tests, split call sites updated for the
  new parameter.
- `benches/fixtures_ix.rs` — `deep_chain_leaf(depth, leaf_bytes)` family.
- `docs/reviews/` — this report.

`src/render/*` untouched. No push, no version bumps.
