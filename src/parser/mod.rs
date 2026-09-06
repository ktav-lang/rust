//! Line-oriented Ktav parser. See [`crate::parse`] for the public entry point.

mod bracket;
pub(crate) mod classify;
mod collecting;
mod frame;
pub(crate) mod inline;
mod insert;
mod parse_str;
mod parser;
pub(crate) mod validate;
mod value_start;

pub(crate) use parse_str::{parse_str, parse_str_strict};

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
