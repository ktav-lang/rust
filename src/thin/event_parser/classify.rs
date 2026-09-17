use bumpalo::Bump;

use crate::error::{CompoundKind, Error, ErrorKind, Result, Span};
use crate::parser::classify::{is_float_literal, try_parse_integer};
use crate::parser::inline::InlineBody;
use crate::whitespace::{common_leading_whitespace_prefix_len, is_ktav_whitespace};

use super::super::event::Event;
use super::super::inline_emit::fast_plain_decimal_i64;

use super::state::{Collecting, MultilineMode, PathShape};

// ---------------------------------------------------------------------------
// Bracket kind
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum BracketKind {
    Object = 0,
    Array = 1,
}

impl BracketKind {
    pub(super) fn close(self) -> char {
        match self {
            BracketKind::Object => '}',
            BracketKind::Array => ']',
        }
    }
    pub(super) fn to_compound(self) -> CompoundKind {
        match self {
            BracketKind::Object => CompoundKind::Object,
            BracketKind::Array => CompoundKind::Array,
        }
    }
}

// ---------------------------------------------------------------------------
// Value-start classification (mirrors parser/classify.rs for 0.5.0)
// ---------------------------------------------------------------------------

pub(super) enum ValueStart<'a> {
    Scalar(&'a str),
    Integer(&'a str),
    Float(&'a str),
    Null,
    Bool(bool),
    EmptyObject,
    EmptyArray,
    OpenObject,
    OpenArray,
    OpenMultilineStripped,
    OpenMultilineVerbatim,
    /// Inline compound (§ 5.2 rules 6–9): scanned directly into events
    /// by [`scan_inline_events`] at the match site.
    InlineCompound(InlineBody),
}

pub(super) enum Separator<'a> {
    Raw(&'a str),
    Plain,
}

#[inline]
pub(super) fn require_sep_end(
    rest: &str,
    line_num: usize,
    body_off: u32,
    trimmed_span: Span,
) -> Result<()> {
    // § 3.3 fixed class; `rest` is a suffix of one pre-split line (§ 3.2),
    // so LF/CR cannot occur.
    if rest.is_empty() || rest.starts_with(is_ktav_whitespace) {
        Ok(())
    } else {
        Err(Error::Structured(ErrorKind::MissingSeparatorSpace {
            line: line_num as u32,
            column: 0,
            marker: ':',
            span: Span::new(body_off, trimmed_span.end),
        }))
    }
}

/// Locate `trimmed` inside `raw` and produce its absolute span.
pub(super) fn trimmed_span_in(raw: &str, trimmed: &str, line_start: u32) -> Span {
    if trimmed.is_empty() {
        return Span::new(line_start, line_start);
    }
    let raw_ptr = raw.as_ptr() as usize;
    let trim_ptr = trimmed.as_ptr() as usize;
    debug_assert!(trim_ptr >= raw_ptr && trim_ptr - raw_ptr <= raw.len());
    let off = (trim_ptr - raw_ptr) as u32;
    let start = line_start + off;
    Span::new(start, start + trimmed.len() as u32)
}

#[inline]
pub(super) fn classify_separator<'a>(after_colon: &'a str) -> Separator<'a> {
    if let Some(rest) = after_colon.strip_prefix(':') {
        return Separator::Raw(rest);
    }
    // Under spec 0.5.0, `:i` and `:f` typed markers are removed.
    Separator::Plain
}

/// Map a value event to the same kind label the owned parser's
/// `kind_label` (src/parser/insert.rs) produces — these strings appear
/// verbatim in `ConflictKind::Overwrite` diagnostics (§ 6.3).
#[inline]
pub(super) fn event_label(ev: &Event<'_>) -> &'static str {
    match ev {
        Event::Null => "null",
        Event::Bool(_) => "bool",
        Event::Integer(_) => "integer",
        Event::Float(_) => "float",
        Event::Str(_) => "string",
        Event::BeginArray => "array",
        Event::BeginObject => "object",
        _ => unreachable!("not a value-start event"),
    }
}

/// The [`PathShape`] a keyed value establishes: an `Event::BeginObject`
/// opener makes the path an Object; anything else (including an array,
/// which is a leaf under the owned parser's model) is a leaf of the
/// event's kind.
#[inline]
pub(super) fn path_shape_of(ev: &Event<'_>) -> PathShape {
    match ev {
        Event::BeginObject => PathShape::Object,
        other => PathShape::Leaf(event_label(other)),
    }
}

/// Classify a value body per § 5.2 rules 1-15 (0.5.0). Inline
/// compounds (rules 6-9) are reported as [`ValueStart::InlineCompound`]
/// — the closer triage and event emission happen in
/// [`scan_inline_events`] at the match site, so this function no
/// longer raises inline-compound errors.
#[inline]
pub(super) fn classify<'a>(trimmed: &'a str, bump: &'a Bump) -> Result<ValueStart<'a>> {
    if trimmed == "{" {
        return Ok(ValueStart::OpenObject);
    }
    if trimmed == "[" {
        return Ok(ValueStart::OpenArray);
    }

    // § 5.2 rules 6-9: inline compounds — triaged and scanned by
    // `scan_inline_events` at the call site (the closer scan decides
    // BadEscape / Unterminated / Malformed / closed there). Empty
    // compounds shortcut first.
    if trimmed.starts_with('{') {
        if trimmed.ends_with('}')
            // Empty inline compound (mirror parser/classify.rs).
            && trimmed[1..trimmed.len() - 1]
                .trim_matches(is_ktav_whitespace)
                .is_empty()
        {
            return Ok(ValueStart::EmptyObject);
        }
        return Ok(ValueStart::InlineCompound(InlineBody::Object));
    }

    if trimmed.starts_with('[') {
        if trimmed.ends_with(']')
            // Empty inline compound (mirror parser/classify.rs).
            && trimmed[1..trimmed.len() - 1]
                .trim_matches(is_ktav_whitespace)
                .is_empty()
        {
            return Ok(ValueStart::EmptyArray);
        }
        return Ok(ValueStart::InlineCompound(InlineBody::Array));
    }

    // Multi-line string openers
    match trimmed {
        "(" => return Ok(ValueStart::OpenMultilineStripped),
        "((" => return Ok(ValueStart::OpenMultilineVerbatim),
        "()" | "(())" => return Ok(ValueStart::Scalar("")),
        _ => {}
    }

    // Spec 0.7 § 5.2: only the bare tokens `(` / `((` open multi-line
    // strings; anything else starting with `(` is an ordinary inline
    // scalar and falls through to classification below (mirrors
    // parser/classify.rs; fixture `inline/paren_scalar_is_string`).

    // § 5.2 rules 10-12: keywords
    match trimmed {
        "null" => return Ok(ValueStart::Null),
        "true" => return Ok(ValueStart::Bool(true)),
        "false" => return Ok(ValueStart::Bool(false)),
        _ => {}
    }

    // § 5.2 rule 13: integer literal
    // Fast path for plain decimal (most common case in configs): ASCII
    // digits only, no sign / underscore / base prefix. The input is
    // already canonical — skip itoa formatting and bump allocation.
    if let Some(_val) = fast_plain_decimal_i64(trimmed) {
        return Ok(ValueStart::Integer(trimmed));
    }
    // General path: prefixed, signed, or underscored literals.
    if let Some(val) = try_parse_integer(trimmed) {
        let mut buf = itoa::Buffer::new();
        let canonical = buf.format(val);
        let s = bump.alloc_str(canonical);
        return Ok(ValueStart::Integer(s));
    }

    // § 5.2 rule 14: float literal
    if is_float_literal(trimmed) {
        // If the literal has no underscores we can parse it directly
        // without allocating a cleaned String.
        let has_underscore = trimmed.as_bytes().contains(&b'_');
        if has_underscore {
            let cleaned: String = trimmed.chars().filter(|&c| c != '_').collect();
            if let Ok(val) = cleaned.parse::<f64>() {
                if !val.is_nan() && !val.is_infinite() {
                    let mut buf = ryu::Buffer::new();
                    let canonical = buf.format(val);
                    let s = bump.alloc_str(canonical);
                    return Ok(ValueStart::Float(s));
                }
            }
        } else if let Ok(val) = trimmed.parse::<f64>() {
            if !val.is_nan() && !val.is_infinite() {
                let mut buf = ryu::Buffer::new();
                let canonical = buf.format(val);
                // If ryu reproduces the input, the original slice is
                // canonical — skip the bump allocation.
                if canonical == trimmed {
                    return Ok(ValueStart::Float(trimmed));
                }
                let s = bump.alloc_str(canonical);
                return Ok(ValueStart::Float(s));
            }
        }
    }

    // § 5.2 rule 15: String
    Ok(ValueStart::Scalar(trimmed))
}

// ---------------------------------------------------------------------------
// Multi-line finalize (identical semantics to parser.rs)
// ---------------------------------------------------------------------------

pub(super) fn finalize_multiline<'a>(c: Collecting<'a>, bump: &'a Bump) -> &'a str {
    match c.mode {
        MultilineMode::Verbatim if c.lines.len() == 1 => c.lines[0],
        MultilineMode::Verbatim => {
            let joined = c.lines.join("\n");
            bump.alloc_str(&joined)
        }
        MultilineMode::Stripped if c.lines.len() == 1 => {
            let only = c.lines[0];
            // Multiline stripped finalize (mirror parser/collecting.rs).
            if only.trim_matches(is_ktav_whitespace).is_empty() {
                ""
            } else {
                only.trim_start_matches(is_ktav_whitespace)
                    .trim_end_matches(is_ktav_whitespace)
            }
        }
        MultilineMode::Stripped => {
            let dedented = dedent(&c.lines);
            bump.alloc_str(&dedented)
        }
    }
}

fn dedent(lines: &[&str]) -> String {
    let common_len = common_leading_whitespace_prefix_len(lines.iter().copied());

    // Multiline dedent capacity (mirror parser/collecting.rs): the
    // common prefix is removed only from non-blank lines, so it is
    // subtracted per line. This must stay a per-line subtraction, never
    // `common_len * lines.len()`: that product overflows 32-bit usize
    // on a valid document (R8-F4: common_len = lines.len() = 65_536
    // blank lines included) before any saturating guard could see it.
    // Every non-blank line's own leading run is at least `common_len`
    // bytes (the shared-prefix scan caps at the shortest run), so the
    // subtraction cannot underflow.
    let mut cap: usize = lines
        .iter()
        .filter(|l| !l.trim_matches(is_ktav_whitespace).is_empty())
        .map(|l| l.len() - common_len)
        .sum();
    cap = cap.saturating_add(lines.len());
    let mut out = String::with_capacity(cap);

    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // Multiline dedent (mirror parser/collecting.rs).
        if l.trim_matches(is_ktav_whitespace).is_empty() {
            // blank line
        } else if common_len > 0 && l.len() >= common_len {
            // Char-boundary safety (mirror parser/collecting.rs):
            // `common_len` is the byte length of a code-point sequence
            // that § 5.6 makes a prefix of every non-blank line's own
            // leading run; each run is cut at a char boundary by
            // `leading_whitespace_run`, and the shared-prefix comparison
            // advances only over whole matched code points (ASCII bytes
            // 1:1, non-ASCII in `char` steps). `common_len` therefore
            // lands on a char boundary of this line and the slice below
            // cannot panic.
            out.push_str(l[common_len..].trim_end_matches(is_ktav_whitespace));
        } else {
            out.push_str(l.trim_end_matches(is_ktav_whitespace));
        }
    }
    out
}
