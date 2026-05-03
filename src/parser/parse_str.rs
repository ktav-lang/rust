//! The crate-level entry point for parsing a `&str` into a [`Value`].

use crate::error::Error;
use crate::value::Value;

use super::parser::Parser;

/// Parse Ktav text into a [`Value`]. Iterates the input via
/// [`str::lines`] — each iteration yields a `&str` slice into the
/// original buffer, so no per-line `String` allocation occurs.
///
/// In addition to the per-line pointer, we maintain a `cumulative_byte`
/// counter that tracks the byte offset of the current line's first
/// byte inside `text`. The parser uses this to compute byte-offset
/// [`crate::error::Span`]s for every emitted [`crate::error::ErrorKind`].
pub(crate) fn parse_str(text: &str) -> Result<Value, Error> {
    let mut parser = Parser::new();
    // Walk lines while tracking the byte offset of the line start. We
    // can't use `str::lines()` here because it strips the terminator
    // and we need to know the terminator width to advance the counter
    // (LF = 1, CRLF = 2, none-at-EOF = 0).
    let bytes = text.as_bytes();
    let mut line_start: usize = 0;
    let mut line_num: usize = 0;
    while line_start <= bytes.len() {
        // Find the next '\n' (if any).
        let nl = bytes[line_start..].iter().position(|&b| b == b'\n');
        let (end, next_start) = match nl {
            Some(rel) => (line_start + rel, line_start + rel + 1),
            None => {
                if line_start == bytes.len() {
                    break;
                }
                (bytes.len(), bytes.len() + 1) // sentinel to exit after this iteration
            }
        };
        // Strip a trailing '\r' from the line content (so logical
        // content matches `str::lines()` semantics).
        let content_end = if end > line_start && bytes[end - 1] == b'\r' {
            end - 1
        } else {
            end
        };
        // SAFETY: the source is `&str`, and we sliced at byte boundaries
        // determined by ASCII separators (`\n` / `\r`), which are always
        // on UTF-8 char boundaries.
        let line: &str = &text[line_start..content_end];
        line_num += 1;
        parser.handle_line(line, line_num, line_start as u32)?;
        line_start = next_start;
    }
    parser.finish(bytes.len() as u32)
}
