//! R17-F1 regression gate: the owned-path key adapter delivers an inline
//! owned key through `visit_str` (no heap round-trip), while a heap-backed
//! key still moves its buffer into the target via `visit_string` (R16-F3).
//! Counts real heap allocations through a forwarding global allocator (the
//! `tests/key_name_alloc.rs` pattern); that file is the writer-side gate
//! and stays untouched.
//!
//! Exactly one `#[test]` fn: the allocator counters are process-wide, and
//! the default multi-thread harness would interleave other tests'
//! allocations into the measured windows.

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt;
use std::marker::PhantomData;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::{
    Deserialize, DeserializeOwned, Deserializer, Error, IgnoredAny, MapAccess, Visitor,
};

use ktav::de::from_value;
use ktav::value::{ObjectMap, Scalar, Value};

/// 47 bytes: beyond the 24-byte inline capacity of `Scalar` (compact_str
/// 0.9.0, 64-bit), so the key spills to heap while staying a legal bare key.
const LONG_KEY: &str = "a_very_long_map_key_name_beyond_inline_capacity";

/// Pulls exactly one key out of an object without building any intermediate
/// map — the key deserializes through `T`'s own serde methods, the paired
/// value and any remaining entries go through `IgnoredAny`.
struct ExtractKey<T> {
    key: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ExtractKey<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for KeyVisitor<T> {
            type Value = ExtractKey<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an object with a key to extract")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let key = map
                    .next_key::<T>()?
                    .ok_or_else(|| Error::custom("expected at least one entry"))?;
                map.next_value::<IgnoredAny>()?;
                while map.next_key::<IgnoredAny>()?.is_some() {
                    map.next_value::<IgnoredAny>()?;
                }
                Ok(ExtractKey { key })
            }
        }

        deserializer.deserialize_map(KeyVisitor(PhantomData))
    }
}

/// Slice-only string target: consumes just the `&str` — mirrors how serde's
/// human-readable `Ipv4Addr` (FromStr visitor) or compact_str's `visit_str`
/// reads a key without needing an owned heap buffer.
struct KeyLen(usize);

impl<'de> Deserialize<'de> for KeyLen {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenVisitor;

        impl<'de> Visitor<'de> for LenVisitor {
            type Value = KeyLen;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string key")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(KeyLen(v.len()))
            }
        }

        deserializer.deserialize_str(LenVisitor)
    }
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

fn one_entry_object(key: Scalar) -> Value {
    let mut obj = ObjectMap::default();
    obj.insert(key, Value::Null);
    Value::Object(obj)
}

/// Control: the same text read as a plain scalar `Value` through
/// `ValueDeserializer` — no key adapter involved. A short scalar is inline,
/// so any allocation here would come from the target type itself.
fn scalar_control<T: DeserializeOwned>(text: &str) -> (u64, T) {
    counted(|| from_value::<T>(Value::String(text.into())).unwrap())
}

fn object_key_ptr(value: &Value) -> usize {
    match value {
        Value::Object(obj) => obj.keys().next().unwrap().as_str().as_ptr() as usize,
        other => panic!("expected object, got {other:?}"),
    }
}

#[test]
fn key_read_allocation_matrix() {
    // Warm-up, uncounted: on some platforms (observed on macOS CI) the
    // first heap allocation on a fresh thread pays for one-time runtime
    // setup (TLS registration, panic machinery, ...) that has nothing to
    // do with the code under test. Absorb it here, before any window
    // below is measured, so every count reflects only the operation it
    // names — see the identical rationale in tests/key_name_alloc.rs.
    let _ = from_value::<ExtractKey<Scalar>>(one_entry_object("port".into())).unwrap();

    // --- short inline keys: the key read must not allocate (R17-F1) ---

    // Scalar target: compact_str 0.9.0 deserializes via `deserialize_str` +
    // `visit_str`, which inlines short text with no heap buffer.
    let obj = one_entry_object("port".into());
    let (n, got) = counted(|| from_value::<ExtractKey<Scalar>>(obj).unwrap());
    assert_eq!(n, 0, "Scalar short key allocated {n} times");
    assert_eq!(got.key.as_str(), "port");
    let (c, _) = scalar_control::<Scalar>("port");
    assert_eq!(c, 0, "scalar control allocated {c} times");

    // Option<Scalar>: the adapter always yields `visit_some`, then the same
    // inline path.
    let obj = one_entry_object("port".into());
    let (n, got) = counted(|| from_value::<ExtractKey<Option<Scalar>>>(obj).unwrap());
    assert_eq!(n, 0, "Option<Scalar> short key allocated {n} times");
    assert_eq!(got.key.as_deref(), Some("port"));

    // Ipv4Addr: serde's human-readable impl is `deserialize_str` + FromStr —
    // the result never needs a string heap buffer.
    let obj = one_entry_object("127.0.0.1".into());
    let (n, got) = counted(|| from_value::<ExtractKey<Ipv4Addr>>(obj).unwrap());
    assert_eq!(n, 0, "Ipv4Addr short key allocated {n} times");
    assert_eq!(got.key, Ipv4Addr::new(127, 0, 0, 1));
    let (c, _) = scalar_control::<Ipv4Addr>("127.0.0.1");
    assert_eq!(c, 0, "Ipv4Addr control allocated {c} times");

    // Slice-only visitor: nothing may be copied out of the key.
    let obj = one_entry_object("port".into());
    let (n, got) = counted(|| from_value::<ExtractKey<KeyLen>>(obj).unwrap());
    assert_eq!(n, 0, "slice-only visitor short key allocated {n} times");
    assert_eq!(got.key.0, 4);
    let (c, _) = scalar_control::<KeyLen>("port");
    assert_eq!(c, 0, "slice-only visitor control allocated {c} times");

    // String target: exactly one allocation — the target's own buffer,
    // matching the control; the adapter must add no temporary on top.
    let obj = one_entry_object("port".into());
    let (n, got) = counted(|| from_value::<ExtractKey<String>>(obj).unwrap());
    assert_eq!(n, 1, "String short key allocated {n} times");
    assert_eq!(got.key, "port");
    let (c, _) = scalar_control::<String>("port");
    assert_eq!(c, 1, "String control allocated {c} times");

    // u32 target: numeric methods parse the borrowed slice (unchanged).
    let obj = one_entry_object("42".into());
    let (n, got) = counted(|| from_value::<ExtractKey<u32>>(obj).unwrap());
    assert_eq!(n, 0, "u32 short key allocated {n} times");
    assert_eq!(got.key, 42);

    // --- long heap-backed keys: the buffer moves, zero allocation (R16-F3,
    // must not regress to an unconditional `visit_str` heap-copy) ---

    // String target receives the moved buffer.
    let source = one_entry_object(LONG_KEY.into());
    let key_ptr = object_key_ptr(&source);
    let (n, got) = counted(|| from_value::<ExtractKey<String>>(source).unwrap());
    assert_eq!(n, 0, "String long key allocated {n} times");
    assert_eq!(
        got.key.as_ptr() as usize,
        key_ptr,
        "String long key buffer must be moved, not copied"
    );
    assert_eq!(got.key.len(), 47);

    // Scalar target keeps the same zero-copy hand-off.
    let source = one_entry_object(LONG_KEY.into());
    let key_ptr = object_key_ptr(&source);
    let (n, got) = counted(|| from_value::<ExtractKey<Scalar>>(source).unwrap());
    assert_eq!(n, 0, "Scalar long key allocated {n} times");
    assert_eq!(
        got.key.as_ptr() as usize,
        key_ptr,
        "Scalar long key buffer must be moved, not copied"
    );
    assert_eq!(got.key.len(), 47);
}
