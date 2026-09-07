//! Release-binary measurement harness for the owned-vs-thin allocation
//! experiment (a3). Self-contained: no criterion, no test harness.
//!
//! Run: `cargo run --release --example a3_measure [-- --quick]`
//!
//! Prints TSV lines to stdout:
//! - `TIMING\t<scenario>\t<batch>\t<ns_per_iter>` per measured batch
//! - `SUMMARY\t<scenario>\t<min>\t<median>` after all batches
//! - `ALLOCS\t<scenario>\t<allocs_per_iter>\t<bytes_per_iter>` after all timing

#[path = "../benches/fixtures.rs"]
mod fixtures;

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use ktav::Value;

// ---------------------------------------------------------------------------
// Counting global allocator (phase-gated)
// ---------------------------------------------------------------------------

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: ptr/layout pair came from the caller of GlobalAlloc,
        // whose contract guarantees they are valid for System::dealloc.
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Runs `f` exactly `iters` times with allocation counting enabled,
/// returning `(alloc_count, alloc_bytes)` for the whole run.
fn counted<F: FnMut()>(iters: usize, mut f: F) -> (usize, usize) {
    COUNTING.store(false, Ordering::Relaxed);
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    for _ in 0..iters {
        f();
    }
    COUNTING.store(false, Ordering::Relaxed);
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

// ---------------------------------------------------------------------------
// Custom deterministic documents (index-driven, no randomness)
// ---------------------------------------------------------------------------

/// ~50 KB of records exercising escapes, quoted keys and dotted keys.
fn key_heavy() -> String {
    // 7 lines per record, each ~150 bytes average.
    let records = 52;
    let mut s = String::with_capacity(records * 7 * 160);
    for i in 0..records {
        s.push_str("plain_");
        s.push_str(&i.to_string());
        s.push_str(": value_");
        s.push_str(&i.to_string());
        s.push('\n');

        s.push_str("service.");
        s.push_str(&i.to_string());
        s.push_str(".host: 10.0.0.");
        s.push_str(&(i % 256).to_string());
        s.push('\n');

        s.push_str("\"quoted key ");
        s.push_str(&i.to_string());
        s.push_str("\": literal value ");
        s.push_str(&i.to_string());
        s.push('\n');

        s.push_str("dot\\.ted_");
        s.push_str(&i.to_string());
        s.push_str(": ");
        s.push_str(&i.to_string());
        s.push('\n');

        s.push_str("esc_");
        s.push_str(&i.to_string());
        // Inline-array value: this parser version decodes \uXXXX escapes
        // only inside inline compounds — a plain top-level scalar keeps
        // them literal — so the escape-forces-String rule is exercised
        // via `[...]`. Decodes to Array([String("true")]).
        s.push_str(": [\\u0074rue]\n");

        s.push_str("\"q\\:colon_");
        s.push_str(&i.to_string());
        s.push_str("\": ");
        s.push_str(&i.to_string());
        s.push('\n');

        // 7th line: mixed-type inline array — every element goes through
        // classify_inline_scalar (string, integer, bool, null, float).
        s.push_str("mix_");
        s.push_str(&i.to_string());
        s.push_str(": [t");
        s.push_str(&i.to_string());
        s.push_str(", ");
        s.push_str(&i.to_string());
        s.push_str(", true, null, 1.5]\n");
    }
    s
}

/// ~50 KB of records with float literals in varied magnitudes.
fn float_heavy() -> String {
    // 3 lines per record, ~45 bytes average.
    let records = 1_100;
    let mut s = String::with_capacity(records * 3 * 50);
    for i in 0..records {
        match i % 3 {
            0 => {
                s.push_str("r_");
                s.push_str(&i.to_string());
                // Single-line inline array so the value goes through
                // parse_inline_value_raw / classify_inline_scalar (where
                // canonical_float runs), not classify_value_start.
                s.push_str(": [");
                s.push_str(&(i % 100).to_string());
                s.push('.');
                s.push_str(&format!("{:03}", i % 1000));
                s.push_str("]\n");
            }
            1 => {
                s.push_str("big_");
                s.push_str(&i.to_string());
                s.push_str(": [");
                s.push_str(&(10_000_000 + i).to_string());
                s.push_str(".5]\n");
            }
            _ => {
                s.push_str("tiny_");
                s.push_str(&i.to_string());
                s.push_str(": [0.00");
                s.push_str(&(i % 10).to_string());
                s.push_str("]\n");
            }
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

fn validate_key_heavy(text: &str) {
    let root: Value = match ktav::parse(text) {
        Ok(v) => v,
        Err(e) => panic!("key_heavy fixture failed to parse: {e}"),
    };
    let obj = root
        .as_object()
        .unwrap_or_else(|| panic!("key_heavy: root is not an Object"));
    let dot = obj
        .get("dot.ted_7")
        .unwrap_or_else(|| panic!("key_heavy: missing key 'dot.ted_7' (escaped dot not honored?)"));
    assert_eq!(dot.as_integer(), Some("7"), "key_heavy: dot.ted_7 drift");
    let esc = obj
        .get("esc_7")
        .unwrap_or_else(|| panic!("key_heavy: missing key 'esc_7'"));
    let esc_items = esc
        .as_array()
        .unwrap_or_else(|| panic!("key_heavy: esc_7 is not an Array — generator shape drift"));
    assert_eq!(
        esc_items.first().and_then(Value::as_str),
        Some("true"),
        "key_heavy: \\u0074rue did not decode to String \"true\""
    );
    assert!(
        esc_items.first().and_then(Value::as_bool).is_none(),
        "key_heavy: esc_7 inferred as Bool — escape-forces-String violated"
    );
    let quoted = obj
        .get("quoted key 7")
        .unwrap_or_else(|| panic!("key_heavy: missing key 'quoted key 7'"));
    assert_eq!(
        quoted.as_str(),
        Some("literal value 7"),
        "key_heavy: quoted key 7 drift"
    );
    let service = obj
        .get("service")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("key_heavy: 'service' is not an Object"));
    let svc7 = service
        .get("7")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("key_heavy: service.7 is not an Object"));
    assert!(
        svc7.get("host").is_some(),
        "key_heavy: service.7.host missing"
    );
    let mix = obj
        .get("mix_7")
        .unwrap_or_else(|| panic!("key_heavy: missing key 'mix_7'"));
    let mix_arr = mix
        .as_array()
        .unwrap_or_else(|| panic!("key_heavy: mix_7 is not an Array"));
    assert_eq!(mix_arr.len(), 5, "key_heavy: mix_7 should have 5 elements");
    assert_eq!(mix_arr[0].as_str(), Some("t7"), "key_heavy: mix_7[0] drift");
    assert_eq!(
        mix_arr[1].as_integer(),
        Some("7"),
        "key_heavy: mix_7[1] drift"
    );
    assert_eq!(
        mix_arr[2].as_bool(),
        Some(true),
        "key_heavy: mix_7[2] drift"
    );
    assert!(mix_arr[3].is_null(), "key_heavy: mix_7[3] drift");
    assert_eq!(
        mix_arr[4].as_float(),
        Some("1.5"),
        "key_heavy: mix_7[4] drift"
    );
}

fn validate_float_heavy(text: &str) {
    let root: Value = match ktav::parse(text) {
        Ok(v) => v,
        Err(e) => panic!("float_heavy fixture failed to parse (float literal shape wrong?): {e}"),
    };
    let obj = root
        .as_object()
        .unwrap_or_else(|| panic!("float_heavy: root is not an Object"));
    for key in ["r_0", "big_1", "tiny_2"] {
        let v = obj
            .get(key)
            .unwrap_or_else(|| panic!("float_heavy: missing key '{key}'"));
        let arr = v.as_array().unwrap_or_else(|| {
            panic!("float_heavy: '{key}' is not an Array — generator shape drift")
        });
        assert_eq!(
            arr.len(),
            1,
            "float_heavy: '{key}' should hold exactly 1 element"
        );
        assert!(
            arr.first().and_then(Value::as_float).is_some(),
            "float_heavy: '{key}[0]' is not a Float (as_float() == None) — literal shape drift"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

fn own_small_1k(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("own_small_1k parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn own_medium_50k(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("own_medium_50k parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn own_key_heavy(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("own_key_heavy parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn own_float_heavy(input: &str) -> usize {
    let v = ktav::parse(input).unwrap_or_else(|e| panic!("own_float_heavy parse error: {e}"));
    black_box(&v).as_object().map_or(0, |o| o.len())
}

fn thin_small_1k(input: &str) -> usize {
    let mut events = 0usize;
    ktav::parse_events(input, |_e| {
        events += 1;
    })
    .unwrap_or_else(|e| panic!("thin_small_1k parse error: {e}"));
    black_box(events)
}

fn thin_key_heavy(input: &str) -> usize {
    let mut events = 0usize;
    ktav::parse_events(input, |_e| {
        events += 1;
    })
    .unwrap_or_else(|e| panic!("thin_key_heavy parse error: {e}"));
    black_box(events)
}

fn thin_float_heavy(input: &str) -> usize {
    let mut events = 0usize;
    ktav::parse_events(input, |_e| {
        events += 1;
    })
    .unwrap_or_else(|e| panic!("thin_float_heavy parse error: {e}"));
    black_box(events)
}

// ---------------------------------------------------------------------------
// Measurement driver
// ---------------------------------------------------------------------------

fn ns_per_iter_batches(
    scenario: &str,
    input: &str,
    f: fn(&str) -> usize,
    batches: usize,
    min_ms: u128,
) -> Vec<f64> {
    // Warm-up: at least `min_ms`, discarded.
    let start = Instant::now();
    let mut warm = 0usize;
    while start.elapsed().as_millis() < min_ms {
        f(input);
        warm += 1;
    }
    black_box(warm);

    let mut results = Vec::with_capacity(batches);
    for batch in 0..batches {
        let start = Instant::now();
        let mut iters = 0usize;
        while start.elapsed().as_millis() < min_ms {
            f(input);
            iters += 1;
        }
        let total_ns = start.elapsed().as_nanos() as f64;
        let per_iter = total_ns / iters as f64;
        results.push(per_iter);
        println!("TIMING\t{scenario}\t{batch}\t{per_iter:.1}");
    }
    results
}

fn measure_allocs(scenario: &str, input: &str, f: fn(&str) -> usize, iters: usize) {
    let (allocs, bytes) = counted(iters, || {
        black_box(f(input));
    });
    let allocs_per_iter = allocs / iters;
    let bytes_per_iter = bytes / iters;
    println!("ALLOCS\t{scenario}\t{allocs_per_iter}\t{bytes_per_iter}");
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

type Scenario<'a> = (&'a str, &'a str, fn(&str) -> usize, usize);

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let batches = if quick { 3 } else { 7 };
    let min_ms = if quick { 100 } else { 200 };

    let small = fixtures::small_1k();
    let medium = fixtures::medium_50k();
    fixtures::validate_synth(&small);
    fixtures::validate_synth(&medium);
    let keys = key_heavy();
    let floats = float_heavy();
    validate_key_heavy(&keys);
    validate_float_heavy(&floats);

    let scenarios: &[Scenario<'_>] = &[
        (
            "own_small_1k",
            &small,
            own_small_1k as fn(&str) -> usize,
            500,
        ),
        ("own_medium_50k", &medium, own_medium_50k, 25),
        ("own_key_heavy", &keys, own_key_heavy, 25),
        ("own_float_heavy", &floats, own_float_heavy, 25),
        ("thin_small_1k", &small, thin_small_1k, 500),
        ("thin_key_heavy", &keys, thin_key_heavy, 25),
        ("thin_float_heavy", &floats, thin_float_heavy, 25),
    ];

    for (name, input, f, alloc_iters) in scenarios {
        let mut per_batch = ns_per_iter_batches(name, input, *f, batches, min_ms);
        let min = per_batch.iter().cloned().fold(f64::INFINITY, f64::min);
        let med = median(&mut per_batch);
        println!("SUMMARY\t{name}\t{min:.1}\t{med:.1}");
        measure_allocs(name, input, *f, *alloc_iters);
    }
}
