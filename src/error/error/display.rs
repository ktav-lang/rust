use std::fmt::{self, Display};

use super::{CompoundKind, ConflictKind, Error, ErrorKind, ReasonCode};

impl Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReasonCode::ScalarRoot => write!(
                f,
                "ScalarRoot: the document root is not an Object or an Array (spec § 5.9.0)"
            ),
            ReasonCode::EmptyKeyName => write!(
                f,
                "EmptyKeyName: an Object pair's name is the empty string (spec § 5.9.0)"
            ),
            ReasonCode::NonFiniteFloat => {
                write!(f, "NonFiniteFloat: a Float is NaN or ±Infinity (spec § 5.9.0)")
            }
            ReasonCode::CRByte => write!(
                f,
                "CRByte: a String contains a CR byte (spec § 5.9.0 / § 5.9.7)"
            ),
            ReasonCode::BothFormsRequired => write!(
                f,
                "BothFormsRequired: the multi-line String needs both forms, a segment trimming to '))' and a segment trimming to ')' (spec § 5.9.7)"
            ),
            ReasonCode::TrailingWhitespaceCollision => write!(
                f,
                "TrailingWhitespaceCollision: a segment trims to '))' and some content line has trailing whitespace (spec § 5.9.7)"
            ),
            ReasonCode::LeadingWhitespaceCollision => write!(
                f,
                "LeadingWhitespaceCollision: a segment trims to '))' and every non-blank segment shares leading whitespace at the same position (spec § 5.9.7)"
            ),
        }
    }
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
            ErrorKind::LossyScalar {
                line,
                body,
                canonical,
                ..
            } => write!(
                f,
                "Line {}: LossyScalar: '{}' would be inferred as a number and silently canonicalised to '{}'; append '::' to keep it a String or write the canonical form",
                line, body, canonical
            ),
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
            ErrorKind::UnterminatedQuotedKey { line, .. } => write!(
                f,
                "Line {}: UnterminatedQuotedKey: quoted key segment not closed on the same line",
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
            Error::Unrepresentable(code) => write!(f, "{}", code),
            Error::UnrepresentableAt { code, path } => {
                write!(f, "{}", code)?;
                if !path.is_empty() {
                    write!(f, " at {:?}", path)?;
                }
                Ok(())
            }
            Error::InvalidUtf8 { valid_up_to } => write!(
                f,
                "InvalidUtf8: input is not valid UTF-8; first invalid byte sequence at byte offset {}",
                valid_up_to
            ),
        }
    }
}
