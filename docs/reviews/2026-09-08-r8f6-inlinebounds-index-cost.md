# R8-F6 measurement: what the InlineBounds index actually costs

Finding: R8-F6 (P3) in `2026-09-08-1533-spec-0.7-rust-review-round-8.md` —
the sorted pair vec + per-lookup binary search is `O(N + C log C)` for `C`
recorded compound spans in an inline body of `N` bytes, and "may add work
compared with the consistently short local scans" when a document has many
short shallow compounds. Explicitly not a bug; measurement requested before
any replacement.

**REVISED 2026-09-08 (round-9 review, R9-F2/R9-F3).** The first version of
this report concluded "the index costs nothing material" on evidence that
did not carry that claim: its A/B ran in the instrumented lib-test binary
(cfg(test) ix counters plus the counting global allocator), its `off` side
re-ran live scans instead of consulting a different index, and its batches
were not alternated. Its verdict is withdrawn and replaced below by an
isolated, uninstrumented attribution: **the index is a real 5–37% of parse
time on compound-dense single-line shapes and under 1% on mainstream
shapes (0.72%/0.58% and 0.38%/0.34% on the two mainstream-shaped rows
below after the R10-F3 arithmetic correction, 0% where C=0).
The sorted-`Vec` index is KEPT as a conservative engineering choice.** The
measured data below is unchanged — only its interpretation is corrected.

## What each instrument is valid for (revised)

1. `ix_probe` deterministic counters (`src/parser/inline.rs:1141`,
   `#[cfg(test)]`, thread-local since R9-F2): calls / hits / total / max
   binary-search steps per lookup site, sort calls / elements /
   comparisons, recorded-body counts. Primary instrument; production
   -equivalent counts. Now an ordinary (non-ignored) test,
   `ix_probe_r8f6_index_counters` (src/parser/tests.rs:2389); its output
   was re-verified byte-identical across the R9-F2 refactor (base
   `fea9513` vs `cef7124`, 22/22 `IX` lines equal).
2. The bypass A/B, `ix_probe_r8f6_memo_lookup_ab`
   (src/parser/tests.rs:2649, `#[ignore]` since R9-F2): the bypass makes
   both lookup sites return `None` and forces the live-dispatch fallback.
   The batch delta is therefore **cached-vs-uncached PARSING — the memo's
   benefit — under lib-test instrumentation**, NOT the isolated cost of
   the binary search and not a comparison of index choices. The first
   version's "the switch's engagement is proven by a zero counter delta"
   control stays (now thread-locally exact), and the first version's
   claim that its batches were "alternated" is corrected: the loop runs
   all `on` batches and then all `off` batches
   (`for (label, bypass) in [("on", false), ("off", true)]`), two
   sequential blocks with no defence against drift.
3. `bench_ix` (a2-harness `h-meas-post/src/bench_ix.rs`, NEW): the
   isolated instrument. A harness binary links `ktav` as a plain
   dependency, so the library is compiled WITHOUT `cfg(test)` — no ix
   counters, no counting allocator. It measures (a) production parse
   wall clock per shape (`SCEN`), (b) the production lookup sequence in
   isolation — a byte-faithful copy of `known_closer`'s non-test body
   (`MIKC` point micros; `MSWEEP` full sweeps at C=2/1024/4096), (c) the
   index-CHOICE comparison the first version lacked: binary search vs a
   monotonic cursor vs a passed boundary — all three KEEPING the computed
   boundaries, never re-parsing, and (d) boundary-sort cost (`MISORT`).
   There is no memo on/off switch here: the memo has no production
   switch, and forking the parser for measurement would reopen the
   R8-F2 machine-divergence defect family.

Fixture family (`benches/fixtures_ix.rs`, mounted by the probe tests in
`src/parser/tests.rs`): single-line inline bodies chosen to MAXIMISE `C` at
depth 1–2 — `k: [{}×n]` (3-byte items, n up to 4096 on one 12.3 KB line),
`k: [{a:1}×n]`, `k: {a_i: {}×n}`, `k: [[{}]×n]` (C = 2n, depth 2), plus
`many_inline_trees` (2000 lines × C=2 trees), `deep_chain` (32/96 — the
opposite shape), the harness `inline_doc` mix and the `synth` 50 KB
mainstream mix. Shape validation test pins item counts; the 4096-item line
is asserted >12,000 bytes (no line cap shrinking C).

## Deterministic counters (final tree; P = `parse`, E = `parse_events` — identical counts, same machines)

Re-verified identical after the R9-F2 thread-local refactor (base
`fea9513` vs `cef7124`): 22/22 `IX` lines equal.

| shape | doc B | walked B | C (max/tree) | kc hits/calls | kc steps (max) | oca steps (max) | sorts | sort elems | sort cmps |
|---|---|---|---|---|---|---|---|---|---|
| wide_arr_tiny_64 | 197 | 193 | 64 | 64/64 | 448 (7) | 448 (7) | 1 | 64 | 63 |
| wide_arr_tiny_1024 | 3077 | 3073 | 1024 | 1024/1024 | 11264 (11) | 11264 (11) | 1 | 1024 | 1023 |
| wide_arr_tiny_4096 | 12293 | 12289 | 4096 | 4096/4096 | 53248 (13) | 53248 (13) | 1 | 4096 | 4095 |
| wide_arr_small_1024 | 6149 | 6145 | 1024 | 1024/1024 | 11264 (11) | 11264 (11) | 1 | 1024 | 1023 |
| wide_obj_tiny_1024 | 9135 | 9131 | 1024 | 1024/1024 | 11264 (11) | 11264 (11) | 1 | 1024 | 1023 |
| two_level_512 | 2565 | 2561 | 1024 | 1024/1024 | 11264 (11) | 11264 (11) | 1 | 1024 | 9578 |
| many_trees_2000 | 40890 | 26000 | 4000 (2) | 4000/4000 | 8000 (2) | 8000 (2) | 2000 | 4000 | 2000 |
| deep_chain_32 | 164 | 161 | 31 | 31/31 | 186 (6) | 186 (6) | 1 | 31 | 30 |
| deep_chain_96 | 484 | 481 | 95 | 95/95 | 760 (8) | 760 (8) | 1 | 95 | 94 |
| inline_doc_50k | 50072 | 45842 | 1240 (2) | 1240/1240 | 2480 (2) | 2480 (2) | 620 | 1240 | 620 |
| synth_50k | 51211 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Two corrections to the first version's reading of this table:

- "The sort is O(C), not O(C log C)" — a run of already-ascending keys is
  INPUT that pdqsort's run detection exploits; it is fully COMPATIBLE with
  the `O(C log C)` upper bound, not a refutation of it. Only the
  depth-2 interleaved pop order pays the log factor here
  (`two_level_512`: 9578 comparisons for 1024 elements).
- Lookups stay LOGARITHMIC regardless of how cheap one comparison is on
  this machine: ~log2(C) comparisons per lookup (13 at C = 4096). A cheap
  comparison lowers the constant, never the asymptotics.

## Isolated, uninstrumented measurement (bench_ix, R9-F3 instrument)

Protocol: 6 rounds pinned to one logical processor (Job Object, `capt 1`)
plus 1 unpinned warm-up round; each round runs 9 shapes × P/E through the
bench_ab protocol (2 warmups, calibrate to ≥40 ms, 9 batches, ns/iter
normalized by each run's own `iters=` line) and the micros; every value
below reports the per-round minimum AND the median across the six pinned
rounds' minima.

Isolated lookup cost (ns per lookup, all hits — the real access pattern;
the production `known_closer` body including the span check):

| strategy | C=2 | C=1024 | C=4096 |
|---|---|---|---|
| bin (production binary search) | 2.56 / 2.75 | 10.77 / 13.55 | 13.83 / 16.63 |
| cursor (monotonic, boundaries kept) | 2.43 / 2.64 | 1.59 / 1.68 | 1.69 / 1.80 |
| direct (passed boundary, O(1)) | 1.67 / 1.69 | 0.93 / 0.99 | 0.86 / 0.98 |

Point micros (`MIKC`, C=1024 table): hit 12.10 / 12.67 ns, miss
10.50 / 11.53 ns — consistent with the sweep. Sort (`MISORT`, C=1024,
incl. the 16 KB clone the loop performs): ascending 600 / 736 ns, reverse
801 / 1013 ns.

**CORRECTED 2026-09-09 (round-10 review, R10-F3).** The table below
originally used only ONE of KC calls / OCA calls for `many_trees_2000`
and `inline_doc_50k` — the two shapes whose recorded pairs come from
BOTH an Object and an Array level, so both lookup sites fire once per
pair. `lookups` for those two rows is corrected from 4000/1240 to
8000/2480 (verified by re-running `ix_probe_r8f6_index_counters`
against the current tree: `many_trees_2000` kc=4000/4000 calls,
oca=4000/4000 calls; `inline_doc_50k` kc=1240/1240, oca=1240/1240 —
sum, not either alone). This is an arithmetic recheck of the existing
timing data with the report's own published per-lookup `bin` rates and
`SCEN` denominators, not a new measurement.

Attribution (share of the whole-parse wall clock the binary searches
could account for = (kc+oca calls) × bin rate ÷ SCEN; all numbers
min / median):

| shape | lookups | C | SCEN P µs (min/med) | attributed µs | share |
|---|---|---|---|---|---|
| wide_arr_tiny_4096 | 8192 | 4096 | 327 / 367 | 113 / 136 | **35% / 37%** |
| wide_arr_tiny_1024 | 2048 | 1024 | 68 / 83 | 22 / 28 | **33% / 34%** |
| two_level_512 | 2048 | 1024 | 168 / 182 | 22 / 28 | 13% / 15% |
| wide_obj_tiny_1024 | 2048 | 1024 | 207 / 246 | 22 / 28 | 11% / 11% |
| deep_chain_96 | 190 | 95 | 63 / 64 | ≤2.6 | ≤4% |
| wide_arr_small_1024 | 2048 | 1024 | 462 / 607 | 22 / 28 | 5% / 5% |
| many_trees_2000 | 8000 | 2 | 2851 / 3825 | 20.48 / 22.00 | 0.72% / 0.58% |
| inline_doc_50k | 2480 | 2 | 1656 / 1981 | 6.35 / 6.82 | 0.38% / 0.34% |
| synth_50k | 0 | — | 582 / 680 | 0 | 0% |

The corrected `many_trees_2000` / `inline_doc_50k` shares roughly
double but stay the same order of magnitude and change nothing about
the kept-index decision; they do mean the headline "≤0.4% on
mainstream" below no longer follows from this table as stated for
these two rows — the verdict is restated to name the shape it actually
holds for.

E-leg shares are the same attributed µs over larger denominators (e.g.
`wide_arr_tiny_4096:E` 510 / 596 µs → 22–23%).

Noise assessment (honest): even the per-round MIN of 9 batches moves
1.34×–1.96× across the six pinned rounds on whole-parse SCEN
(`many_trees_2000:P` round-minima 2.85–5.59 ms), so absolute SCEN numbers
are quoted as a min–median band and any wall-clock difference under ±20%
is unmeasurable on this machine. The per-lookup micros are much steadier
(spread 1.14×–1.29× for bin C=2/4096; one drift round reached 1.72× at
C=1024). The attribution is a MODEL — isolated sweeps of the exact lookup
sequence, table sizes matched to the counters, applied against the same
binary's whole-parse totals — not a marginal A/B of production code; the
memo's recording cost (pairs pushed during the live scan) is not
separately instrumented, and in-parse cache/branch state may differ from
the sweep. The band above is the honest resolution this machine supports,
and it is enough to settle the question the first version got wrong.

What a different index could win (both sides keeping the boundaries): a
monotonic cursor costs ~1.6–1.8 ns/lookup vs 10.8–16.6 ns for the binary
search at C≥1024, so `wide_arr_tiny_4096` could shed roughly a third of
its parse and `wide_arr_tiny_1024` ~30%. At C=2 — every mainstream shape
— cursor 2.64 vs bin 2.75 ns/lookup is parity within noise: there is
nothing to win where the document does not concentrate C on one line.

### The bypass A/B: memo benefit (instrumented; kept as an `#[ignore]`d side-instrument)

Three rounds on the pre-R9 code (r1 unpinned, r2/r3 pinned), 27 batches
pooled per mode; ns/iter per batch's own iters; min (mean) reported —
VALID only as cached-vs-uncached parsing under instrumentation:

| shape | memo=on µs | memo=off µs | off/on (min) |
|---|---|---|---|
| wide_arr_tiny_4096 | 464.2 (698.1) | 5548.2 (6761.2) | 12.0× |
| wide_arr_tiny_1024 | 99.7 (164.0) | 437.9 (548.2) | 4.4× |
| wide_arr_small_1024 | 500.1 (700.0) | 1075.5 (1441.2) | 2.1× |
| wide_obj_tiny_1024 | 261.7 (352.0) | 1235.4 (1484.0) | 4.7× |
| two_level_512 | 196.2 (250.6) | 484.4 (579.7) | 2.5× |
| deep_chain_96 | 59.9 (73.9) | 343.8 (423.9) | 5.7× |
| many_trees_2000 | 3263.4 (4095.3) | 3091.9 (4191.0) | 0.95× (noise) |

Reading: the memo pays for its lookups everywhere it fires at scale —
the fallback re-runs full live scans, which dwarf even the 33–37% the
searches cost on the pathological shapes. These numbers are NOT
production timings (instrumented binary) and are not comparable across
binaries.

## Revised verdict

- The index is NOT free, and the first version's "no material cost" was
  wrong for the shapes the review worried about: on single-line
  compound-dense documents the binary searches are a measured 33–37% of
  the parse. On mainstream-shaped documents (`many_trees_2000`,
  `inline_doc_50k` — many small trees rather than one deep or wide
  compound) the index costs under 1% of the parse (0.72%/0.58% and
  0.38%/0.34% respectively, corrected in R10-F3) and is never consulted
  in vain (100% hit rate, exactly 2·C lookups).
- The sorted-`Vec` index is KEPT. The alternatives' win is concentrated
  on pathological single-line shapes with C in the thousands; taking it
  would add a second lookup machine next to the two scanners rounds 3
  through 9 just finished reconciling (R8-F2 byte-identity discipline),
  for a benefit mainstream documents cannot measure.
- **Scope of the "unchanged" claim (corrected in R10-F3):** `src/` as a
  whole is NOT byte-identical to `cc58308` — R9-F1 (`fea9513`) changed
  inline key-position and separator-scan behavior (some inputs now get
  a different error category) and R9-F4 (`f8ac675`) changed the writer
  helper; `git diff cc58308..HEAD -- src` shows both. What IS
  byte-identical to `cc58308` outside `#[cfg(test)]` is narrower and is
  the only thing this report's "no corpus diff needed" claim actually
  rests on: the `InlineBounds` struct's `known_closer` and
  `opener_close_at` lookup bodies (the `#[cfg(not(test))]` line in each
  is textually the same `self.pairs.binary_search_by_key(...)` call the
  pre-instrumentation code had) and the `pairs.sort_unstable_by_key(|p|
  p.0)` call itself — verified directly against
  `git diff cc58308..HEAD -- src/parser/inline.rs`. The A/B's own
  positive controls and the counters test's every-suite-run check are
  what actually cover this narrower claim; the corpus is not a
  substitute for either.

## Reproduce

- Counters (ordinary test, every suite run):
  `cargo test --release --lib ix_probe_r8f6_index_counters -- --nocapture`
- Wall-clock instrument (ignored, single-threaded):
  `cargo test --release --lib ix_probe_r8f6_wall_clock -- --ignored --test-threads=1 --nocapture`
- Bypass A/B (ignored, single-threaded):
  `cargo test --release --lib ix_probe_r8f6_memo_lookup_ab -- --ignored --test-threads=1 --nocapture`
- Uninstrumented isolated instrument (a2-harness `h-meas-post`):
  `cargo build --release --bin bench_ix` then
  `capt 1 target/release/bench_ix.exe` (pin optional). Summary lines
  carry min/median; raw batch lines must be normalized by their own
  `iters=` line. Raw rounds:
  `D:/system_artefact/Temp/ktav-a2-harness/ix_r9_b1..b7.txt`;
  counters base/post positive control:
  `ix_r9_counters_base.ix` vs `ix_r9_counters_post.ix` (identical);
  corpus diff: `ix_r9_corpus.diff` (trailer-only).
- First-version raw runs (pre-correction):
  `ix_counters_*.txt`, `ix_wallclock_r*.txt`, `ix_ab_r*.txt`.

Commits: b9d5e6d, 77e7d9c, 4961040, 1225795, fb065a0 (first version,
`r8-index`); cef7124 (R9-F2 probe isolation + first correction); this
file's R10-F3 pass (attribution arithmetic and provenance scope) is a
documentation-only correction with no accompanying commit hash of its
own.

## Verification (revised tree, all re-run)

- `cargo build` — pass
- `cargo test` with NO skip flags — 52 suites, 1071 passed, 0 failed,
  3 ignored (the two timing probes now `#[ignore]`d and validated by
  their own invocation: wall-clock 15.0 s ok, A/B 9.5 s ok with both
  positive controls passing on the thread-local state)
- `cargo clippy --all-targets -- -D warnings` — pass
- `cargo fmt --check` — clean
- `Cargo.toml` — unchanged (version 0.6.4, spec-version 0.6.4)
