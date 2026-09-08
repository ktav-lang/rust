# R8-F6 measurement: what the InlineBounds index actually costs

Finding: R8-F6 (P3) in `2026-09-08-1533-spec-0.7-rust-review-round-8.md` —
the sorted pair vec + per-lookup binary search is `O(N + C log C)` for `C`
recorded compound spans in an inline body of `N` bytes, and "may add work
compared with the consistently short local scans" when a document has many
short shallow compounds. Explicitly not a bug; measurement requested before
any replacement.

**Verdict: the index costs nothing material. No production code was
changed.** Measured with deterministic counters first; then a direct
memo-on/memo-off A/B on the real parse path. The index is the cheap side of
the trade everywhere, including exactly the shape the review worried about:
the live-dispatch fallback it replaced is 2–12× slower on every
compound-dense shape, and parity (within noise) on small-C mainstream
shapes.

## Instruments

1. `ix_probe` (`src/parser/inline.rs`, `#[cfg(test)]` only): atomic counters
   at the two lookup sites (`known_closer`, `opener_close_at`) recording
   calls / hits / total / max binary-search steps, at the boundary sort
   (calls / elements / comparisons via the identical `sort_unstable_by`
   closure std implements `sort_unstable_by_key` with), and per recorded
   body (count, bytes, total and max pairs). Release builds compile none of
   it; every `#[cfg(not(test))]` statement is byte-identical to `cc58308`.
2. Direct A/B: the memo is a proven pure memo, so a cfg(test) `BYPASS`
   switch makes both lookup sites return `None` and forces the
   live-dispatch fallback — a byte-identical parse (controlled per shape:
   Debug-equality of the parsed Values) without the index lookups. The
   switch's engagement is proven per shape by a zero lookup-counter delta
   across the bypassed parse. Timing `memo=on` vs `memo=off` in ONE binary
   with alternated batches avoids the cross-binary calibration trap
   entirely (ns/iter normalized per run's own `iters=` line regardless).
3. Wall-clock SCEN batches (bench_ab protocol: 2 warmups, calibrate to
   >= 40 ms, 9 batches) and two micros (ns/op of `known_closer` hit/miss on
   a C=1024 table; ns per `sort_unstable_by_key` of a C=1024 table).

Fixture family (`benches/fixtures_ix.rs`, mounted by the probe tests in
`src/parser/tests.rs`): single-line inline bodies chosen to MAXIMISE `C` at
depth 1–2 — `k: [{}×n]` (3-byte items: the most lookups per body byte the
grammar can spell, n up to 4096 on one 12.3 KB line), `k: [{a:1}×n]`,
`k: {a_i: {}×n}`, `k: [[{}]×n]` (C = 2n, depth 2), plus `many_inline_trees`
(2000 lines × C=2 trees), `deep_chain` (32/96 — the opposite shape), the
harness `inline_doc` mix and the `synth` 50 KB mainstream mix. Shape
validation test pins item counts; the 4096-item line is asserted >12,000
bytes (no line cap shrinking C).

## Deterministic counters (final tree; P = `parse`, E = `parse_events` — identical counts, same machines)

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

Observations the review's `O(C log C)` model does not predict:

- Lookups are exactly 2·C per shape at a 100% hit rate (one
  `opener_close_at` per gated opener in split, one `known_closer` per
  compound-valued segment dispatch), ~log2(C) steps each. No miss ever
  pays on these shapes; a miss would fall to the live dispatch, which the
  A/B shows is far more expensive than any search.
- The sort is O(C) comparisons, not O(C log C), on every all-sibling
  shape: recorded pop order for siblings is already opener-ascending, and
  pattern-defeating quicksort detects the run — exactly C−1 comparisons
  (63 / 1023 / 4095 / 94 / 30 above). Only the depth-2 interleaved pop
  order defeats the run detection (`two_level_512`: 9578 comparisons for
  1024 elements, still under the 10240 a random order costs).
- Index operations per walked byte: worst case here is
  `wide_arr_tiny_4096` at ~9.0 (110,591 ops / 12,289 B). Mainstream:
  `inline_doc_50k` ~0.12, `many_trees_2000` ~0.69, `synth_50k` exactly 0 —
  the synth mix has no single-line inline compounds, so it never touches
  the index at all.

## Timings (secondary; this machine is noisy — treat <±20% as unmeasurable)

SCEN wall-clock per iteration (min of 9 batches, normalized by each run's
own `iters=` line; rounds r2/r3 on the final code, r1 on a binary whose
production code was identical): P-path mins ranged, e.g.
`wide_arr_tiny_4096:P` 558.8 µs (12293 B), `wide_arr_tiny_1024:P` 122.1 µs,
`deep_chain_96:P` 81.7 µs, `synth_50k:P` 850.7 µs, `inline_doc_50k:P`
3.45 ms, `many_trees_2000:P` 5.20 ms. Direction of same-code batch
comparisons flipped between batches on unchanged code, consistent with the
series' noise history; none of the conclusions below rests on
cross-binary wall-clock.

Micros (release, min of 9 batches, two rounds): `known_closer` on the
C=1024 table: hit 23.8–24.6 ns/op, miss 18.1–20.0 ns/op (11 search steps +
call overhead). `sort_unstable_by_key` of C=1024 (incl. a 16 KB clone of
the table in the loop): ascending 469–516 ns, reverse 796–848 ns. The real
all-sibling pop order is the ascending case.

### Direct A/B: memo lookups on vs off (same binary, controls pass)

Three rounds (r1 unpinned, r2/r3 pinned to one logical processor via a Job
Object), 9 batches per mode per round, 27 batches pooled per mode; ns/iter
per batch's own iters; min (mean) reported:

| shape | memo=on µs | memo=off µs | off/on (min) |
|---|---|---|---|
| wide_arr_tiny_4096 | 464.2 (698.1) | 5548.2 (6761.2) | **12.0×** |
| wide_arr_tiny_1024 | 99.7 (164.0) | 437.9 (548.2) | 4.4× |
| wide_arr_small_1024 | 500.1 (700.0) | 1075.5 (1441.2) | 2.1× |
| wide_obj_tiny_1024 | 261.7 (352.0) | 1235.4 (1484.0) | 4.7× |
| two_level_512 | 196.2 (250.6) | 484.4 (579.7) | 2.5× |
| deep_chain_96 | 59.9 (73.9) | 343.8 (423.9) | 5.7× |
| many_trees_2000 | 3263.4 (4095.3) | 3091.9 (4191.0) | 0.95× (noise) |

Removing the index lookups makes every compound-dense shape 2–12× SLOWER
— min and mean agree, and the gaps are 5–50× the noise band — because the
fallback re-runs full live scans (`find_matching_close` /
`scan_inline_closer` / per-dispatch machine setup) over each span, which
dwarfs log2(C) in-table steps. `many_trees_2000` (C=2 per tree, where the
live dispatch is also trivial) is parity within noise: 0.95× on mins, ~1.0×
on means.

## Verdict

- The index is NOT a material cost. It is the cheap half of the trade on
  every shape measured, including the review's hypothesized worst case
  (many short shallow compounds at depth 1–2): there the memo path is
  ~10× faster than the scans it replaced.
- Model attribution bounds what any replacement could still win: on
  `wide_arr_tiny_4096`, 8192 lookups × ~2.2 ns/step ≈ 35–40% of the
  memo-on parse is the binary searches themselves (consistent with the
  micro); a boundary-ID pass-down or a monotonic cursor could recover a
  fraction of that on this pathological 12 KB single-line shape, and
  <1% anywhere realistic. Not worth touching machines R8-F2 just finished
  reconciling.
- Deliberate non-change: production `src/` code is byte-identical to
  `cc58308` outside `#[cfg(test)]` blocks; the corpus diff is therefore
  not applicable, and the A/B carried its own positive controls
  (per-shape Debug-equality of parsed Values; zero-lookup-delta proof of
  switch engagement) on top of the standing memo purity test.

## Reproduce

`cargo test --release --lib parser::tests::ix_probe -- --nocapture`
(counters `IX` lines, wall-clock `SCEN`, micros `MIKC`/`MISORT`, A/B `AB`
lines). Raw runs: `D:/system_artefact/Temp/ktav-a2-harness/ix_counters_*.txt`,
`ix_wallclock_r*.txt`, `ix_ab_r*.txt`, `ix_fulltest*.txt`.

Commits on `r8-index`: b9d5e6d (ix_probe counters), 77e7d9c (wip fixtures +
probe tests), 4961040 (micro body-slice fix), 1225795 (A/B + controls),
fb065a0 (lint fixes, fixtures_ix split), plus this report.

## Verification (all on the final tree)

- `cargo build` — pass
- `cargo test` — 52 suites, 1069 passed, 0 failed, 1 ignored (baseline
  1065 + 4 new probe tests: shape validation, counters, wall-clock, A/B)
- `cargo clippy --all-targets -- -D warnings` — pass
- `cargo fmt --check` — clean
- `Cargo.toml` — unchanged vs `cc58308` (version 0.6.4,
  spec-version 0.6.4)
