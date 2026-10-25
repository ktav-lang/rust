use std::fmt::Display;
use std::io;

use super::{Error, ErrorKind, ReasonCode, Span};

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
    /// form [`Error::Syntax`], [`Error::Unrepresentable`],
    /// [`Error::UnrepresentableAt`] and parser-internal `Other` errors
    /// that lack a line number.
    pub fn line(&self) -> Option<u32> {
        match self {
            Error::Structured(k) => k.line(),
            Error::InvalidUtf8 { line, .. } => Some(*line),
            _ => None,
        }
    }

    /// Returns the byte-offset span associated with the error, if
    /// available. `None` for [`Error::Io`], [`Error::Message`],
    /// [`Error::Syntax`], [`Error::Unrepresentable`] and
    /// [`Error::UnrepresentableAt`]. [`Error::InvalidUtf8`] covers the
    /// first invalid UTF-8 sequence. May return `Some(Span::EMPTY)` for an
    /// internal-state structured error that has no meaningful source
    /// range.
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Structured(k) => Some(k.span()),
            Error::InvalidUtf8 { span, .. } => Some(*span),
            _ => None,
        }
    }

    /// Returns the § 5.9.0 reason code if this is an
    /// [`Error::Unrepresentable`] or [`Error::UnrepresentableAt`]
    /// writer rejection, else `None`.
    pub fn reason_code(&self) -> Option<ReasonCode> {
        match self {
            Error::Unrepresentable(code) => Some(*code),
            Error::UnrepresentableAt { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// Byte offset of the first invalid UTF-8 sequence if this is an
    /// [`Error::InvalidUtf8`] error, else `None`.
    pub fn valid_up_to(&self) -> Option<usize> {
        match self {
            Error::InvalidUtf8 { valid_up_to, .. } => Some(*valid_up_to),
            _ => None,
        }
    }
}

impl ErrorKind {
    /// 1-based line number; `None` only for internal errors without context.
    pub fn line(&self) -> Option<u32> {
        match self {
            ErrorKind::MissingSeparatorSpace { line, .. }
            | ErrorKind::InvalidTypedScalar { line, .. }
            | ErrorKind::LossyScalar { line, .. }
            | ErrorKind::DuplicateKey { line, .. }
            | ErrorKind::KeyPathConflict { line, .. }
            | ErrorKind::EmptyKey { line, .. }
            | ErrorKind::InvalidKey { line, .. }
            | ErrorKind::UnbalancedBracket { line, .. }
            | ErrorKind::InlineNonEmptyCompound { line, .. }
            | ErrorKind::MissingSeparator { line, .. }
            | ErrorKind::UnterminatedInlineCompound { line, .. }
            | ErrorKind::UnterminatedQuotedKey { line, .. }
            | ErrorKind::MalformedInlineCompound { line, .. }
            | ErrorKind::BadEscapeSequence { line, .. }
            | ErrorKind::OrphanLineAfterTopLevelInline { line, .. }
            | ErrorKind::UnclosedCompound { line, .. } => Some(*line),
            ErrorKind::Other { line, .. } => *line,
        }
    }

    /// Byte-offset span covering the offending source region.
    pub fn span(&self) -> Span {
        match self {
            ErrorKind::MissingSeparatorSpace { span, .. }
            | ErrorKind::InvalidTypedScalar { span, .. }
            | ErrorKind::LossyScalar { span, .. }
            | ErrorKind::DuplicateKey { span, .. }
            | ErrorKind::KeyPathConflict { span, .. }
            | ErrorKind::EmptyKey { span, .. }
            | ErrorKind::InvalidKey { span, .. }
            | ErrorKind::UnclosedCompound { span, .. }
            | ErrorKind::UnbalancedBracket { span, .. }
            | ErrorKind::InlineNonEmptyCompound { span, .. }
            | ErrorKind::MissingSeparator { span, .. }
            | ErrorKind::UnterminatedInlineCompound { span, .. }
            | ErrorKind::UnterminatedQuotedKey { span, .. }
            | ErrorKind::MalformedInlineCompound { span, .. }
            | ErrorKind::BadEscapeSequence { span, .. }
            | ErrorKind::OrphanLineAfterTopLevelInline { span, .. }
            | ErrorKind::Other { span, .. } => *span,
        }
    }
}
