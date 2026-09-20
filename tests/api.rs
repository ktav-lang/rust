//! The crate's public API exercised end to end: the text format
//! (parsing, keys and quoting, README/formatter surfaces), the
//! zero-copy thin event stream, Value-level semantics and wire
//! representation including the C ABI.

#[path = "fixtures/common/mod.rs"]
mod common;
#[path = "suite/api/text.rs"]
mod text;
#[path = "suite/api/thin.rs"]
mod thin;
#[path = "suite/api/value.rs"]
mod value;
