//! Source-edit поверх существующего wire-протокола значений.

use super::envelope::{envelope, message_envelope, not_utf8_envelope};
use super::wire::wire_to_any_value;
use crate::edit::EditError;
use crate::{Error, ErrorEnvelope};

/// Заменить значение по UTF-8 textual path; результат — исходный текст, не JSON.
/// Source errors имеют координаты0.9; ошибки пути не получают source span.
pub fn edit_value(src: &[u8], path: &[u8], replacement: &[u8]) -> Result<Vec<u8>, String> {
    let source = source_text(src)?;
    let path = std::str::from_utf8(path)
        .map_err(|err| message_envelope(format!("path is not valid UTF-8: {err}"), ""))?;
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
