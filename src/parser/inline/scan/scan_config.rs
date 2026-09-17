//! Configuration types for the shared single-loop inline scanner (quirk inventory, scope frames, ScanCfg and its six configs).

use crate::error::Error;

// ---------------------------------------------------------------------------
// Shared single-loop inline scanner
// ---------------------------------------------------------------------------
//
// `split_top_level`, `split_top_level_fast`, `find_matching_close` and
// `scan_inline_closer` are thin wrappers over ONE byte-at-a-time state
// machine ([`Scanner::run`]). The three former hand-written scanners
// duplicated overlapping state machines and successive review rounds
// each found a defect fixed in one copy but not another; the shared
// loop carries every policy decision as an associated `const` of the
// [`ScanCfg`] trait so each (entry point × quote-mode) instantiation
// monomorphizes into its own specialized copy where every policy check
// folds away at compile time — no `dyn`, no runtime enum dispatch in
// the byte loop.
//
// Preserved-quirk inventory (all LOAD-BEARING current behavior).
// Former quirks 1-2 were REMOVED in R8-F2 — they made the fast and
// quote-aware machines disagree on the meaning of the same quote-free
// bytes (a `[` after a closed empty Array stayed a structural opener
// in fast mode, and key-position residue leaked into Array spans), so
// a memo recorded by one mode could lie about the other:
// 1. REMOVED R8-F2: every find/scan config now clears `value_start`
//    at a nested close (`CLEAR_VS_ON_NESTED_CLOSE`) — the closed
//    compound consumed the value position (§ 5.8.5).
// 2. REMOVED R8-F2: every config now sets `in_key = (b == b'{')` at
//    an opener and restores it from a kind-matched pop.
// 3. split maps an unterminated quoted key to
//    `UnterminatedInlineCompound`, and dotted-key-then-EOF to
//    whitespace-only-rest → no trailing segment, else `EmptyKey`;
//    find/scan map both to `None`/`NotFound`.
// 4. REMOVED R9-F1: every config now derives the comma's value_start
//    from the SAME scope kind as its in_key — after an Object comma
//    that position is a key position (§ 4: <inline-pair> begins with
//    <key>), so `value_start = !scope_kind`; an Array comma stays a
//    value position.
// 5. scan validates escapes (full-length advance,
//    `BadEscapeSequence` precedence); find/split skip 2 bytes
//    unvalidated (validation happens later in `process_escapes`).
// 6. `prev` tracking (every TRACK_RAW config, i.e. find and scan):
//    ws arm sets prev to the run's last byte; quoted-span continue
//    sets prev to the closing quote byte; `\\` sets prev to `\\`.
// 7. the lone-`:`-in-value rule (R5-F2) clears `value_start` unless
//    the next byte is `:`.
// 8. The segment-start skip block runs only when
//     `TRACK_QUOTES && in_key && seg_start`.
//
// R7-F1: the find configs previously kept naive opener counting, no
// raw-marker tracking and no value-position tracking — frozen at the
// unification as `OPENER_GATED = false`. A literal mid-scalar `[`
// then pushed a PHANTOM Array scope; the next comma derived key
// context from it, quoted keys stopped being opaque, and a key's `}`
// was mistaken for the body closer. Byte meaning (opener
// structurality, raw mode, quoted-key opacity, crossed closers) is
// now IDENTICAL across find and scan; the configs differ only in
// result shape (`Option<usize>` vs [`InlineCloserScan`]) and
// validation mode (find never validates escapes).

/// Per open compound: the opener kind plus the enclosing key-position,
/// segment-start and raw state, packed into ONE byte. The scope stack
/// pushes/pops one element per nesting level, so the element size is
/// hot on deeply-nested documents; the former 4-byte struct
/// (kind + three `bool`s) regressed them measurably against the
/// pre-unification `Vec<u8>` opener stack.
///
/// Bit layout: bit 0 = opener kind (0: `{`-scope, 1: `[`-scope),
/// bit 1 = saved `in_key`, bit 2 = saved `seg_start`, bit 3 = saved `raw`.
#[derive(Clone, Copy)]
pub(in crate::parser::inline) struct ScopeFrame(u8);

impl ScopeFrame {
    /// `kind` is the opener byte, `b'{'` or `b'['`.
    #[inline]
    pub(in crate::parser::inline) fn pack(
        kind: u8,
        saved_in_key: bool,
        saved_seg_start: bool,
        saved_raw: bool,
    ) -> Self {
        ScopeFrame(
            ((kind == b'[') as u8)
                | ((saved_in_key as u8) << 1)
                | ((saved_seg_start as u8) << 2)
                | ((saved_raw as u8) << 3),
        )
    }

    /// The stored opener kind, as the opener byte it was packed from.
    #[inline]
    pub(in crate::parser::inline) fn kind(self) -> u8 {
        if self.0 & 1 == 0 {
            b'{'
        } else {
            b'['
        }
    }

    #[inline]
    pub(in crate::parser::inline) fn saved_in_key(self) -> bool {
        self.0 & 0b0010 != 0
    }

    #[inline]
    pub(in crate::parser::inline) fn saved_seg_start(self) -> bool {
        self.0 & 0b0100 != 0
    }

    #[inline]
    pub(in crate::parser::inline) fn saved_raw(self) -> bool {
        self.0 & 0b1000 != 0
    }
}

/// Why the shared scanner loop stopped.
pub(in crate::parser::inline) enum ScanStop {
    /// Depth returned to 0 at `idx`; `byte` is the closer byte
    /// (`}`/`]`) seen there.
    Closer { idx: usize, byte: u8 },
    /// The seg-start whitespace skip consumed the rest of the input.
    EofAfterWsSkip,
    /// A quoted key segment never closed.
    UnterminatedQuote,
    /// [`scan_escape`] rejection with the ready-made error payload.
    BadEscape(Error),
    /// Input exhausted without any of the above.
    Exhausted,
}

/// Compile-time policy knobs of the shared scanner. Every `const` is
/// folded away by monomorphization, so each config's byte loop is the
/// specialized machine of exactly one former hand-written scanner.
pub(in crate::parser::inline) trait ScanCfg {
    /// Quote/segment-start tracking (§ 5.3.3). `false` for every
    /// *Fast config.
    const TRACK_QUOTES: bool;
    /// Maintain the scope stack (find and scan, all four configs);
    /// `false` for split (its `OPENER_JUMP` sub-scans keep the stack empty).
    const USE_STACK: bool;
    /// split: sub-scan a candidate nested compound with its own pair
    /// ([`scan_inline_closer`]) and jump over it.
    const OPENER_JUMP: bool;
    /// find/scan: BOTH closer kinds decrement the shared depth and
    /// the depth-0 closer must be the body's own kind; split: unused
    /// (`}`/`]` are literal there).
    const DECREMENT_ANY_CLOSER: bool;
    /// split: `}`/`]` are ordinary content (§ 5.8.5 shields commas
    /// only inside value-start compounds).
    const CLOSER_LITERAL: bool;
    /// scan configs only ([`scan_inline_closer`]): [`scan_escape`]
    /// validation with `BadEscapeSequence` precedence (§ 5.2
    /// rules-6–9 preamble); find defers escape validation to
    /// `process_escapes` so the nested find-first dispatch keeps
    /// `BadEscapeSequence` precedence via its scan fallback.
    const VALIDATE_ESCAPES: bool;
    /// find + scan: `::` raw marker (§ 5.4) + `prev` byte tracking;
    /// split never tracks raw.
    const TRACK_RAW: bool;
    /// split: record a segment at every loop-seen comma.
    const COMMA_SPLITS: bool;
    /// find/scan: comma key context comes from the CURRENT scope (the
    /// stack top, R5-F1/R6-F1); split: from the body kind.
    const COMMA_CTX_SCOPE: bool;
    /// find + scan: an unescaped comma ends raw mode (R5-F3).
    const COMMA_CLEARS_RAW: bool;
    /// a key `:` sets `value_start` (find + scan + split).
    const COLON_SETS_VS: bool;
    /// split: a non-key `:` clears `value_start` — `:` is never § 3.3
    /// whitespace, so split's old `_` fall-through applies.
    const COLON_ELSE_CLEAR_VS: bool;
    /// Restore `in_key` from a kind-matched popped scope.
    const RESTORE_IN_KEY_ON_MATCH: bool;
    /// scan-slow/find-slow: TRUE — a kind-matched closer restores the
    /// saved `seg_start`; the fast configs track no segments.
    const RESTORE_SEG_ON_MATCH: bool;
    /// scan-slow/find-slow: restore saved raw state on a kind match
    /// (§ 5.8.5, R4-F1).
    const RESTORE_RAW_ON_MATCH: bool;
    /// find-slow + scan-slow: a closer that matches NO scope kind
    /// still forces `seg_start = false`.
    const MISMATCH_SEG_FALSE: bool;
    /// every find/scan config (R8-F2, was fast-only quirk 1): a
    /// nested closer clears `value_start` — the closed compound
    /// consumed the value position (§ 5.8.5), so a following
    /// `{`/`[` is content after a closed value, not a new opener.
    const CLEAR_VS_ON_NESTED_CLOSE: bool;
    /// scan configs only: record every nested (opener, closer) pair
    /// into `pairs_out` as the walk pops each scope. Pure side output —
    /// changes no decision. Recording is only PURE for spans whose
    /// entry state equals the standalone scan's seed (see
    /// [`InlineBounds`]), which is why only the gate scans record.
    const RECORD_BOUNDS: bool;
}

/// `find_matching_close`, quote-free fast path: byte-for-byte the
/// [`ScanFast`] machine minus escape validation (R7-F1). `in_key` and
/// `seg_start` are maintained with the same semantics as [`FindQ`]
/// (R8-F2) even though no quote byte can exist: `in_key` still
/// classifies a `:` as key-separator vs value content, and find must
/// mean exactly what the quote-aware machine means by every byte.
pub(in crate::parser::inline) struct FindFast;
impl ScanCfg for FindFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = false;
}

/// `find_matching_close`, slow path (quote bytes present): the
/// [`ScanQ`] machine minus escape validation (R7-F1). The same
/// per-scope key-position tracking, value-position gating and
/// raw-marker rules as the root scan — only the result shape
/// (`Option<usize>`, no error construction) and the missing escape
/// validation differ.
pub(in crate::parser::inline) struct FindQ;
impl ScanCfg for FindQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = false;
}

/// `scan_inline_closer`, quote-free fast path. `value_start` marks an
/// unconsumed value position (§ 5.8.5); both closer kinds decrement
/// the shared depth; a nested close clears `value_start` and an opener
/// re-derives `in_key` — byte-identical to [`ScanQ`] on quote-free
/// slices (R8-F2 closed the former fast-only quirks), which is what
/// lets one recorded boundary memo serve a consumer in either mode.
pub(in crate::parser::inline) struct ScanFast;
impl ScanCfg for ScanFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = true;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = true;
}

/// `scan_inline_closer`, slow path (quote bytes present): the
/// per-level key-position machine plus the same value-position /
/// raw-marker tracking as the fast path; closers restore the
/// enclosing scope's key, segment-start and raw state (R4-F2) and
/// clear `value_start`.
pub(in crate::parser::inline) struct ScanQ;
impl ScanCfg for ScanQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = true;
    const OPENER_JUMP: bool = false;
    const DECREMENT_ANY_CLOSER: bool = true;
    const CLOSER_LITERAL: bool = false;
    const VALIDATE_ESCAPES: bool = true;
    const TRACK_RAW: bool = true;
    const COMMA_SPLITS: bool = false;
    const COMMA_CTX_SCOPE: bool = true;
    const COMMA_CLEARS_RAW: bool = true;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = false;
    const RESTORE_IN_KEY_ON_MATCH: bool = true;
    const RESTORE_SEG_ON_MATCH: bool = true;
    const RESTORE_RAW_ON_MATCH: bool = true;
    const MISMATCH_SEG_FALSE: bool = true;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = true;
    const RECORD_BOUNDS: bool = true;
}

/// `split_top_level`, quote-free fast path (also serving object bodies
/// without quote bytes). Implements the § 5.8.5 value-start rule via
/// sub-scans; `}`/`]` are ordinary content.
pub(in crate::parser::inline) struct SplitFast;
impl ScanCfg for SplitFast {
    const TRACK_QUOTES: bool = false;
    const USE_STACK: bool = false;
    const OPENER_JUMP: bool = true;
    const DECREMENT_ANY_CLOSER: bool = false;
    const CLOSER_LITERAL: bool = true;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = false;
    const COMMA_SPLITS: bool = true;
    const COMMA_CTX_SCOPE: bool = false;
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
    const RECORD_BOUNDS: bool = false;
}

/// `split_top_level`, object body with quote bytes: quoted KEYS are
/// comma-opaque (§ 5.3.3), quotes in value positions are content.
pub(in crate::parser::inline) struct SplitQ;
impl ScanCfg for SplitQ {
    const TRACK_QUOTES: bool = true;
    const USE_STACK: bool = false;
    const OPENER_JUMP: bool = true;
    const DECREMENT_ANY_CLOSER: bool = false;
    const CLOSER_LITERAL: bool = true;
    const VALIDATE_ESCAPES: bool = false;
    const TRACK_RAW: bool = false;
    const COMMA_SPLITS: bool = true;
    const COMMA_CTX_SCOPE: bool = false;
    const COMMA_CLEARS_RAW: bool = false;
    const COLON_SETS_VS: bool = true;
    const COLON_ELSE_CLEAR_VS: bool = true;
    const RESTORE_IN_KEY_ON_MATCH: bool = false;
    const RESTORE_SEG_ON_MATCH: bool = false;
    const RESTORE_RAW_ON_MATCH: bool = false;
    const MISMATCH_SEG_FALSE: bool = false;
    const CLEAR_VS_ON_NESTED_CLOSE: bool = false;
    const RECORD_BOUNDS: bool = false;
}
