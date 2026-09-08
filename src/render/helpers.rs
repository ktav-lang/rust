//! Small primitives shared by the rendering functions.

use crate::error::{Error, ReasonCode, Result};
use crate::parser::classify;
use crate::value::Value;
use crate::whitespace::{
    common_leading_whitespace_prefix_len, is_inline_whitespace, is_ktav_whitespace,
};

pub(super) const INDENT: &str = "    ";

/// A top-level Array must be wrapped in `[` / `]` whenever its first
/// item, rendered on its own line, would itself be read as a complete
/// root by § 5.0.1. Only the *first* item matters — root-kind
/// detection looks at the first content line only — so
/// `Array(["x", {..}])` is already unambiguous (a bare scalar first
/// line means "root is an Array") and needs no wrapping.
///
/// **Wider than § 5.9.3 as written.** The spec derives the wrap rule
/// from § 5.0.1 rules 4/5 only — "specifically a lone `{` (rule 4) or
/// a lone `[` (rule 5)" — which covers non-empty compounds, since
/// those open with a brace on its own line. But rules 2/3 (a *closed*
/// inline `{ … }` / `[ … ]` as the first content line) create exactly
/// the same ambiguity, and an empty compound item emits as precisely
/// that: bare `Array([{}])` renders as the single line `{}`, which
/// rule 2 reads as "the root IS this Object" — one item short. Worse,
/// rule 2 also forbids further content lines, so `Array([{}, "x"])`
/// renders as `"{}\nx\n"` and fails to parse outright
/// (`OrphanLineAfterTopLevelInline`, § 6.14).
///
/// Both were reproduced against 0.6.2 on `emit_canonical` as well as
/// `render`, so this is a lossless-writer fix, not a new convention —
/// but the § 5.9.3 wording should be widened to name rules 2/3
/// alongside 4/5. Tracked in `ktav-lang/spec`.
pub(crate) fn first_item_needs_wrap(item: &Value) -> bool {
    matches!(item, Value::Object(_) | Value::Array(_))
}

/// Push `\u` followed by exactly four UPPERCASE hex digits naming
/// `ch` (§ 3.7.1). Used only where no named escape exists.
fn push_unicode_escape(out: &mut String, ch: char) {
    out.push_str(&format!("\\u{:04X}", ch as u32));
}

/// Emit a key (flat, single-segment — the byte string stored on the
/// Object) into `out`, choosing **bare** or **quoted** form per spec
/// 0.7 § 5.9.10 and applying the corresponding re-escape recipe so
/// the parser reads the same segment back.
///
/// Form selection (§ 5.9.10): quoted (delimiter `"` unconditional)
/// iff any of —
/// - (a) the segment contains a structural byte (`.` `:` `,` `{` `}`
///   `[` `]` `(` `)`) anywhere, or its first/last code point is a
///   § 3.3 whitespace code point other than LF/CR (a quoted segment
///   is never trimmed, and never admits LF/CR raw at all, so quoting
///   buys nothing for those two); or
/// - (b) the segment begins with `"`, `'`, or `` ` `` — a leading
///   quote character would open a `<quoted-segment>` on re-parse; or
/// - (c) `root_first_key` is set and the segment begins with U+FEFF —
///   bare form would place the BOM encoding at byte offset 0 of the
///   document, indistinguishable from the metadata byte-order mark
///   (§ 5.9.12); or
/// - (d) the segment begins with the two-byte sequence `##` — no bare
///   escape changes the raw first two bytes § 5.1 rule 2 inspects, so
///   only quoted form avoids the comment dispatch.
///
/// Escaping a backslash, LF, CR, a control byte, or DEL does NOT
/// trigger quoted form by itself: quoted form escapes those
/// identically, so quoting buys nothing.
///
/// `root_first_key` is true ONLY when this key is the first-
/// serialized key of the ROOT Object at byte offset 0 of the
/// document (rule (c) / § 5.9.12).
///
/// This is used for the leaf key of a pair (and any non-dotted
/// segment). Callers that want to emit a dotted path (multiple
/// segments separated by an UNescaped `.`) join multiple calls with a
/// literal `.` between them.
pub(crate) fn push_escaped_key_segment(key: &str, root_first_key: bool, out: &mut String) {
    if key_segment_needs_quotes(key, root_first_key) {
        push_quoted_key(key, out);
    } else {
        push_bare_key(key, out);
    }
}

/// § 5.9.10 form selection — see [`push_escaped_key_segment`].
fn key_segment_needs_quotes(key: &str, root_first_key: bool) -> bool {
    let first = key.chars().next();
    let last = key.chars().next_back();

    // (b) leading quote character.
    if matches!(first, Some('"') | Some('\'') | Some('`')) {
        return true;
    }
    // (c) root's first-serialized key beginning with U+FEFF.
    if root_first_key && first == Some('\u{FEFF}') {
        return true;
    }
    // (d) leading `##` comment collision.
    if key.as_bytes().starts_with(b"##") {
        return true;
    }
    // (a) structural bytes anywhere.
    if key.bytes().any(|b| {
        matches!(
            b,
            b'.' | b':' | b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')'
        )
    }) {
        return true;
    }
    // (a) edge whitespace — except LF/CR, which a quoted segment never
    // admits raw, so quoting buys nothing for them.
    let edge_ws = |c: Option<char>| c.is_some_and(is_inline_whitespace);
    edge_ws(first) || edge_ws(last)
}

/// Bare form (§ 5.9.10 bullet recipe): named escapes for `\` `.` `:`
/// `,` `{` `}` `[` `]` LF CR; `\uXXXX` for other control bytes < 0x20
/// (except tab/VT/FF) and DEL; `\uXXXX` also for a § 3.3 whitespace
/// code point at the first or last code-point position (named `\n`/
/// `\r` when applicable) — unescaped it would be trimmed away on
/// re-parse. Interior whitespace (including tab) stays raw.
fn push_bare_key(key: &str, out: &mut String) {
    let first_ch = key.chars().next();
    let last_ch = key.chars().next_back();
    let edge_ws = |c: Option<char>| c.is_some_and(is_inline_whitespace);

    // Fast path: nothing to escape anywhere, and neither edge code
    // point is § 3.3 whitespace → push the whole string verbatim.
    let needs_rewrite = key.bytes().any(|b| {
        matches!(
            b,
            b'\\' | b'.' | b':' | b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')' | b'\n' | b'\r'
        ) || (b < 0x20 && !matches!(b, b'\t' | 0x0B | 0x0C))
            || b == 0x7F
    }) || edge_ws(first_ch)
        || edge_ws(last_ch);
    if !needs_rewrite {
        out.push_str(key);
        return;
    }

    out.reserve(key.len() + 8);
    let last_idx = key.char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
    for (i, ch) in key.char_indices() {
        let at_edge = i == 0 || i == last_idx;
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            // § 3.3 whitespace: raw in the interior, `\uXXXX` at an
            // edge (named `\n`/`\r` already handled above). Unreachable
            // at an edge in bare form — rule (a) routed the key to
            // quoted form — but kept for completeness.
            c if is_ktav_whitespace(c) => {
                if at_edge {
                    push_unicode_escape(out, c);
                } else {
                    out.push(c);
                }
            }
            // Structural bytes: unreachable in bare form (rule (a)
            // routed the key to quoted form), named escape if ever.
            '.' => out.push_str("\\."),
            ':' => out.push_str("\\:"),
            ',' => out.push_str("\\,"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7F => push_unicode_escape(out, c),
            c => out.push(c),
        }
    }
}

/// Quoted form (§ 5.9.10): `"` delimiter, escaping only `\"`, `\\`,
/// `\n`, `\r`, control bytes < 0x20 (except tab/VT/FF, which
/// `<dq-char>` admits raw) and DEL as `\uXXXX`. Structural bytes,
/// other quote characters, and ANY edge whitespace stay raw — a
/// quoted segment's content is never trimmed (§ 5.3.3).
fn push_quoted_key(key: &str, out: &mut String) {
    out.push('"');
    for ch in key.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if ((c as u32) < 0x20 && !matches!(c, '\t' | '\u{0B}' | '\u{0C}'))
                || (c as u32) == 0x7F =>
            {
                push_unicode_escape(out, c)
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// True if a one-line String body cannot be emitted on the value line
/// at all and must take a multi-line form: the parser trims the body
/// after `:` / `::` with `str::trim()` (§ 4) — the full Unicode
/// `White_Space` set, so an NBSP/NEL/ideographic-space edge is lost
/// the same way an ASCII one is; § 5.9.7 also routes control bytes
/// (other than `TAB`) to the multi-line form. Bodies containing `LF`
/// are multi-line by definition. The first/last-char checks use the
/// FULL § 3.3 class ([`is_ktav_whitespace`]) because the predicate
/// mirrors the parser's body trim (`str::trim()` equivalent, § 4);
/// LF/CR are already covered by the third clause (`b < 0x20`), so a
/// body with LF/CR at an edge is caught regardless.
pub(crate) fn string_needs_multiline(s: &str) -> bool {
    s.chars().next().is_some_and(is_ktav_whitespace)
        || s.chars().next_back().is_some_and(is_ktav_whitespace)
        || s.bytes().any(|b| b < 0x20 && b != b'\t')
}

/// The § 5.9.7 / § 5.9.0 error for a `CR` byte in a String, coded
/// [`ReasonCode::CRByte`]: no text form can hold it (verbatim blocks
/// split on line terminators), so the writer must reject the Value
/// rather than emit a document that parses to a different one.
pub(crate) fn cr_error() -> Error {
    Error::Unrepresentable(ReasonCode::CRByte)
}

/// Which multi-line form a String body should take.
#[derive(Debug)]
pub(crate) enum MultilineForm {
    /// `(` … `)` — parser dedents by the common leading whitespace.
    Stripped,
    /// `((` … `))` — parser copies content lines byte-for-byte.
    Verbatim,
}

/// Pick the multi-line form that reproduces `s` byte-for-byte on
/// re-parse, or return `Error::Unrepresentable` with the matching
/// § 5.9.0 reason code when neither form can (spec § 5.6.1 / § 5.9.0).
///
/// Verbatim breaks on a content line trimming to `))` (it would close
/// the block). Stripped breaks on a line trimming to `)` (it would
/// close the block too), on a whitespace-only line (§ 5.6 blanks it),
/// and on a line with trailing whitespace (spec 0.7 § 5.6 strips it).
/// Stripped also loses leading indentation exactly when the non-blank
/// lines share a common leading-whitespace prefix (§ 5.6 dedents by
/// it), so the decision uses the shared § 5.6 scan
/// [`common_leading_whitespace_prefix_len`] — the very computation
/// both parsers apply on re-parse. "At least one line is unindented"
/// is sufficient for an empty prefix but not necessary: lines whose
/// leading runs differ at some position (a TAB line against a SPACE
/// line, U+2000 against U+2001) share no common prefix at all (§ 5.6)
/// and stay lossless in stripped form.
///
/// The § 5.6 common-prefix scan runs only when the cheap checks above
/// leave the form undecided; on the common paths (verbatim, or the
/// pretty writers' unconditionally-safe stripped) the function returns
/// before it. The scan iterates `s.split('\n')` directly — no
/// intermediate `Vec` of lines.
///
/// `prefer_stripped` keeps the pretty renderers' historical choice
/// (indented stripped output when it is unconditionally safe); the
/// canonical writer passes `false` and prefers verbatim (§ 5.9.7).
pub(crate) fn choose_multiline_form(s: &str, prefer_stripped: bool) -> Result<MultilineForm> {
    let mut sole_single = false;
    let mut sole_double = false;
    let mut ws_only_line = false;
    let mut indented_line = false;
    let mut trailing_ws_line = false;
    for line in s.split('\n') {
        // Callers reject CR bytes (§ 5.9.7) before reaching here and
        // this loop re-splits on LF, so each `line` is terminator-free:
        // use the INLINE whitespace view.
        let trimmed = line.trim_matches(is_inline_whitespace);
        match trimmed {
            ")" => sole_single = true,
            "))" => sole_double = true,
            _ => {}
        }
        if line.is_empty() {
            continue;
        }
        if trimmed.is_empty() {
            ws_only_line = true;
        } else if line.starts_with(is_inline_whitespace) {
            indented_line = true;
        }
        if line.trim_end_matches(is_inline_whitespace).len() != line.len() {
            trailing_ws_line = true;
        }
    }

    let stripped_safe = !sole_single && !ws_only_line && !indented_line && !trailing_ws_line;
    let verbatim_ok = !sole_double;

    if prefer_stripped && stripped_safe {
        Ok(MultilineForm::Stripped)
    } else if verbatim_ok {
        Ok(MultilineForm::Verbatim)
    } else {
        // Only this fallback — reached when the cheap collision checks
        // above left both forms undecided — needs the § 5.6 scan, so it
        // is the one place it runs: the common cases (verbatim, or the
        // pretty writers' unconditionally-safe stripped) return before
        // it. The scan walks `s.split('\n')` directly; no intermediate
        // vector of lines is materialised anywhere in this function.
        //
        // § 5.6: the parser removes from every non-blank line the longest
        // leading § 3.3 whitespace prefix that is identical
        // code-point-for-code-point across every non-blank line's own
        // leading run; blank lines do not participate. Stripped bodies are
        // emitted at indent 0, so the emission is lossless exactly when
        // that common prefix is EMPTY. Same scan as the parsers' dedent
        // (src/parser/collecting.rs, src/thin/event_parser.rs): one notion
        // of "common prefix", no second § 3.3 list.
        let has_common_indent = common_leading_whitespace_prefix_len(s.split('\n')) != 0;
        let stripped_lossless =
            !sole_single && !ws_only_line && !has_common_indent && !trailing_ws_line;
        if stripped_lossless {
            Ok(MultilineForm::Stripped)
        } else {
            // § 5.9.7 / § 5.9.0: verbatim is blocked (a segment trims to
            // `))`) and stripped cannot hold the body losslessly. The
            // remaining blockers map onto the three named collision codes:
            // - a segment trimming to `)`        → BothFormsRequired
            // - any line with trailing whitespace → TrailingWhitespaceCollision
            // - otherwise the residual rejection is exactly § 5.9.7's
            //   condition: every non-blank segment shares at least one
            //   leading whitespace code point in the same position. With
            //   `sole_single` and `trailing_ws_line` both false, a
            //   whitespace-only line is impossible (it would have set
            //   `trailing_ws_line` — its every byte is trailing
            //   whitespace), so `stripped_lossless` can only have failed
            //   via `has_common_indent`: the shared prefix § 5.6's dedent
            //   would strip from the content on re-parse.
            //                                     → LeadingWhitespaceCollision
            let code = if sole_single {
                ReasonCode::BothFormsRequired
            } else if trailing_ws_line {
                ReasonCode::TrailingWhitespaceCollision
            } else {
                ReasonCode::LeadingWhitespaceCollision
            };
            Err(Error::Unrepresentable(code))
        }
    }
}

/// Item-context variant of [`needs_raw_marker`]: a bare array-item
/// line has extra collisions a pair body does not — a leading `##` is
/// a comment line (§ 3.4), a leading `::` is the raw-marker itself,
/// and a sole `]` / `}` closes the enclosing compound.
pub(crate) fn item_needs_raw_marker(s: &str) -> bool {
    needs_raw_marker(s) || s.starts_with("##") || s.starts_with("::") || matches!(s, "]" | "}")
}

/// § 5.9.6 "Bare String item", first-item exclusion: when the item is
/// the FIRST item of an Array root, the bare form is additionally not
/// used if the body satisfies § 5.0.1 rule 6's phase-1 pair-candidate
/// test — a first unescaped `:` or `::` separator (§ 4's
/// separator-scanning rule) with a non-empty raw prefix before it,
/// where a plain `:` is satisfied only when followed by whitespace or
/// end-of-line. Only the Array root's first item is exposed to
/// § 5.0.1's root-kind detection, so only that position is tested.
///
/// § 5.9.6 tests the BARE one-line body against the parser's test.
/// A bare one-line array item never has edge whitespace
/// (`string_needs_multiline` already routed those to the multi-line
/// form), so `body` is exactly the parser's trimmed first content
/// line, and the two tests share one implementation by construction:
/// this delegates to the parser's own
/// [`crate::parser::classify::is_pair_shape`].
///
/// Note that an unterminated leading quote makes the separator scan
/// return no separator at all (the quote swallows the colon), so such
/// bodies stay bare — fixture:
/// `quoted_keys/unterminated_double_quote_first_line_falls_back`.
pub(crate) fn bare_item_is_pair_candidate(body: &str) -> bool {
    crate::parser::classify::is_pair_shape(body)
}

/// True if the value must be emitted with `::` so that the parser does not
/// re-interpret it as a compound (`{...}` / `[...]`), a JSON keyword
/// (`null` / `true` / `false`), a number literal (§ 3.6), or a multi-line
/// opener (`(` / `((`).
pub(crate) fn needs_raw_marker(s: &str) -> bool {
    // Fast path: most scalars don't start with whitespace, so we can check
    // the first byte directly and skip `trim_start`'s whole-string scan.
    match s.as_bytes().first() {
        None => false,
        Some(&b' ') | Some(&b'\t') => {
            needs_raw_marker_slow(s.trim_start_matches(is_inline_whitespace))
        }
        Some(&b'{') | Some(&b'[') => true,
        Some(_) => needs_raw_marker_content(s),
    }
}

fn needs_raw_marker_content(s: &str) -> bool {
    // Keywords, and the bare multi-line openers `(` / `((`, must use `::`.
    // `()` / `(())` parse as the empty String (§ 5.2 rule 5). Other
    // `(`-prefixed bodies (e.g. `(tail`) re-parse as the same String and
    // need no marker (spec 0.7 fixture `inline/paren_scalar_is_string`).
    if matches!(s, "null" | "true" | "false" | "(" | "((" | "()" | "(())") {
        return true;
    }
    // § 5.2 rules 13–14: number literals must be forced to String via `::`
    if classify::matches_integer_grammar(s) || classify::matches_float_grammar(s) {
        return true;
    }
    false
}

#[cold]
#[inline(never)]
fn needs_raw_marker_slow(t: &str) -> bool {
    t.starts_with('{') || t.starts_with('[') || needs_raw_marker_content(t)
}

/// Push `level * INDENT.len()` spaces into `out`. Uses slice copies of a
/// const all-spaces string so the hot path is a single `push_str` →
/// vectorised memcpy instead of a per-level loop of 4-byte pushes.
pub(super) fn push_indent(out: &mut String, level: usize) {
    const SPACES: &str = "                                                                "; // 64
    let mut remaining = level * INDENT.len();
    if remaining == 0 {
        return;
    }
    out.reserve(remaining);
    while remaining > 0 {
        let chunk = remaining.min(SPACES.len());
        out.push_str(&SPACES[..chunk]);
        remaining -= chunk;
    }
}
