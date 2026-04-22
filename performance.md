# Performance notes — `ktav` Rust crate

This document collects every principle, library choice, and pattern used to
get the Rust implementation of Ktav to its current speed. It is written
against the state as of commit `ceb8184` (phase 6) and draws on the full
commit history from the initial import forward.

The crate was accelerated from a naïve first cut to the current numbers
across seven perf rounds. Target fixture: `parse_to_struct/100_upstreams_typed`
(22 KB of Ktav with 100 nested records). Typical representative moves:

| Round                | Snapshot impact on typed deserialize |
|----------------------|--------------------------------------|
| initial import       | baseline                             |
| phase 1              | indent, validation, borrowed multiline |
| phase 2              | `itoa` / `ryu` (serializer hot path) |
| phase 3              | `CompactString` in Value              |
| phase 4              | capacity hints, `IndexMap::entry`, fast paths |
| phase 5              | **bump-arena for `ThinValue`: −34 %** |
| phase 6              | memchr shortcut on multiline terminator |

The cumulative effect is large: deserialize time dropped from the first
internal measurement (~**~1 ms** on an early revision) to ~**207 µs** —
roughly a **5× improvement**, without a single `unsafe` block in `src/`.

The remainder of this document explains *why* each move helped, *how* it
interacts with the rest of the code, and — equally important — **which
seemingly obvious optimizations were measured and rejected**.

---

## 1. Philosophy: measure, don't guess

Every number in this doc comes from Criterion long-sample benchmarks (50
samples × 5 s measurement, 2 s warmup). The failed-experiment list at the
end of this doc is a long one: `scan_line` (unified forward scan), byte-match
classification, `Bump::with_capacity`, custom integer parsers, `lto = "fat"`
for the bench profile. Each of those *looked* like it should help; each of
them measured worse or flat.

The working rules:

1. **Land one change at a time**, bench it against the previous commit as a
   criterion baseline, decide to commit or revert on measured signal — not
   on "it should be faster because…".
2. **A uniform direction across all benchmarks is signal**; a mixed bag is
   almost always noise. Thermal drift and rebuild layout shifts routinely
   move individual benches by ±5 % between runs.
3. **Trust the standard library.** `str::trim`, `str::find`, memchr-backed
   `<[u8]>::contains`, and `str::parse::<u16>` all ship with SIMD/highly
   tuned implementations. Hand-rolled forward scans that do "the same work
   in one pass" routinely lose because LLVM can't vectorize them.
4. **Invariants beat early exits.** If the pipeline guarantees input is
   already `trim_end`'d, downstream code should not call `trim()` again.
   Thread the invariant through types/comments; save the bytes.
5. **No `unsafe`.** A stated project rule (`src/` grep is clean — the only
   `unsafe` tokens are in comments describing code that used to exist).
   Every allocation trick below is in safe Rust.

---

## 2. Memory principles

### 2.1 Arena allocation (bumpalo) — the single biggest win

The `from_str<T>` path used to allocate per-compound: every `ThinValue::Array`
and `ThinValue::Object` held a heap `Vec`, every multi-line string held a
heap `String`, every multi-line collecting buffer was another `Vec<&str>`.
For a 100-record input that worked out to ~500 heap allocations per call,
with matching frees at the end.

**Phase 5** moved the whole zero-copy path into a `bumpalo::Bump`:

```rust
// src/lib.rs
pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    let bump = bumpalo::Bump::new();
    let thin = thin::parse_thin(s, &bump)?;
    T::deserialize(thin::ThinDeserializer::new(thin))
}
```

Inside `thin/parser.rs`, `Frame::Object { pairs, .. }` and
`Frame::Array { items }` hold `bumpalo::collections::Vec<'a, _>`. Subvectors
are allocated via `BumpVec::with_capacity_in(4, bump)`. Multi-line content
that has to be re-assembled goes through `Bump::alloc_str`. When `from_str`
returns, one `Drop` on the `Bump` frees every byte in constant time.

Measured effect: **−34 % on `parse_to_struct`, −29 % on `roundtrip`,
−10 % on `multiline_dedent/*`** vs. phase 4.

**Caveat (phase 11 reject):** `Bump::with_capacity(s.len())` — pre-sizing
the arena to the input — *hurt* by 7–27 %. Lazy chunk growth amortizes
cheaply; an up-front large chunk forces the allocator to hand out a
first-class large block before anyone asks for it.

### 2.2 Zero-copy borrowing: `ThinValue<'a>`

`ThinValue` carries slices straight out of the input buffer wherever the
text form is a contiguous substring of that buffer:

```rust
// src/thin/value.rs
pub enum ThinValue<'a> {
    Null,
    Bool(bool),
    Integer(&'a str),
    Float(&'a str),
    Str(&'a str),
    Array(BumpVec<'a, ThinValue<'a>>),
    Object(BumpVec<'a, (&'a str, ThinValue<'a>)>),
}
```

Key design decisions:

* **Object keys are `&'a str`** — no `String` wrapper. Dotted paths
  (`a.b.c`) produce three slices into the same buffer via
  `str::split('.')`, still zero-allocation.
* **No `Cow`.** Before phase 5 `Integer`/`Float`/`Str` were
  `Cow<'a, str>`; phase 5 collapsed them to `&'a str`. Normalization cases
  (leading `+` strip, multi-line dedent) copy into the arena via
  `Bump::alloc_str`, and the returned slice is still `&'a str`.
* **The deserializer prefers `visit_borrowed_str`**, so structs declared
  as `String` get a single copy at the serde boundary and arena slices
  declared as `&'de str` skip even that.

### 2.3 Inline short strings: `CompactString`

The owned `Value` path (`parse()` → `Value`) keeps scalar text in
`compact_str::CompactString`. It is `Deref<Target = str>`, so callers treat
it as a `String`, but:

* Strings ≤ 24 bytes on 64-bit live *inline* in the 24-byte struct — no
  heap allocation for port numbers, identifiers, short hostnames, booleans-
  as-string.
* Longer strings fall back to a heap allocation transparently.

Used pervasively:

```rust
// src/value/value.rs
pub type Scalar = CompactString;

// src/value/object_map.rs
pub type ObjectMap = IndexMap<Scalar, Value, FxBuildHasher>;
```

Phase 3 swapped `String → CompactString` across `Value`, the parser, and
the serializer. The landed rule: never hold scalar text as `String` if
`CompactString` can substitute; pay the heap allocation only when the
scalar is genuinely long.

### 2.4 Capacity hints, everywhere

`Vec::new()` and `String::new()` don't allocate — that's the right default
for conditionally populated collections. But when the eventual size is
known-even-loosely, hinting removes one or more doubling reallocations:

```rust
// Owned parser (src/parser/frame.rs)
Frame::Object { pairs: ObjectMap::with_capacity_and_hasher(4, FxBuildHasher), ... }

// Owned parser Collecting (src/parser/collecting.rs)
Self { mode, lines: Vec::with_capacity(8) }

// Thin parser (src/thin/parser.rs)
pairs: BumpVec::with_capacity_in(4, bump)
```

The serializer opens the output buffer at 2 KB instead of 256 bytes; for a
22 KB target that removes **seven** doubling reallocs:

```rust
// src/ser/text_serializer.rs
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let mut out = String::with_capacity(2048);
    ...
}
```

**Size-hint source:** when serde hands us `len: usize` (struct field count,
tuple len, known-len map), use it:

```rust
// src/ser/value_serializer.rs
fn serialize_struct(self, _name: &'static str, len: usize) -> Result<StructSerializer> {
    Ok(StructSerializer {
        entries: IndexMap::with_capacity_and_hasher(len, FxBuildHasher),
    })
}
```

**Caveat:** the phase-5-arena path uses `with_capacity_in(4, bump)`, not
`new_in(bump)`. Phase 7 tested swapping to `new_in` (lazy) — it helped
owned-parse slightly but *hurt* thin parse by 5 % because typical objects
in the test fixtures have 4–6 keys, so reserving 4 up front skips a
doubling.

---

## 3. Library choices

| Crate | Role | Why |
|---|---|---|
| `bumpalo` (`3`, `collections` feature) | Arena for thin-path temporaries | One drop frees everything; `BumpVec` for arena-rooted growable vectors |
| `compact_str` (`0.9`, `serde` feature) | Inline short-string storage | 24 inline bytes covers typical config scalars with zero heap traffic |
| `indexmap` (`2`, `serde` feature) | Ordered `Scalar → Value` maps | Preserves struct-field order through ser/de; supports `entry()` API |
| `rustc-hash` (`2`) | `FxBuildHasher` for `IndexMap` | Faster than `DefaultHasher`; Ktav is a config parser, not hash-flood-resistant |
| `itoa` (`1`) | Integer → text in serializer | Skips `fmt::Formatter` overhead |
| `ryu` (`1`) | `f32`/`f64` → text | Shortest-round-trip float representation, fast |
| `serde` (`1`, `derive`) | De/serialization | Required contract |

Dev-only: `criterion` for measurement, `serde_json` (`preserve_order` +
`arbitrary_precision`) for conformance oracles.

**Why `IndexMap` and not `HashMap`:** `HashMap` would destroy struct field
order, which breaks round-trip serialization. `IndexMap` gives O(1) lookup
*and* insertion-order iteration for the price of a parallel `Vec` of
indices. Under `FxBuildHasher` it's faster than `HashMap` with the default
SipHash.

**Why `FxBuildHasher`:** SipHash is hash-flood-resistant at the cost of
throughput. A configuration parser is not exposed to adversarial inputs in
the threat model this crate targets; Fx gives a measurable speedup and
deterministic iteration order.

---

## 4. Code patterns

### 4.1 Fast paths for the common case

The hottest code path picks the dominant input shape and handles it without
going through the full generality. Two examples.

**Non-dotted keys** — in Ktav you *can* write `a.b.c: 10` but you rarely
do. Split the insertion function so the dotted case is a fallback and the
non-dotted case is a flat series of checks:

```rust
// src/thin/parser.rs
fn insert_pair<'a>(
    target: &mut BumpVec<'a, (&'a str, ThinValue<'a>)>,
    path: &'a str,
    value: ThinValue<'a>,
    line_num: usize,
    bump: &'a Bump,
) -> Result<()> {
    // Fast path: no dotted segments — the vast majority of inserts.
    if !path.as_bytes().contains(&b'.') {
        if !is_valid_key(path) { ... }
        if target.iter().any(|(k, _)| *k == path) { ... /* dup */ }
        target.push((path, value));
        return Ok(());
    }
    insert_at_path(target, path, value, line_num, path, bump)
}
```

The owned parser mirrors this in `src/parser/insert.rs`.

**`IndexMap::entry()` over `contains_key` + `insert`** — the old code took
two hash lookups per insertion (check, then insert). The `entry` API
consolidates to one:

```rust
// src/parser/insert.rs
match table.entry(path.into()) {
    Entry::Occupied(_) => Err(/* duplicate */),
    Entry::Vacant(v) => { v.insert(value); Ok(()) }
}
```

For dotted insertion the gain is larger — the naïve code did
`contains_key`, `insert`, `get_mut` (three lookups); `entry().or_insert_with`
does it in one.

### 4.2 Invariant-tracking to avoid redundant work

The parser used to call `trim()` three or four times per line:
`raw.trim()` at the top of `handle_line`, then `line[..colon].trim()`,
then `after.trim()`, then `classify()` would call `text.trim()` again.
Each trim is two byte scans (start + end).

Phase 4 fixed each redundant call by threading the invariant through:

* `handle_line` does `raw.trim()` once.
* Downstream receives already-trimmed input; comments and types document
  the invariant.
* Functions that need only leading- or trailing-whitespace cleanup use
  `trim_start`/`trim_end` — not the full `trim()`.

Typical doc-comment:

```rust
/// `trimmed` MUST be trim_start'ed (no leading whitespace). Trailing
/// whitespace has been removed earlier in the pipeline (raw.trim() at
/// the top of handle_line). Don't trim again here — we'd pay O(len)
/// for a no-op on the hot path.
fn classify<'a>(trimmed: &'a str, line_num: usize) -> Result<ValueStart<'a>> { ... }
```

### 4.3 `memchr`-backed shortcut for rare-byte checks

Phase 6: inside a multi-line string, every line used to do `raw.trim()` *
compare against `")"` or `"))"`. Most multi-line content has no `)` byte
at all. `<[u8]>::contains(&b')')` is memchr-backed — a single SIMD scan
that returns false in microseconds for typical payload lines. Guard the
expensive work with it:

```rust
// src/thin/parser.rs (phase 6)
if let Some(ref mut collecting) = self.collecting {
    if raw.as_bytes().contains(&b')') {
        let trimmed = raw.trim();
        let term = match collecting.mode { ... };
        if trimmed == term { /* close */ }
    }
    collecting.lines.push(raw);
    return Ok(());
}
```

All ten benchmarks moved in the same direction (−0.7 % to −7.6 %) — a
small but clean win, precisely because the shortcut is trivially correct
and stdlib does the heavy lifting.

### 4.4 Static-slice pool for constant content

The serializer writes indentation (4 spaces per level) tens of thousands of
times in a large render. The first implementation did `out.extend(iter::repeat(' ').take(n))`,
which LLVM compiles to a per-byte loop. The second made the same writes
through `unsafe { out.as_mut_vec() }.extend_from_slice(...)`. The third —
`unsafe`-free and faster — is a pre-built all-spaces `const` string sliced
into the output via `push_str`:

```rust
// src/render/helpers.rs
pub(super) fn push_indent(out: &mut String, level: usize) {
    const SPACES: &str =
        "                                                                "; // 64
    let mut remaining = level * INDENT.len();
    if remaining == 0 { return; }
    out.reserve(remaining);
    while remaining > 0 {
        let chunk = remaining.min(SPACES.len());
        out.push_str(&SPACES[..chunk]);
        remaining -= chunk;
    }
}
```

`push_str` compiles to `memcpy`, which is SIMD-friendly. The same pattern
lives in `src/ser/text_serializer.rs` (`write_indent`).

### 4.5 `#[cold]` / `#[inline(never)]` for slow paths

When a function has a hot branch and a rarely-taken slow branch, splitting
the slow part out with `#[cold] #[inline(never)]` keeps the hot path free
of unrelated code (better icache and branch prediction):

```rust
// src/render/helpers.rs
pub(super) fn needs_raw_marker(s: &str) -> bool {
    match s.as_bytes().first() {
        None => false,
        Some(&b' ') | Some(&b'\t') => needs_raw_marker_slow(s.trim_start()),
        Some(&b'{') | Some(&b'[') => true,
        Some(_) => matches!(s, "null" | "true" | "false" | "(" | "((" | "()" | "(())"),
    }
}

#[cold]
#[inline(never)]
fn needs_raw_marker_slow(t: &str) -> bool {
    t.starts_with('{') || t.starts_with('[')
        || matches!(t, "null" | "true" | "false" | "(" | "((" | "()" | "(())")
}
```

The hot path — 99 % of scalars that don't start with whitespace — branches
on one byte and returns. The whitespace-prefix case pays the cost of a
full `trim_start` and gets a proper check, but that path is marked cold so
it lives elsewhere in the binary.

### 4.6 Byte-level validation

Key validation rejects ASCII whitespace and a small set of reserved bytes.
UTF-8 decoding is overkill for this — the forbidden set is pure ASCII, so
iterate bytes directly:

```rust
// src/parser/validate.rs
#[inline]
pub(super) fn is_valid_key(k: &str) -> bool {
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            b.is_ascii_whitespace()
                || matches!(b, b'[' | b']' | b'{' | b'}' | b':' | b'#')
        })
}
```

`b.is_ascii_whitespace()` is a byte test; `matches!` compiles to a bitmask.
The old `char::is_whitespace` over `s.chars()` decoded UTF-8 for every
byte, and also *matched* Unicode whitespace (NBSP etc.), which the Ktav
spec explicitly allows.

### 4.7 Write-directly-into-buffer serialization

An earlier serializer did `let s = format!(...); out.push_str(&s);` for
each field. Commit `60d57d9` replaced every such call with
`write!(out, "{...}")` or the direct typed helpers:

```rust
// src/ser/text_serializer.rs
fn push_int_pair<I: itoa::Integer>(out: &mut String, v: I) {
    out.push_str(":i ");
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}
```

`itoa::Buffer` is a fixed-size `[u8; 40]` inside the caller's stack — no
allocation — and `Buffer::format` is the specialized integer-to-ASCII
routine. `ryu::Buffer` plays the same role for floats. This was phase 2.

### 4.8 `Cow` at boundaries (before arena)

Pre-phase-5 thin path kept `Cow<'a, str>` in `ThinValue` — borrowed when
the text was a buffer slice, owned when it had to be normalized. Phase 5
removed the `Cow` (arena took over the "owned when normalized" case), but
the pattern remains in `finalize_multiline` conceptually: single-line
cases return a direct buffer borrow; multi-line cases copy into the arena.

The principle: **have one place in the pipeline where `Borrowed` vs
`Owned` is decided, not a boolean sprinkled through the code.**

### 4.9 Single-line fast paths in multi-line finalization

A multi-line string with exactly one content line is common enough to be
worth a special case — both because it saves the `Vec::join` allocation
and because for `stripped` mode the single line's `trim_start` is the
final answer:

```rust
// src/thin/parser.rs
fn finalize_multiline<'a>(c: Collecting<'a>, bump: &'a Bump) -> &'a str {
    match c.mode {
        MultilineMode::Verbatim if c.lines.len() == 1 => c.lines[0],
        MultilineMode::Verbatim => { /* join into arena */ }
        MultilineMode::Stripped if c.lines.len() == 1 => {
            let only = c.lines[0];
            if only.trim().is_empty() { "" } else { only.trim_start() }
        }
        MultilineMode::Stripped => { /* dedent into arena */ }
    }
}
```

---

## 5. Serde integration

### 5.1 Deserializer prefers borrowed visits

The thin deserializer always routes to `visit_borrowed_str` when the
content is arena- or input-rooted:

```rust
// src/thin/deserializer.rs
fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    match self.value {
        ThinValue::Str(s) | ThinValue::Integer(s) | ThinValue::Float(s) => {
            visitor.visit_borrowed_str(s)
        }
        ...
    }
}
```

A caller asking for `&'de str` gets it zero-copy; a caller asking for
`String` still gets one copy at the boundary (serde's `String` visitor
copies a borrowed `&str` once), but no extra copy to reach the visitor.

Same applies to `deserialize_bytes` → `visit_borrowed_bytes`.

### 5.2 Borrowed keys through a mini-deserializer

Struct-field lookup compares the declared field name to the incoming key
string. The thin path exposes keys as `&'de str`, via a purpose-built
deserializer that forwards everything relevant to `visit_borrowed_str`:

```rust
// src/thin/deserializer.rs
struct BorrowedStrDeserializer<'de> {
    value: &'de str,
}

impl<'de> Deserializer<'de> for BorrowedStrDeserializer<'de> {
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_borrowed_str(self.value)
    }
    ...
}
```

Serde's default `IntoDeserializer<'_, _>` for `String` forces an allocation
per key; the borrowed variant skips it entirely.

### 5.3 `visit_str` vs `visit_string` on `CompactString`

Subtle: on the owned-`Value` path, scalar text lives in `CompactString`.
Owned-path `deserialize_str` could call `visitor.visit_string(s.into_string())`,
but that forces an allocation for every inline-short scalar. Instead:

```rust
// src/de/value_deserializer.rs
fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
    match &self.value {
        Value::String(s) | Value::Integer(s) | Value::Float(s) => visitor.visit_str(s),
        ...
    }
}
```

`visit_str(&s)` hands a borrowed `&str` — serde's own `String` visitor
allocates *once* if needed, *zero times* for inline variants where the
caller took `&'de str`. The old `visit_string(s.into_string())` cost a
heap allocation per scalar regardless.

### 5.4 `deserialize_any` still surfaces numeric types

Typed markers (`:i 123`) should arrive in `serde_json::Value` as an
integer, not a string. `deserialize_any` tries the numeric coercion first:

```rust
// src/thin/deserializer.rs
ThinValue::Integer(s) => {
    if let Ok(i) = s.parse::<i64>() { visitor.visit_i64(i) }
    else if let Ok(u) = s.parse::<u64>() { visitor.visit_u64(u) }
    else { visitor.visit_borrowed_str(s) } // arbitrary-precision fallback
}
```

Fallback preserves arbitrary-precision literals as strings — never silently
drops digits.

---

## 6. What didn't work, and why (the reject pile)

These were all measured; each has a commit history or a documented reason
they were rolled back. They are *more* useful than the commits that
landed: they draw the line between "natural-looking optimization" and
"actual optimization".

### 6.1 Unified forward line-scanner (phase 6, reverted)

Idea: replace `raw.trim()` + `trimmed.find(':')` + `"}"` / `"]"` equality
checks with one forward pass that tracks start, end, and first colon in
one loop. Looks like a clear win — fewer passes, same information.

**Measured:** `parse_to_value` **+14 % to +58 %** across benches. Reverted.

**Why it lost:** `str::trim` and `str::find` are memchr/SIMD backed in
stdlib. They scan at L1-cache speed. A hand-written forward loop with
*two* conditionals per byte (`== b':'`, `is_ascii_whitespace`) defeats
vectorization — LLVM can't prove the scans are independent, so it emits
scalar code. Two SIMD passes beat one scalar pass by a factor of three.

**Lesson:** when the stdlib already has a tight primitive for your scan,
don't replace it with a "more efficient" loop that fuses two different
checks. Call the primitives back-to-back; the cost is nearly free.

### 6.2 Byte-match fast-out in `classify` (phase 9, reverted)

Idea: if the first byte of the trimmed value isn't `{`/`[`/`(`, skip the
whole chain of `== "{"`, `starts_with('{')`, …, and return `Scalar` right
away.

**Measured:** +1 % to +5 % across `parse_to_value`, neutral on
`parse_to_struct`. Reverted.

**Why it lost:** LLVM was already constant-folding and reordering the
cascade. The extra explicit match arm adds code and occasionally defeats
the optimizer's reorder, costing more than it saves.

### 6.3 `Bump::with_capacity(text.len())` (phase 11, reverted)

Idea: Pre-size the bump arena so it never has to grow.

**Measured:** +7 % to +27 % across benches. Reverted.

**Why it lost:** `Bump::new()`'s default chunk size and doubling
strategy are tuned. Handing it a single large chunk up-front forces
a big allocator call before there's a user for any of it, and
interacts badly with per-call reuse patterns on warm allocators.

### 6.4 `BumpVec::new_in(bump)` instead of `with_capacity_in(4, bump)` (phase 7, reverted)

Idea: Lazy allocation — don't reserve anything until the first `push`.

**Measured:** `parse_to_value` improved 2–10 %, but
`parse_to_struct` regressed 5 %. Reverted on the typed-path regression.

**Why it lost:** Typical objects in the benchmark have 4–6 keys.
`with_capacity_in(4)` skips one doubling (0 → 4); `new_in` takes the
full 0 → 4 → 8 sequence on the hot path.

### 6.5 In-place dedent in `bumpalo::String` (phase 8, reverted)

Idea: Build the dedented multi-line string directly in a `bumpalo::String`,
skipping the `std::String` → `Bump::alloc_str` copy chain.

**Measured:** `multiline_dedent/*` **+6 % to +15 %**. Reverted.

**Why it lost:** `bumpalo::String` has slightly more bookkeeping per
`push_str` than `std::String` (alignment, chunk boundary checks). Over
many pushes the overhead exceeds the single terminating copy saved.

### 6.6 Custom `parse_u64_fast` / `parse_i64_fast` (phase 13, reverted)

Idea: Hand-rolled byte loops for `u8/u16/u32/u64/i8/i16/i32/i64`, falling
through to `str::parse` on anything unusual.

**Measured:** `parse_to_struct` **±0.0 %**. Reverted.

**Why it lost:** `str::parse::<u16>` is already tight — a few dozen
instructions, no allocation. LLVM inlined it. The hand-rolled version
did exactly the same work.

### 6.7 `lto = "fat"` for the bench profile (not committed)

Idea: Enable fat LTO just for benches.

**Measured:** `parse_to_value` **+40–60 %** (regression), but `render` saw
`−23 %`. Net negative across the suite.

**Why it lost:** Fat LTO aggressively inlines across crate boundaries,
which in a parser swells the hot function past L1 icache capacity. The
render path is smaller and benefits; the parse path gets punished by
instruction-cache pressure. Kept out of `Cargo.toml`; LTO belongs in a
separate, deliberately-scoped experiment.

### 6.8 `scan_line`-based optimizations for array items (phase 10)

Only applicable if `scan_line` from phase 6 had landed — since it didn't,
this never materialized. Its concept (skip colon-search in array context)
is *already* realized in the current code: `handle_array_item` doesn't
call `find(':')` at all.

---

## 7. Benchmarking methodology

### 7.1 Criterion defaults

```bash
cargo bench --bench parse -- \
    --save-baseline <name> \
    --warm-up-time 2 \
    --measurement-time 5 \
    --sample-size 50
```

Five-second measurement with 50 samples and two-second warmup is the
`*_precise` profile used for phase-to-phase comparisons.

### 7.2 Short-sample is for draft signal only

A 30-second run (`--warm-up-time 1 --measurement-time 2 --sample-size 20`)
is fast but carries ±10 % noise. Useful as a "did this completely break?"
sanity check. Never commit-or-revert a marginal change (±5 %) on a
short-sample result — rerun long.

### 7.3 Ablation for layered changes

When a phase bundles several independent patches, revert files one group
at a time and rebench each permutation. The Phase 4 ablation (in the
commit history) showed that one of its three groups gave most of the
signal and another was flat; that's only visible through ablation.

### 7.4 Comparing across commits

Criterion writes numbered baselines under `target/criterion/<bench>/<name>/`.
The JSON under `estimates.json` has `mean.point_estimate` in nanoseconds;
diffing two baselines is a one-liner:

```bash
for bench in parse_to_value/100_upstreams/22KB parse_to_struct/100_upstreams_typed ...; do
  a=$(grep -o 'point_estimate":[0-9.]*' "target/criterion/$bench/arena_precise/estimates.json" | head -1 | grep -o '[0-9.]*$')
  b=$(grep -o 'point_estimate":[0-9.]*' "target/criterion/$bench/phase6_memchr/estimates.json" | head -1 | grep -o '[0-9.]*$')
  awk "BEGIN{printf \"$bench: %+.1f%%\n\", ($b-$a)/$a*100}"
done
```

### 7.5 Thermal and rebuild noise

Individual bench results move ±5 % between adjacent long-sample runs even
with no code change. Causes:

* CPU thermal state (first run hotter than third on a laptop).
* Minor code changes trigger full crate recompile with different inline
  layout; bench binaries from different cargo runs have different TLB
  behavior.

A rule of thumb: **a single-digit percentage delta on one bench is
always noise. A uniform direction across ≥8 benches is signal.** That's
why phase 6 was committed (all ten went down) and phase 9 was rejected
(net mixed, within noise).

---

## 8. The rules, distilled

1. **Arena over per-node heap** for short-lived object trees.
2. **Borrow before owning** — `&'a str` and `Cow` at boundaries.
3. **Inline short strings** — `CompactString` for 24-byte fast path.
4. **FxHash + IndexMap** — fast hashing + insertion-order iteration.
5. **Capacity hints** at every collection creation where size is
   loosely knowable.
6. **One lookup per insert** — `entry().or_insert_with`, not
   `contains_key` + `insert`.
7. **Fast paths** for the dominant shape; dotted/complex cases branch
   off first.
8. **memchr shortcuts** to skip expensive `trim`/`compare` chains.
9. **Byte-level scanning** for ASCII-only grammars; never decode UTF-8
   for rejection-class checks.
10. **Static slice pools** for constant writes in hot loops
    (`const SPACES`).
11. **`#[cold]` + `#[inline(never)]`** for rarely-taken slow paths to
    free the hot icache.
12. **`itoa` / `ryu`** for number formatting; never `write!("{}", ...)`
    inside a hot loop.
13. **Invariant-threading** — document that input is already trimmed
    and don't re-trim; match `trim_end`/`trim_start` to the real need.
14. **`visit_borrowed_str` / `visit_borrowed_bytes`** on deserialize
    paths; a borrowed-key mini-deserializer for struct fields.
15. **`visit_str(&s)` over `visit_string(s.into_string())`** for
    inline-short string storage.
16. **Trust the standard library**. `str::trim`, `str::find`, `<[u8]>::contains`,
    `str::parse::<T>` are SIMD/tight-asm tuned. Replacing them with
    "unified" hand-rolled scans almost always regresses.
17. **Measure, don't guess**. Uniform direction across benches = signal;
    mixed bag = noise. Revert on flat results.
18. **No `unsafe`.** Every principle above is implementable in safe
    Rust; if an optimization seems to need `unsafe`, there's almost
    always a safe equivalent at the same speed or faster
    (`write_indent` went from `unsafe { as_mut_vec() }` to the
    `const SPACES` chunk loop and got slightly faster).

---

## 9. What's left (the wall)

Future gains require either:

* **Breaking the public `Value` API** to arena-back the owned parser too
  (est. +10–15 % on `parse()`), or
* **Custom-derive macros** bypassing serde's virtual dispatch for
  hot types (est. +15–30 % on `from_str<T>`), or
* **PGO** (est. +5–15 %, compile-time only).

Each of those is either API-breaking or a structural shift. Inside the
current "prose" of the crate — `#[derive(Serialize, Deserialize)]` over
safe Rust — the measured optimization surface is empirically exhausted.
