//! Line-oriented Ktav parser. See [`crate::parse`] for the public entry point.
//!
//! The tree below groups the parser by what each part does rather than
//! by file: `syntax` classifies what a line says, `build` turns that
//! into a `Value`, `parser` is the state machine that drives both,
//! `inline` handles `{ ... }` and `[ ... ]`, and `fmt_parser` is the
//! trivia-preserving fork the formatter uses. The `use` declarations
//! below re-export every module at its original path, so the rest of the
//! crate still writes `crate::parser::classify` and friends.

mod build;
pub(crate) mod fmt_parser;
pub(crate) mod inline;
mod parser;
mod syntax;

use build::frame;
pub(crate) use build::{insert, validate};
pub(crate) use syntax::classify;
use syntax::{bracket, collecting, value_start};

pub(crate) use parser::parse_str::{parse_str, parse_str_strict};

/// Byte length of the byte-order mark (U+FEFF) to skip at the very
/// start of a document: 3 (its UTF-8 encoding `EF BB BF`) when the
/// text begins with it, 0 otherwise. Spec § 3.1: a parser-conforming
/// implementation MUST skip exactly one leading U+FEFF before any
/// other byte is examined; a U+FEFF anywhere else is ordinary
/// content (§ 3.3 does not classify it as whitespace).
pub(crate) fn leading_bom_len(text: &str) -> usize {
    if text.as_bytes().starts_with(b"\xEF\xBB\xBF") {
        3
    } else {
        0
    }
}

#[cfg(test)]
mod tests;
