//! Lossless-замена значения по пути без форматирования всего документа.
//!
//! Обычно меняются только байты цели. Dotted-, inline- и блочная формы
//! сохраняются. Inline-предки раскрываются при многострочном тексте,
//! росте строки сверх max(100, прежняя длина в Unicode scalar values)
//! или не менее четырёх escapes с накладными расходами от 25%.
//! Раскрытие затрагивает только необходимую цепочку обёрток; переносимые
//! скаляры перекодируются при необходимости. Если lossless-блок невозможен,
//! сохраняется inline. Комментарии и байты вне диапазона правки, BOM,
//! существующие EOL и наличие/отсутствие финального EOL сохраняются.
//! Новые уровни блока используют четыре пробела. No-op всегда byte-identical.
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
//! Replacement, insertion, removal and key renaming share the same
//! lossless guarantees. [`insert_value`] adds a *new* final Object key
//! (existing key: [`EditError::AlreadyExists`], never an overwrite;
//! insertion into an Array is an explicit `Unsupported`);
//! [`remove_value`] deletes an existing leaf, subtree or Array element
//! (empty ancestors are kept — removing the only dotted leaf keeps its
//! parent as an empty Object); [`rename_key`] renames an existing
//! Object member in place (identical key: byte-identical no-op).
//! Descending through a scalar, a missing key, or an out-of-bounds
//! index is [`EditError::NotFound`].
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

use std::borrow::Cow;

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
    /// The addressed Object key already exists; insertion never
    /// overwrites, and renaming onto an existing sibling is refused.
    AlreadyExists,
    /// No lossless single-range patch exists for this target or
    /// replacement; the reason describes which limitation fired.
    Unsupported {
        /// Human-readable explanation of the limitation.
        reason: String,
    },
}

/// Заменить значение по `path` в исходном `text`. Байты вне цели или
/// необходимых раскрываемых inline-обёрток сохраняются; см. [правила](self).
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
    crate::document::parser::edit::splice::apply(text, &segments, replacement)
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
    // Сегменты уже декодированы; передача заимствованных кусков без
    // промежуточного Vec<String>.
    crate::document::parser::edit::splice::apply(text, path, replacement)
}

/// Вставить новый ключ `path` (полный путь; последний сегмент — новый
/// ключ Object) со значением `replacement`. Родитель обязан
/// существовать и быть Object; существующий ключ —
/// [`EditError::AlreadyExists`] (никогда не overwrite), вставка в
/// Array — явный [`EditError::Unsupported`]. Новая пара добавляется в
/// конец Object; см. [правила](self).
pub fn insert_value(text: &str, path: &str, replacement: &Value) -> Result<String, EditError> {
    let segments = parse_path(path)?;
    insert_value_segments(text, &segments, replacement)
}

/// [`insert_value`] with pre-decoded literal segments: the last
/// segment is the new Object key; an empty segment list is
/// [`EditError::NotFound`].
pub fn insert_value_segments<S: AsRef<str>>(
    text: &str,
    path: &[S],
    replacement: &Value,
) -> Result<String, EditError> {
    crate::document::parser::edit::splice::insert_apply(text, path, replacement)
}

/// Удалить существующее значение по `path`: лист, целое поддерево или
/// элемент массива (хвост массива сдвигается естественно). Пустые
/// предки не удаляются: удаление единственного dotted-листа сохраняет
/// родителя пустым Object; блочная форма остаётся блочной, inline —
/// inline, где это возможно. Комментарии внутри удаляемого контейнера
/// — [`EditError::Unsupported`].
pub fn remove_value(text: &str, path: &str) -> Result<String, EditError> {
    let segments = parse_path(path)?;
    remove_value_segments(text, &segments)
}

/// [`remove_value`] with pre-decoded literal segments; an empty
/// segment list is [`EditError::NotFound`].
pub fn remove_value_segments<S: AsRef<str>>(text: &str, path: &[S]) -> Result<String, EditError> {
    crate::document::parser::edit::splice::remove_apply(text, path)
}

/// Переименовать ключ существующего члена Object по `path` в
/// `new_key` (один буквальный декодированный сегмент; точка и backslash
/// внутри — часть имени, не синтаксис пути). Позиция, порядок и
/// значение сохраняются; идентичный ключ — byte-identical no-op;
/// занятый сосед — [`EditError::AlreadyExists`]; индекс массива и
/// невоспроизводимый ключ — [`EditError::Unsupported`].
pub fn rename_key(text: &str, path: &str, new_key: &str) -> Result<String, EditError> {
    let segments = parse_path(path)?;
    rename_key_segments(text, &segments, new_key)
}

/// [`rename_key`] with pre-decoded literal segments; an empty segment
/// list is [`EditError::NotFound`].
pub fn rename_key_segments<S: AsRef<str>>(
    text: &str,
    path: &[S],
    new_key: &str,
) -> Result<String, EditError> {
    crate::document::parser::edit::splice::rename_apply(text, path, new_key)
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
            EditError::AlreadyExists => {
                write!(f, "edit target already exists; insertion never overwrites")
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
/// Сегменты без escapes возвращаются заимствованными срезами входа.
fn parse_path(path: &str) -> Result<Vec<Cow<'_, str>>, EditError> {
    if path.is_empty() {
        return Err(EditError::InvalidPath { offset: 0 });
    }
    let bytes = path.as_bytes();
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    // Owned-аккумулятор появляется только после первого escape в
    // сегменте; до того сегмент остаётся borrowed срезом.
    let mut cur: Option<String> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                if seg_start == i && cur.is_none() {
                    return Err(EditError::InvalidPath { offset: i });
                }
                segments.push(match cur.take() {
                    Some(s) => Cow::Owned(s),
                    None => Cow::Borrowed(&path[seg_start..i]),
                });
                seg_start = i + 1;
                i += 1;
            }
            b'\\' => match bytes.get(i + 1) {
                Some(b'.') | Some(b'\\') => {
                    let cur = cur.get_or_insert_with(|| String::from(&path[seg_start..i]));
                    cur.push(if bytes[i + 1] == b'.' { '.' } else { '\\' });
                    i += 2;
                }
                _ => {
                    return Err(EditError::InvalidPath { offset: i });
                }
            },
            _ => {
                let ch_len = utf8_len(bytes[i]);
                if let Some(cur) = cur.as_mut() {
                    cur.push_str(&path[i..i + ch_len]);
                }
                i += ch_len;
            }
        }
    }
    if seg_start == bytes.len() && cur.is_none() {
        // Пустой последний сегмент (хвост '.').
        return Err(EditError::InvalidPath {
            offset: bytes.len(),
        });
    }
    segments.push(match cur {
        Some(s) => Cow::Owned(s),
        None => Cow::Borrowed(&path[seg_start..]),
    });
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
