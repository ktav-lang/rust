//! Measurement-only arena instrumentation, gated behind `#[cfg(test)]`.
//!
//! Measures reserved-vs-actually-used bump-arena bytes across contrasting
//! documents, for both arena entry points (`parse_events_merged` — the
//! `from_str` path — and `parse_events_raw` — the public `parse_events`
//! path) plus the whole `from_str` allocation cost via a counting global
//! allocator (a wrapper that forwards ALL FOUR `GlobalAlloc` operations to
//! `System` unchanged, with counters that separate alloc / zeroed /
//! realloc / dealloc). Run with:
//! `cargo test arena_probe -- --ignored --test-threads=1 --nocapture`
//!
//! `used` = cumulative reserved bytes minus the free tail of the current
//! chunk (found by probe-filling 1-byte allocations until a new chunk is
//! created). Bytes abandoned in retired chunks count as used — bumpalo
//! never frees them, so they are unrecoverable footprint until arena drop.
//!
//! The tests are `#[ignore]`d because the counting global allocator is
//! process-wide for the lib test binary: the counters are only meaningful
//! in a deterministic single-threaded measurement window, and the normal
//! suite must not depend on (or pay for reading) them.
//!
//! # Measured verdict (2026-09-08, worktree of aeab4c8, bumpalo 3.14)
//!
//! - `Bump::with_capacity(n)` really reserves `round_up_page(n + 64) - 64`
//!   (bumpalo 3.14 `new_chunk_memory_details`), so the `6N + 4096` policy
//!   lands on whole 4 KiB pages (`reserved_init` is that value after
//!   bumpalo's page rounding).
//! - Dot-free dense documents (short fields): used/reserved_policy
//!   ≈ 1.01 — the reservation is well-matched, nothing grows. Dense
//!   dotted-key documents overshoot it modestly (1.27x / 1.41x at 50K /
//!   500K, one graceful chunk growth) — staging + per-path key tables
//!   are the extra demand.
//! - Event-sparse documents (comments-only, one long string, early error):
//!   the events vec's unused capacity is ~all of the claim (payload
//!   ≈ 0.03%) — the review's A1 premise, confirmed as a TRANSIENT
//!   over-claim of ~6N bytes, freed when the parse call returns.
//! - Reopen-merge path (from_str on dotted-key docs): per-compound
//!   64-item buffers + key tables (thin/merge.rs) push `used` to ~5.4x
//!   reserved_init → 2 graceful chunk growths. One N-proportional
//!   constant is too big for sparse and too small for merge docs.
//! - A bounded initial reserve with growth (BumpVec doubling) would
//!   ABANDON each old buffer unrecoverably in the arena, raising the
//!   total claim on small/medium dense docs — so the current
//!   policy stands unchanged; this module is the evidence.

#[path = "../benches/fixtures.rs"]
mod fixtures;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use bumpalo::Bump;

// ---------------------------------------------------------------------------
// Counting global allocator
// ---------------------------------------------------------------------------

/// Counting wrapper: every operation forwards to the matching `System`
/// method unchanged (realloc = the platform HeapReAlloc path on Windows for
/// ordinary alignments). Per-op counters: alloc = fresh uninitialized
/// allocations; zeroed = fresh zero-filled allocations, separate from
/// alloc; realloc = in-place-or-move grow/shrink, bytes = requested NEW
/// size; dealloc = releases, bytes = the released layout. A `Vec` doubling
/// ten times is 1 alloc + 9 reallocs; all bytes are requested layout bytes.
struct CountingAlloc;

static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static REALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static ZEROED_COUNT: AtomicU64 = AtomicU64::new(0);
static ZEROED_BYTES: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Relaxed is fine: the probe test only reads counts in a
        // single-threaded window, no cross-thread ordering needed.
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Relaxed is fine: the probe test only reads counts in a
        // single-threaded window, no cross-thread ordering needed.
        ZEROED_COUNT.fetch_add(1, Ordering::Relaxed);
        ZEROED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        // SAFETY: standard forwarding wrapper — we validate nothing and
        // defer entirely to `System`'s own safety guarantees.
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Relaxed is fine: the probe test only reads counts in a
        // single-threaded window, no cross-thread ordering needed.
        REALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        REALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        // SAFETY: the ptr/layout pair is the caller-guaranteed grown
        // allocation per the GlobalAlloc contract; we validate nothing and
        // defer entirely to `System`'s own safety guarantees.
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

/// All eight global-allocator counters at one instant.
#[derive(Clone, Copy, Default)]
struct Snapshot {
    alloc_count: u64,
    alloc_bytes: u64,
    zeroed_count: u64,
    zeroed_bytes: u64,
    realloc_count: u64,
    realloc_bytes: u64,
    dealloc_count: u64,
    dealloc_bytes: u64,
}

fn alloc_snapshot() -> Snapshot {
    Snapshot {
        alloc_count: ALLOC_COUNT.load(Ordering::Relaxed),
        alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        zeroed_count: ZEROED_COUNT.load(Ordering::Relaxed),
        zeroed_bytes: ZEROED_BYTES.load(Ordering::Relaxed),
        realloc_count: REALLOC_COUNT.load(Ordering::Relaxed),
        realloc_bytes: REALLOC_BYTES.load(Ordering::Relaxed),
        dealloc_count: DEALLOC_COUNT.load(Ordering::Relaxed),
        dealloc_bytes: DEALLOC_BYTES.load(Ordering::Relaxed),
    }
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

/// One document's arena numbers (identical schema for both arena windows).
#[derive(Debug, PartialEq, Clone)]
struct Row {
    name: &'static str,
    doc_len: usize,
    reserved_policy: usize,
    reserved_init: usize,
    used: usize,
    events: Option<usize>,
    /// Events-vec claim: len*24 / cap*24, captured from the stream before
    /// it drops. None on parse error (stream is gone).
    events_len_b: Option<usize>,
    events_cap_b: Option<usize>,
    reopen_count: Option<usize>,
    chunks_after: usize,
    grew_chunks: usize,
}

/// Whole-`from_str` allocation cost for one document.
#[derive(Debug, PartialEq, Clone)]
struct AllocRow {
    name: &'static str,
    ok: bool,
    alloc_count: u64,
    alloc_bytes: u64,
    realloc_count: u64,
    realloc_bytes: u64,
    zeroed_count: u64,
    zeroed_bytes: u64,
    dealloc_count: u64,
    dealloc_bytes: u64,
}

fn chunk_count(bump: &mut Bump) -> usize {
    bump.iter_allocated_chunks().count()
}

/// Probe-fill the current chunk with 1-byte allocations until a new chunk
/// appears; free tail = probes that fit. Requires `&mut` for both the probe
/// layout alloc and the `iter_allocated_chunks` counter readback.
fn probe_used(bump: &mut Bump, reserved_post: usize) -> usize {
    let one = Layout::from_size_align(1, 1).unwrap();
    let mut k = 0usize;
    for _ in 0..reserved_post {
        let before = bump.allocated_bytes();
        bump.alloc_layout(one);
        if bump.allocated_bytes() != before {
            break;
        }
        k += 1;
    }
    reserved_post - k
}

/// Measure one document against one arena entry point (`parse_events_raw`
/// or `parse_events_merged`) using the crate's `Bump::with_capacity` policy.
fn measure_arena(name: &'static str, doc: &str, raw: bool) -> Row {
    let reserved_policy = (doc.len() / 4).saturating_mul(24) + 4096;
    let mut bump = Bump::with_capacity(reserved_policy);
    let reserved_init = bump.allocated_bytes();
    let chunks_before = chunk_count(&mut bump);
    let (events, events_len_b, events_cap_b, reopen_count) = if raw {
        match crate::thin::parse_events_raw(doc, &bump) {
            Ok((stream, reopens)) => (
                Some(stream.len()),
                Some(stream.len() * 24),
                Some(stream.capacity() * 24),
                Some(reopens),
            ),
            Err(_) => (None, None, None, None),
        }
    } else {
        match crate::thin::parse_events_merged(doc, &bump) {
            Ok(stream) => (
                Some(stream.len()),
                Some(stream.len() * 24),
                Some(stream.capacity() * 24),
                None,
            ),
            Err(_) => (None, None, None, None),
        }
    };
    let reserved_post = bump.allocated_bytes();
    let chunks_after = chunk_count(&mut bump);
    let used = probe_used(&mut bump, reserved_post);
    Row {
        name,
        doc_len: doc.len(),
        reserved_policy,
        reserved_init,
        used,
        events,
        events_len_b,
        events_cap_b,
        reopen_count,
        chunks_after,
        grew_chunks: chunks_after - chunks_before,
    }
}

/// Measure the whole `from_str` allocation cost (doc built first,
/// outside the snapshot window, so fixture construction isn't counted).
/// `ktav::Value` is not `DeserializeOwned`, so a fully dynamic
/// `serde_json::Value` target (dev-dependency) drives the full path.
fn measure_from_str(name: &'static str, doc: &str) -> AllocRow {
    let s0 = alloc_snapshot();
    let res = crate::from_str::<serde_json::Value>(doc);
    let s1 = alloc_snapshot();
    AllocRow {
        name,
        ok: res.is_ok(),
        alloc_count: s1.alloc_count - s0.alloc_count,
        alloc_bytes: s1.alloc_bytes - s0.alloc_bytes,
        realloc_count: s1.realloc_count - s0.realloc_count,
        realloc_bytes: s1.realloc_bytes - s0.realloc_bytes,
        zeroed_count: s1.zeroed_count - s0.zeroed_count,
        zeroed_bytes: s1.zeroed_bytes - s0.zeroed_bytes,
        dealloc_count: s1.dealloc_count - s0.dealloc_count,
        dealloc_bytes: s1.dealloc_bytes - s0.dealloc_bytes,
    }
}

/// The 11 deterministic documents under test.
fn documents() -> Vec<(&'static str, String)> {
    let mut inline_50k = String::new();
    let mut i = 0usize;
    while inline_50k.len() < 50_000 {
        inline_50k.push_str(&format!("a{i}: {{x: {i}, y: {i}, z: {i}}}\n"));
        i += 1;
    }
    let mut comment = String::new();
    let mut i = 0usize;
    while comment.len() < 50_000 {
        comment.push_str(&format!("## comment line {i} padding padding\n"));
        i += 1;
    }
    let comments_only_50k = comment.clone();
    let comment_heavy_50k = format!("{comment}final: 1\n");
    vec![
        ("small_1k", fixtures::small_1k()),
        ("medium_50k", fixtures::medium_50k()),
        ("large_500k", fixtures::large_500k()),
        (
            "short_fields_2k",
            (0..2000).map(|i| format!("k{i}: 1\n")).collect(),
        ),
        ("inline_50k", inline_50k),
        (
            "one_long_string_50k",
            format!("big: {}\n", "x".repeat(49_000)),
        ),
        ("comment_heavy_50k", comment_heavy_50k),
        ("comments_only_50k", comments_only_50k),
        ("empty", String::new()),
        (
            "err_early_50k",
            format!("key:value\n{}", fixtures::medium_50k()),
        ),
        (
            "err_mid_50k",
            fixtures::inject_bad_line(&fixtures::medium_50k()).0,
        ),
    ]
}

fn measure_all() -> (Vec<Row>, Vec<Row>, Vec<AllocRow>) {
    let mut merged = Vec::new();
    let mut raw = Vec::new();
    let mut alloc = Vec::new();
    for (name, doc) in documents() {
        merged.push(measure_arena(name, &doc, false));
        raw.push(measure_arena(name, &doc, true));
        alloc.push(measure_from_str(name, &doc));
    }
    (merged, raw, alloc)
}

fn print_table_1(title: &str, rows: &[Row]) {
    // policy = (N/4)*24 + 4096; reserved_init is bumpalo's page-rounded request.
    println!("{title}");
    println!(
        "{:<20} {:>8} {:>13} {:>12} {:>8} {:>8} {:>9} {:>9} {:>9} {:>9} {:>7} {:>12} {:>11}",
        "name",
        "N bytes",
        "reserved_init",
        "used",
        "used/res",
        "events",
        "ev_len_b",
        "ev_cap_b",
        "payload%",
        "other_b",
        "reopens",
        "chunks_after",
        "grew_chunks"
    );
    for r in rows {
        let ratio = r.used as f64 / r.reserved_policy as f64;
        // other_b on the merged window includes the abandoned raw stream
        // plus merge.rs's per-compound item buffers/key tables; on the raw
        // window it's non-event spill only.
        let (ev_len, ev_cap, payload, other) = match (r.events_len_b, r.events_cap_b) {
            (Some(l), Some(c)) => {
                let pct = if c == 0 {
                    0.0
                } else {
                    l as f64 / c as f64 * 100.0
                };
                (
                    l.to_string(),
                    c.to_string(),
                    format!("{pct:.2}"),
                    (r.used - c).to_string(),
                )
            }
            _ => ("-".into(), "-".into(), "-".into(), "-".into()),
        };
        println!(
            "{:<20} {:>8} {:>13} {:>12} {:>8.2} {:>8} {:>9} {:>9} {:>9} {:>9} {:>7} {:>12} {:>11}",
            r.name,
            r.doc_len,
            r.reserved_init,
            r.used,
            ratio,
            r.events
                .map(|e| e.to_string())
                .unwrap_or_else(|| "-".into()),
            ev_len,
            ev_cap,
            payload,
            other,
            r.reopen_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
            r.chunks_after,
            r.grew_chunks,
        );
    }
    println!();
}

fn print_table_2(rows: &[AllocRow]) {
    println!("Table 2: whole from_str cost (per-operation allocation-counting global allocator)");
    println!(
        "alloc = fresh `alloc` calls; realloc = `realloc` calls (bytes = requested new size); \
         zeroed = fresh `alloc_zeroed` calls; dealloc = releases; bytes are requested layout bytes"
    );
    println!(
        "{:<20} {:>6} {:>11} {:>11} {:>13} {:>13} {:>12} {:>12} {:>13} {:>13}",
        "name",
        "result",
        "alloc_count",
        "alloc_bytes",
        "realloc_count",
        "realloc_bytes",
        "zeroed_count",
        "zeroed_bytes",
        "dealloc_count",
        "dealloc_bytes"
    );
    for r in rows {
        println!(
            "{:<20} {:>6} {:>11} {:>11} {:>13} {:>13} {:>12} {:>12} {:>13} {:>13}",
            r.name,
            if r.ok { "ok" } else { "err" },
            r.alloc_count,
            r.alloc_bytes,
            r.realloc_count,
            r.realloc_bytes,
            r.zeroed_count,
            r.zeroed_bytes,
            r.dealloc_count,
            r.dealloc_bytes,
        );
    }
    println!();
}

#[test]
#[ignore]
fn arena_probe_reserved_vs_used_report() {
    // Run the whole measurement twice and require every number to match
    // exactly — a determinism self-check against accidental dependence
    // on allocator state, hash iteration, or environment.
    let run1 = measure_all();
    let run2 = measure_all();
    let (m1, r1, a1) = &run1;
    let (m2, r2, a2) = &run2;
    assert_eq!(m1, m2, "merged-path rows differ between runs");
    assert_eq!(r1, r2, "raw-path rows differ between runs");
    assert_eq!(a1, a2, "from_str alloc rows differ between runs");
    // Error docs must actually error; valid docs must actually parse.
    for row in a1 {
        let expect_ok = !row.name.starts_with("err_");
        assert_eq!(
            row.ok, expect_ok,
            "unexpected from_str result for {}",
            row.name
        );
    }
    print_table_1(
        "Table 1a: Window A — parse_events_merged (from_str arena path)",
        m1,
    );
    print_table_1(
        "Table 1b: Window B — parse_events_raw (public parse_events arena path)",
        r1,
    );
    print_table_2(a1);
}
