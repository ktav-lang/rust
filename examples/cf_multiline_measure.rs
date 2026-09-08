//! Counting-only measurement for the MULTI-LINE float classification path
//! (`classify_value_start` in `src/parser/classify.rs`), complementing
//! `a3_measure.rs`, whose `float_heavy` fixture exercises the INLINE path
//! (`classify_inline_scalar`) via `: [...]` values. Plain `key: <float>`
//! lines are the shape measured here.
//!
//! Run: `cargo run --release --example cf_multiline_measure`
//!
//! Prints TSV lines to stdout, four per scenario (ALLOCS / ZEROED /
//! REALLOCS / DEALLOCS with per-op counts and requested bytes). The
//! counting global allocator forwards ALL FOUR `GlobalAlloc` operations
//! to `System` unchanged (review R7-F4); realloc bytes are the requested
//! NEW size. A `Vec` doubling ten times is ONE allocation plus NINE
//! reallocations. Allocation counts are deterministic; timings are not
//! taken here (`a3_timing.rs` is the timing instrument).

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ktav::Value;

// ---------------------------------------------------------------------------
// Counting global allocator (phase-gated) — same corrected shape as
// examples/a3_measure.rs.
// ---------------------------------------------------------------------------

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static ZEROED_COUNT: AtomicUsize = AtomicUsize::new(0);
static ZEROED_BYTES: AtomicUsize = AtomicUsize::new(0);
static REALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static REALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        // SAFETY: standard forwarding wrapper — we validate nothing and
        // defer entirely to `System`'s own safety guarantees.
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ZEROED_COUNT.fetch_add(1, Ordering::Relaxed);
            ZEROED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        // SAFETY: standard forwarding wrapper — we validate nothing and
        // defer entirely to `System`'s own safety guarantees.
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            REALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            REALLOC_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
        // SAFETY: the ptr/layout pair is the caller-guaranteed grown
        // allocation per the GlobalAlloc contract; we validate nothing and
        // defer entirely to `System`'s own safety guarantees.
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING.load(Ordering::Relaxed) {
            DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            DEALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        // SAFETY: ptr/layout pair came from the caller of GlobalAlloc,
        // whose contract guarantees they are valid for System::dealloc.
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Per-operation allocation counters for one counting window.
#[derive(Clone, Copy, Default)]
struct AllocStats {
    alloc_count: usize,
    alloc_bytes: usize,
    zeroed_count: usize,
    zeroed_bytes: usize,
    realloc_count: usize,
    realloc_bytes: usize,
    dealloc_count: usize,
    dealloc_bytes: usize,
}

/// Resets all counters, runs `f` exactly `iters` times with counting
/// enabled, returning the per-operation stats for the whole run.
fn counted<F: FnMut()>(iters: usize, mut f: F) -> AllocStats {
    COUNTING.store(false, Ordering::Relaxed);
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    ZEROED_COUNT.store(0, Ordering::Relaxed);
    ZEROED_BYTES.store(0, Ordering::Relaxed);
    REALLOC_COUNT.store(0, Ordering::Relaxed);
    REALLOC_BYTES.store(0, Ordering::Relaxed);
    DEALLOC_COUNT.store(0, Ordering::Relaxed);
    DEALLOC_BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    for _ in 0..iters {
        f();
    }
    COUNTING.store(false, Ordering::Relaxed);
    AllocStats {
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
// Documents
// ---------------------------------------------------------------------------

/// Multi-line float-heavy document: plain `key: <float>` lines (no `: [...]`
/// inline compounds), so every value reaches `classify_value_start`.
/// Three magnitudes per record, mirroring `a3_measure`'s `float_heavy`:
/// - decimal region (0.01 <= |v| < 1e7): canonical_float returns the ryu form via `s.to_string()` — one String allocation on the lax path;
/// - >= 1e7: canonical_float takes the scientific path (format! + normalise) — several allocations on the lax path;
/// - < 1e-2: same scientific path.
fn float_multiline_mixed() -> String {
    let records = 1_100;
    let mut s = String::with_capacity(records * 3 * 40);
    for i in 0..records {
        match i % 3 {
            0 => {
                s.push_str("d_");
                s.push_str(&i.to_string());
                s.push_str(": ");
                s.push_str(&(i % 100).to_string());
                s.push('.');
                s.push_str(&format!("{:03}", i % 1000));
                s.push('\n');
            }
            1 => {
                s.push_str("big_");
                s.push_str(&i.to_string());
                s.push_str(": ");
                s.push_str(&(10_000_000 + i).to_string());
                s.push_str(".5\n");
            }
            _ => {
                s.push_str("tiny_");
                s.push_str(&i.to_string());
                s.push_str(": 0.00");
                s.push_str(&(i % 10).to_string());
                s.push('\n');
            }
        }
    }
    s
}

/// Multi-line float document whose every literal is already in the ryu
/// decimal form (all in (0.01, 1e7), no trailing zeros): the lax branch
/// takes the `canonical == trimmed` fast return, strict still computes
/// `canonical_float` for the comparison.
fn float_multiline_canonical() -> String {
    let records = 1_100;
    let mut s = String::with_capacity(records * 40);
    for i in 0..records {
        let whole = i % 9_000_000 + 1;
        s.push_str("c_");
        s.push_str(&i.to_string());
        s.push_str(": ");
        s.push_str(&whole.to_string());
        s.push_str(".125\n");
    }
    s
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

fn validate_float_multiline(text: &str) {
    let root: Value = match ktav::parse(text) {
        Ok(v) => v,
        Err(e) => panic!("float_multiline fixture failed to parse: {e}"),
    };
    let obj = root
        .as_object()
        .unwrap_or_else(|| panic!("float_multiline: root is not an Object"));
    // Stored lax forms are the RYU forms — never the § 5.9.8 scientific
    // rendering (that comparison happens in strict mode only).
    for (key, want) in [
        ("d_0", "0.0"),          // "0.000" → ryu zero form
        ("d_3", "3.003"),        // already canonical decimal
        ("big_1", "10000001.5"), // ryu form; strict canonical would be "1.00000015e7"
        ("tiny_2", "0.002"),     // ryu form; strict canonical would be "2e-3"
    ] {
        let v = obj
            .get(key)
            .unwrap_or_else(|| panic!("float_multiline: missing key '{key}'"));
        assert_eq!(
            v.as_float(),
            Some(want),
            "float_multiline: '{key}' stored form drift"
        );
    }
    for key in ["big_1", "tiny_2"] {
        let v = obj.get(key).unwrap_or_else(|| panic!("missing {key}"));
        assert!(
            v.as_float().is_some(),
            "float_multiline: '{key}' is not a Float"
        );
    }
}

fn validate_canonical_doc(text: &str) {
    let root: Value = match ktav::parse(text) {
        Ok(v) => v,
        Err(e) => panic!("canonical float doc failed to parse: {e}"),
    };
    let obj = root
        .as_object()
        .unwrap_or_else(|| panic!("canonical float doc: root is not an Object"));
    let v = obj
        .get("c_0")
        .unwrap_or_else(|| panic!("canonical float doc: missing c_0"));
    assert_eq!(v.as_float(), Some("1.125"), "canonical doc c_0 drift");
    // Strict must accept it (every literal already canonical).
    if let Err(e) = ktav::parse_strict(text) {
        panic!("canonical float doc must pass strict: {e}");
    }
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

fn lax_mixed(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("lax_mixed parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn lax_canonical(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("lax_canonical parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn strict_canonical(input: &str) -> usize {
    let v =
        ktav::parse_strict(input).unwrap_or_else(|e| panic!("strict_canonical parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn measure(scenario: &str, input: &str, f: fn(&str) -> usize, iters: usize) {
    let s = counted(iters, || {
        black_box(f(input));
    });
    println!(
        "ALLOCS\t{scenario}\t{}\t{}",
        s.alloc_count / iters,
        s.alloc_bytes / iters
    );
    println!(
        "ZEROED\t{scenario}\t{}\t{}",
        s.zeroed_count / iters,
        s.zeroed_bytes / iters
    );
    println!(
        "REALLOCS\t{scenario}\t{}\t{}",
        s.realloc_count / iters,
        s.realloc_bytes / iters
    );
    println!(
        "DEALLOCS\t{scenario}\t{}\t{}",
        s.dealloc_count / iters,
        s.dealloc_bytes / iters
    );
}

fn main() {
    let mixed = float_multiline_mixed();
    let canonical = float_multiline_canonical();
    validate_float_multiline(&mixed);
    validate_canonical_doc(&canonical);

    measure("lax_float_multiline_mixed", &mixed, lax_mixed, 25);
    measure(
        "lax_float_multiline_canonical",
        &canonical,
        lax_canonical,
        25,
    );
    measure(
        "strict_float_multiline_canonical",
        &canonical,
        strict_canonical,
        25,
    );
}
