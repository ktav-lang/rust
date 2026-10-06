use crate::{Error, ErrorEnvelope, Span};

pub(super) fn envelope(err: &Error, source: &str) -> String {
    let mut env = ErrorEnvelope::from_error(err, source);
    // Spec § 6: every source-content parse error carries a 1-based line.
    // `from_error` takes the line from `Error::line()`, which is `None`
    // for an EOF-detected `UnclosedCompound` even though the error
    // carries an opener..EOF span. The C ABI layer has the source, so
    // the line is derived from the span's start (the opener line) here
    // rather than shipping a null. Errors with no source position (Io,
    // writer rejections) have no span and keep their honest nulls.
    if env.line.is_none() && !source.is_empty() {
        if let Some(span) = env.span {
            env.line = Some(line_of_offset(source.as_bytes(), span.start as usize));
        }
    }
    env.to_json()
}

/// Wrap a non-`ktav::Error` diagnostic (a serde_json failure, the
/// top-level check) so it leaves through the same envelope channel as
/// everything else.
pub(super) fn message_envelope(text: impl Into<String>, source: &str) -> String {
    envelope(&Error::Message(text.into()), source)
}

/// The raw-`InvalidUtf8` envelope for a source-bytes entry point whose
/// input failed `std::str::from_utf8` (spec § 3.1/§ 6.15): class
/// `InvalidUtf8` — never `Message` — with the § 6 location computed
/// over the original bytes. The span covers the offending sequence
/// (`Utf8Error::error_len`), or runs to end-of-input when the sequence
/// is truncated at EOF (`error_len` is `None`); the line counts § 3.2
/// terminators over the raw bytes, before any BOM removal or trimming.
pub(super) fn not_utf8_envelope(src: &[u8], err: std::str::Utf8Error) -> String {
    let valid_up_to = err.valid_up_to();
    let mut end = valid_up_to + err.error_len().unwrap_or(src.len() - valid_up_to);
    // An illegal lead byte is reported by `std` as a one-byte sequence,
    // but any following continuation bytes belong to the same ill-formed
    // subsequence; the spec (§ 6.15) fixes only the span's start, so the
    // span swallows them to cover the offending region as a whole.
    if err.error_len() == Some(1) {
        while src.get(end).is_some_and(|b| (0x80..=0xBF).contains(b)) {
            end += 1;
        }
    }
    let mut env = ErrorEnvelope::from_error(
        &Error::InvalidUtf8 { valid_up_to },
        // No valid source text exists, so `line_text` honestly comes out
        // null; `line` and `span` are filled in from the raw bytes below.
        "",
    );
    // `Span` is u32-based; a >4 GiB document is beyond what `Span` can
    // address anywhere in this crate, so saturate (same trade-off as
    // `Error::span`).
    let start = u32::try_from(valid_up_to).unwrap_or(u32::MAX);
    let end = u32::try_from(end).unwrap_or(u32::MAX);
    env.line = Some(line_of_offset(src, valid_up_to));
    env.span = Some(Span::new(start, end));
    env.body = Some(format!("input is not valid UTF-8: {err}"));
    env.to_json()
}

/// 1-based line number of raw byte offset `pos`, counting each LF, lone
/// CR and CR LF pair as exactly one terminator (spec § 3.2). A whole
/// terminator counts only when it ends at or before `pos`, so an offset
/// pointing at a terminator belongs to the line that terminator closes.
/// Counted over the raw bytes, before BOM removal or trimming (§ 6.15);
/// `pos` beyond the end clamps to the end.
pub(super) fn line_of_offset(bytes: &[u8], pos: usize) -> u32 {
    let mut line: u32 = 1;
    let mut i = 0;
    let bound = pos.min(bytes.len());
    while i < bound {
        match bytes[i] {
            b'\n' => {
                line += 1;
                i += 1;
            }
            b'\r' => {
                if bytes.get(i + 1) == Some(&b'\n') {
                    // CRLF is ONE terminator and counts only for offsets
                    // strictly after its LF byte.
                    if i + 2 <= pos {
                        line += 1;
                    }
                    i += 2;
                } else {
                    line += 1;
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    line
}
