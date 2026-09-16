//! Error types.

pub mod envelope;
pub mod error;
pub mod result;

pub use envelope::ErrorEnvelope;
pub use error::{CompoundKind, ConflictKind, Error, ErrorKind, ReasonCode, Span};
pub use result::Result;

#[cfg(test)]
mod tests;
