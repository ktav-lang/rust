//! The crate-level entry point for parsing a `&str` into a [`Value`].

use memchr::{memchr, memchr2};

use crate::error::Error;
use crate::value::Value;

use super::parser::Parser;

/// Parse Ktav text into a [`Value`]. Iterates the input via byte scanning
/// — each iteration yields a `&str` slice into the original buffer, so no
/// per-line `String` allocation occurs.
///
/// Per spec § 3.2, three line terminators are recognised:
/// - `LF`   (`0x0A`)
/// - `CR`   (`0x0D`)
/// - `CR LF` (`0x0D 0x0A`)
///
/// A bare `CR` (not followed by `LF`) is its own terminator.
pub(crate) fn parse_str(text: &str) -> Result<Value, Error> {
    parse_str_impl(text, false)
}

/// Strict variant of [`parse_str`]: lossy scalars are rejected with
/// [`crate::ErrorKind::LossyScalar`]. See [`crate::parse_strict`].
pub(crate) fn parse_str_strict(text: &str) -> Result<Value, Error> {
    parse_str_impl(text, true)
}

fn parse_str_impl(text: &str, strict: bool) -> Result<Value, Error> {
    let mut parser = Parser::new(strict);
    let bytes = text.as_bytes();

    // Fast path for the overwhelmingly common case: LF-only input
    // (no CR bytes). This avoids the per-byte branch on `\r` in the
    // inner scan loop and lets the compiler emit a tighter scan.
    if memchr(b'\r', bytes).is_none() {
        // LF-only fast path: memchr-backed `\n` splitting that mirrors
        // `text.split('\n')` exactly (including the trailing empty line
        // after a final `\n`).
        let mut line_start: usize = 0;
        let mut line_num: usize = 0;
        loop {
            let end = memchr(b'\n', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            line_num += 1;
            let line: &str = &text[line_start..end];
            parser.handle_line(line, line_num, line_start as u32)?;
            if end == bytes.len() {
                break;
            }
            line_start = end + 1;
        }
        return parser.finish(bytes.len() as u32);
    }

    let mut line_start: usize = 0;
    let mut line_num: usize = 0;
    while line_start <= bytes.len() {
        if line_start == bytes.len() {
            break;
        }
        // Scan for the next line terminator: CR, LF, or CR LF — via
        // SIMD memchr2.
        let pos = memchr2(b'\n', b'\r', &bytes[line_start..])
            .map(|p| line_start + p)
            .unwrap_or(bytes.len());
        let content_end = pos;
        let next_start = if pos < bytes.len() {
            if bytes[pos] == b'\r' {
                // CR LF → one terminator; bare CR → one terminator.
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'\n' {
                    pos + 2
                } else {
                    pos + 1
                }
            } else {
                // LF
                pos + 1
            }
        } else {
            // EOF without terminator — sentinel to exit after this iteration.
            bytes.len() + 1
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
