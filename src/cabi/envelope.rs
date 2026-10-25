use crate::{Error, ErrorEnvelope};

#[cfg(test)]
pub(super) use crate::error::source::line_of_offset;

pub(super) fn envelope(err: &Error, source: &str) -> String {
    ErrorEnvelope::from_error(err, source).to_json()
}

/// Неструктурированная диагностика использует тот же JSON-канал.
pub(super) fn message_envelope(text: impl Into<String>, source: &str) -> String {
    envelope(&Error::Message(text.into()), source)
}

/// Координаты UTF-8 совпадают с прямым Rust API, без ABI-only исправлений.
pub(super) fn not_utf8_envelope(src: &[u8], err: std::str::Utf8Error) -> String {
    let error = crate::error::source::invalid_utf8(src, err);
    let mut env = ErrorEnvelope::from_error(&error, "");
    env.body = Some(format!("input is not valid UTF-8: {err}"));
    env.to_json()
}
