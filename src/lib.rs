//! # Ktav — a plain configuration format
//!
//! JSON5-shaped, but with no quotes, no commas, and dotted keys for nesting.
//! A document is an implicit top-level object. Native `serde` integration:
//! any type implementing `Serialize` / `Deserialize` (including
//! `#[derive]`-generated ones) round-trips through Ktav out of the box.
//!
//! ## Syntax
//!
//! ```text
//! # comment             — any line starting with '#'
//! key: value            — scalar; `key` may be a dotted path (a.b.c: 10)
//! key:: value           — scalar; value is ALWAYS a literal string
//! key: { ... }          — multi-line object; `}` closes on its own line
//! key: [ ... ]          — multi-line array; `]` closes on its own line
//! key: {}  /  key: []   — empty compound, inline
//! :: value              — (inside an array) literal-string item
//! ```
//!
//! ## Structured errors
//!
//! The parser returns [`Error::Structured`] for every parse failure since
//! `0.1.5`. Each [`ErrorKind`] variant carries a 1-based `line` and a
//! byte-offset [`Span`] you can slice the original input with.
//!
//! ```text
//! use ktav::{parse, Error, ErrorKind};
//!
//! // The first line anchors the document as an Object; the malformed
//! // `port:8080` on line 2 then errors with a `MissingSeparatorSpace`
//! // (a first-line `port:8080` would now be a top-level Array
//! // bare-scalar item — spec § 5.0.1).
//! let src = "anchor: ok\nport:8080\n";
//! match parse(src) {
//!     Ok(_) => unreachable!(),
//!     Err(Error::Structured(ErrorKind::MissingSeparatorSpace { line, span, .. })) => {
//!         assert_eq!(line, 2);
//!         // The span covers the offending body chunk glued to the marker.
//!         assert_eq!(span.slice(src), Some("8080"));
//!     }
//!     Err(other) => panic!("unexpected error: {other:?}"),
//! }
//! ```
//! (See `tests/error_accessors.rs` for the executed test.)
//!
//! ## Example
//!
//! See [`tests/doc_example.rs`](../tests/doc_example.rs) for the executed
//! version of this snippet — it exercises the full parse → struct → render
//! → parse round-trip:
//!
//! ```text
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Upstream {
//!     host: String,
//!     port: u16,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Config {
//!     port: u16,
//!     upstreams: Vec<Upstream>,
//! }
//!
//! let text = "\
//! port: 8080
//!
//! upstreams: [
//!     {
//!         host: a.example
//!         port: 1080
//!     }
//!     {
//!         host: b.example
//!         port: 1080
//!     }
//! ]
//! ";
//! let cfg: Config = ktav::from_str(text).unwrap();
//! assert_eq!(cfg.port, 8080);
//! assert_eq!(cfg.upstreams.len(), 2);
//!
//! let back = ktav::to_string(&cfg).unwrap();
//! let round: Config = ktav::from_str(&back).unwrap();
//! assert_eq!(cfg, round);
//! ```
#![allow(clippy::module_inception)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
pub mod parser;
pub mod render;
pub mod ser;
pub mod thin;
pub mod value;

pub use error::{CompoundKind, ConflictKind, Error, ErrorKind, Result, Span};
pub use thin::{parse_events, ParseEvent};
pub use value::{ObjectMap, Value};

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Parse a Ktav document from a string into a raw [`Value`]. Useful when
/// you want to inspect or manipulate the document generically. For
/// deserializing into a user type, prefer [`from_str`].
pub fn parse(text: &str) -> Result<Value> {
    parser::parse_str(text)
}

/// Parse a Ktav document from a string and deserialize it into `T`. Uses
/// the zero-copy event path: the parser tokenizes the document into a
/// flat `Vec<Event>` (object keys and single-line scalars are borrowed
/// directly from `s`), and serde walks that vec linearly without ever
/// materialising a tree. Compound nesting is bracketed by
/// `BeginObject`/`EndObject` events instead of nested allocations.
pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    // Pre-size the bump arena to avoid re-allocations during event parsing.
    // Each Event is 24 bytes; the pre-allocated count is ~text.len()/4.
    // Adding overhead for the bump metadata and per-object-level BumpVecs.
    let arena_bytes = (s.len() / 4).saturating_mul(24) + 4096;
    let bump = bumpalo::Bump::with_capacity(arena_bytes);
    let events = thin::parse_events_raw(s, &bump)?;
    let mut cursor = thin::EventCursor::new(&events);
    T::deserialize(thin::EventDeserializer::new(&mut cursor))
}

/// Parse a Ktav document from a file path and deserialize it into `T`.
pub fn from_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let text = fs::read_to_string(path)?;
    from_str(&text)
}

/// Serialize `value` as a Ktav document string. Uses the direct text
/// serializer — no `Value` intermediate.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    ser::to_string(value)
}

/// Serialize `value` as a Ktav document and write it to `path`.
pub fn to_file<T: ?Sized + Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<()> {
    let text = to_string(value)?;
    fs::write(path, text)?;
    Ok(())
}

/// Render a [`Value`] with **every scalar coerced to a String** —
/// typed integers (`:i`), typed floats (`:f`), booleans, and null
/// are flattened to their textual form and emitted via the raw-
/// marker `::`. Compounds (Object / Array) preserve their structure;
/// only leaf scalars are coerced. The output round-trips back
/// through the parser as the same set of String scalars.
///
/// Useful for "everything is a string" dumps — e.g. for downstream
/// consumers that don't understand typed markers, or for diff-
/// friendly canonical text.
pub fn to_string_force_strings(value: &Value) -> Result<String> {
    render::to_string_force_strings(value)
}

/// Emit a canonical Ktav serialisation of `value` (spec § 5.9).
///
/// The top-level value must be an Object or an Array (§ 5.0.1).
/// Returns an error for any other variant, or if a String contains a
/// `CR` byte (not representable in canonical form, § 5.9.7).
///
/// Two writer-conforming implementations fed the same [`Value`] MUST produce
/// identical output (§ 8.2).
pub fn emit_canonical(value: &Value) -> Result<String> {
    render::emit_canonical(value)
}
