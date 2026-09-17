//! Separator classification for pair-lines (`::` raw-string marker vs plain `:`).

use crate::error::{Error, ErrorKind, Span};
use crate::whitespace::is_ktav_whitespace;

// ---------------------------------------------------------------------------
// Separator classification for pair-lines.
//
// Under 0.5.0, after the first `:` of `key: value`, the slice can begin
// with:
//   - `:` + whitespace/EOL   → raw-string marker `::`
//   - anything else          → plain `:` separator; the rest is the body
//
// The old `:i`/`:f` typed markers are removed in 0.5.0.
// ---------------------------------------------------------------------------

pub(in crate::parser) enum Separator<'a> {
    /// `::` followed by the body (leading whitespace not yet trimmed).
    Raw(&'a str),
    /// Plain `:` — body already lacks the leading separator char.
    Plain(&'a str),
}

/// Enforce the "separator followed by whitespace or end-of-line" rule
/// from spec § 5.3 / § 5.4. Returns `Err(MissingSeparatorSpace)` for the
/// `key:value` / `key::value` / `port:i42` / `ratio:f0.5` shapes where
/// the body is glued to the separator.
pub(in crate::parser) fn require_sep_end(
    rest: &str,
    line_num: usize,
    line_start: u32,
    column: u32,
    body_off: u32,
    trimmed_span: Span,
) -> Result<(), Error> {
    // § 3.3 fixed class; `rest` is a suffix of one pre-split line (§ 3.2),
    // so LF/CR cannot occur.
    if rest.is_empty() || rest.starts_with(is_ktav_whitespace) {
        Ok(())
    } else {
        // Compute span: from the marker through the glued body chars
        // (i.e. up to the trimmed-line end).
        let _ = line_start;
        let span = Span::new(body_off, trimmed_span.end);
        Err(Error::Structured(ErrorKind::MissingSeparatorSpace {
            line: line_num as u32,
            column,
            marker: ':',
            span,
        }))
    }
}
pub(in crate::parser) fn classify_separator(after_colon: &str) -> Separator<'_> {
    if let Some(rest) = after_colon.strip_prefix(':') {
        return Separator::Raw(rest);
    }
    // Under spec 0.5.0, `:i` and `:f` typed markers are removed.
    // Everything that isn't `::` is a plain `:` separator.
    Separator::Plain(after_colon)
}
