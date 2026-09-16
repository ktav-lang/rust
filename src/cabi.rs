//! The shareable part of the C ABI shim.
//!
//! The six language bindings each used to keep a private copy of the same
//! FFI shim around the `ktav` crate. This module carries the part of that
//! shim that is genuinely shareable: the wire-value decoding, the six
//! document operations, the error-envelope encoding, the ABI version and
//! the native-library naming convention.
//!
//! # No exported symbols here
//!
//! This module deliberately contains **no `extern "C"` and no
//! `#[no_mangle]`**. Symbols defined in a dependency rlib are not
//! guaranteed to survive into a downstream cdylib — Rust may strip them
//! as dead code. Instead, the [`crate::declare_cabi!`] macro expands the
//! exported symbols into the *calling* crate, where the export is
//! guaranteed by construction.
//!
//! # Wire format
//!
//! Between host and Rust we exchange **JSON**, not a custom binary. Ktav's
//! Integer and Float scalars do not map 1:1 onto JSON numbers (JSON cannot
//! represent arbitrary-precision integers, and it loses the
//! Integer/Float distinction), so they travel as tagged wrappers:
//!
//! ```text
//! Value::Integer(s) <=> {"$i": "<digits>"}
//! Value::Float(s)   <=> {"$f": "<text>"}
//! ```
//!
//! Everything else maps to the obvious JSON shape. Object key order is
//! preserved via [`indexmap`] (the `preserve_order` serde_json feature).
//!
//! # Error encoding
//!
//! Every `Err` payload of the six operations is already the
//! [`ErrorEnvelope`] JSON — `ErrorEnvelope::from_error(&err,
//! source).to_json()`: nine fields, always all nine, in the fixed order
//! `error, reason, line, line_text, span, path, body, canonical,
//! spec_section`, with absent information an explicit `null` and `path`
//! an array of exact decoded key segments. Errors that are not
//! [`Error`]s (serde_json failures, the top-level check) are wrapped as
//! `Error::Message` and encoded the same way, so hosts never have to
//! sniff plain text vs JSON. For operations whose input was Ktav source
//! text the source is passed, so `line_text` populates; for the
//! JSON-input family an empty source is passed and the envelope emits
//! honest nulls.
//!
//! # [`ABI_VERSION`]
//!
//! The exported-shape version, exposed as `ktav_abi_version()` by the
//! macro. Bump it when an exported function is added, removed, or its
//! signature or ownership contract changes, or when the error encoding
//! changes. Adding a field to the error envelope does **not** oblige a
//! bump — the contract is "nine fields, absent means null".

use indexmap::IndexMap;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map as JsonMap, Value as Json};

use crate::value::{ObjectMap, Value};
use crate::{Error, ErrorEnvelope};

/// The version of the exported C ABI *shape*: the set of `ktav_*`
/// functions the [`crate::declare_cabi!`] expansion produces, their
/// signatures, and the error encoding. It changes almost never — the
/// crate version changes every release, this must not. A host compares
/// `ktav_abi_version()` against the value it was compiled against and
/// refuses to load on mismatch, instead of corrupting memory.
///
/// Bump when an exported function is added, removed, or its signature
/// or ownership contract changes, or when the error encoding changes.
/// Adding a *field* to the error envelope does not oblige a bump: the
/// envelope's contract is already "nine fields, absent means null".
pub const ABI_VERSION: u32 = 1;

/// The crate version as a NUL-terminated byte string. The macro's
/// `ktav_version()` returns a pointer into this static; the macro
/// cannot use `env!("CARGO_PKG_VERSION")` itself, because that would
/// report the *calling* crate's version.
pub const VERSION_BYTES: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

/// The native-library file name the binding loaders search for, for an
/// `os`/`arch` pair. Accepted spellings: `os` ∈ {`"windows"`,
/// `"macos"`, `"darwin"`, `"linux"`}, `arch` ∈ {`"amd64"`,
/// `"x86_64"`, `"arm64"`, `"aarch64"`}; `None` for anything else. The
/// output uses exactly the strings the `php`, `csharp` and `golang`
/// loaders already derive independently:
///
/// ```text
/// ktav_cabi-windows-{arch}.dll
/// libktav_cabi-darwin-{arch}.dylib
/// libktav_cabi-linux-{arch}.so
/// arch ∈ { amd64, arm64 }
/// ```
///
/// Host loaders cannot call this — they run before the library is
/// loaded — but build scripts, packaging steps and tests can, which
/// makes the convention executable instead of aspirational.
pub fn library_file_name(os: &str, arch: &str) -> Option<String> {
    let (prefix, ext) = match os {
        "windows" => ("ktav_cabi-windows-", ".dll"),
        "macos" | "darwin" => ("libktav_cabi-darwin-", ".dylib"),
        "linux" => ("libktav_cabi-linux-", ".so"),
        _ => return None,
    };
    let arch = match arch {
        "amd64" | "x86_64" => "amd64",
        "arm64" | "aarch64" => "arm64",
        _ => return None,
    };
    Some(format!("{prefix}{arch}{ext}"))
}

fn envelope(err: &Error, source: &str) -> String {
    ErrorEnvelope::from_error(err, source).to_json()
}

/// Wrap a non-`ktav::Error` diagnostic (a serde_json failure, the
/// top-level check) so it leaves through the same envelope channel as
/// everything else.
fn message_envelope(text: impl Into<String>, source: &str) -> String {
    envelope(&Error::Message(text.into()), source)
}

fn not_utf8_envelope(err: std::str::Utf8Error) -> String {
    // The input is not text, so there is no source line to pass and
    // `line_text` honestly comes out null.
    message_envelope(format!("input is not valid UTF-8: {err}"), "")
}

/// Parse Ktav source text bytes into a JSON wire document.
pub fn loads(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let value = crate::parse(input).map_err(|err| envelope(&err, input))?;
    let json = value_to_json(&value);
    serde_json::to_vec(&json)
        .map_err(|err| message_envelope(format!("internal: encode JSON: {err}"), ""))
}

/// Like [`loads`], but with strict scalar inference: lossy numeric
/// literals (`1.10`, `+1`) are rejected instead of normalized.
pub fn loads_strict(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let value = crate::parse_strict(input).map_err(|err| envelope(&err, input))?;
    let json = value_to_json(&value);
    serde_json::to_vec(&json)
        .map_err(|err| message_envelope(format!("internal: encode JSON: {err}"), ""))
}

/// Render a JSON wire document back to Ktav source text.
pub fn dumps(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::render::render(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Render a JSON wire document to Ktav source, coercing every scalar to
/// a literal string.
pub fn dumps_force_strings(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::to_string_force_strings(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Render a JSON wire document to canonical Ktav text (fixed key order,
/// minimal quoting).
pub fn emit_canonical(src: &[u8]) -> Result<Vec<u8>, String> {
    let value = wire_to_value(src)?;
    let text = crate::emit_canonical(&value).map_err(|err| envelope(&err, ""))?;
    Ok(text.into_bytes())
}

/// Format Ktav source text bytes. The input is Ktav source text, NOT a
/// JSON wire value. Formatting preserves comments and blank-line
/// grouping and is a fixed point: `format(format(x)) == format(x)`.
pub fn format(src: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(src).map_err(not_utf8_envelope)?;
    let text = crate::format_str(input).map_err(|err| envelope(&err, input))?;
    Ok(text.into_bytes())
}

/// The dumps-family front half: JSON wire bytes -> `Value`, with every
/// failure — serde_json parse, tagged-payload validation, the
/// top-level object/array rule — reported as an envelope. The input
/// was a wire value, not Ktav text, so envelopes are built against an
/// empty source (honest nulls for `line` / `line_text` / `span`).
fn wire_to_value(src: &[u8]) -> Result<Value, String> {
    let wire: WireValue = serde_json::from_slice(src)
        .map_err(|err| message_envelope(format!("input JSON: {err}"), ""))?;
    let value = wire.into_value().map_err(|e| message_envelope(e, ""))?;
    if !matches!(value, Value::Object(_) | Value::Array(_)) {
        return Err(message_envelope(
            "top-level Ktav document must be an object or array",
            "",
        ));
    }
    Ok(value)
}

fn value_to_json(v: &Value) -> Json {
    match v {
        Value::Null => Json::Null,
        Value::Bool(b) => Json::Bool(*b),
        Value::Integer(s) => {
            let mut m = JsonMap::new();
            m.insert("$i".to_string(), Json::String(s.to_string()));
            Json::Object(m)
        }
        Value::Float(s) => {
            let mut m = JsonMap::new();
            m.insert("$f".to_string(), Json::String(s.to_string()));
            Json::Object(m)
        }
        Value::String(s) => Json::String(s.to_string()),
        Value::Array(a) => Json::Array(a.iter().map(value_to_json).collect()),
        Value::Object(o) => {
            let mut m = JsonMap::new();
            for (k, val) in o {
                m.insert(k.to_string(), value_to_json(val));
            }
            Json::Object(m)
        }
    }
}

/// A JSON wire value that understands both plain JSON values and the
/// `{"$i": ...}` / `{"$f": ...}` tagged wrappers, preserving object key
/// order via `indexmap`.
pub enum WireValue {
    /// JSON `null`.
    Null,
    /// A JSON boolean.
    Bool(bool),
    /// An integer literal, tagged `{"$i": "<digits>"}` on the wire.
    Integer(String),
    /// A float literal, tagged `{"$f": "<text>"}` on the wire.
    Float(String),
    /// A JSON string.
    String(String),
    /// A JSON array.
    Array(Vec<WireValue>),
    /// A JSON object with insertion order preserved.
    Object(IndexMap<String, WireValue>),
}

impl WireValue {
    fn into_value(self) -> Result<Value, String> {
        match self {
            WireValue::Null => Ok(Value::Null),
            WireValue::Bool(b) => Ok(Value::Bool(b)),
            WireValue::Integer(s) => {
                validate_integer(&s)?;
                Ok(Value::Integer(s.into()))
            }
            WireValue::Float(s) => {
                validate_float(&s)?;
                Ok(Value::Float(s.into()))
            }
            WireValue::String(s) => Ok(Value::String(s.into())),
            WireValue::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for w in items {
                    out.push(w.into_value()?);
                }
                Ok(Value::Array(out))
            }
            WireValue::Object(m) => {
                let mut obj = ObjectMap::with_capacity_and_hasher(m.len(), Default::default());
                for (k, v) in m {
                    obj.insert(k.into(), v.into_value()?);
                }
                Ok(Value::Object(obj))
            }
        }
    }
}

fn validate_integer(s: &str) -> Result<(), String> {
    let rest = s.strip_prefix('-').unwrap_or(s);
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("$i payload not an integer literal: {s:?}"));
    }
    Ok(())
}

/// True when a number's source text is a float lexical form (`.` or
/// exponent) rather than an integer literal.
fn looks_like_float(s: &str) -> bool {
    s.bytes().any(|b| b == b'.' || b == b'e' || b == b'E')
}

fn validate_float(s: &str) -> Result<(), String> {
    if s.parse::<f64>().is_err() {
        return Err(format!("$f payload not a finite decimal: {s:?}"));
    }
    if !looks_like_float(s) {
        return Err(format!("$f payload must contain '.' or exponent: {s:?}"));
    }
    Ok(())
}

/// The string payload of a `$i` / `$f` tag (or of serde_json's
/// arbitrary-precision sentinel): anything but string-like is a
/// wire-contract violation.
fn tagged_payload<E: de::Error>(tag: &str, v: WireValue) -> Result<String, E> {
    match v {
        WireValue::String(s) | WireValue::Integer(s) | WireValue::Float(s) => Ok(s),
        _ => Err(de::Error::custom(format!("{tag} payload must be a string"))),
    }
}

impl<'de> Deserialize<'de> for WireValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = WireValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_unit<E: de::Error>(self) -> Result<WireValue, E> {
                Ok(WireValue::Null)
            }
            fn visit_none<E: de::Error>(self) -> Result<WireValue, E> {
                Ok(WireValue::Null)
            }
            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<WireValue, D::Error> {
                WireValue::deserialize(d)
            }
            fn visit_bool<E: de::Error>(self, b: bool) -> Result<WireValue, E> {
                Ok(WireValue::Bool(b))
            }
            fn visit_i64<E: de::Error>(self, n: i64) -> Result<WireValue, E> {
                Ok(WireValue::Integer(n.to_string()))
            }
            fn visit_u64<E: de::Error>(self, n: u64) -> Result<WireValue, E> {
                Ok(WireValue::Integer(n.to_string()))
            }
            fn visit_f64<E: de::Error>(self, n: f64) -> Result<WireValue, E> {
                if !n.is_finite() {
                    return Err(E::custom("NaN / ±Infinity not allowed in Ktav"));
                }
                // Bare JSON floats get the ":f" wire form with a forced
                // decimal point so render's grammar check is satisfied.
                let mut s = format!("{n}");
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    s.push_str(".0");
                }
                Ok(WireValue::Float(s))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<WireValue, E> {
                Ok(WireValue::String(v.to_string()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<WireValue, E> {
                Ok(WireValue::String(v))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<WireValue, A::Error> {
                let mut out = Vec::new();
                while let Some(item) = seq.next_element()? {
                    out.push(item);
                }
                Ok(WireValue::Array(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<WireValue, A::Error> {
                let Some(k1) = map.next_key::<String>()? else {
                    return Ok(WireValue::Object(IndexMap::new()));
                };
                let v1: WireValue = map.next_value()?;
                let second_key: Option<String> = map.next_key()?;

                // serde_json built with `arbitrary_precision` — which this
                // crate's dependency graph can force via feature
                // unification — delivers every number that is not an exact
                // i64/u64 (all floats, integers beyond 64 bits) as a
                // single-entry map keyed by its crate-private sentinel, the
                // value being the number's exact source text. Recover the
                // number so bare JSON numbers keep decoding losslessly;
                // builds without the feature never hit this branch.
                if second_key.is_none() && k1 == "$serde_json::private::Number" {
                    let payload = tagged_payload(&k1, v1)?;
                    return Ok(if looks_like_float(&payload) {
                        WireValue::Float(payload)
                    } else {
                        WireValue::Integer(payload)
                    });
                }

                if second_key.is_none() && (k1 == "$i" || k1 == "$f") {
                    let payload = tagged_payload(&k1, v1)?;
                    return Ok(if k1 == "$i" {
                        WireValue::Integer(payload)
                    } else {
                        WireValue::Float(payload)
                    });
                }

                let mut out: IndexMap<String, WireValue> = IndexMap::new();
                out.insert(k1, v1);
                if let Some(k2) = second_key {
                    let v2: WireValue = map.next_value()?;
                    out.insert(k2, v2);
                    while let Some((k, v)) = map.next_entry::<String, WireValue>()? {
                        out.insert(k, v);
                    }
                }
                Ok(WireValue::Object(out))
            }
        }

        d.deserialize_any(V)
    }
}

/// Expand the full exported C ABI surface into the CALLING crate: six
/// document functions, `ktav_free`, `ktav_version` and
/// `ktav_abi_version`. Symbols defined here are exported by
/// construction — a dependency rlib's symbols would not survive into a
/// downstream cdylib.
///
/// Invoke it exactly once, at the top level of the binding's cdylib:
/// `ktav::declare_cabi!();`
///
/// Ownership contract: every returned buffer is freed by the caller
/// through `ktav_free(ptr, len)` exactly once, with the length it was
/// returned with. Return code is `0` on success and `1` on error, with
/// `out_err` holding the JSON error envelope on error.
///
/// The invoking crate must be edition ≤ 2021: edition 2024 renamed the
/// attribute to `#[unsafe(no_mangle)]`.
#[macro_export]
macro_rules! declare_cabi {
    () => {
        #[inline]
        unsafe fn ktav_cabi_emit(
            buf: ::std::vec::Vec<u8>,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
        ) {
            let mut boxed = buf.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            // SAFETY: the boxed slice is intentionally leaked here; the
            // caller frees it via `ktav_free` with the returned length.
            unsafe {
                ::std::mem::forget(boxed);
            }
            *out_buf = ptr;
            *out_len = len;
        }

        unsafe fn ktav_cabi_emit_err(
            msg: ::std::string::String,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) {
            let bytes = msg.into_bytes();
            let mut boxed = bytes.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr() as *mut ::core::ffi::c_char;
            // SAFETY: same leak-for-ownership contract as `ktav_cabi_emit`.
            unsafe {
                ::std::mem::forget(boxed);
            }
            *out_err = ptr;
            *out_err_len = len;
        }

        /// Parse Ktav source text to a JSON wire document.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_loads(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::loads(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Parse Ktav source text (strict scalars) to a JSON wire document.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_loads_strict(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::loads_strict(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to Ktav source text.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_dumps(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::dumps(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to Ktav source, coercing every
        /// scalar to a literal string.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_dumps_force_strings(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::dumps_force_strings(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to canonical Ktav text.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_emit_canonical(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::emit_canonical(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Format Ktav source text (input is Ktav source, NOT a JSON wire
        /// value); preserves comments and blank-line grouping.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_format(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::format(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Free a buffer returned by any `ktav_*` output function.
        /// Null/zero is a no-op.
        ///
        /// # Safety
        /// Must be called exactly once per returned buffer, with the same
        /// length it was returned with.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_free(ptr: *mut u8, len: usize) {
            if ptr.is_null() || len == 0 {
                return;
            }
            // SAFETY: the pointer was produced by `ktav_cabi_emit` /
            // `ktav_cabi_emit_err` as a leaked boxed slice of exactly
            // `len` bytes, and this runs once per buffer.
            let _ = unsafe {
                ::std::boxed::Box::from_raw(::core::ptr::slice_from_raw_parts_mut(ptr, len))
            };
        }

        /// NUL-terminated static version string of the `ktav` crate, for
        /// sanity checks that the loader picked up the right file.
        #[no_mangle]
        pub extern "C" fn ktav_version() -> *const ::core::ffi::c_char {
            $crate::cabi::VERSION_BYTES.as_ptr() as *const ::core::ffi::c_char
        }

        /// Version of the exported C ABI *shape*. Compare against the
        /// `ktav::cabi::ABI_VERSION` the host was compiled against and
        /// refuse to load on mismatch.
        #[no_mangle]
        pub extern "C" fn ktav_abi_version() -> u32 {
            $crate::cabi::ABI_VERSION
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope_json(payload: &str) -> Json {
        let v: Json = serde_json::from_str(payload)
            .unwrap_or_else(|e| panic!("Err payload is not JSON: {e}: {payload}"));
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "error",
                "reason",
                "line",
                "line_text",
                "span",
                "path",
                "body",
                "canonical",
                "spec_section",
            ]
        );
        v
    }

    #[test]
    fn abi_version_is_deliberate() {
        // Changing this value is a breaking change for every host: update
        // this test and the module docs together with the bump.
        assert_eq!(ABI_VERSION, 1);
    }

    #[test]
    fn library_file_name_matches_the_binding_loaders() {
        assert_eq!(
            library_file_name("windows", "amd64").as_deref(),
            Some("ktav_cabi-windows-amd64.dll")
        );
        assert_eq!(
            library_file_name("windows", "arm64").as_deref(),
            Some("ktav_cabi-windows-arm64.dll")
        );
        assert_eq!(
            library_file_name("macos", "amd64").as_deref(),
            Some("libktav_cabi-darwin-amd64.dylib")
        );
        assert_eq!(
            library_file_name("darwin", "arm64").as_deref(),
            Some("libktav_cabi-darwin-arm64.dylib")
        );
        assert_eq!(
            library_file_name("linux", "amd64").as_deref(),
            Some("libktav_cabi-linux-amd64.so")
        );
        assert_eq!(
            library_file_name("linux", "arm64").as_deref(),
            Some("libktav_cabi-linux-arm64.so")
        );
        // Rust-const target spellings alias onto the same files.
        assert_eq!(
            library_file_name("linux", "x86_64").as_deref(),
            Some("libktav_cabi-linux-amd64.so")
        );
        assert_eq!(
            library_file_name("macos", "aarch64").as_deref(),
            Some("libktav_cabi-darwin-arm64.dylib")
        );
        assert_eq!(library_file_name("freebsd", "amd64"), None);
        assert_eq!(library_file_name("linux", "riscv64"), None);
    }

    #[test]
    fn loads_wire_form_pins_order_and_tags() {
        let out = loads(b"port: 8080\nhost: a.example\n").unwrap();
        assert_eq!(out, br#"{"port":{"$i":"8080"},"host":"a.example"}"#);
    }

    #[test]
    fn loads_reports_envelope_on_parse_error() {
        let err = loads(b"anchor: ok\nport:8080\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "MissingSeparatorSpace");
        assert_eq!(v["line"], 2);
        assert_eq!(v["line_text"], "port:8080");
        assert!(v["span"].is_object());
        assert!(v["body"].is_null());
        // `MissingSeparatorSpace` has a governing spec section, so the
        // envelope populates it rather than emitting null.
        assert_eq!(v["spec_section"], "§6.10");
    }

    #[test]
    fn loads_reports_envelope_on_invalid_utf8() {
        let err = loads(&[0xFF, 0xFE]).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "Message");
        assert!(
            v["body"]
                .as_str()
                .unwrap()
                .starts_with("input is not valid UTF-8"),
            "body = {:?}",
            v["body"]
        );
        assert!(v["line"].is_null());
        assert!(v["line_text"].is_null());
        assert!(v["span"].is_null());
    }

    #[test]
    fn loads_strict_reports_lossy_scalar_envelope() {
        let err = loads_strict(b"a: 1.10\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "LossyScalar");
        assert_eq!(v["body"], "1.10");
        assert_eq!(v["canonical"], "1.1");
        assert_eq!(v["line"], 1);
    }

    #[test]
    fn dumps_error_payloads_are_envelopes() {
        let err = dumps(b"{").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "Message");
        assert!(
            v["body"].as_str().unwrap().starts_with("input JSON"),
            "body = {:?}",
            v["body"]
        );

        let err = dumps(br#""just a string""#).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(
            v["body"],
            "top-level Ktav document must be an object or array"
        );
    }

    #[test]
    fn dumps_bare_float_survives_arbitrary_precision() {
        let text = dumps(br#"{"r": 3.5}"#).unwrap();
        let text = std::str::from_utf8(&text).unwrap();
        assert!(text.contains("r: 3.5"), "text = {text:?}");
        assert_eq!(
            crate::parse(text).unwrap(),
            crate::parse("r: 3.5\n").unwrap()
        );
    }

    #[test]
    fn dumps_big_integer_survives_arbitrary_precision() {
        let text = dumps(br#"{"n": 123456789012345678901234567890}"#).unwrap();
        let text = std::str::from_utf8(&text).unwrap();
        assert!(
            text.contains("123456789012345678901234567890"),
            "text = {text:?}"
        );
        assert_eq!(
            crate::parse(text).unwrap(),
            crate::parse("n: 123456789012345678901234567890\n").unwrap()
        );
    }

    #[test]
    fn emit_canonical_smoke_and_force_strings_coercion() {
        let wire = br#"{"a":{"$i":"7"},"b":[true,null,"s"]}"#;
        let canon = emit_canonical(wire).unwrap();
        let text = std::str::from_utf8(&canon).unwrap();
        let from_canonical = crate::parse(text).unwrap();
        // `loads` re-encodes to wire form; `dumps` produces the Ktav text
        // that both must agree on when parsed back.
        let dumped = dumps(wire).unwrap();
        let from_dumps = crate::parse(std::str::from_utf8(&dumped).unwrap()).unwrap();
        assert_eq!(from_canonical, from_dumps);

        let forced = dumps_force_strings(br#"{"a":{"$i":"7"}}"#).unwrap();
        let text = std::str::from_utf8(&forced).unwrap();
        let v = crate::parse(text).unwrap();
        match v {
            Value::Object(o) => assert!(matches!(o.get("a"), Some(Value::String(_)))),
            other => panic!("expected object, got {other:?}"),
        }
    }

    #[test]
    fn format_is_a_fixed_point_and_normalises_inline() {
        let src: &[u8] = b"a: {x: 1}\n";
        let out = format(src).unwrap();
        assert_ne!(out, src, "inline compound should be normalised");
        let out_text = std::str::from_utf8(&out).unwrap();
        crate::parse(out_text).unwrap();
        let again = format(&out).unwrap();
        assert_eq!(again, out, "format must be a fixed point");

        // Comments survive formatting, in place.
        let out = format(b"## c\na: 1\n").unwrap();
        assert!(out.starts_with(b"## c\n"), "out = {out:?}");
    }

    #[test]
    fn format_reports_envelope_with_source_position() {
        let err = format(b"anchor: ok\nport:8080\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "MissingSeparatorSpace");
        assert_eq!(v["line"], 2);
        assert_eq!(v["line_text"], "port:8080");
    }

    #[test]
    fn version_bytes_are_nul_terminated() {
        assert!(VERSION_BYTES.ends_with(&[0u8]));
        assert_eq!(
            &VERSION_BYTES[..VERSION_BYTES.len() - 1],
            env!("CARGO_PKG_VERSION").as_bytes()
        );
    }
}
