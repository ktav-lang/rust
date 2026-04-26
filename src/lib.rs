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
//! ## Example
//!
//! See [`tests/doc_example.rs`](../tests/doc_example.rs) for the executed
//! version of this snippet — it exercises the full parse → struct → render
//! → parse round-trip:
//!
//! ```rust,ignore
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
mod thin;
pub mod value;

pub use error::{Error, Result};
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
    let bump = bumpalo::Bump::new();
    let events = thin::parse_events(s, &bump)?;
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
