//! Key-context escape processing, colon scanning, and dotted-key splitting (spec section 3.7 + 5.3).

use std::borrow::Cow;

use crate::error::{Error, Span};
use crate::whitespace::{inline_whitespace_ascii, is_inline_whitespace};
use memchr::memchr2;

use super::escapes::process_escapes;

#[cfg(test)]
use super::ix_probe;

// ---------------------------------------------------------------------------
// Key-context escape processing (spec 0.6.0 § 3.7 + § 5.3)
// ---------------------------------------------------------------------------

/// Outcome of scanning key text for the pair separator (spec 0.7 § 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColonScan {
    /// Byte offset of the first unescaped `:` outside quoted segments.
    Found(usize),
    /// No unescaped `:` anywhere outside quoted segments.
    Absent,
    /// A quoted segment opened at a segment-start position and never
    /// closed: the rest of the line, colon included, is segment
    /// content (§ 5.3.3 "Unterminated quoted segments").
    UnterminatedQuote,
}

/// True iff `b` opens a quoted key segment (spec 0.7 § 5.3.3).
pub(super) fn is_quote_byte(b: u8) -> bool {
    b == b'"' || b == b'\'' || b == b'`'
}

/// True iff `bytes` contains any quote byte. Cheap early-out for the
/// scanners: without a quote byte, quoted-segment tracking cannot
/// change the outcome, so callers keep their SIMD fast paths.
///
/// R10-F1: one call hands a slice of up to `bytes.len()` bytes to each
/// of the three `contains` passes, so the true byte-view count of one
/// call is between 1x and 3x the length recorded by
/// `ix_probe::record_quote_prescan` (test-only, not visible outside
/// `#[cfg(test)]` builds); comparisons across parser versions hold
/// because the factor is the same.
pub(crate) fn has_quote_bytes(bytes: &[u8]) -> bool {
    #[cfg(test)]
    ix_probe::record_quote_prescan(bytes.len());
    bytes.contains(&b'"') || bytes.contains(&b'\'') || bytes.contains(&b'`')
}

/// Skip line-bounded § 3.3 whitespace; LF/CR cannot occur here (lines
/// are
/// pre-split). Returns the index of the first non-whitespace byte at or
/// after `i`. Classification MUST NOT delegate to a host
/// Unicode-whitespace primitive (§ 3.3); it comes from the shared § 3.3
/// module ([`crate::whitespace`]) via [`inline_whitespace_at`].
pub(super) fn skip_segment_ws(s: &str, mut i: usize) -> usize {
    while let Some(len) = inline_whitespace_at(s, i) {
        i += len;
    }
    i
}

/// Byte length of the § 3.3 whitespace code point starting at byte
/// offset `i`, or `None` if the code point at `i` is not whitespace.
/// Classification comes from the shared § 3.3 module
/// ([`crate::whitespace`]) — never a host Unicode-whitespace primitive
/// (`char::is_whitespace` MUST NOT be delegated to, even though it
/// matches the list today): the four single-byte ASCII members take the
/// byte fast path ([`inline_whitespace_ascii`]), the decoded-char check
/// is [`is_inline_whitespace`] (the § 3.3 set minus LF/CR; LF/CR cannot
/// occur here — lines are pre-split per § 3.2). A mid-code-point
/// (non-boundary) offset is never whitespace: callers scan
/// byte-at-a-time and may sit on a continuation byte of a
/// non-whitespace character.
pub(super) fn inline_whitespace_at(s: &str, i: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let b = *bytes.get(i)?;
    if inline_whitespace_ascii(b) {
        return Some(1);
    }
    if b < 0x80 || !s.is_char_boundary(i) {
        return None;
    }
    let ch = s[i..].chars().next()?;
    if is_inline_whitespace(ch) {
        Some(ch.len_utf8())
    } else {
        None
    }
}

/// Scan key text for the pair separator, treating quoted-segment
/// content as opaque (spec 0.7 § 4's separator-scanning rule: the
/// separator is the first unescaped `:`, with the content of any
/// `<quoted-segment>` encountered along the way treated as opaque).
/// A quoted segment only OPENS at a segment-start position: the very
/// start of the text, or immediately after an unescaped `.` (plus
/// line-bounded whitespace, since `<raw-segment> ::= (ws) <segment>
/// (ws)`).
///
/// R10-F1: candidate-driven instead of a full quote-prescan plus a
/// byte-at-a-time walk over the WHOLE pair text. The escape-aware fast
/// scan (`find_unescaped_colon_fast`, memchr) proposes the first
/// unescaped `:` candidate; only that candidate's KEY PREFIX
/// (`s[..cand]`, never the value) is checked for quote-opacity. A
/// candidate whose prefix ends outside every quoted segment is the
/// separator; a candidate inside a closed-or-open quoted span resumes
/// after the span's closer (never at a segment start); a span that
/// never closes is `UnterminatedQuote`. The full quote-aware walk
/// ([`scan_unescaped_colon_slow`]) remains only for the corner where
/// no unescaped candidate exists at all but quote bytes are present —
/// the `Absent` vs `UnterminatedQuote` distinction there does not
/// depend on finding a colon. Escape-awareness is carried by the fast
/// scan itself (`\` consumes the next byte); the prefix walk mirrors
/// the slow machine byte for byte over its range.
pub(crate) fn scan_unescaped_colon(s: &str) -> ColonScan {
    let bytes = s.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = find_unescaped_colon_fast(&s[from..]) {
        let cand = from + rel;
        match key_prefix_quote_state(s, from, cand) {
            KeyPrefixQuote::Opaque => return ColonScan::Found(cand),
            KeyPrefixQuote::OpenSegment { resume } => from = resume,
            KeyPrefixQuote::Unterminated => return ColonScan::UnterminatedQuote,
        }
    }
    // No further escape-aware colon candidate anywhere. Without quote
    // bytes nothing can be unterminated; with them the slow walk
    // decides Absent vs UnterminatedQuote (it cannot return `Found`:
    // the fast scan just proved no unescaped `:` exists).
    if has_quote_bytes(bytes) {
        return scan_unescaped_colon_slow(s);
    }
    ColonScan::Absent
}

/// Quote-state of `s[from..cand]` at the candidate offset `cand`
/// (R10-F1): is a `<quoted-segment>` open there, did one open at a
/// segment-start position and never close, or is the candidate outside
/// every segment? Walks ONLY the key prefix `[from, cand)` — the
/// value's own quotes and colons are irrelevant to where the key ends.
/// `from` is either 0 (segment start, the whole text's beginning) or
/// one past a quoted span's closer (NOT a segment start) — the same
/// resume state [`scan_unescaped_colon_slow`] carries.
enum KeyPrefixQuote {
    /// No quoted segment is open at the candidate offset.
    Opaque,
    /// A quoted segment is open at the candidate; `resume` is one past
    /// its closing quote.
    OpenSegment { resume: usize },
    /// A segment-start quote whose span never closes: the candidate is
    /// inside it (§ 5.3.3 "Unterminated quoted segments").
    Unterminated,
}

fn key_prefix_quote_state(s: &str, from: usize, cand: usize) -> KeyPrefixQuote {
    let bytes = s.as_bytes();
    // No quote byte in the prefix: no segment can open (openings need a
    // quote byte at a segment start) and none can be unterminated.
    if !has_quote_bytes(&bytes[from..cand]) {
        return KeyPrefixQuote::Opaque;
    }
    let mut i = from;
    let mut seg_start = from == 0;
    while i < cand {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < cand && is_quote_byte(bytes[i]) {
                return match quoted_span_end(bytes, i) {
                    // The colon is inside this span iff the span reaches
                    // past it (the closer cannot sit ON `cand` — that
                    // byte is `:`). A span closing before the candidate
                    // just resumes the walk after its closer.
                    Some(end) if end > cand => KeyPrefixQuote::OpenSegment { resume: end + 1 },
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => KeyPrefixQuote::Unterminated,
                };
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape lead: consume the escaped byte too (a lone trailing `\` overshoots the prefix, which the loop guard makes safe — the colon at `cand` is unescaped, so this cannot hide it)
            b'.' => {
                seg_start = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    KeyPrefixQuote::Opaque
}

/// The pre-R10-F1 full quote-aware walk (spec 0.7 § 4 + § 5.3.3), kept
/// for `scan_unescaped_colon`'s no-candidate corner: byte-at-a-time
/// segment tracking over the WHOLE text. Precondition: the caller has
/// established that no unescaped `:` candidate exists
/// ([`find_unescaped_colon_fast`] over the whole text returned `None`),
/// so this can only return `Absent` or `UnterminatedQuote`.
fn scan_unescaped_colon_slow(s: &str) -> ColonScan {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut seg_start = true; // position 0 is a segment start
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                return match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => ColonScan::UnterminatedQuote,
                };
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape lead: consume the escaped byte too (a lone trailing `\` overshoots `bytes.len()`, which the loop guard makes safe — scan just ends, as in `find_unescaped_colon`)
            b'.' => {
                seg_start = true;
                i += 1;
            }
            b':' => return ColonScan::Found(i),
            _ => i += 1,
        }
    }
    ColonScan::Absent
}

/// SIMD-accelerated escape-aware `:` scan: memchr2 jumps to the next
/// candidate byte (`\` or `:`). When we land on `\` we skip the
/// escaped byte and resume; when we land on `:` we return it.
fn find_unescaped_colon_fast(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let rel = memchr2(b'\\', b':', &bytes[i..])?;
        let abs = i + rel;
        if bytes[abs] == b':' {
            return Some(abs);
        }
        // `\` — skip the escaped byte (whatever it is). At EOL just
        // stop: the caller will report a key-without-separator error
        // of its own.
        i = abs + 2;
    }
    None
}

/// Find the byte offset of the first **unescaped** `:` in `s`. Returns
/// `None` if every `:` is preceded by `\`, or if a quoted key segment
/// opened at a segment-start position and swallowed the rest of the
/// line (spec 0.7 § 5.3.3 "Unterminated quoted segments" — callers
/// needing to distinguish that case use [`scan_unescaped_colon`]).
/// Spec 0.6.0 § 5.3 — the pair separator is the first unescaped `:`
/// (or `::`).
///
/// `\` consumes the next byte; pairs of `\\` reset to "no pending
/// escape". This intentionally does not validate the escape sequence —
/// validation is deferred to `decode_key_segment` so a glued
/// `BadEscapeSequence` error fires at the right call site.
pub(crate) fn find_unescaped_colon(s: &str) -> Option<usize> {
    match scan_unescaped_colon(s) {
        ColonScan::Found(p) => Some(p),
        _ => None,
    }
}

/// Split a key string into dotted segments at **unescaped** `.` bytes
/// (spec 0.6.0 § 4 / § 5.3). The returned slices reference the input;
/// callers run `decode_key_segment` on each segment to materialise the
/// final byte form.
pub(crate) fn split_key_path(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    if !has_quote_bytes(bytes) {
        // SIMD-accelerated escape-aware split: memchr2 jumps to the next
        // candidate byte (`\` or `.`). On `\` skip the escaped byte; on
        // `.` cut a segment.
        let mut start = 0;
        let mut i = 0;
        while i < bytes.len() {
            let rel = match memchr2(b'\\', b'.', &bytes[i..]) {
                Some(p) => p,
                None => break,
            };
            let abs = i + rel;
            if bytes[abs] == b'.' {
                out.push(&s[start..abs]);
                start = abs + 1;
                i = abs + 1;
            } else {
                // `\` — escape consumes the next byte.
                if abs + 1 < bytes.len() {
                    i = abs + 2;
                } else {
                    // Lone trailing `\` — let decoding report it.
                    i = abs + 1;
                }
            }
        }
        out.push(&s[start..]);
        return out;
    }
    // Slow path (quote bytes present): quoted segments are opaque to
    // `.` splitting (spec 0.7 § 5.3.3). A segment opens only at a
    // segment-start position: position 0 or after an unescaped `.`
    // (plus line-bounded whitespace).
    let mut start = 0;
    let mut i = 0;
    let mut seg_start = true;
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => {
                        // Defensive: callers only reach here with
                        // well-formed key text (the colon scan already
                        // proved the separator exists), so an unterminated
                        // span here keeps the remainder as one segment.
                        break;
                    }
                }
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape consumes the next byte
            b'.' => {
                out.push(&s[start..i]);
                start = i + 1;
                i += 1;
                seg_start = true;
            }
            _ => i += 1,
        }
    }
    out.push(&s[start..]);
    out
}

/// Returns `true` iff the key string contains no `.` separator at any
/// unescaped position. Used by callers that take a non-dotted fast
/// path; callers still need `decode_key_segment` to materialise the
/// final byte form when the segment contains a `\`.
pub(crate) fn key_is_single_segment(s: &str) -> bool {
    let bytes = s.as_bytes();
    if !has_quote_bytes(bytes) {
        // SIMD-accelerated escape-aware scan via memchr2.
        let mut i = 0;
        while i < bytes.len() {
            let rel = match memchr2(b'\\', b'.', &bytes[i..]) {
                Some(p) => p,
                None => return true,
            };
            let abs = i + rel;
            if bytes[abs] == b'.' {
                return false;
            }
            // `\` — skip the escaped byte.
            i = abs + 2;
        }
        return true;
    }
    // Slow path (quote bytes present): dots inside quoted segments are
    // not separators (spec 0.7 § 5.3.3). Segment-start tracking as in
    // [`split_key_path`].
    let mut i = 0;
    let mut seg_start = true;
    while i < bytes.len() {
        if seg_start {
            i = skip_segment_ws(s, i);
            if i < bytes.len() && is_quote_byte(bytes[i]) {
                match quoted_span_end(bytes, i) {
                    Some(end) => {
                        i = end + 1;
                        seg_start = false;
                        continue;
                    }
                    None => {
                        // Defensive (see `split_key_path`): treat the
                        // unterminated span as one segment.
                        return true;
                    }
                }
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => i += 2, // escape consumes the next byte
            b'.' => return false,
            _ => i += 1,
        }
    }
    true
}

/// Find the matching unescaped closer for a quoted key segment opened
/// at `open_idx` (spec 0.7 § 5.3.3). `bytes[open_idx]` must be `"`,
/// `'`, or `` ` ``. Backslash-escaped bytes are skipped (`\"` does not
/// close a `"` segment). Returns `None` when no closer exists before
/// the end of input — the segment then swallows the entire remainder.
pub(crate) fn quoted_span_end(bytes: &[u8], open_idx: usize) -> Option<usize> {
    let quote = bytes[open_idx];
    let mut j = open_idx + 1;
    while j < bytes.len() {
        if bytes[j] == b'\\' {
            j += 2;
            continue;
        }
        if bytes[j] == quote {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Decode a single key segment per spec 0.7 § 3.7 / § 5.3.3. The
/// segment must not contain unescaped `.` or `:` (callers are expected
/// to split on those first). A segment whose first byte is `"`, `'`,
/// or `` ` `` is a `<quoted-segment>` (§ 5.3.3): the outer delimiter
/// pair is stripped and `process_escapes` is applied to the interior
/// only, with NO trimming of the interior (quoted content is never
/// trimmed — `" a "` decodes to the 3-char key ` a `). Callers must
/// have validated the segment with `validate::check_key` first.
/// Returns a borrowed slice when there is no escape at all: a quoted
/// segment with no `\` in its interior borrows the interior slice, and
/// a bare segment with no `\` borrows the input as-is — no allocation.
/// Any `\` decodes via [`process_escapes`] and yields an owned String,
/// including `\.`/`\:`, which decode to literal `.`/`:` that are NOT
/// separators. Errors with `BadEscapeSequence` on an unknown `\X`;
/// identical escape table to [`process_escapes`].
pub(crate) fn decode_key_segment<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
) -> Result<Cow<'a, str>, Error> {
    // Quoted segment (spec 0.7 § 5.3.3): strip the outer delimiter pair,
    // decode the interior only. Callers must have validated the segment
    // with `check_key` (properly closed, nothing after the closer).
    if !input.is_empty() {
        let first = input.as_bytes()[0];
        if first == b'"' || first == b'\'' || first == b'`' {
            debug_assert!(input.len() >= 2 && input.as_bytes()[input.len() - 1] == first);
            let interior = &input[1..input.len() - 1];
            if !interior.as_bytes().contains(&b'\\') {
                return Ok(Cow::Borrowed(interior));
            }
            return process_escapes(interior, line_num, span);
        }
    }
    // Bare segment path (unchanged)
    if !input.as_bytes().contains(&b'\\') {
        return Ok(Cow::Borrowed(input));
    }
    process_escapes(input, line_num, span)
}
