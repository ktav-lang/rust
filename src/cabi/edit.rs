//! Source-edit поверх существующего wire-протокола значений.

use super::envelope::{envelope, message_envelope, not_utf8_envelope};
use super::wire::wire_to_any_value;
use crate::edit::EditError;
use crate::{Error, ErrorEnvelope};

/// Заменить значение по UTF-8 textual path; результат — исходный текст, не JSON.
/// Source errors имеют координаты0.9; ошибки пути не получают source span.
pub fn edit_value(src: &[u8], path: &[u8], replacement: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let path = path_str(path)?;
    let replacement = wire_to_any_value(replacement)?;
    crate::edit::edit_value(source, path, &replacement)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// Заменить значение по JSON-массиву буквальных decoded segments.
/// Пустой массив — EditNotFound; нестроковые segments — transport Message.
pub fn edit_value_segments(src: &[u8], path: &[u8], replacement: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let segments: Vec<String> = serde_json::from_slice(path)
        .map_err(|err| message_envelope(format!("path segments JSON: {err}"), ""))?;
    let replacement = wire_to_any_value(replacement)?;
    crate::edit::edit_value_segments(source, &segments, &replacement)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// Textual path: UTF-8 транспорт; ошибка — Message без source-позиций.
fn path_str(path: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(path)
        .map_err(|err| message_envelope(format!("path is not valid UTF-8: {err}"), ""))
}

/// Новый ключ: UTF-8 литерал (один decoded сегмент), не JSON и не path.
fn new_key_str(new_key: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(new_key)
        .map_err(|err| message_envelope(format!("new key is not valid UTF-8: {err}"), ""))
}

fn source_text(src: &[u8]) -> Result<&str, String> {
    let source = std::str::from_utf8(src).map_err(|err| not_utf8_envelope(src, err))?;
    // Сохраняем structured parser error до преобразования в EditError::Unsupported.
    crate::parse(source).map_err(|err| envelope(&err, source))?;
    Ok(source)
}

fn edit_envelope(error: EditError) -> String {
    let message = error.to_string();
    let mut env = ErrorEnvelope::from_error(&Error::Message(message.clone()), "");
    env.message = message;
    match error {
        EditError::InvalidPath { offset } => {
            env.error = "EditInvalidPath".into();
            env.body = Some(offset.to_string());
        }
        EditError::NotFound => {
            env.error = "EditNotFound".into();
            env.body = None;
        }
        EditError::AlreadyExists => {
            env.error = "EditAlreadyExists".into();
            env.body = None;
        }
        EditError::Unsupported { reason } => {
            env.error = "EditUnsupported".into();
            env.body = Some(reason);
        }
    }
    env.to_json()
}

/// Вставить новый ключ Object по UTF-8 textual path; replacement —
/// tagged JSON wire (скаляры допустимы). Source проверяется первым.
pub fn insert_value(src: &[u8], path: &[u8], replacement: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let path = path_str(path)?;
    let replacement = wire_to_any_value(replacement)?;
    crate::edit::insert_value(source, path, &replacement)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// [`insert_value`] с JSON-массивом буквальных decoded segments.
pub fn insert_value_segments(
    src: &[u8],
    path: &[u8],
    replacement: &[u8],
) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let segments: Vec<String> = serde_json::from_slice(path)
        .map_err(|err| message_envelope(format!("path segments JSON: {err}"), ""))?;
    let replacement = wire_to_any_value(replacement)?;
    crate::edit::insert_value_segments(source, &segments, &replacement)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// Удалить существующее значение по UTF-8 textual path (два входных
/// буфера, без replacement).
pub fn remove_value(src: &[u8], path: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let path = path_str(path)?;
    crate::edit::remove_value(source, path)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// [`remove_value`] с JSON-массивом буквальных decoded segments.
pub fn remove_value_segments(src: &[u8], path: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let segments: Vec<String> = serde_json::from_slice(path)
        .map_err(|err| message_envelope(format!("path segments JSON: {err}"), ""))?;
    crate::edit::remove_value_segments(source, &segments)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// Переименовать ключ по textual path; `new_key` — UTF-8 литеральный
/// decoded сегмент (не JSON-строка и не path-синтаксис).
pub fn rename_key(src: &[u8], path: &[u8], new_key: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let path = path_str(path)?;
    let new_key = new_key_str(new_key)?;
    crate::edit::rename_key(source, path, new_key)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}

/// [`rename_key`] с JSON-массивом буквальных decoded segments.
pub fn rename_key_segments(src: &[u8], path: &[u8], new_key: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let segments: Vec<String> = serde_json::from_slice(path)
        .map_err(|err| message_envelope(format!("path segments JSON: {err}"), ""))?;
    let new_key = new_key_str(new_key)?;
    crate::edit::rename_key_segments(source, &segments, new_key)
        .map(String::into_bytes)
        .map_err(edit_envelope)
}
