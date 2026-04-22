//! Convenience `Result` alias.

use super::error::Error;

/// Convenience alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
