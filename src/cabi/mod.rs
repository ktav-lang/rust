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
//! source).to_json()`: ten fields, always all ten, in the fixed order
//! `error, reason, line, line_text, span, path, body, canonical,
//! spec_section, message`, with absent information an explicit `null`,
//! `path` an array of exact decoded key segments, and `message` the
//! `Display` rendering, which is never null. Errors that are not
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
//! bump — the contract is "every field always present, absent means
//! null", and a longer object still satisfies it.

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
/// envelope's contract is already "every field always present,
/// absent means null".
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

mod declare;
mod envelope;
mod ops;
mod wire;

pub use ops::{
    canonical_from_source, dumps, dumps_force_strings, emit_canonical, format, loads, loads_strict,
};
pub use wire::WireValue;

#[cfg(test)]
mod tests;
