//! The unified `Error` enum plus its std / serde `Error` trait impls.

use std::fmt::{self, Display};
use std::io;

/// The single error type returned by every entry point.
#[derive(Debug)]
pub enum Error {
    /// I/O error while reading from disk.
    Io(io::Error),
    /// Syntax error in a Ktav document; message includes the line number.
    ///
    /// The message may carry a category prefix in the form
    /// `"Line N: Category: ..."` for specific error classes such as
    /// `InvalidTypedScalar` (a malformed body after a `:i` / `:f`
    /// typed-scalar marker). Callers generally match on the category via
    /// `str::contains` — see `tests/edge_cases/typed_markers.rs` for
    /// examples.
    Syntax(String),
    /// A custom message produced by `serde` during (de)serialization —
    /// for example a type mismatch or a missing field.
    Message(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Syntax(m) => write!(f, "Syntax error: {}", m),
            Error::Message(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
