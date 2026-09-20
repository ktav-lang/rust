//! R8-F6 probe tests: deterministic `InlineBounds` index-cost counters.

use super::ix_fixtures;
use super::S;
use crate::parser::inline::InlineBounds;
// --- R8-F6 probe: deterministic InlineBounds index-cost counters -----------

#[test]
fn ix_probe_r8f6_fixture_shapes_are_valid() {
    use crate::Value;

    // Wide arrays of tiny compounds: one Array of n empty Objects.
    let doc = ix_fixtures::wide_line_arr_tiny(64);
    let v = crate::parse(&doc).expect("wide_arr_tiny must parse");
    let arr = v.as_object().unwrap().get("k").unwrap().as_array().unwrap();
    assert_eq!(arr.len(), 64);
    assert!(arr
        .iter()
        .all(|it| matches!(it, Value::Object(o) if o.is_empty())));

    // Wide objects of tiny compounds: one Object with n empty Objects.
    let doc = ix_fixtures::wide_line_obj_tiny(64);
    let v = crate::parse(&doc).expect("wide_obj_tiny must parse");
    let obj = v
        .as_object()
        .unwrap()
        .get("k")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(obj.len(), 64);
    assert!(obj
        .iter()
        .all(|(_, it)| matches!(it, Value::Object(o) if o.is_empty())));

    // Two-level: Array of n one-Arrays of one empty Object.
    let doc = ix_fixtures::wide_line_two_level(64);
    let v = crate::parse(&doc).expect("two_level must parse");
    let arr = v.as_object().unwrap().get("k").unwrap().as_array().unwrap();
    assert_eq!(arr.len(), 64);
    for it in arr {
        let inner = it.as_array().expect("two-level item must be an Array");
        assert_eq!(inner.len(), 1);
        assert!(matches!(inner[0], Value::Object(ref o) if o.is_empty()));
    }

    // Many small trees: root of `lines` pairs, each a 2-item Array.
    let doc = ix_fixtures::many_inline_trees(50);
    let v = crate::parse(&doc).expect("many_trees must parse");
    let root = v.as_object().unwrap();
    assert_eq!(root.len(), 50);
    for (k, it) in root {
        let arr = it
            .as_array()
            .unwrap_or_else(|| panic!("{k}: expected Array"));
        assert_eq!(arr.len(), 2, "{k}: expected 2 items");
    }

    // Deep chain: the "k" spine nests exactly `depth` Objects down
    // (root key is "k"; each nested level repeats the key "a").
    let doc = ix_fixtures::deep_chain(32);
    let v = crate::parse(&doc).expect("deep_chain must parse");
    let mut depth = 0usize;
    let mut cur = &v;
    while let Some(obj) = cur.as_object() {
        let key = if depth == 0 { "k" } else { "a" };
        cur = obj
            .get(key)
            .unwrap_or_else(|| panic!("deep_chain spine key {key} missing"));
        depth += 1;
        if cur.as_object().is_none() {
            break;
        }
    }
    assert_eq!(depth, 33, "deep_chain spine depth drift");

    // The long single lines really are long (no accidental line cap
    // shrinking C below the intended shape).
    assert!(ix_fixtures::wide_line_arr_tiny(4096).len() > 12_000);
}

fn ix_print_counters(
    name: &str,
    path: &str,
    doc_bytes: usize,
    s: &crate::parser::inline::ix_probe::Snapshot,
) {
    println!(
        "IX\t{name}\t{path}\tdoc_bytes={doc_bytes}\tbodies={}\tbody_bytes={}\tpairs={}\tpairs_max={}\tkc={}/{}\tkc_steps={}\tkc_steps_max={}\toca={}/{}\toca_steps={}\toca_steps_max={}\tsort={}\tsort_elems={}\tsort_cmps={}",
        s.bodies, s.body_bytes, s.pairs_total, s.pairs_max,
        s.kc_hits, s.kc_calls, s.kc_steps, s.kc_steps_max,
        s.oca_hits, s.oca_calls, s.oca_steps, s.oca_steps_max,
        s.sort_calls, s.sort_elems, s.sort_cmps,
    );
}

/// R8-F6 measurement: run both parse paths over the whole shape space
/// and print the deterministic ix_probe counters per shape and path.
/// Run filtered with --nocapture: `cargo test --release --lib ix_probe -- --nocapture`.
#[test]
fn ix_probe_r8f6_index_counters() {
    use std::fmt::Write as _;

    let shapes: Vec<(&str, String)> = vec![
        ("wide_arr_tiny_64", ix_fixtures::wide_line_arr_tiny(64)),
        ("wide_arr_tiny_1024", ix_fixtures::wide_line_arr_tiny(1024)),
        ("wide_arr_tiny_4096", ix_fixtures::wide_line_arr_tiny(4096)),
        (
            "wide_arr_small_1024",
            ix_fixtures::wide_line_arr_small(1024),
        ),
        ("wide_obj_tiny_1024", ix_fixtures::wide_line_obj_tiny(1024)),
        ("two_level_512", ix_fixtures::wide_line_two_level(512)),
        ("many_trees_2000", ix_fixtures::many_inline_trees(2000)),
        ("deep_chain_32", ix_fixtures::deep_chain(32)),
        ("deep_chain_96", ix_fixtures::deep_chain(96)),
        ("inline_doc_50k", {
            // Same generator as the harness bench_ab `inline_from_str`
            // scenario (verbatim here: benches/fixtures.rs has no copy).
            let mut out = String::with_capacity(51_200);
            let mut i = 0u32;
            while out.len() < 50_000 {
                let _ = writeln!(
                    out,
                    "k{i}: {{a: {i}, b: text item {i}, c: [{i}, {}, {i}], d:: raw {i}, e: {{deep: {}.5}}}}",
                    i + 1,
                    i % 10
                );
                i += 1;
            }
            out
        }),
        ("synth_50k", ix_fixtures::medium_50k()),
    ];

    for (name, doc) in &shapes {
        let doc_bytes = doc.len();
        // Owned path (`parse`).
        crate::parser::inline::ix_probe::reset();
        let v = crate::parse(doc).unwrap_or_else(|e| panic!("{name}: owned parse failed: {e}"));
        let s = crate::parser::inline::ix_probe::snapshot();
        assert!(!v.as_object().expect(name).is_empty());
        ix_print_counters(name, "P", doc_bytes, &s);

        // Thin path (`parse_events`).
        crate::parser::inline::ix_probe::reset();
        let mut events = 0usize;
        crate::parse_events(doc, |_ev| {
            events += 1;
        })
        .unwrap_or_else(|e| panic!("{name}: thin parse failed: {e}"));
        let s = crate::parser::inline::ix_probe::snapshot();
        assert!(events > 0);
        ix_print_counters(name, "E", doc_bytes, &s);
    }
}

fn ix_run_parse(input: &str) -> bool {
    std::hint::black_box(crate::parse(input).is_ok())
}

fn ix_run_events(input: &str) -> bool {
    std::hint::black_box(crate::parse_events(input, |_ev| {}).is_ok())
}

/// R8-F6 wall-clock side-instrument (NOISY MACHINE — deterministic
/// counters above are the primary instrument). INSTRUMENTED: this runs
/// in the lib test binary, i.e. the cfg(test) build with ix_probe
/// counters live and the counting global allocator of src/arena_probe
/// in place — its numbers are NOT production timings; the
/// uninstrumented instrument is the a2-harness binary `bench_ix`.
/// Same scenario names as the counters test, bench_ab-style output: `SCEN <shape>:<path>
/// iters=<n>` then nine `SCEN <shape>:<path> batch=<i> nanos=<ns>`
/// lines; normalize per iteration with each run's own iters line.
/// Plus micro lines: `MIKC ...` (ns/op of `known_closer` hit/miss over
/// the wide_arr_small body's C=1024 pairs) and `MISORT ...` (ns per
/// `sort_unstable_by_key` of C elements in the all-sibling pop order,
/// which is already ascending).
///
/// #[ignore] (R9-F2): calibration loops and nine timing batches per
/// scenario must not run in the ordinary suite. Run explicitly, in a
/// single-threaded window of its own:
/// `cargo test --release --lib ix_probe_r8f6_wall_clock -- --ignored --test-threads=1 --nocapture`
#[test]
#[ignore]
fn ix_probe_r8f6_wall_clock() {
    use std::fmt::Write as _;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    let shapes: Vec<(&str, String)> = vec![
        ("wide_arr_tiny_1024", ix_fixtures::wide_line_arr_tiny(1024)),
        ("wide_arr_tiny_4096", ix_fixtures::wide_line_arr_tiny(4096)),
        (
            "wide_arr_small_1024",
            ix_fixtures::wide_line_arr_small(1024),
        ),
        ("wide_obj_tiny_1024", ix_fixtures::wide_line_obj_tiny(1024)),
        ("two_level_512", ix_fixtures::wide_line_two_level(512)),
        ("many_trees_2000", ix_fixtures::many_inline_trees(2000)),
        ("deep_chain_96", ix_fixtures::deep_chain(96)),
        ("inline_doc_50k", {
            // Same generator as the harness bench_ab `inline_from_str`
            // scenario (verbatim here: benches/fixtures.rs has no copy).
            let mut out = String::with_capacity(51_200);
            let mut i = 0u32;
            while out.len() < 50_000 {
                let _ = writeln!(
                    out,
                    "k{i}: {{a: {i}, b: text item {i}, c: [{i}, {}, {i}], d:: raw {i}, e: {{deep: {}.5}}}}",
                    i + 1,
                    i % 10
                );
                i += 1;
            }
            out
        }),
        ("synth_50k", ix_fixtures::medium_50k()),
    ];

    for (name, doc) in &shapes {
        for (path, run) in [
            ("P", ix_run_parse as fn(&str) -> bool),
            ("E", ix_run_events),
        ] {
            // 2 untimed warmups, then calibrate to one >= 40 ms batch.
            for _ in 0..2 {
                run(doc);
            }
            let mut iters: u64 = 1;
            loop {
                let t = Instant::now();
                for _ in 0..iters {
                    black_box(run(doc));
                }
                if t.elapsed() >= Duration::from_millis(40) || iters >= (1 << 22) {
                    break;
                }
                iters *= 2;
            }
            println!("SCEN {name}:{path} iters={iters}");
            for batch in 0..9u32 {
                let t = Instant::now();
                for _ in 0..iters {
                    black_box(run(doc));
                }
                println!(
                    "SCEN {name}:{path} batch={batch} nanos={}",
                    t.elapsed().as_nanos() as u64
                );
            }
        }
    }

    // Micro: known_closer over the wide_arr_small_1024 body — C=1024
    // pairs, hits on opener slices, misses on a mid-item slice.
    let line = ix_fixtures::wide_line_arr_small(1024);
    // The closer scan wants the compound body itself (leading `[`),
    // not the whole `k: [...]` line.
    let body = line.trim_end_matches('\n');
    let body = &body["k: ".len()..];
    let mut pairs = Vec::new();
    let verdict =
        crate::parser::inline::scan_inline_closer_with_bounds(body, b'[', b']', 0, S, &mut pairs);
    assert!(matches!(
        verdict,
        crate::parser::inline::InlineCloserScan::Found(_)
    ));
    assert_eq!(
        pairs.len(),
        1024,
        "wide_arr_small body must record 1024 pairs"
    );
    let bounds = InlineBounds::over(body, &pairs);
    // Children are `{a:1}` at stride 6 from offset 1.
    let hit_slice = &body[1..6];
    assert_eq!(bounds.known_closer(hit_slice), Some(4));
    let miss_slice = &body[3..8];
    assert_eq!(bounds.known_closer(miss_slice), None);
    for (label, slice, expect) in [("hit", hit_slice, 4usize), ("miss", miss_slice, 0usize)] {
        let mut iters: u64 = 1;
        loop {
            let t = Instant::now();
            for _ in 0..iters {
                black_box(bounds.known_closer(black_box(slice)));
            }
            if t.elapsed() >= Duration::from_millis(40) || iters >= (1 << 24) {
                break;
            }
            iters *= 2;
        }
        let mut best: u128 = u128::MAX;
        for _ in 0..9u32 {
            let t = Instant::now();
            for _ in 0..iters {
                black_box(bounds.known_closer(black_box(slice)));
            }
            best = best.min(t.elapsed().as_nanos());
        }
        println!("MIKC C=1024 kind={label} expect={expect} iters={iters} min_batch_nanos={best} ns_per_op={}", best as f64 / iters as f64);
    }

    // Micro: sort cost of the boundary vec in the all-sibling pop
    // order — siblings pop left-to-right, so the recorded order for
    // these shapes is already ascending; reverse brackets the worst
    // case pdqsort would face on this table.
    for (order_label, table) in [
        (
            "asc",
            (0..1024).map(|i| (i * 6, i * 6 + 4)).collect::<Vec<_>>(),
        ),
        (
            "reverse",
            (0..1024)
                .rev()
                .map(|i| (i * 6, i * 6 + 4))
                .collect::<Vec<_>>(),
        ),
    ] {
        let mut iters: u64 = 1;
        loop {
            let t = Instant::now();
            for _ in 0..iters {
                let mut t2 = table.clone();
                t2.sort_unstable_by_key(|p| p.0);
                black_box(&t2);
            }
            if t.elapsed() >= Duration::from_millis(40) || iters >= (1 << 20) {
                break;
            }
            iters *= 2;
        }
        let mut best: u128 = u128::MAX;
        for _ in 0..9u32 {
            let t = Instant::now();
            for _ in 0..iters {
                let mut t2 = table.clone();
                t2.sort_unstable_by_key(|p| p.0);
                black_box(&t2);
            }
            best = best.min(t.elapsed().as_nanos());
        }
        println!("MISORT C=1024 order={order_label} iters={iters} min_batch_nanos={best} ns_per_sort_incl_clone={}", best as f64 / iters as f64);
    }
}
/// R8-F6 direct A/B, RELABELED by the round-9 review (R9-F3): the
/// bypass forces the live-dispatch fallback, so the batch delta is
/// cached-vs-uncached PARSING — the memo's benefit — measured in the
/// instrumented lib-test binary (cfg(test) ix counters + the counting
/// global allocator). It is NOT the isolated cost of the binary
/// search, and not a comparison of index choices; that isolated,
/// uninstrumented measurement lives in the a2-harness binary
/// `bench_ix` (binary search vs monotonic cursor vs passed boundary,
/// both sides keeping the recorded boundaries). Positive controls per
/// shape: (1) the bypassed parse must produce a Value whose Debug
/// matches the memo-on parse exactly; (2) with the bypass engaged the
/// ix_probe lookup counters must be zero (a vacuous A/B would show
/// zeros anyway — this proves the switch really flipped).
///
/// #[ignore] (R9-F2): timing batches must not run in the ordinary
/// suite. Run explicitly, single-threaded:
/// `cargo test --release --lib ix_probe_r8f6_memo_lookup_ab -- --ignored --test-threads=1 --nocapture`
#[test]
#[ignore]
fn ix_probe_r8f6_memo_lookup_ab() {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::parser::inline::ix_probe;

    let shapes: Vec<(&str, String)> = vec![
        ("wide_arr_tiny_1024", ix_fixtures::wide_line_arr_tiny(1024)),
        ("wide_arr_tiny_4096", ix_fixtures::wide_line_arr_tiny(4096)),
        (
            "wide_arr_small_1024",
            ix_fixtures::wide_line_arr_small(1024),
        ),
        ("wide_obj_tiny_1024", ix_fixtures::wide_line_obj_tiny(1024)),
        ("two_level_512", ix_fixtures::wide_line_two_level(512)),
        ("many_trees_2000", ix_fixtures::many_inline_trees(2000)),
        ("deep_chain_96", ix_fixtures::deep_chain(96)),
    ];

    for (name, doc) in &shapes {
        // Positive control 1: bypassed parse is byte-identical.
        ix_probe::reset();
        let on = crate::parse(doc).unwrap_or_else(|e| panic!("{name}: memo parse failed: {e}"));
        let lookups_before_bypass = {
            let s = ix_probe::snapshot();
            s.kc_calls + s.oca_calls
        };
        let bypassed = ix_probe::set_bypass(true);
        let off = crate::parse(doc).unwrap_or_else(|e| panic!("{name}: bypass parse failed: {e}"));
        let lookup_calls_while_bypassed = {
            let s = ix_probe::snapshot();
            s.kc_calls + s.oca_calls - lookups_before_bypass
        };
        drop(bypassed);
        assert_eq!(
            format!("{on:?}"),
            format!("{off:?}"),
            "{name}: bypass changed the parse result"
        );
        // Positive control 2: the switch really rerouted every lookup
        // (delta across the bypassed parse only — the memo-on parse's
        // counts precede the baseline snapshot).
        assert_eq!(
            lookup_calls_while_bypassed, 0,
            "{name}: bypass did not engage"
        );

        for (label, bypass) in [("on", false), ("off", true)] {
            let _mode = ix_probe::set_bypass(bypass);
            for _ in 0..2 {
                black_box(crate::parse(doc).is_ok());
            }
            let mut iters: u64 = 1;
            loop {
                let t = Instant::now();
                for _ in 0..iters {
                    black_box(crate::parse(doc).is_ok());
                }
                let e = t.elapsed();
                if e >= Duration::from_millis(40) || iters >= (1 << 22) {
                    break;
                }
                iters *= 2;
            }
            for batch in 0..9u32 {
                let t = Instant::now();
                for _ in 0..iters {
                    black_box(crate::parse(doc).is_ok());
                }
                let e = t.elapsed();
                println!(
                    "AB {name} memo={label} iters={iters} batch={batch} nanos={}",
                    e.as_nanos() as u64
                );
            }
        }
    }
}
