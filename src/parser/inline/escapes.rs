//! Escape processing for inline scalar values (spec section 3.7).

use std::borrow::Cow;

use crate::error::{Error, ErrorKind, Span};

// ---------------------------------------------------------------------------
// Escape processing (section 3.7)
// ---------------------------------------------------------------------------

/// One recognised § 3.7 escape, scanned at `bytes[i]` (which must be
/// `\`). The variant determines the sequence's total byte length.
pub(super) enum RecognisedEscape {
    /// Single-character escape: the char to emit; total length is 2 bytes.
    Simple(char),
    /// `\uXXXX` mapping to an ordinary (non-surrogate) code point; total
    /// length is 6 bytes.
    Unicode(char),
    /// `\uXXXX\uXXXX` surrogate pair, combined; total length is 12 bytes.
    SurrogatePair(char),
}

impl RecognisedEscape {
    /// Total byte length of the escape sequence (2 for simple escapes,
    /// 6 for `\uXXXX`, 12 for a full surrogate pair).
    pub(super) fn len(&self) -> usize {
        match self {
            RecognisedEscape::Simple(_) => 2,
            RecognisedEscape::Unicode(_) => 6,
            RecognisedEscape::SurrogatePair(_) => 12,
        }
    }
}

/// Scan one escape sequence. Returns the offending sequence text (the
/// exact `BadEscapeSequence` payload) on any unrecognised or malformed
/// form; the recognised table and every error string are identical to
/// `process_escapes`' decode — this is the single source of both.
pub(super) fn scan_escape(bytes: &[u8], i: usize) -> Result<RecognisedEscape, String> {
    if i + 1 >= bytes.len() {
        // Backslash at end of line
        return Err("\\<end-of-line>".to_string());
    }
    let next = bytes[i + 1];
    let simple = match next {
        b'\\' => Some('\\'),
        b',' => Some(','),
        b'}' => Some('}'),
        b']' => Some(']'),
        b'{' => Some('{'),
        b'[' => Some('['),
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b'.' => Some('.'),
        b':' => Some(':'),
        b'"' => Some('"'),
        b'\'' => Some('\''),
        b'`' => Some('`'),
        _ => None,
    };
    if let Some(ch) = simple {
        return Ok(RecognisedEscape::Simple(ch));
    }
    if next == b'u' {
        // `\uXXXX`: exactly four ASCII hex digits (spec 0.7 § 3.7.1).
        // Validate up-front so nothing is consumed on a malformed escape.
        if i + 6 > bytes.len() || !bytes[i + 2..i + 6].iter().all(|b| b.is_ascii_hexdigit()) {
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        // All-ASCII validated slice: `from_utf8` cannot fail.
        let hex = std::str::from_utf8(&bytes[i + 2..i + 6]).unwrap();
        let value = u32::from_str_radix(hex, 16).expect("4 ASCII hex digits");
        if (0xD800..=0xDBFF).contains(&value) {
            // High surrogate: must pair with an immediately following
            // low-surrogate `\uXXXX` (12 bytes total).
            if i + 12 <= bytes.len()
                && bytes[i + 6] == b'\\'
                && bytes[i + 7] == b'u'
                && bytes[i + 8..i + 12].iter().all(|b| b.is_ascii_hexdigit())
            {
                let low_hex = std::str::from_utf8(&bytes[i + 8..i + 12]).unwrap();
                let low = u32::from_str_radix(low_hex, 16).expect("4 ASCII hex digits");
                if (0xDC00..=0xDFFF).contains(&low) {
                    let combined = 0x10000 + (value - 0xD800) * 0x400 + (low - 0xDC00);
                    // 0x10000..=0x10FFFF by construction.
                    let ch = char::from_u32(combined).expect("valid surrogate pair");
                    return Ok(RecognisedEscape::SurrogatePair(ch));
                }
            }
            // Lone high surrogate (end of input, non-`\u` text, malformed
            // second escape, or non-low value).
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        if (0xDC00..=0xDFFF).contains(&value) {
            // Lone low surrogate.
            return Err(render_malformed_unicode_escape(bytes, i));
        }
        // Ordinary BMP code point.
        let ch = char::from_u32(value).expect("BMP non-surrogate value");
        return Ok(RecognisedEscape::Unicode(ch));
    }
    // Invalid escape
    if next < 0x80 {
        Err(format!("\\{}", next as char))
    } else {
        Err(format!("\\<0x{:02X}>", next))
    }
}

/// Process escape sequences in an inline scalar value.
///
/// Recognised sequences (14, spec 0.7 § 3.7 / § 3.7.1):
///   `\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:`,
///   `\"`, `\'`, `` \` `` and `\uXXXX` (exactly four hex digits,
///   case-insensitive). A `\uXXXX` in the high-surrogate range must be
///   immediately followed by a low surrogate `\uXXXX` (combined into a
///   single scalar value); lone surrogates and malformed `\u` forms are
///   `BadEscapeSequence`. The three quote escapes exist for the quoted
///   key form (§ 5.3.3) but are recognised in every context where
///   escapes are recognised at all (§ 3.7).
/// Any other `\X` is a `BadEscapeSequence` error.
///
/// Returns `Cow::Borrowed` iff the input contains no `\` at all;
/// `Cow::Owned` iff at least one recognised escape was decoded.
pub(crate) fn process_escapes<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
) -> Result<Cow<'a, str>, Error> {
    // Fast path: if no backslash, the input is already clean — return it
    // borrowed, no allocation at all.
    if !input.as_bytes().contains(&b'\\') {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let esc = match scan_escape(bytes, i) {
                Err(sequence) => {
                    return Err(Error::Structured(ErrorKind::BadEscapeSequence {
                        line: line_num as u32,
                        span,
                        sequence,
                    }));
                }
                Ok(esc) => esc,
            };
            let len = esc.len();
            match esc {
                RecognisedEscape::Simple(ch)
                | RecognisedEscape::Unicode(ch)
                | RecognisedEscape::SurrogatePair(ch) => out.push(ch),
            }
            i += len;
        } else {
            // Safe because we're iterating over valid UTF-8
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    Ok(Cow::Owned(out))
}

/// Render a malformed `\u` escape for the error payload: `\u` followed
/// by up to four raw bytes from the input; ASCII bytes as chars, others
/// as `<0xXX>`.
fn render_malformed_unicode_escape(bytes: &[u8], i: usize) -> String {
    let end = usize::min(i + 6, bytes.len());
    let mut seq = String::from("\\u");
    for &b in &bytes[i + 2..end] {
        if b < 0x80 {
            seq.push(b as char);
        } else {
            seq.push_str(&format!("<0x{:02X}>", b));
        }
    }
    seq
}
