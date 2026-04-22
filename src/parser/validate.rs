//! Rules for valid keys and dotted paths.

#[inline]
pub(super) fn is_valid_key(k: &str) -> bool {
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            b.is_ascii_whitespace() || matches!(b, b'[' | b']' | b'{' | b'}' | b':' | b'#')
        })
}
