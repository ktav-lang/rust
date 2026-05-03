//! Error types.

pub mod error;
pub mod result;

pub use error::{CompoundKind, ConflictKind, Error, ErrorKind, Span};
pub use result::Result;

#[cfg(test)]
mod tests;
