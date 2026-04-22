//! The crate-level entry point for parsing a `&str` into a [`Value`].

use crate::error::Error;
use crate::value::Value;

use super::parser::Parser;

/// Parse Ktav text into a [`Value`]. Iterates the input via
/// [`str::lines`] — each iteration yields a `&str` slice into the
/// original buffer, so no per-line `String` allocation occurs.
pub(crate) fn parse_str(text: &str) -> Result<Value, Error> {
    let mut parser = Parser::new();
    for (idx, line) in text.lines().enumerate() {
        parser.handle_line(line, idx + 1)?;
    }
    parser.finish()
}
