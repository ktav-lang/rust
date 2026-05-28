//! Rules for valid keys and dotted paths (spec § 4).
//!
//! Under spec 0.5.0:
//! - Each key segment is trimmed of leading/trailing ASCII whitespace
//!   before validation.
//! - Internal whitespace (space / tab) is allowed inside segments.
//! - `#` is allowed (single `#` has no special meaning in 0.5.0).
//! - Forbidden bytes: `:`, `,`, `{`, `}`, `[`, `]`, `(`, `)`, line
//!   terminators (`LF`, `CR`).
//! - Empty-after-trim → `EmptyKey`.

#[inline]
pub(super) fn is_valid_key(k: &str) -> bool {
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            matches!(
                b,
                b':' | b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')' | b'\n' | b'\r'
            )
        })
}
