//! The unified `Error` enum plus its std / serde `Error` trait impls.
//!
//! ## Structured vs free-form errors
//!
//! Historically every parse failure was funneled into `Error::Syntax(String)`
//! and callers (notably `editor/lsp/src/diagnostics.rs` and the seven
//! non-Rust bindings) recovered structure (line, column, category) by
//! re-parsing the formatted message with regexes. Starting with `0.1.5`
//! the parser emits `Error::Structured(ErrorKind)` instead — every call
//! site retains `(line, column, kind)` so callers no longer have to
//! reverse-engineer it. `0.1.6` extends every variant with a [`Span`]
//! describing the exact byte range in the source input, and adds three
//! new variants (`UnbalancedBracket`, `InlineNonEmptyCompound`,
//! `MissingSeparator`) promoted out of `ErrorKind::Other`.
//!
//! `Error::Syntax(String)` is **kept** as a public variant for backward
//! compatibility with `0.1.x` callers that pattern-match on the string.
//! The parser itself never constructs it any more, but external code
//! (tests, downstream wrappers) is free to.
//!
//! ## Display parity (the contract)
//!
//! `<ErrorKind as Display>` is *byte-identical* to what the matching
//! `Error::Syntax(format!(…))` site produced in `0.1.4` for the seven
//! categories pinned in `tests/error_format.rs`. The three new
//! categories added in `0.1.6` get their own pinned strings in the same
//! file.
//!
//! ## Forward compatibility
//!
//! Both `Error` and the three error-kind enums are `#[non_exhaustive]`
//! so future additive variants don't constitute a breaking change.

use std::fmt::{self, Display};
use std::io;

/// A byte-offset range inside the original input string. `start..end`
/// is half-open (Rust convention). `start == end` denotes an
/// insertion-point span (no bytes covered).
///
/// For convenience, [`Span::slice`] returns the substring covered by
/// the span and [`Span::line_col`] returns the 1-based line and
/// 0-based column at the span's start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Inclusive byte offset of the first covered byte.
    pub start: u32,
    /// Exclusive byte offset of the byte past the covered range.
    pub end: u32,
}

impl Span {
    /// Construct a new span. No checks are performed — callers are
    /// expected to keep `start <= end` and both within `input.len()`.
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Empty / placeholder span. Used for `Other`-class internal-state
    /// errors where no source range is meaningful.
    pub const EMPTY: Span = Span { start: 0, end: 0 };

    /// Borrow the substring covered by this span out of `input`.
    /// Returns `None` if the offsets are outside `input` or fall on
    /// non-UTF-8 character boundaries.
    pub fn slice<'a>(&self, input: &'a str) -> Option<&'a str> {
        let start = self.start as usize;
        let end = self.end as usize;
        if start > end || end > input.len() {
            return None;
        }
        if !input.is_char_boundary(start) || !input.is_char_boundary(end) {
            return None;
        }
        Some(&input[start..end])
    }

    /// Compute the 1-based line and 0-based column corresponding to
    /// this span's `start` offset within `input`. Counts `\n` as a
    /// line break; `\r` is treated as part of the previous line.
    /// Out-of-range start clamps to `input.len()`.
    pub fn line_col(&self, input: &str) -> (u32, u32) {
        let start = (self.start as usize).min(input.len());
        let bytes = input.as_bytes();
        let mut line: u32 = 1;
        let mut last_nl: usize = 0;
        for (i, &b) in bytes.iter().enumerate().take(start) {
            if b == b'\n' {
                line += 1;
                last_nl = i + 1;
            }
        }
        let col = (start - last_nl) as u32;
        (line, col)
    }
}

/// The single error type returned by every entry point.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// I/O error while reading from disk.
    Io(io::Error),
    /// Structured parse error — preferred form, emitted by the parser
    /// since `0.1.5`. Carries `(line, [span], kind)` for callers that
    /// need to drive IDE diagnostics, pretty-printers, etc.
    Structured(ErrorKind),
    /// Free-form syntax error (legacy). The parser no longer constructs
    /// this directly; new code should prefer `Error::Structured`. Kept
    /// public for `0.1.x` backward-compat.
    ///
    /// The message may carry a category prefix in the form
    /// `"Line N: Category: ..."` for specific error classes such as
    /// `InvalidTypedScalar`.
    Syntax(String),
    /// A custom message produced by `serde` during (de)serialization —
    /// for example a type mismatch or a missing field.
    Message(String),
}

/// Structured parse-error category. Each variant carries the
/// information the parser had at the point of failure; `Display` for
/// `ErrorKind` reproduces byte-for-byte the string the legacy
/// `Error::Syntax(format!(…))` call site produced in `0.1.4` for the
/// seven categories that existed then; the three new categories added
/// in `0.1.6` (`UnbalancedBracket`, `InlineNonEmptyCompound`,
/// `MissingSeparator`) have their own pinned Display strings.
///
/// `#[non_exhaustive]` so future kinds can be added without breakage.
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// A separator (`:`, `::`, `:i`, `:f`) is glued to its body without
    /// the mandatory whitespace / end-of-line. Spec § 5.3 / § 5.4.
    ///
    /// `column` is the 0-based column inside the line; `span` covers
    /// the offending separator-and-glued-body region.
    MissingSeparatorSpace {
        line: u32,
        column: u32,
        marker: char,
        span: Span,
    },
    /// **Spec 0.5.0 § 6.9 (reserved).**
    ///
    /// In 0.1.x this was emitted for invalid `:i` / `:f` bodies. In 0.5.0
    /// typed markers no longer exist. This variant is kept for backward
    /// compatibility but the parser never constructs it.
    #[doc(hidden)]
    InvalidTypedScalar {
        line: u32,
        marker: char,
        body: String,
        span: Span,
    },
    /// Two pairs in the same object share the same key.
    DuplicateKey { line: u32, key: String, span: Span },
    /// A dotted-key insertion clashed with an existing entry. `kind`
    /// distinguishes the three concrete shapes the parser observes.
    KeyPathConflict {
        line: u32,
        path: String,
        kind: ConflictKind,
        span: Span,
    },
    /// `: value` — the line had no key before the separator.
    EmptyKey { line: u32, span: Span },
    /// A key contained an invalid byte (whitespace, `[`, `]`, `{`, `}`,
    /// `:`, `#`) or was empty after segmentation.
    InvalidKey { line: u32, key: String, span: Span },
    /// The document ended while a compound (object, array, multi-line
    /// string) was still open. `span` covers from the opener through
    /// EOF (or `Span::EMPTY` if the opener wasn't tracked).
    UnclosedCompound { kind: CompoundKind, span: Span },
    /// A closer (`}` or `]`) appeared without a matching open compound,
    /// or with the wrong shape relative to the open one.
    /// `expected` records what was actually open (or could be opened),
    /// and `found` records the literal closer character that appeared.
    UnbalancedBracket {
        line: u32,
        span: Span,
        expected: CompoundKind,
        found: char,
    },
    /// **Spec 0.5.0 § 6.7 (reserved).**
    ///
    /// In 0.1.x this was emitted when a non-empty compound appeared on a
    /// single line. In 0.5.0 inline non-empty compounds are valid (§ 5.8).
    /// This variant is kept for backward compatibility but the parser never
    /// constructs it.
    #[doc(hidden)]
    InlineNonEmptyCompound { line: u32, span: Span, body: String },
    /// A non-blank, non-comment, non-closer, non-array-item line lacks
    /// the `:` that would make it a `key: value` pair. The whole
    /// trimmed line is the offending span.
    MissingSeparator { line: u32, span: Span },
    /// A `{` or `[` in a value-position is not followed by a matching
    /// `}` / `]` on the same line. Spec § 6.11.
    UnterminatedInlineCompound { line: u32, span: Span },
    /// A structural defect inside a closed inline compound — leading
    /// comma, double comma, empty array item, missing pair separator.
    /// Spec § 6.12.
    MalformedInlineCompound {
        line: u32,
        span: Span,
        detail: String,
    },
    /// A `\X` form inside an inline scalar value where `X` is not one
    /// of the eight recognised escape characters. Spec § 6.13.
    BadEscapeSequence {
        line: u32,
        span: Span,
        sequence: String,
    },
    /// A non-blank, non-comment line that appears after a top-level
    /// inline compound root has already been fully consumed.
    /// Spec § 6.14.
    OrphanLineAfterTopLevelInline { line: u32, span: Span },
    /// Escape hatch for parser failures that don't map to one of the
    /// canonical categories — currently only parser-internal invariant
    /// violations (`pending key already set`, `closed compound without
    /// pending key`, `multi-line string closed without pending key`).
    Other {
        line: Option<u32>,
        message: String,
        span: Span,
    },
}

/// Sub-classification for `ErrorKind::KeyPathConflict`.
///
/// `#[non_exhaustive]` so future conflict shapes can be added without
/// breaking pattern-match exhaustiveness in downstream crates.
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    /// Existing scalar/array would have to be overwritten with an
    /// object-shaped value (or vice versa).
    Overwrite {
        existing: &'static str,
        new_kind: &'static str,
    },
    /// A dotted-key tried to descend into / through an existing scalar
    /// (`db: 1` then `db.x: 2`).
    BlockedByValue,
    /// A synthetic dotted-key prefix was re-opened after an
    /// intervening different prefix had closed it (event-stream
    /// path only).
    SyntheticReopen,
}

/// Sub-classification for `ErrorKind::UnclosedCompound` and
/// `ErrorKind::UnbalancedBracket`.
///
/// `#[non_exhaustive]` so future compound flavours can be added
/// without breakage.
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundKind {
    Object,
    Array,
    MultilineStripped,
    MultilineVerbatim,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::MissingSeparatorSpace { line, .. } => write!(
                f,
                "Line {}: MissingSeparatorSpace: separator must be followed by whitespace or end of line",
                line
            ),
            ErrorKind::InvalidTypedScalar { line, body, .. } => {
                write!(f, "Line {}: InvalidTypedScalar: {}", line, body)
            }
            ErrorKind::DuplicateKey { line, key, .. } => {
                write!(f, "Line {}: duplicate key '{}'", line, key)
            }
            ErrorKind::KeyPathConflict { line, path, kind, .. } => match kind {
                ConflictKind::Overwrite { existing, new_kind } => write!(
                    f,
                    "Line {}: conflict at '{}' \u{2014} cannot overwrite {} with {}",
                    line, path, existing, new_kind
                ),
                ConflictKind::BlockedByValue => write!(
                    f,
                    "Line {}: conflict at '{}' \u{2014} an existing value blocks the path",
                    line, path
                ),
                ConflictKind::SyntheticReopen => write!(
                    f,
                    "Line {}: conflict at '{}' \u{2014} synthetic dotted-key prefix already closed by an intervening different prefix; group lines with the same prefix together",
                    line, path
                ),
            },
            ErrorKind::EmptyKey { line, .. } => write!(f, "Empty key at line {}", line),
            ErrorKind::InvalidKey { line, key, .. } => {
                write!(f, "Invalid key at line {}: '{}'", line, key)
            }
            ErrorKind::UnclosedCompound { kind, .. } => match kind {
                CompoundKind::Object => write!(f, "Unclosed object at end of input"),
                CompoundKind::Array => write!(f, "Unclosed array at end of input"),
                CompoundKind::MultilineStripped | CompoundKind::MultilineVerbatim => {
                    write!(f, "Unclosed multi-line string at end of input")
                }
            },
            ErrorKind::UnbalancedBracket {
                line,
                expected,
                found,
                ..
            } => {
                let opener = match expected {
                    CompoundKind::Object => '{',
                    CompoundKind::Array => '[',
                    // Multi-line markers can't be opened by `{` / `[`,
                    // but if a future caller wires this up we still
                    // produce sensible output.
                    CompoundKind::MultilineStripped | CompoundKind::MultilineVerbatim => '(',
                };
                write!(
                    f,
                    "Line {}: UnbalancedBracket: '{}' without matching '{}'",
                    line, found, opener
                )
            }
            ErrorKind::InlineNonEmptyCompound { line, body, .. } => write!(
                f,
                "Line {}: InlineNonEmptyCompound: inline non-empty {} is not supported; put entries on separate lines",
                line, body
            ),
            ErrorKind::MissingSeparator { line, .. } => write!(
                f,
                "Line {}: MissingSeparator: object entries must be 'key: value' pairs",
                line
            ),
            ErrorKind::UnterminatedInlineCompound { line, .. } => write!(
                f,
                "Line {}: UnterminatedInlineCompound: inline compound not closed on the same line",
                line
            ),
            ErrorKind::MalformedInlineCompound { line, detail, .. } => {
                write!(
                    f,
                    "Line {}: MalformedInlineCompound: {}",
                    line, detail
                )
            }
            ErrorKind::BadEscapeSequence { line, sequence, .. } => write!(
                f,
                "Line {}: BadEscapeSequence: invalid escape sequence '{}'",
                line, sequence
            ),
            ErrorKind::OrphanLineAfterTopLevelInline { line, .. } => write!(
                f,
                "Line {}: OrphanLineAfterTopLevelInline: content after root-level inline compound",
                line
            ),
            ErrorKind::Other { message, .. } => f.write_str(message),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Structured(k) => write!(f, "Syntax error: {}", k),
            Error::Syntax(m) => write!(f, "Syntax error: {}", m),
            Error::Message(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

// ---------------------------------------------------------------------------
// Convenience accessors on `Error`
// ---------------------------------------------------------------------------

impl Error {
    /// Returns the 1-based line number associated with the error, if
    /// available. `None` for [`Error::Io`], [`Error::Message`], free-
    /// form [`Error::Syntax`], EOF-detected `UnclosedCompound`, and
    /// the parser-internal `Other` variants that lack a line number.
    pub fn line(&self) -> Option<u32> {
        match self {
            Error::Structured(k) => k.line(),
            _ => None,
        }
    }

    /// Returns the byte-offset span associated with the error, if
    /// available. `None` for [`Error::Io`], [`Error::Message`] and
    /// [`Error::Syntax`]. May return `Some(Span::EMPTY)` for an
    /// internal-state structured error that has no meaningful source
    /// range.
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Structured(k) => Some(k.span()),
            _ => None,
        }
    }
}

impl ErrorKind {
    /// 1-based line number, or `None` for variants where the failure
    /// is detected at EOF or carries no line context.
    pub fn line(&self) -> Option<u32> {
        match self {
            ErrorKind::MissingSeparatorSpace { line, .. }
            | ErrorKind::InvalidTypedScalar { line, .. }
            | ErrorKind::DuplicateKey { line, .. }
            | ErrorKind::KeyPathConflict { line, .. }
            | ErrorKind::EmptyKey { line, .. }
            | ErrorKind::InvalidKey { line, .. }
            | ErrorKind::UnbalancedBracket { line, .. }
            | ErrorKind::InlineNonEmptyCompound { line, .. }
            | ErrorKind::MissingSeparator { line, .. }
            | ErrorKind::UnterminatedInlineCompound { line, .. }
            | ErrorKind::MalformedInlineCompound { line, .. }
            | ErrorKind::BadEscapeSequence { line, .. }
            | ErrorKind::OrphanLineAfterTopLevelInline { line, .. } => Some(*line),
            ErrorKind::UnclosedCompound { .. } => None,
            ErrorKind::Other { line, .. } => *line,
        }
    }

    /// Byte-offset span covering the offending source region.
    pub fn span(&self) -> Span {
        match self {
            ErrorKind::MissingSeparatorSpace { span, .. }
            | ErrorKind::InvalidTypedScalar { span, .. }
            | ErrorKind::DuplicateKey { span, .. }
            | ErrorKind::KeyPathConflict { span, .. }
            | ErrorKind::EmptyKey { span, .. }
            | ErrorKind::InvalidKey { span, .. }
            | ErrorKind::UnclosedCompound { span, .. }
            | ErrorKind::UnbalancedBracket { span, .. }
            | ErrorKind::InlineNonEmptyCompound { span, .. }
            | ErrorKind::MissingSeparator { span, .. }
            | ErrorKind::UnterminatedInlineCompound { span, .. }
            | ErrorKind::MalformedInlineCompound { span, .. }
            | ErrorKind::BadEscapeSequence { span, .. }
            | ErrorKind::OrphanLineAfterTopLevelInline { span, .. }
            | ErrorKind::Other { span, .. } => *span,
        }
    }
}
