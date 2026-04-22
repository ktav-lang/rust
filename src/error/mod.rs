//! Error types.

pub mod error;
pub mod result;

pub use error::Error;
pub use result::Result;

#[cfg(test)]
mod tests;
