use super::{Error, Span};

/// Координаты исходных байтов; CRLF считается одним терминатором.
pub(crate) fn line_col(bytes: &[u8], pos: usize) -> (u32, u32) {
    let bound = pos.min(bytes.len());
    let mut line = 1u32;
    let mut line_start = 0usize;
    let mut i = 0;
    while i < bound {
        match bytes[i] {
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                if i + 2 > bound {
                    break;
                }
                i += 2;
                line = line.saturating_add(1);
                line_start = i;
            }
            b'\r' | b'\n' => {
                i += 1;
                line = line.saturating_add(1);
                line_start = i;
            }
            _ => i += 1,
        }
    }
    (line, u32::try_from(bound - line_start).unwrap_or(u32::MAX))
}

pub(crate) fn line_of_offset(bytes: &[u8], pos: usize) -> u32 {
    line_col(bytes, pos).0
}

/// UTF-8 проверяется до грамматики; диапазон сохраняет исходные байты.
pub(crate) fn invalid_utf8(source: &[u8], error: std::str::Utf8Error) -> Error {
    let valid_up_to = error.valid_up_to();
    let mut end = valid_up_to + error.error_len().unwrap_or(source.len() - valid_up_to);
    if error.error_len() == Some(1) {
        while source
            .get(end)
            .is_some_and(|byte| (0x80..=0xBF).contains(byte))
        {
            end += 1;
        }
    }
    Error::InvalidUtf8 {
        valid_up_to,
        line: line_of_offset(source, valid_up_to),
        span: Span::new(
            u32::try_from(valid_up_to).unwrap_or(u32::MAX),
            u32::try_from(end).unwrap_or(u32::MAX),
        ),
    }
}
