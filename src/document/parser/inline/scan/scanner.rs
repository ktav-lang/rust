//! The shared single-loop inline scanner (the byte-at-a-time state machine).

use crate::error::{Error, ErrorKind, Span};

use super::scan_config::{ScanCfg, ScanStop, ScopeFrame};
use super::split::{scan_inline_closer, InlineBounds, InlineCloserScan};
use crate::parser::inline::escapes::scan_escape;
use crate::parser::inline::keys::{
    inline_whitespace_at, is_quote_byte, quoted_span_end, skip_segment_ws,
};

/// The shared byte-at-a-time scanner. All hot state lives in fields
/// that [`Scanner::run`] copies into LOCALS for the whole loop (the
/// former hand-written loops kept state in register-allocatable
/// locals; a field-per-iteration machine measured +15–20% slower on
/// deeply-nested quote-free documents) and writes back once after the
/// loop.
pub(in crate::document::parser::inline) struct Scanner<'a, 'b, C: ScanCfg> {
    input: &'a str,
    bytes: &'a [u8],
    open: u8,
    close: u8,
    body_object: bool,
    line_num: usize,
    span: Span,
    pub(in crate::document::parser::inline) i: usize,
    pub(in crate::document::parser::inline) depth: i32,
    pub(in crate::document::parser::inline) in_key: bool,
    pub(in crate::document::parser::inline) seg_start: bool,
    pub(in crate::document::parser::inline) value_start: bool,
    raw: bool,
    prev: u8,
    stack: Vec<ScopeFrame>,
    pub(in crate::document::parser::inline) segments: Vec<&'a str>,
    pub(in crate::document::parser::inline) seg_at: usize,
    input_off: usize, // offset of input[0] in bounds coordinates
    bounds: InlineBounds<'b>,
    open_at: Vec<(usize, bool, i32)>, // parallels `stack`: (opener offset, pure, entry depth)
    pub(in crate::document::parser::inline) pairs_out: Vec<(usize, usize)>, // recorded (opener, closer) pairs
    _cfg: std::marker::PhantomData<C>,
}

impl<'a, 'b, C: ScanCfg> Scanner<'a, 'b, C> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::document::parser::inline) fn new(
        input: &'a str,
        open: u8,
        close: u8,
        body_object: bool,
        line_num: usize,
        span: Span,
        bounds: InlineBounds<'b>,
    ) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            open,
            close,
            body_object,
            line_num,
            span,
            i: 0,
            depth: 0,
            in_key: false,
            seg_start: false,
            value_start: false,
            raw: false,
            prev: open,
            stack: Vec::new(),
            segments: Vec::new(),
            seg_at: 0,
            input_off: input.as_ptr() as usize - bounds.origin,
            bounds,
            open_at: Vec::new(),
            pairs_out: Vec::new(),
            _cfg: std::marker::PhantomData,
        }
    }

    pub(in crate::document::parser::inline) fn run(&mut self) -> ScanStop {
        let input = self.input;
        let bytes = self.bytes;
        let len = bytes.len();
        let open = self.open;
        let close = self.close;
        let body_object = self.body_object;
        let mut i = self.i;
        let mut depth = self.depth;
        let mut in_key = self.in_key;
        let mut seg_start = self.seg_start;
        let mut value_start = self.value_start;
        let mut raw = self.raw;
        let mut prev = self.prev;
        let mut seg_at = self.seg_at;
        let mut stack = std::mem::take(&mut self.stack);
        let mut segments = std::mem::take(&mut self.segments);
        let mut open_at = std::mem::take(&mut self.open_at);
        let mut pairs_out = std::mem::take(&mut self.pairs_out);

        let stop = loop {
            if i >= len {
                break ScanStop::Exhausted;
            }

            // (1) quoted key segment start: only when quote tracking,
            // in a key position, at a fresh segment start (quirk 10).
            if C::TRACK_QUOTES && in_key && seg_start {
                i = skip_segment_ws(input, i);
                // The skip may consume every byte that remains: after
                // a trailing comma (or a dotted-key `.`) only
                // whitespace can follow, leaving `i` at EOF;
                // `bytes[i]` below then indexed out of bounds
                // (R3-F1).
                if i >= len {
                    break ScanStop::EofAfterWsSkip;
                }
                if is_quote_byte(bytes[i]) {
                    match quoted_span_end(bytes, i) {
                        Some(end) => {
                            // `prev` takes the closing quote byte —
                            // what a per-byte loop would leave in
                            // `prev` after the span (TRACK_RAW only).
                            if C::TRACK_RAW {
                                prev = bytes[end];
                            }
                            i = end + 1;
                            seg_start = false;
                            continue;
                        }
                        None => break ScanStop::UnterminatedQuote,
                    }
                }
                seg_start = false;
            }

            let b = bytes[i];
            match b {
                b'\\' => {
                    if C::VALIDATE_ESCAPES {
                        // § 5.2 rules-6–9 preamble: an invalid escape
                        // beats the rule 8/9 decision
                        // (`BadEscapeSequence` precedence). Advance by
                        // the FULL escape length (unlike find/split's
                        // `i += 2`, which suffices there because hex
                        // digits are not structural and validation
                        // happens later in `process_escapes`). `prev`
                        // becomes `\\` so an escaped `:` never forms a
                        // `::` raw marker (R5-F4).
                        match scan_escape(bytes, i) {
                            Err(seq) => {
                                break ScanStop::BadEscape(Error::Structured(
                                    ErrorKind::BadEscapeSequence {
                                        line: self.line_num as u32,
                                        span: self.span,
                                        sequence: seq,
                                    },
                                ))
                            }
                            Ok(esc) => {
                                // A recognized escape in value
                                // position consumes the scalar start
                                // (§ 3.7 / § 5.8.5, R4-F4): the
                                // decoded byte cannot reopen
                                // structural dispatch.
                                value_start = false;
                                if C::TRACK_RAW {
                                    prev = b'\\';
                                }
                                i += esc.len();
                                continue;
                            }
                        }
                    }
                    // find/split: skip the escaped character, no
                    // validation (validation happens later in
                    // `process_escapes`).
                    value_start = false;
                    i += 2;
                    continue;
                }
                b':' => {
                    if in_key {
                        // Key/value boundary: quotes after this are
                        // content.
                        in_key = false;
                        if C::COLON_SETS_VS {
                            value_start = true;
                        }
                    } else if C::TRACK_RAW && prev == b':' && value_start {
                        // `::` raw marker (§ 5.4 — in array bodies
                        // too): the rest of the item's value is a
                        // String — braces/brackets in it are content.
                        raw = true;
                    } else if C::TRACK_RAW {
                        // A lone `:` opening the value's scalar
                        // (§ 5.8.5, R5-F2) consumes the value
                        // position: a following `{`/`[` is literal
                        // content, not a nested opener. A `:`
                        // immediately followed by another `:` stays
                        // armed for the `::` marker check above on
                        // the next iteration.
                        if value_start && bytes.get(i + 1) != Some(&b':') {
                            value_start = false;
                        }
                    } else if C::COLON_ELSE_CLEAR_VS {
                        // `:` is never § 3.3 whitespace: split's old
                        // `_` fall-through for a non-key colon.
                        value_start = false;
                    }
                }
                b',' => {
                    if C::COMMA_SPLITS {
                        segments.push(&input[seg_at..i]);
                        seg_at = i + 1;
                    }
                    // Next pair / item begins: re-derive key context
                    // from the CURRENT scope (the innermost opener on
                    // the stack), not the outermost one (R5-F1/R6-F1)
                    // — an Object scope starts a key position, an
                    // Array scope stays a value position (§ 5.3.3
                    // "Keys only"). split has no stack (OPENER_JUMP):
                    // its commas always sit at body depth, so the body
                    // kind is the current scope kind there.
                    let scope_object = if C::COMMA_CTX_SCOPE {
                        stack.last().map_or(body_object, |f| f.kind() == b'{')
                    } else {
                        body_object
                    };
                    in_key = scope_object;
                    if C::TRACK_QUOTES {
                        seg_start = scope_object;
                    }
                    // An unescaped comma ALWAYS ends raw mode (R5-F3).
                    if C::COMMA_CLEARS_RAW {
                        raw = false;
                    }
                    // R9-F1: value_start mirrors the same scope kind —
                    // after an Object comma the next position is a KEY
                    // position (§ 4: <inline-pair> begins with <key>),
                    // so the opener gate must reject a bracket there
                    // instead of phantom-opening a compound whose
                    // closer then eats the body's own closer.
                    value_start = !scope_object;
                }
                b'.' if C::TRACK_QUOTES && in_key => {
                    seg_start = true;
                }
                b'{' | b'[' => {
                    let gated_open = value_start && !raw;
                    if !gated_open {
                        // Mid-scalar (or raw-mode) opener: a literal
                        // byte with no structural meaning; balancing
                        // is irrelevant (R3-F4, § 5.8.5) — commas
                        // inside it still split.
                        value_start = false;
                    } else if C::OPENER_JUMP {
                        // split: sub-scan the candidate nested
                        // compound with its own pair. The matching
                        // closer is found with the value-start-aware
                        // `scan_inline_closer` (a mid-scalar opener
                        // inside the span must not count);
                        // `find_matching_close` would balance naive
                        // byte counts and skip spans that per
                        // § 5.8.5 are NOT one compound.
                        let (o, c) = if b == b'{' {
                            (b'{', b'}')
                        } else {
                            (b'[', b']')
                        };
                        // Consult the gate scan's boundary map first: a
                        // hit is a pure memo of the sub-scan below (see
                        // [`InlineBounds`]), so the jump target and the
                        // state writes are identical; a miss re-runs the
                        // live sub-scan. Only split configs consult
                        // (OPENER_JUMP): find-config walks never read
                        // the map, and find_matching_close itself is
                        // unchanged — a memo hit can therefore never
                        // mask a BadEscapeSequence the live find
                        // dispatch would raise, because
                        // known_closer/consult hits only ever return
                        // spans the validating scan walked without any
                        // bad escape.
                        if let Some(end_abs) = self.bounds.opener_close_at(self.input_off + i) {
                            let end = end_abs - self.input_off; // closer index within `input`
                            if end < len {
                                // Skip over the entire nested compound
                                // (same writes as the Found branch
                                // below).
                                i = end + 1;
                                value_start = false;
                                if C::TRACK_QUOTES {
                                    seg_start = false;
                                }
                                continue;
                            }
                        }
                        if let InlineCloserScan::Found(pos) =
                            scan_inline_closer(&input[i..], o, c, self.line_num, self.span)
                        {
                            // Skip over the entire nested compound.
                            i += pos + 1;
                            value_start = false;
                            if C::TRACK_QUOTES {
                                seg_start = false;
                            }
                            continue;
                        }
                        // No closer inside the body — treat as
                        // literal byte (mid-value brace). Escapes are
                        // validated later by `process_escapes`, so a
                        // `BadEscape` scan result is deliberately not
                        // propagated here.
                        value_start = false;
                        i += 1;
                        continue;
                    } else {
                        // Count in place.
                        if C::DECREMENT_ANY_CLOSER || b == open {
                            depth += 1;
                        }
                        if C::USE_STACK {
                            // Save the enclosing key-position and raw
                            // state (§ 5.8.5, R4-F1 / R6-F1).
                            stack.push(ScopeFrame::pack(b, in_key, seg_start, raw));
                            if C::RECORD_BOUNDS {
                                // Entry state of this span vs the
                                // standalone scan's seed: the opener
                                // arm re-derives in_key/seg_start/
                                // value_start/prev identically in both
                                // modes (R8-F2), so every span is
                                // recorded. Purity still has the two
                                // pop-time gates below (crossed-closer
                                // and stop-byte checks).
                                open_at.push((i, true, depth));
                            }
                        }
                        // After an array's `[` the next position is
                        // still a value position (its first item,
                        // § 5.8.5); after a nested `{` comes key
                        // context. find never reads `value_start`, so
                        // this write is harmless there. `in_key` is
                        // re-derived IDENTICALLY in both modes (R8-F2,
                        // was fast-only quirk 2): fast residue let a
                        // span's `:` be classified as a key separator
                        // in one mode and value content in the other.
                        value_start = b == b'[';
                        in_key = b == b'{';
                        if C::TRACK_QUOTES {
                            seg_start = b == b'{';
                        }
                    }
                }
                b'}' | b']' if !C::CLOSER_LITERAL => {
                    if !C::DECREMENT_ANY_CLOSER && b != close {
                        // find's other-kind closer: restore-only, no
                        // depth change.
                        if C::TRACK_QUOTES {
                            let want = if b == b']' { b'[' } else { b'{' };
                            if stack.last().is_some_and(|f| f.kind() == want) {
                                let f = stack.pop().unwrap();
                                in_key = f.saved_in_key();
                                seg_start = if C::RESTORE_SEG_ON_MATCH {
                                    f.saved_seg_start()
                                } else {
                                    // The closer itself consumed a
                                    // position, so segment-start
                                    // tracking stays off until the
                                    // next re-arm (quirk 3).
                                    false
                                };
                            } else {
                                seg_start = false;
                            }
                        }
                    } else {
                        // Both closer kinds decrement the shared
                        // depth: a nested compound of the OTHER
                        // delimiter type still closes (an array item
                        // may be an object and vice versa). An
                        // unescaped closer ALWAYS ends raw mode and is
                        // structural (R5-F3):
                        // `<inline-raw-scalar>` terminates on the
                        // FIRST unescaped `,`, `}`, or `]` regardless
                        // of which scope it belongs to — raw mode only
                        // makes leading openers literal (§ 5.8.5).
                        raw = false;
                        depth -= 1;
                        if depth == 0 {
                            // A closer that returns depth to zero
                            // must be the body's own closer; a crossed
                            // one (e.g. `[{a: 1]`) is not a matching
                            // closer (§ 5.2's matching-closer rule).
                            break ScanStop::Closer { idx: i, byte: b };
                        }
                        // Nested closer: pop it only when it matches
                        // the most recently opened compound kind
                        // (compare the stored OPENER to the closer's
                        // matching opener, R4-F2), so crossed closers
                        // don't corrupt tracking.
                        let want = if b == b']' { b'[' } else { b'{' };
                        if C::RECORD_BOUNDS {
                            // The span's standalone scan stops at the FIRST
                            // closer that returns its own counter to zero —
                            // global `depth` back to `entry - 1`. That byte is
                            // now, and the top frame is the span in question
                            // (everything opened above it died at strictly
                            // higher depths). Kind match: the pop below records
                            // a pure pair. Crossed: the standalone said
                            // `NotFound` HERE, so the frame must never record,
                            // no matter how far the walk continues past it.
                            match open_at.last_mut() {
                                Some(top) if depth == top.2 - 1 => {
                                    if !stack.last().is_some_and(|f| f.kind() == want) {
                                        top.1 = false;
                                    }
                                }
                                // Defense in depth: the walk somehow passed
                                // the stop byte without stopping — treat the
                                // span as dead.
                                Some(top) if depth < top.2 - 1 => top.1 = false,
                                _ => {}
                            }
                        }
                        if stack.last().is_some_and(|f| f.kind() == want) {
                            let f = stack.pop().unwrap();
                            if C::RECORD_BOUNDS {
                                let (open, pure, entry) =
                                    open_at.pop().expect("open_at parallels the scope stack");
                                if pure && depth == entry - 1 {
                                    pairs_out.push((open, i));
                                }
                            }
                            if C::RESTORE_IN_KEY_ON_MATCH {
                                in_key = f.saved_in_key();
                            }
                            if C::TRACK_QUOTES {
                                seg_start = if C::RESTORE_SEG_ON_MATCH {
                                    f.saved_seg_start()
                                } else {
                                    // The `}`/`]` itself consumed a
                                    // position, so segment-start
                                    // tracking stays off until the
                                    // next re-arm (quirk 3).
                                    false
                                };
                                if C::RESTORE_RAW_ON_MATCH {
                                    // Per-scope raw tracking
                                    // (§ 5.8.5, R4-F1 / R4-F2).
                                    raw = f.saved_raw();
                                }
                            }
                        } else if C::MISMATCH_SEG_FALSE && C::TRACK_QUOTES {
                            seg_start = false;
                        }
                        // The closed compound consumed the value
                        // position (§ 5.8.5) — identical in every
                        // find/scan config (R8-F2, was quirk 1).
                        if C::CLEAR_VS_ON_NESTED_CLOSE {
                            value_start = false;
                        }
                    }
                }
                _ => {
                    // § 3.3 whitespace: skip the whole code point
                    // without consuming the value-start position
                    // (R4-F3) — NBSP's 0xC2 lead byte must not clear
                    // `value_start`, and a per-byte `i += 1` would
                    // strand its 0xA0 continuation byte mid-code-point.
                    // `prev` takes the run's LAST byte (TRACK_RAW
                    // only) — whitespace bytes are never `:` or `\`,
                    // so `::` raw-marker detection sees exactly what
                    // the per-byte loop would see.
                    if let Some(ws_len) = inline_whitespace_at(input, i) {
                        if C::TRACK_RAW {
                            prev = bytes[i + ws_len - 1];
                        }
                        i += ws_len;
                        continue;
                    }
                    // Non-whitespace content consumes the value-start
                    // position.
                    value_start = false;
                }
            }
            prev = b;
            i += 1;
        };

        self.i = i;
        self.depth = depth;
        self.in_key = in_key;
        self.seg_start = seg_start;
        self.value_start = value_start;
        self.raw = raw;
        self.prev = prev;
        self.seg_at = seg_at;
        self.stack = stack;
        self.segments = segments;
        self.open_at = open_at;
        self.pairs_out = pairs_out;
        stop
    }
}
