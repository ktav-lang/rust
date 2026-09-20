# The Ktav C ABI (0.8.0)

This document specifies the C ABI that the Ktav language bindings
(Go, Java, PHP, C#, JS, Python) load and call. It is written for the
authors of the binding loaders: everything a host needs to resolve
symbols, own memory and decode results is stated here once, instead of
being re-derived per language.

The contract is split between two pieces of the `ktav` crate:

- **`ktav::cabi`** (behind the off-by-default `cabi` feature) is the
  safe half: wire decoding, the seven document operations, the error
  envelope, `ABI_VERSION` and `library_file_name`. It contains no
  exported symbols.
- **`ktav::declare_cabi!()`** expands the ten exported `#[no_mangle]`
  symbols *into the calling cdylib*. The export is guaranteed by
  construction: symbols defined in a dependency rlib are not guaranteed
  to survive into a downstream cdylib, symbols defined in the cdylib's
  own code are.

Getting started:

```toml
# Cargo.toml of the binding's native crate:
#   [dependencies] ktav = { version = "0.8", features = ["cabi"] }
#   [lib] crate-type = ["cdylib"]
```

```rust
// src/lib.rs — the whole body:
ktav::declare_cabi!();
```

## Symbols

The macro exports exactly ten symbols. The seven document functions
share one signature (shown for `ktav_loads`; the others differ only in
name):

```c
int ktav_loads(const uint8_t* src, size_t src_len,
               uint8_t** out_buf, size_t* out_len,
               char** out_err, size_t* out_err_len);
```

| Symbol | C signature | Notes |
|---|---|---|
| `ktav_loads` | shared signature | Ktav source text in → JSON wire out |
| `ktav_loads_strict` | shared signature | Ktav source text in (strict scalars, spec § 3.6 / § 5.2) → JSON wire out |
| `ktav_dumps` | shared signature | JSON wire in → Ktav text out |
| `ktav_dumps_force_strings` | shared signature | JSON wire in → Ktav text out, every scalar coerced via `::` |
| `ktav_emit_canonical` | shared signature | JSON wire in → canonical Ktav text out (spec § 5.9) |
| `ktav_format` | shared signature | Ktav source text in → formatted Ktav text out |
| `ktav_canonical_from_source` | shared signature | Ktav source text in → canonical Ktav text out, without passing through a host value |
| `ktav_free` | `void ktav_free(uint8_t* ptr, size_t len)` | frees a returned buffer |
| `ktav_version` | `const char* ktav_version(void)` | NUL-terminated crate version, static storage |
| `ktav_abi_version` | `uint32_t ktav_abi_version(void)` | see [ABI version](#abi-version) |

Every document function returns `0` on success and `1` on error. All
four output pointers are nulled at entry, before any input is touched.
Invalid UTF-8 input is an error (an envelope on `out_err`), never a
panic and never a silent lossy replacement.

## Ownership contract

- Every returned buffer — the success payload on `out_buf` *and* the
  error envelope on `out_err` — is allocated by the callee and must be
  freed by the caller with `ktav_free(ptr, len)` exactly once, passing
  the same length the buffer was returned with.
- `ktav_free` is a no-op when `ptr` is null or `len` is zero.
- The host must free through the same library that returned the
  buffer. Each loaded copy of the shim has its own allocator; freeing
  a buffer through a different library's `ktav_free` is undefined
  behavior.

## Wire format

JSON in both directions. Ktav's Integer and Float scalars do not map
onto JSON numbers (JSON cannot represent arbitrary-precision integers
and loses the Integer/Float distinction), so they travel as tagged
wrappers:

```text
{"$i": "<digits>"}   Integer
{"$f": "<text>"}     Float
```

Everything else uses the plain JSON shape (`null`, `true`, `false`,
strings, arrays, objects). Object key order is preserved in both
directions.

Hosts may send bare JSON numbers: 64-bit integers and floats are
accepted on input. An integer beyond 64 bits must use the `$i`
wrapper.

## Error encoding

On error, `out_err` holds the JSON envelope produced by
`ErrorEnvelope::from_error(...).to_json()`: ten fields, always all
ten, in the fixed order

```text
error, reason, line, line_text, span, path, body, canonical, spec_section,
message
```

Absent information is an explicit `null`, never an omitted key.
`path` is an array of exact decoded key segments, never a joined
string.

`message` is the error's `Display` rendering, verbatim, and is the
only field that is never `null`. **A host surfaces this as its
exception text instead of assembling prose from the other nine
fields.** Before it existed the envelope carried structured data and
no message, so each binding invented a rendering — and the five that
did all disagreed, with each other and with what the crate itself
printed for the same input. It is appended last, so the nine original
fields keep the positions they shipped with.

Uniformity rule: *every* error leaves as an envelope, including
serde_json failures and the top-level check, which are wrapped as
`Error::Message` first. Hosts must not sniff plain text against JSON.

`body` carries the `Message`/`Syntax` payload verbatim. When the input
was a JSON wire value, the source passed to the envelope is empty and
the position fields are honest nulls; when the input was Ktav source
text, `line`, `line_text` and `span` populate.

## Operation semantics

- **`ktav_loads` / `ktav_loads_strict`** — parse Ktav source text to
  the JSON wire form. The strict variant additionally rejects lossy
  scalars per spec § 3.6 / § 5.2.
- **`ktav_dumps`** — render a JSON wire document as Ktav text. The
  top level must be an Object or an Array; anything else fails with
  exactly `top-level Ktav document must be an object or array`.
- **`ktav_dumps_force_strings`** — as `ktav_dumps`, but every scalar
  is coerced to a literal string via the `::` marker.
- **`ktav_emit_canonical`** — render the spec § 5.9 canonical form.
  Two calls with the same input produce the same bytes.
- **`ktav_format`** — format Ktav source text: comments preserved
  verbatim, runs of blank lines collapsed to one, key order never
  changed, and the transform is a fixed point
  (`format(format(x)) == format(x)`). For a document with no comments
  and no blank lines it equals `emit_canonical` of its parse.
- **`ktav_canonical_from_source`** — canonical form straight from Ktav
  source text, never passing through a host value. Prefer it over
  `loads` followed by `emit_canonical` in any language whose number
  type cannot carry the Integer/Float distinction: routing `1.0`
  through such a value yields `1`, and § 5.9 becomes unreachable. The
  JavaScript binding discovered this when a byte-exact canonical check
  failed on ten float fixtures.

## ABI version

`ktav::cabi::ABI_VERSION`, currently `1`, is exported as
`ktav_abi_version()`. Hosts **must** compare it at load time against
the value they were built against and refuse to load on mismatch.

A bump is obliged by: adding, removing, or changing the signature or
ownership contract of an exported function; changing the error
encoding. Appending a *field* to the end of the error envelope does
**not** oblige a bump: absent information is already an explicit
`null`, and a host that reads fields by name is unaffected by a longer
object. Inserting or reordering a field would, because it moves every
field after it. The crate version is a different axis and never stands
in for the ABI version.

## Artifact naming and search

The native library file names follow one convention:

```text
ktav_cabi-windows-{arch}.dll
libktav_cabi-darwin-{arch}.dylib
libktav_cabi-linux-{arch}.so
arch ∈ { amd64, arm64 }
```

`ktav::cabi::library_file_name(os, arch) -> Option<String>` is the
executable form of the table. It accepts `x86_64`/`aarch64` aliases
for `amd64`/`arm64` and returns `None` outside the table. Host
loaders cannot call it (they run before the library exists), but build
scripts and packaging steps can.

Setting `$KTAV_LIB_PATH` overrides the search entirely.

This convention is derived independently by the `php`, `csharp` and
`golang` loaders and pinned by `ktav::cabi` unit tests.

## Where the tests live

- `tests/cabi-fixture/` — a cdylib fixture whose entire body is one
  `declare_cabi!()` invocation.
- `tests/cabi_load.rs` — builds the fixture, dlopens it, resolves all
  ten symbols and calls through them.
- `tests/cabi_corpus.rs` — the 221-fixture wire round-trip plus the
  canonical byte oracles.
