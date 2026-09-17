//! Splitting inline bodies on top-level commas, delimiter matching, and closer scanning.

use crate::error::{Error, ErrorKind, Span};
use crate::whitespace::is_inline_whitespace;

use super::scan_config::{FindFast, FindQ, ScanCfg, ScanFast, ScanQ, ScanStop, SplitFast, SplitQ};
use super::scanner::Scanner;
use crate::parser::inline::keys::{find_unescaped_colon, has_quote_bytes};

#[cfg(test)]
use crate::parser::inline::ix_probe;

// ---------------------------------------------------------------------------
// Splitting on top-level commas
// ---------------------------------------------------------------------------

/// Which inline-compound body is being split (spec 0.7 § 5.3.3
/// "Keys only"): in an object body, quotes at key-segment-start are
/// quoted KEYS and opaque to comma splitting; in an array body every
/// position is a value position, so quotes are ordinary content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineBody {
    Object,
    Array,
}

/// Precomputed (opener, closer) byte offsets of every nested compound
/// span inside one top-level inline body, relative to the body's first
/// byte, sorted by opener. Produced by the § 5.2 gate scan
/// ([`scan_inline_closer_with_bounds`]); consumed as a pure memo by
/// split's opener jump and the per-value tri-state dispatch.
///
/// Purity invariant: a recorded pair `(o, c)` exists iff
/// [`scan_inline_closer`] over the same span returns `Found(c - o)`.
/// Two properties make this hold:
///
/// 1. ENTRY STATE. The recording walk and the standalone scan are the
///    same machine, and at the span's opener their states agree: same
///    seeds, and the opener arm re-derives in_key/seg_start/value_start/
///    prev identically in BOTH modes (R8-F2 closed the former
///    fast-only residue quirks), and gated openers always run with
///    `raw = false`. On a quote-free slice the fast and quote-aware
///    machines are byte-identical — no quote byte can arm the
///    quote/segment arms and every remaining policy const coincides —
///    so a recorded span is pure for EITHER consumer slice shape (the
///    bounded value and the split suffix), whichever mode the slice's
///    own byte set selects at dispatch time. Recording is
///    unconditional at push; the pop-time gates below still kill any
///    span whose own stop byte says the standalone would disagree.
/// 2. STOP BYTE. The standalone scan's counter is the walk's shared
///    `depth` offset by the value captured at the opener, so it returns
///    to zero exactly where global `depth` returns to `entry - 1`. That
///    byte is the standalone's stop, and the top frame is always the
///    span in question there (frames opened above die at strictly
///    higher depths). A kind-matched closer at that byte is what the
///    pop records; a crossed closer there is where the standalone
///    stops with `NotFound`, so the frame is marked dead and never
///    records, however far the walk continues past it — a later
///    kind-matched pop cannot satisfy `depth == entry - 1` (its depth
///    is strictly below), which is the second gate on recording.
///
/// No `unsafe`: every pointer use is an `as usize`
/// comparison/subtraction on slices of the same allocation — every
/// slice passed to [`InlineBounds::known_closer`] descends from the
/// body the bounds were built over, so the subtraction stays within
/// one allocation.
#[derive(Clone, Copy)]
pub(crate) struct InlineBounds<'a> {
    pub(in crate::parser::inline) origin: usize, // address of the top body's first byte (coordinate origin)
    pairs: &'a [(usize, usize)],
}

impl<'a> InlineBounds<'a> {
    /// Empty bounds whose coordinate origin is `input` itself (gate
    /// scans and find: they never consult, and `input_off` becomes 0).
    pub(crate) fn for_input(input: &str) -> Self {
        InlineBounds {
            origin: input.as_ptr() as usize,
            pairs: &[],
        }
    }

    /// Bounds over `top`, offsets relative to `top`'s first byte.
    pub(crate) fn over<'b>(top: &str, pairs: &'b [(usize, usize)]) -> InlineBounds<'b> {
        InlineBounds {
            origin: top.as_ptr() as usize,
            pairs,
        }
    }

    /// If `s`'s first byte opens a recorded span whose closer lies
    /// within `s`, the closer's offset relative to `s`. `Some` here is
    /// byte-for-byte what the find-first dispatch's
    /// `find_matching_close` would return (identical machines; a
    /// recorded span provably contains no bad escape, the only
    /// find/scan divergence). `None` — no record, or the recorded
    /// closer falls beyond this body — means "not a memo for this
    /// slice": the caller MUST run the live dispatch.
    pub(crate) fn known_closer(&self, s: &str) -> Option<usize> {
        let off = s.as_ptr() as usize - self.origin;
        #[cfg(test)]
        if ix_probe::bypass_engaged() {
            return None;
        }
        #[cfg(test)]
        let idx = {
            // Counted twin of `binary_search_by_key` — the identical
            // probe closure `binary_search_by_key` itself uses — so
            // test builds see the same iteration sequence plus its
            // step count. Non-test builds keep the original line.
            let mut steps = 0usize;
            let idx = self.pairs.binary_search_by(|p| {
                steps += 1;
                p.0.cmp(&off)
            });
            ix_probe::record_kc_lookup(steps);
            idx
        };
        #[cfg(not(test))]
        let idx = self.pairs.binary_search_by_key(&off, |p| p.0);
        let idx = idx.ok()?;
        let rel = self.pairs[idx].1 - off;
        let hit = (rel < s.len()).then_some(rel);
        #[cfg(test)]
        if hit.is_some() {
            ix_probe::record_kc_hit();
        }
        hit
    }

    /// Closer absolute offset of a recorded span whose opener sits at
    /// absolute offset `abs`, for split's opener jump.
    pub(in crate::parser::inline) fn opener_close_at(&self, abs: usize) -> Option<usize> {
        #[cfg(test)]
        if ix_probe::bypass_engaged() {
            return None;
        }
        #[cfg(test)]
        let idx = {
            let mut steps = 0usize;
            let idx = self.pairs.binary_search_by(|p| {
                steps += 1;
                p.0.cmp(&abs)
            });
            ix_probe::record_oca_lookup(steps);
            idx
        };
        #[cfg(not(test))]
        let idx = self.pairs.binary_search_by_key(&abs, |p| p.0);
        let idx = idx.ok()?;
        #[cfg(test)]
        ix_probe::record_oca_hit();
        Some(self.pairs[idx].1)
    }
}

// R8-F6 measurement-only counters. Compiled ONLY under `cfg(test)`; a
// release build carries none of this. They record the deterministic
// operation counts of the `InlineBounds` index — binary-search steps
// per lookup site (calls/hits/total/max steps), the boundary sort
// (calls/elements/comparisons), and the recorded bodies themselves
// (count, total body bytes, total and max recorded pairs) — so the
// index's cost can be measured against the O(N) walk instead of
// guessed at. See the ix_probe_* tests in src/parser/tests.rs.
//
// R9-F2: the state is THREAD-LOCAL. The former process-global atomics
// let any concurrent parse bump the counters and let a probe's BYPASS
// flip flip the lookup path of every other test in the process, so no
// probe's snapshot window was isolated from the rest of the suite (a
// mutex held only by probe tests does not close that interleaving).
// Each thread now sees only its own window, and `set_bypass` returns
// an RAII guard so an unwinding test cannot leak the bypassed mode to
// the next test on the same thread (the timing probes are `#[ignore]`d
// and run with `--test-threads=1`).

// ---------------------------------------------------------------------------
// Splitting on top-level commas (wrappers over the shared machine)
// ---------------------------------------------------------------------------

/// Split `input` on unescaped `,` at nesting depth 0.
///
/// Unlike a naive brace-counting approach, this correctly handles the
/// section 5.8.5 "mid-value brace literal" rule: only a `{` or `[` that
/// is the first non-whitespace code point of a VALUE is skipped over as
/// a nested compound. A mid-scalar `{`/`[` is literal data even when it
/// happens to balance — it neither nests nor shields the commas inside
/// it, which still split. A value-start opener without a matching
/// closer is treated as literal (the value parser will handle it later
/// per the mid-value-brace rule).
///
/// In [`InlineBody::Object`] mode, quoted key segments (spec 0.7
/// § 5.3.3) are opaque to comma splitting — `{"a,b": 1, c: 2}`
/// splits into two pairs — while quotes in value positions are
/// ordinary content (`a: "x,y", b: 2` splits inside the quotes). An
/// unterminated quoted key segment raises `UnterminatedInlineCompound`.
///
/// `has_quotes` (R10-F1) is the ROOT body's quote presence threaded
/// down from the parse entry — computing it here would re-scan every
/// descendant subtree once per nesting level (`O(N·D)`).
pub(crate) fn split_top_level<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    body: InlineBody,
    bounds: InlineBounds<'_>,
    has_quotes: bool,
) -> Result<Vec<&'a str>, Error> {
    // R10-F1: `has_quotes` is threaded from the parse entry, where it
    // was computed once over the ROOT body. The soundness argument is
    // ASYMMETRIC (R11-F1): every slice split here descends from that
    // body, so root-level ABSENCE guarantees absence here and the
    // fast machine is always safe; root-level PRESENCE implies
    // nothing about this slice — a quote-free level may take the
    // quote-aware machine, which is safe only because the two
    // machines agree on quote-free input (R8-F2, including the last
    // segment: `EofAfterWsSkip` below pushes it exactly like
    // `Exhausted`). No per-level `has_quote_bytes` re-scan remains.
    if body == InlineBody::Array || !has_quotes {
        return Ok(split_top_level_fast(input, line_num, span, body, bounds));
    }

    // Slow path (object body with quote bytes): track key/value and
    // segment-start state so quoted KEYS are comma-opaque.
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, '_, SplitQ> =
        Scanner::new(input, open, close, object, line_num, span, bounds);
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::UnterminatedQuote => {
            // Unterminated quoted key segment (§ 5.3.3).
            Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            }))
        }
        ScanStop::EofAfterWsSkip => {
            // The skip may consume every byte that remains: after a
            // trailing comma (or a dotted-key `.`) only whitespace can
            // follow (R3-F1). Two possible outcomes:
            // Inline view trim: LF/CR cannot occur — this scanner works
            // within one § 3.2-pre-split line.
            if input[sc.seg_at..]
                .trim_matches(is_inline_whitespace)
                .is_empty()
            {
                // Only whitespace after the last comma: a valid
                // trailing comma — emit no final segment (the
                // callers treat an empty last segment identically).
                Ok(sc.segments)
            } else {
                // R11-F1: the remainder is the raw LAST segment — push
                // it exactly like the `Exhausted` branch (and hence
                // `split_top_level_fast`) does. Reaching this stop
                // proves the segment holds no unescaped `:`: the
                // seg-start block only runs while `in_key`, a
                // key-position colon clears `in_key`, and only a
                // body-depth comma can re-arm it (advancing `seg_at`
                // past itself), so a `.` that armed this segment start
                // plus a whitespace skip to EOF means the segment
                // simply has no separator. The callers'
                // `find_unescaped_colon_inline` then raises the § 6.12
                // missing-separator error (§ 5.8.2/§ 5.3: separator
                // finding precedes key validation) — the same verdict
                // the fast machine reaches for the same bytes. The
                // former EmptyKey here mis-categorized: the genuine
                // dotted key with an empty final segment (`a.: 1`) HAS
                // a separator, never reaches this stop, and stays
                // EmptyKey via `insert_value` (§ 6.5).
                sc.segments.push(&input[sc.seg_at..]);
                Ok(sc.segments)
            }
        }
        ScanStop::Exhausted => {
            // Last segment (after final comma, or the whole string if
            // no comma).
            sc.segments.push(&input[sc.seg_at..]);
            Ok(sc.segments)
        }
        ScanStop::Closer { .. } | ScanStop::BadEscape(_) => {
            unreachable!("split scanner cannot stop on a closer or bad escape")
        }
    }
}

/// Quote-free fast path for [`split_top_level`] (also serving object
/// bodies without quote bytes). Implements the § 5.8.5 value-start
/// rule: only a `{`/`[` at the first non-whitespace code point of a
/// value nests (skipped via the value-start-aware
/// `scan_inline_closer`); a mid-scalar opener is a literal byte even
/// when balanced, and a mid-scalar closer byte likewise has no
/// structural effect — commas after it still split.
/// In an object body a value starts only after `:`; in an array body
/// the body start and every position after `,` are value positions
/// (§ 5.3.3 "Keys only").
fn split_top_level_fast<'a>(
    input: &'a str,
    line_num: usize,
    span: Span,
    body: InlineBody,
    bounds: InlineBounds<'_>,
) -> Vec<&'a str> {
    let object = body == InlineBody::Object;
    let (open, close) = if object { (b'{', b'}') } else { (b'[', b']') };
    let mut sc: Scanner<'_, '_, SplitFast> =
        Scanner::new(input, open, close, object, line_num, span, bounds);
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::Exhausted => {
            // Last segment (after final comma, or the whole string if
            // no comma).
            sc.segments.push(&input[sc.seg_at..]);
            sc.segments
        }
        ScanStop::Closer { .. } => {
            unreachable!("quote-free split scanner cannot stop on a closer")
        }
        ScanStop::BadEscape(_) => {
            unreachable!("quote-free split scanner never validates escapes")
        }
        ScanStop::UnterminatedQuote | ScanStop::EofAfterWsSkip => {
            unreachable!("quote-free split scanner tracks no quoted key segments")
        }
    }
}
// ---------------------------------------------------------------------------
// Delimiter matching helpers
// ---------------------------------------------------------------------------

/// Check if `input` is a balanced inline compound: starts with `open`
/// and has a matching `close` at the very end. Returns the last byte
/// index if found.
///
/// For object bodies (`open == b'{'`), brackets inside quoted key
/// segments are opaque to bracket-balance counting (spec 0.7 § 5.3.3:
/// same reason an escaped bracket is). Array bodies' own positions are
/// value positions, so quotes there are content (§ 5.3.3 "Keys only"),
/// but Object scopes nested inside still track key positions — a quoted
/// key segment of a nested object is opaque to `]` counting too (R3-F2:
/// the gate is per nested scope, not the outer opener). Key-position
/// tracking is per nesting level, and BOTH compound kinds open a scope
/// whose kind decides what a `,` begins (R6-F1): an Object scope's comma
/// starts a fresh key position, an Array scope's comma stays a value
/// position, and the enclosing level's key context is saved and restored
/// around each nested body.
///
/// Per R7-F1, openers nest only at a value position (§ 5.8.5); `::`
/// raw values and quoted-key spans are honored exactly as in
/// [`scan_inline_closer`] — the only differences are the result shape
/// ([`Option<usize>`] instead of [`InlineCloserScan`], no error
/// construction) and that escapes are never validated here.
pub(crate) fn find_matching_close(input: &str, open: u8, close: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != open {
        return None;
    }

    // line_num/span: find's configs never build errors, so the
    // placeholder 0 / `Span::EMPTY` values are never observed.
    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking.
        run_find::<FindFast>(input, open, close, open == b'{')
    } else {
        // Slow path (quote bytes present): byte-identical rules for
        // object and array bodies (R7-F1) — key-position tracking is
        // per nested Object scope inside the machine itself.
        run_find::<FindQ>(input, open, close, open == b'{')
    }
}

fn run_find<C: ScanCfg>(input: &str, open: u8, close: u8, object: bool) -> Option<usize> {
    let mut sc: Scanner<'_, '_, C> = Scanner::new(
        input,
        open,
        close,
        object,
        0,
        Span::EMPTY,
        InlineBounds::for_input(input),
    );
    // Same seed state as `run_scan` (R7-F1): find is the identical
    // machine minus escape validation, so `Some(idx)` here is exactly
    // `scan_inline_closer`'s `Found(idx)` whenever no bad escape
    // intervenes.
    sc.i = 1;
    sc.depth = 1;
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    match sc.run() {
        ScanStop::Closer { idx, byte } => (byte == close).then_some(idx),
        ScanStop::EofAfterWsSkip
        | ScanStop::UnterminatedQuote
        | ScanStop::BadEscape(_)
        | ScanStop::Exhausted => None,
    }
}
/// Find the byte offset of an inline pair's separator — the first
/// unescaped `:` outside quoted key segments (§ 4 `<inline-pair>`
/// starts with `<key> (ws) ":" (ws) ...`). Delegates to the shared
/// § 4/§ 5.3 key-separator scanner. NO compound-depth counting (R9-F1):
/// a raw bracket in the key prefix is forbidden by `<key-char>` and is
/// reported as `InvalidKey` at key validation; it must not hide the
/// separator behind phantom depth. Colons inside nested value
/// compounds cannot race this scan: the separator precedes the value,
/// so the first colon outside quoted segments is always the pair's own
/// separator for any prefix that could still be a valid key.
///
/// Quote-aware (spec 0.7 § 5.3.3): the content of a quoted key segment
/// opened at a segment-start position is opaque. A span that never
/// closes swallows the rest of the input — `None` is returned and the
/// caller maps that to its unterminated/unparseable error of choice.
pub(crate) fn find_unescaped_colon_inline(s: &str) -> Option<usize> {
    find_unescaped_colon(s)
}

// ---------------------------------------------------------------------------
// Error helpers
// ---------------------------------------------------------------------------

pub(in crate::parser::inline) fn malformed(line_num: usize, span: Span, detail: &str) -> Error {
    Error::Structured(ErrorKind::MalformedInlineCompound {
        line: line_num as u32,
        span,
        detail: detail.to_string(),
    })
}

/// Error for § 5.2 rule 8's "closer followed by content" shape: the
/// scan found a matching closer, but not at the last byte of the body.
pub(crate) fn malformed_closer_not_at_end(line_num: usize, span: Span) -> Error {
    malformed(
        line_num,
        span,
        "matching closer is not the last byte of the body; non-whitespace content follows the closed inline compound",
    )
}

/// Outcome of the § 5.2 rules 6–9 same-line closer scan for a body
/// beginning with `{` or `[`.
pub(crate) enum InlineCloserScan {
    /// A matching closer (unescaped, depth back to zero) at this byte
    /// offset.
    Found(usize),
    /// No matching closer anywhere on the line — § 5.2 rule 9. Includes
    /// the unterminated-quoted-key case: § 6.16 keeps that
    /// `UnterminatedInlineCompound`, so the scan stops there without
    /// validating anything inside the swallowed span.
    NotFound,
    /// An invalid escape was met while scanning outside quoted
    /// segments — § 5.2 rules 6–9 preamble gives `BadEscapeSequence`
    /// precedence over the rule 8/9 decision. The payload is the
    /// ready-to-return error.
    BadEscape(Error),
}

/// Scan `input` (which MUST start with `open`) with § 5.8's quote-aware,
/// escape-aware delimiter rules and report the matching `close`, for the
/// § 5.2 rules 6–9 dispatch. Shares `find_matching_close`'s quote /
/// key-position state machine (quote tracking whenever quote bytes
/// are present — per nested Object scope, so an Array body still
/// becomes quote-aware inside a nested `{`; per-level key-position
/// tracking; unterminated quoted segment ⇒ `NotFound`) and
/// additionally validates every `\X` outside quoted
/// segments via [`scan_escape`] so `BadEscapeSequence` can take
/// precedence.
///
/// Unlike `find_matching_close`, openers only nest at a VALUE position
/// (§ 5.8.5 mid-value brace rule: only the first non-ws byte of a value
/// decides compound-vs-literal), and a `::` raw marker makes the rest of
/// the pair's value literal — so mid-value braces and raw strings never
/// swallow the body's matching closer. Raw tracking is per scope
/// (§ 5.8.5, R4-F1): a raw value is terminated only by the CURRENT
/// scope's own unescaped delimiter, and closers restore the enclosing
/// scope's key context (R4-F2).
pub(crate) fn scan_inline_closer(
    input: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
) -> InlineCloserScan {
    let bytes = input.as_bytes();
    let object = open == b'{';
    // The caller guarantees `input` starts with `open`; the opener
    // itself is depth 1, so scanning starts at byte 1.
    if bytes.is_empty() || bytes[0] != open {
        return InlineCloserScan::NotFound;
    }
    // R3-F2: the gate is "any quote byte anywhere" — quote tracking
    // itself is per nested Object scope in the machine, not per outer
    // opener. Recording is discarded; use
    // [`scan_inline_closer_with_bounds`] to keep it.
    if !has_quote_bytes(bytes) {
        // Fast path: no quote tracking. `value_start` marks an
        // unconsumed value position (body start in arrays — including
        // the position right after the array's `[`, which is its first
        // item — and after `:`/`,` otherwise); per § 5.8.5 a `{`/`[`
        // only nests there.
        run_scan::<ScanFast>(input, open, close, object, line_num, span, &mut Vec::new())
    } else {
        // Slow path (quote bytes present): the per-level key-position
        // machine plus the same value-position / raw-marker tracking.
        // Object bodies start at their first key segment; array bodies
        // start at their first item — a value position with no key
        // context (§ 5.3.3 "Keys only").
        run_scan::<ScanQ>(input, open, close, object, line_num, span, &mut Vec::new())
    }
}

/// [`scan_inline_closer`] plus the boundary recording: on `Found`,
/// `bounds_out` receives every nested compound span's (opener, closer)
/// byte offset relative to `input`'s first byte, sorted by opener — a
/// pure memo of the scans the callers would otherwise re-run (see
/// [`InlineBounds`] for the purity invariant). On every non-Found
/// outcome `bounds_out` is left empty.
pub(crate) fn scan_inline_closer_with_bounds(
    input: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
    bounds_out: &mut Vec<(usize, usize)>,
) -> InlineCloserScan {
    let bytes = input.as_bytes();
    let object = open == b'{';
    if bytes.is_empty() || bytes[0] != open {
        return InlineCloserScan::NotFound;
    }
    if !has_quote_bytes(bytes) {
        run_scan::<ScanFast>(input, open, close, object, line_num, span, bounds_out)
    } else {
        run_scan::<ScanQ>(input, open, close, object, line_num, span, bounds_out)
    }
}

fn run_scan<C: ScanCfg>(
    input: &str,
    open: u8,
    close: u8,
    object: bool,
    line_num: usize,
    span: Span,
    bounds_out: &mut Vec<(usize, usize)>,
) -> InlineCloserScan {
    let mut sc: Scanner<'_, '_, C> = Scanner::new(
        input,
        open,
        close,
        object,
        line_num,
        span,
        InlineBounds::for_input(input),
    );
    sc.i = 1;
    sc.depth = 1;
    sc.in_key = object;
    sc.seg_start = object;
    sc.value_start = !object;
    sc.pairs_out = std::mem::take(bounds_out);
    let stop = sc.run();
    let mut pairs = std::mem::take(&mut sc.pairs_out);
    match stop {
        // § 5.2's matching-closer rule: a closer that returns depth to
        // zero must be the body's own closer kind (a crossed one, e.g.
        // `[{a: 1]`, is not a matching closer).
        ScanStop::Closer { idx, byte } => {
            if byte == close {
                // Recorded in pop order (= closer order): sort by opener
                // once for the consumers' binary searches. Keys are
                // unique — one span per opener byte — so unstable is
                // fine.
                #[cfg(test)]
                {
                    // Counted twin of the production
                    // `sort_unstable_by_key` below — std implements
                    // that as exactly this `sort_unstable_by` closure —
                    // so test builds see the same algorithm, input and
                    // comparison count plus its count.
                    let mut cmps = 0usize;
                    pairs.sort_unstable_by(|a, b| {
                        cmps += 1;
                        a.0.cmp(&b.0)
                    });
                    ix_probe::record_sort(pairs.len(), cmps);
                    ix_probe::record_body(input.len(), pairs.len());
                }
                #[cfg(not(test))]
                pairs.sort_unstable_by_key(|p| p.0);
                *bounds_out = pairs;
                InlineCloserScan::Found(idx)
            } else {
                InlineCloserScan::NotFound
            }
        }
        // Includes the unterminated-quoted-key case: § 6.16 keeps that
        // `UnterminatedInlineCompound` upstream, so the scan stops
        // there without validating anything inside the swallowed span.
        ScanStop::EofAfterWsSkip | ScanStop::UnterminatedQuote | ScanStop::Exhausted => {
            InlineCloserScan::NotFound
        }
        ScanStop::BadEscape(e) => InlineCloserScan::BadEscape(e),
    }
}
