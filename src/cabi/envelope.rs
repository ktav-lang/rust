use crate::{Error, ErrorEnvelope};

pub(super) fn envelope(err: &Error, source: &str) -> String {
    ErrorEnvelope::from_error(err, source).to_json()
}

/// Wrap a non-`ktav::Error` diagnostic (a serde_json failure, the
/// top-level check) so it leaves through the same envelope channel as
/// everything else.
pub(super) fn message_envelope(text: impl Into<String>, source: &str) -> String {
    envelope(&Error::Message(text.into()), source)
}

pub(super) fn not_utf8_envelope(err: std::str::Utf8Error) -> String {
    // The input is not text, so there is no source line to pass and
    // `line_text` honestly comes out null.
    message_envelope(format!("input is not valid UTF-8: {err}"), "")
}
