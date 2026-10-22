//! Lossless source-level editing: replace the value at a key path,
//! touching only the target's own bytes.
//!
//! Unlike [`parse`](crate::parse) → mutate → serialize round-trips,
//! `edit_value` splices the source text at the exact byte range of the
//! target value. Comments, blank lines, indentation, key spelling,
//! BOM, line-ending style (LF / CR / CRLF), ordering, and EOF are
//! preserved byte-for-byte; only the target value's own bytes change.
//!
//! # Paths
//!
//! The textual form `key1.key2.0.key4` splits on `.`. Inside a
//! segment, `\.` is a literal dot and `\\` a literal backslash; any
//! other escape (or a trailing lone backslash) is an
//! [`EditError::InvalidPath`]. The segment-based [`edit_value_segments`]
//! takes already-decoded literal segments with no extra unescaping: a
//! numeric segment is a string key inside an Object and a zero-based
//! index inside an Array.
//!
//! Only *replacement of an existing value* is supported — no automatic
//! path creation, no removal, no rename. Descending through a scalar,
//! a missing key, or an out-of-bounds index is [`EditError::NotFound`].
//!
//! # Limitations
//!
//! Some targets admit no lossless single-range patch and yield
//! [`EditError::Unsupported`]:
//! - a *noncontiguous* value assembled from dotted-key and reopen
//!   lines (e.g. `a: {}` + a later `a.y: 2`) — replacing the whole `a`
//!   would have to erase neighbouring bytes;
//! - structural replacement of a container whose interior contains
//!   comment lines (they cannot survive a re-render; leaf edits inside
//!   such a container work and keep the comments);
//! - a multiline string with a line equal to `))` (no verbatim form
//!   can hold it).
//!
//! Array-index segments accept ASCII digits only (leading zeros
//! allowed: `01` addresses index 1; signs are not indexes).
//!
//! Every successful patch is re-parsed and semantically verified: the
//! resulting tree must equal the original tree with exactly the target
//! replaced, so a silent corruption is never returned.

use crate::value::Value;

/// Everything that can go wrong with a lossless source edit.
///
/// `InvalidPath` and `NotFound` leave the source text untouched by
/// construction (the text is only borrowed); `Unsupported` means a
/// lossless patch for this (path, replacement) pair does not exist.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// The textual path is malformed: a bad escape (`\x` other than
    /// `\.` and `\\`), a trailing lone backslash, an empty path, or an
    /// empty segment. `offset` is the byte offset within the path
    /// argument where the problem was detected.
    InvalidPath {
        /// Byte offset into the textual path argument.
        offset: usize,
    },
    /// The path does not address an existing value: missing key,
    /// non-numeric or out-of-bounds array index, descent through a
    /// scalar, or an empty path.
    NotFound,
    /// No lossless single-range patch exists for this target or
    /// replacement; the reason describes which limitation fired.
    Unsupported {
        /// Human-readable explanation of the limitation.
        reason: String,
    },
}

/// Replace the value at `path` in the Ktav source `text`, returning the
/// patched document with every byte outside the target value unchanged.
///
/// `path` is the textual dotted form (`a.b\.c.0`); see the [module
/// docs](self) for the escape rules. For pre-decoded segments use
/// [`edit_value_segments`].
///
/// ```text
/// let src = "port: 8080\nhost: db\n";
/// let out = ktav::edit::edit_value(src, "port", &ktav::Value::Integer("9090".into()))?;
/// assert_eq!(out, "port: 9090\nhost: db\n");
/// ```
pub fn edit_value(text: &str, path: &str, replacement: &Value) -> Result<String, EditError> {
    let segments = parse_path(path)?;
    edit_value_segments(text, &segments, replacement)
}

/// [`edit_value`] with pre-decoded literal segments: no additional
/// unescaping is performed. A numeric segment addresses a string key
/// inside an Object and a zero-based index inside an Array.
pub fn edit_value_segments<S: AsRef<str>>(
    text: &str,
    path: &[S],
    replacement: &Value,
) -> Result<String, EditError> {
    if path.is_empty() {
        return Err(EditError::NotFound);
    }
    let segments: Vec<String> = path.iter().map(|s| s.as_ref().to_string()).collect();
    crate::document::parser::edit::splice::apply(text, &segments, replacement)
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::InvalidPath { offset } => write!(
                f,
                "invalid edit path (bad escape or empty segment) at byte {offset}"
            ),
            EditError::NotFound => {
                write!(f, "edit path does not address an existing value")
            }
            EditError::Unsupported { reason } => {
                write!(f, "no lossless source edit: {reason}")
            }
        }
    }
}

impl std::error::Error for EditError {}

/// Разбор текстового пути: '.' — разделитель; `\.` — literal точка;
/// `\\` — literal backslash; прочие escape и хвостовой `\` — ошибка.
fn parse_path(path: &str) -> Result<Vec<String>, EditError> {
    if path.is_empty() {
        return Err(EditError::InvalidPath { offset: 0 });
    }
    let bytes = path.as_bytes();
    let mut segments = Vec::new();
    let mut cur = String::new();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                if cur.is_empty() {
                    return Err(EditError::InvalidPath { offset: i });
                }
                segments.push(std::mem::take(&mut cur));
                i += 1;
            }
            b'\\' => match bytes.get(i + 1) {
                Some(b'.') => {
                    cur.push('.');
                    i += 2;
                }
                Some(b'\\') => {
                    cur.push('\\');
                    i += 2;
                }
                _ => {
                    return Err(EditError::InvalidPath { offset: i });
                }
            },
            _ => {
                let ch_len = utf8_len(bytes[i]);
                cur.push_str(&path[i..i + ch_len]);
                i += ch_len;
            }
        }
    }
    if cur.is_empty() {
        // Пустой последний сегмент (хвост '.').
        return Err(EditError::InvalidPath {
            offset: bytes.len(),
        });
    }
    segments.push(cur);
    Ok(segments)
}

/// Длина UTF-8 последовательности по ведущему байту (1 для ASCII).
fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        // Continuation-байт вне последовательности: UTF-8 вход не может
        // содержать такого; безопасная длина 1 не паникует на границе.
        _ => 1,
    }
}
