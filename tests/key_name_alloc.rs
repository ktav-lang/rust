//! R16-F2 regression gate: the shared map key-name policy
//! (`serialize_key_name`) writes the name directly into the inline-capable
//! `Scalar`, with no intermediate heap `String`. Counts real heap
//! allocations through a forwarding global allocator (the
//! `src/arena_probe.rs` pattern); a reintroduced temporary buffer shows up
//! as exactly one extra allocation for a short name.
//!
//! Exactly one `#[test]` fn: the allocator counters are process-wide, and
//! the default multi-thread harness would interleave other tests'
//! allocations into the measured windows.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::ser::{Serialize, SerializeMap, Serializer};

use ktav::value::Value;
use ktav::{ser, to_string};

/// 47 bytes: beyond the 24-byte inline capacity of `Scalar` (compact_str
/// 0.9.0, 64-bit), so the name spills to heap while staying a legal bare key.
const LONG_KEY: &str = "a_very_long_map_key_name_beyond_inline_capacity";

struct OneEntry<K>(K);

impl<K: Serialize> Serialize for OneEntry<K> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(Some(1))?;
        m.serialize_entry(&self.0, "v")?;
        m.end()
    }
}

#[derive(serde::Serialize)]
struct SameField {
    port: &'static str,
}

struct CountingAlloc;

static ALLOCS: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

fn counted<T>(f: impl FnOnce() -> T) -> (u64, T) {
    let before = ALLOCS.load(Ordering::Relaxed);
    let out = f();
    (ALLOCS.load(Ordering::Relaxed) - before, out)
}

#[test]
fn key_name_writes_directly_into_scalar() {
    // Direct text writer, short name: the only allocation is the output
    // buffer (`String::with_capacity(2048)` in `to_string`) — the key
    // name lives inline in the Scalar.
    let (n, text) = counted(|| to_string(&OneEntry("port")).unwrap());
    assert_eq!(n, 1, "short name took {n} allocations, output: {text:?}");
    assert_eq!(text, "port: v\n");

    // Owned path, differential oracle: struct fields insert the
    // `&'static str` name directly (src/ser/struct_serializer.rs), a map
    // with the same short key must allocate the same amount (the entry
    // insert dominates both windows) and produce an identical Value.
    let (struct_n, struct_v) = counted(|| ser::to_value(&SameField { port: "v" }).unwrap());
    let (map_n, map_v) = counted(|| ser::to_value(&OneEntry("port")).unwrap());
    assert_eq!(struct_v, map_v);
    assert_eq!(
        map_n, struct_n,
        "map path allocated more than the struct path for the same short key name"
    );

    // Long name (> inline capacity): the heap branch stays behaviorally
    // identical on both writer paths. No exact-count assertion — the
    // spill is compact_str's internal policy and did not change here.
    let (map_long_n, map_long_v) = counted(|| ser::to_value(&OneEntry(LONG_KEY)).unwrap());
    match map_long_v {
        Value::Object(m) => {
            assert_eq!(m.len(), 1);
            assert_eq!(m.keys().next().unwrap().as_str(), LONG_KEY);
        }
        other => panic!("expected object, got {other:?}"),
    }
    let (text_long_n, text_long) = counted(|| to_string(&OneEntry(LONG_KEY)).unwrap());
    assert_eq!(text_long, format!("{LONG_KEY}: v\n"));
    // Smoke-only sanity that both long windows actually ran through the
    // counting allocator; values above carry the real assertions.
    assert!(map_long_n > 0 && text_long_n > 0);
}
