//! The unified structured-error envelope — one JSON object for every
//! structured error, parse-time and writer-time alike (decision record
//! for issue rust#12).
//!
//! Before this module existed, a consumer that wanted machine-readable
//! diagnostics had to pick between [`Error::line`] / [`Error::span`]
//! accessors (Rust-only) and free-form `Display` strings (everything
//! else). The envelope is the single wire contract: every error that
//! has a class renders as exactly one nine-field JSON object, so a
//! consumer of any language can switch on `error` / `reason` and read
//! the remaining fields positionally.
//!
//! ```text
//! // A parse-time LossyScalar on the input "a: 1.10\n" (spec §3.6/§5.2);
//! // the span is whatever the parser reports — here the whole pair line:
//! {"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
//!  "span":{"start":0,"end":7},"path":null,"body":"1.10",
//!  "canonical":"1.1","spec_section":"§3.6/§5.2"}
//! ```

use super::error::{Error, ErrorKind, ReasonCode, Span};
use crate::parser::inline::{decode_key_segment, split_key_path};

/// The unified structured-error envelope: one nine-field JSON object
/// for every structured error, whether it was produced at parse time
/// or at writer time (issue rust#12 decision record).
///
/// ## The wire contract
///
/// [`to_json`](ErrorEnvelope::to_json) always emits all nine fields,
/// in this exact order: `error`, `reason`, `line`, `line_text`,
/// `span`, `path`, `body`, `canonical`, `spec_section`. Absent
/// information is an explicit JSON `null`, never an omitted key —
/// consumers can index every field positionally without a schema
/// negotiation step.
///
/// ## Field semantics
///
/// * `error` — the class name: [`ErrorKind::code_name`] for parse-time
///   errors; `"InvalidUtf8"`, `"Syntax"`, `"Message"`, `"Io"` for the
///   top-level variants. The two writer rejections are named apart,
///   matching their [`Error`] variants: `"Unrepresentable"` when the
///   writer could not say where the offending node is (the streaming
///   serde writers), and `"UnrepresentableAt"` when it could (the
///   Value-walking writers, which also populate `path`). A consumer
///   that only cares that a write was refused can match the
///   `reason` code, which is identical in both cases.
/// * `reason` — writer-time only: the § 5.9.0 reason code
///   ([`ReasonCode::code_name`]), e.g. `"NonFiniteFloat"`. `null` for
///   parse-time errors.
/// * `path` — **an array of exact decoded key segments, never a joined
///   string**. A key literally named `a -> b` or `a.b` is one segment
///   and cannot be confused with a two-segment path; there is no
///   separator in the wire contract. This is the binding rule from the
///   decision record. Populated for `DuplicateKey`, `KeyPathConflict`
///   and `InvalidKey` (raw split segments, undecoded — the raw
///   spelling is often exactly why the key was rejected) and for
///   `Error::UnrepresentableAt`; `null` otherwise.
/// * `span` — JSON shape `{"start":N,"end":M}` where `start` and `end`
///   are **byte offsets into the UTF-8 source text**, not UTF-16 code
///   units — the same unit decision as [`Span`] itself. LSP consumers
///   must convert (or negotiate `positionEncoding: "utf-8"`).
/// * `line`, `line_text`, `span` are populated for parse-time errors
///   only; writer-time errors carry no source position and emit honest
///   nulls. This falls out of the [`Error`] accessors rather than
///   being special-cased: writer-time and `Io`/`Message`/`Syntax`
///   errors have no span, and `line_text` is derived from the span.
/// * `body` — the class-specific text payload: `LossyScalar`'s source
///   form, `BadEscapeSequence`'s offending sequence,
///   `MalformedInlineCompound`'s detail, legacy `InvalidTypedScalar`'s
///   body. The top-level [`Error::Message`] and [`Error::Syntax`]
///   variants carry their text here verbatim — it is the only carrier
///   of that diagnostic on the wire (the uniform C ABI error channel
///   wraps non-ktav errors as `Message` and depends on it). `Other`'s
///   parser-internal `message` is deliberately **not** mapped here —
///   it is a diagnostic, not source text — and stays on
///   `Debug`/`Display` only.
/// * `canonical` — `LossyScalar`'s canonical form.
/// * `spec_section` — the real spec section governing the class (e.g.
///   `"§6.2"`); `null` for classes with none (`Io`, `Message`, legacy
///   `Syntax`, parser-internal `Other`).
///
/// ## Rendering
///
/// [`to_json`](ErrorEnvelope::to_json) produces valid JSON for **any**
/// error payload: every string field is escaped per RFC 8259 (control
/// characters as `\u00xx` with lowercase hex digits, matching
/// serde_json's convention; everything else — DEL, non-ASCII,
/// non-BMP — passes through as raw UTF-8, which is valid JSON). No
/// serde runtime dependency is involved.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorEnvelope {
    /// The error class name (see the type-level contract).
    pub error: String,
    /// Writer-time § 5.9.0 reason code, `None` for parse-time errors.
    pub reason: Option<String>,
    /// 1-based source line, parse-time only.
    pub line: Option<u32>,
    /// The source line containing the span's start, parse-time only.
    pub line_text: Option<String>,
    /// Byte-offset span into the UTF-8 source, parse-time only.
    pub span: Option<Span>,
    /// Exact decoded key segments — never a joined string.
    pub path: Option<Vec<String>>,
    /// Class-specific text payload (see the type-level contract).
    pub body: Option<String>,
    /// `LossyScalar`'s canonical form.
    pub canonical: Option<String>,
    /// Governing spec section, e.g. `"§6.2"`.
    pub spec_section: Option<String>,
}

impl ErrorEnvelope {
    /// Build the envelope for any [`Error`] against the source text it
    /// was (or would have been) parsed from. Construction is total:
    /// it never panics and never fails, for any input pair.
    ///
    /// Uniform rules:
    ///
    /// * `line` = [`Error::line`], `span` = [`Error::span`].
    /// * `line_text` = the source line containing `span.start`,
    ///   whenever a span exists and `source` is non-empty; `None`
    ///   otherwise. Writer-time errors have no span, so `line`,
    ///   `line_text` and `span` are all null there — the decision
    ///   record's "writer-time errors emit honest nulls" rule falls
    ///   out of the accessors, it is not special-cased.
    /// * `path` = `Some` only for `DuplicateKey` / `KeyPathConflict`
    ///   (raw text decoded via `split_key_path` + `decode_key_segment`;
    ///   a decode failure falls back to the raw segment spelling) and
    ///   `InvalidKey` (raw split segments, **not** decoded — the raw
    ///   text may be undecodable by definition, and that spelling is
    ///   what the consumer needs to see to fix it); `null` for every
    ///   other kind. `UnrepresentableAt` carries its decoded path.
    pub fn from_error(error: &Error, source: &str) -> ErrorEnvelope {
        let span = error.span();
        let line_text = span.and_then(|s| line_text_at(source, s.start as usize));
        let (name, reason, path, body, canonical, spec_section) = match error {
            Error::Io(_) => ("Io".to_string(), None, None, None, None, None),
            Error::Structured(kind) => (
                kind.code_name().to_string(),
                None,
                kind_path(kind),
                kind_body(kind),
                kind_canonical(kind),
                kind_spec_section(kind).map(str::to_string),
            ),
            Error::Syntax(m) => (
                "Syntax".to_string(),
                None,
                None,
                Some(m.clone()),
                None,
                None,
            ),
            Error::Message(m) => (
                "Message".to_string(),
                None,
                None,
                Some(m.clone()),
                None,
                None,
            ),
            Error::Unrepresentable(code) => (
                "Unrepresentable".to_string(),
                Some(code.code_name().to_string()),
                None,
                None,
                None,
                code_spec_section(code).map(str::to_string),
            ),
            Error::UnrepresentableAt { code, path } => (
                "UnrepresentableAt".to_string(),
                Some(code.code_name().to_string()),
                Some(path.clone()),
                None,
                None,
                code_spec_section(code).map(str::to_string),
            ),
            Error::InvalidUtf8 { .. } => (
                "InvalidUtf8".to_string(),
                None,
                None,
                None,
                None,
                Some("§6.15".to_string()),
            ),
        };
        ErrorEnvelope {
            error: name,
            reason,
            line: error.line(),
            line_text,
            span,
            path,
            body,
            canonical,
            spec_section,
        }
    }

    /// Render the envelope as one JSON object string.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(256);
        self.push_json(&mut out);
        out
    }

    /// Append the JSON object to `out` without an intermediate
    /// allocation. `to_json` delegates here.
    pub fn push_json(&self, out: &mut String) {
        out.push('{');
        out.push_str("\"error\":");
        push_json_string(out, &self.error);
        out.push_str(",\"reason\":");
        push_json_opt_string(out, self.reason.as_deref());
        out.push_str(",\"line\":");
        match self.line {
            Some(line) => push_u32(out, line),
            None => out.push_str("null"),
        }
        out.push_str(",\"line_text\":");
        push_json_opt_string(out, self.line_text.as_deref());
        out.push_str(",\"span\":");
        match self.span {
            Some(span) => {
                out.push_str("{\"start\":");
                push_u32(out, span.start);
                out.push_str(",\"end\":");
                push_u32(out, span.end);
                out.push('}');
            }
            None => out.push_str("null"),
        }
        out.push_str(",\"path\":");
        match &self.path {
            Some(path) => {
                out.push('[');
                for (i, segment) in path.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    push_json_string(out, segment);
                }
                out.push(']');
            }
            None => out.push_str("null"),
        }
        out.push_str(",\"body\":");
        push_json_opt_string(out, self.body.as_deref());
        out.push_str(",\"canonical\":");
        push_json_opt_string(out, self.canonical.as_deref());
        out.push_str(",\"spec_section\":");
        push_json_opt_string(out, self.spec_section.as_deref());
        out.push('}');
    }
}

/// The source line containing byte offset `pos`, or `None` when `pos`
/// is out of bounds or not on a char boundary (defensive — envelopes
/// must build for any error/source pair). `pos == source.len()` (EOF
/// spans) yields the last line. One trailing `\r` is stripped so a
/// CRLF terminator does not leak into the payload.
fn line_text_at(source: &str, pos: usize) -> Option<String> {
    if source.is_empty() {
        return None;
    }
    let pos = pos.min(source.len());
    if !source.is_char_boundary(pos) {
        return None;
    }
    let bytes = source.as_bytes();
    let line_start = bytes[..pos]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |i| i + 1);
    let line_end = bytes[pos..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(source.len(), |i| pos + i);
    let mut line = &source[line_start..line_end];
    if line.ends_with('\r') {
        line = &line[..line.len() - 1];
    }
    Some(line.to_string())
}

/// Decode the raw key-path text carried by `DuplicateKey` /
/// `KeyPathConflict` / `InvalidKey` into exact segments. `InvalidKey`
/// is deliberately not decoded — its raw text may be undecodable by
/// definition, and that exact spelling is the useful payload.
fn kind_path(kind: &ErrorKind) -> Option<Vec<String>> {
    let raw = match kind {
        ErrorKind::DuplicateKey { key, .. } => key,
        ErrorKind::KeyPathConflict { path, .. } => path,
        ErrorKind::InvalidKey { key, .. } => {
            return Some(
                split_key_path(key)
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            );
        }
        _ => return None,
    };
    let mut segments = Vec::with_capacity(4);
    for segment in split_key_path(raw) {
        match decode_key_segment(segment, 0, Span::EMPTY) {
            Ok(decoded) => segments.push(decoded.into_owned()),
            // Fall back to the raw spelling so construction stays total.
            Err(_) => segments.push(segment.to_string()),
        }
    }
    Some(segments)
}

fn kind_body(kind: &ErrorKind) -> Option<String> {
    match kind {
        ErrorKind::LossyScalar { body, .. } | ErrorKind::InvalidTypedScalar { body, .. } => {
            Some(body.clone())
        }
        ErrorKind::BadEscapeSequence { sequence, .. } => Some(sequence.clone()),
        ErrorKind::MalformedInlineCompound { detail, .. } => Some(detail.clone()),
        // `Other`'s parser-internal `message` is deliberately not
        // mapped to `body` — it is a diagnostic, not source text.
        _ => None,
    }
}

fn kind_canonical(kind: &ErrorKind) -> Option<String> {
    match kind {
        ErrorKind::LossyScalar { canonical, .. } => Some(canonical.clone()),
        _ => None,
    }
}

fn kind_spec_section(kind: &ErrorKind) -> Option<&'static str> {
    match kind {
        ErrorKind::MissingSeparatorSpace { .. } => Some("§6.10"),
        ErrorKind::InvalidTypedScalar { .. } => Some("§6.9"),
        ErrorKind::LossyScalar { .. } => Some("§3.6/§5.2"),
        ErrorKind::DuplicateKey { .. } => Some("§6.2"),
        ErrorKind::KeyPathConflict { .. } => Some("§6.3"),
        ErrorKind::EmptyKey { .. } => Some("§6.5"),
        ErrorKind::InvalidKey { .. } => Some("§6.4"),
        ErrorKind::UnclosedCompound { .. } => Some("§6.1"),
        ErrorKind::UnbalancedBracket { .. } => Some("§6.1"),
        ErrorKind::InlineNonEmptyCompound { .. } => Some("§6.7"),
        ErrorKind::MissingSeparator { .. } => Some("§6.6"),
        ErrorKind::UnterminatedInlineCompound { .. } => Some("§6.11"),
        ErrorKind::UnterminatedQuotedKey { .. } => Some("§6.16"),
        ErrorKind::MalformedInlineCompound { .. } => Some("§6.12"),
        ErrorKind::BadEscapeSequence { .. } => Some("§6.13"),
        ErrorKind::OrphanLineAfterTopLevelInline { .. } => Some("§6.14"),
        ErrorKind::Other { .. } => None,
    }
}

fn code_spec_section(code: &ReasonCode) -> Option<&'static str> {
    match code {
        ReasonCode::ScalarRoot | ReasonCode::EmptyKeyName | ReasonCode::NonFiniteFloat => {
            Some("§5.9.0")
        }
        ReasonCode::CRByte
        | ReasonCode::BothFormsRequired
        | ReasonCode::TrailingWhitespaceCollision
        | ReasonCode::LeadingWhitespaceCollision => Some("§5.9.7"),
    }
}

fn push_u32(out: &mut String, value: u32) {
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(value));
}

fn push_json_opt_string(out: &mut String, value: Option<&str>) {
    match value {
        Some(s) => push_json_string(out, s),
        None => out.push_str("null"),
    }
}

/// Push `s` as a `"…"`-quoted JSON string. RFC 8259 escaping:
/// control characters as `\u00xx` with lowercase hex digits (matching
/// serde_json's convention); DEL, non-ASCII and non-BMP characters
/// pass through as raw UTF-8, which is valid JSON — a Rust `str`
/// cannot hold lone surrogates, so no surrogate pairing is needed.
fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let n = c as u32;
                out.push_str("\\u00");
                out.push(HEX[(n >> 4) as usize] as char);
                out.push(HEX[(n & 0xf) as usize] as char);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
