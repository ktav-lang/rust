//! Top-level root-kind detection (spec § 5.0.1).

use crate::error::{Error, ErrorKind, Span};
use crate::value::Value;

use crate::parser::classify::is_pair_shape;
use crate::parser::inline;

/// Result of top-level kind detection (§ 5.0.1).
pub(in crate::document::parser) enum RootResult {
    /// § 5.0.1 rule 2: first content line is a closed inline object.
    InlineObject(Value),
    /// § 5.0.1 rule 3: first content line is a closed inline array.
    InlineArray(Value),
    /// § 5.0.1 rule 4: first content line is a lone `{`.
    ExplicitObject,
    /// § 5.0.1 rule 5: first content line is a lone `[`.
    ExplicitArray,
    /// § 5.0.1 rule 6: pair-shape → implicit Object root.
    Object,
    /// § 5.0.1 rule 7: array-item → implicit Array root.
    Array,
}

/// Top-level kind detection (spec § 5.0.1, 0.5.0).
///
/// Applies all 8 rules in order. Rules 1 (no content lines) and 8
/// (bare `}` / `]`) are handled by the caller.
pub(in crate::document::parser) fn classify_root_kind_050(
    trimmed: &str,
    line_num: usize,
    trimmed_span: Span,
    strict: bool,
) -> Result<RootResult, Error> {
    // Rule 4: lone `{`
    if trimmed == "{" {
        return Ok(RootResult::ExplicitObject);
    }
    // Rule 5: lone `[`
    if trimmed == "[" {
        return Ok(RootResult::ExplicitArray);
    }

    // § 5.0.1 rules 2/3, plus the rules-2–5 addendum: a first content
    // line beginning with `{`/`[` is diagnosed by the same § 5.2 scan as
    // a value body — closed at the end ⇒ the root IS the inline value;
    // closer followed by content ⇒ MalformedInlineCompound (§ 6.12); no
    // closer ⇒ UnterminatedInlineCompound (§ 6.11); BadEscapeSequence
    // wins per the § 5.2 rules-6–9 preamble. This precedence applies
    // before rule 6: such a line is never a pair candidate (`[bad]: 1`).
    if trimmed.starts_with('{') {
        return diagnose_root_inline(trimmed, b'{', b'}', line_num, trimmed_span, strict);
    }
    if trimmed.starts_with('[') {
        return diagnose_root_inline(trimmed, b'[', b']', line_num, trimmed_span, strict);
    }

    // Rule 6/7: pair-shape vs array-item. Use the same heuristic
    // as before.
    if is_pair_shape(trimmed) {
        Ok(RootResult::Object)
    } else {
        Ok(RootResult::Array)
    }
}

/// § 5.0.1 rules 2/3 + rules-2–5 addendum, for a first content line
/// beginning with `{`/`[` that is not a lone opener: the same § 5.2
/// rules 6–9 closer scan as a value body decides closed-inline root vs
/// `MalformedInlineCompound` (§ 6.12) vs `UnterminatedInlineCompound`
/// (§ 6.11), with `BadEscapeSequence` taking precedence (§ 5.2
/// rules-6–9 preamble). Such a line is never a pair candidate.
fn diagnose_root_inline(
    trimmed: &str,
    open: u8,
    close: u8,
    line_num: usize,
    span: Span,
    strict: bool,
) -> Result<RootResult, Error> {
    let mut bounds_pairs = Vec::new();
    match inline::scan_inline_closer_with_bounds(
        trimmed,
        open,
        close,
        line_num,
        span,
        &mut bounds_pairs,
    ) {
        inline::InlineCloserScan::BadEscape(err) => Err(err),
        inline::InlineCloserScan::NotFound => {
            Err(Error::Structured(ErrorKind::UnterminatedInlineCompound {
                line: line_num as u32,
                span,
            }))
        }
        inline::InlineCloserScan::Found(idx) if idx == trimmed.len() - 1 => {
            // `parse_inline_*` already yields the empty compound for
            // `{}` / `[]` (empty inner → default value), so no separate
            // empty shortcut is needed.
            if open == b'{' {
                let bounds = inline::InlineBounds::over(trimmed, &bounds_pairs);
                let value = inline::parse_inline_object(trimmed, line_num, span, strict, bounds)?;
                Ok(RootResult::InlineObject(value))
            } else {
                let bounds = inline::InlineBounds::over(trimmed, &bounds_pairs);
                let value = inline::parse_inline_array(trimmed, line_num, span, strict, bounds)?;
                Ok(RootResult::InlineArray(value))
            }
        }
        inline::InlineCloserScan::Found(_) => {
            Err(inline::malformed_closer_not_at_end(line_num, span))
        }
    }
}
