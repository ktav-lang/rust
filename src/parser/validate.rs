//! Rules for valid keys and dotted paths (spec § 4).
//!
//! Under spec 0.6.0:
//! - Each key segment is trimmed of leading/trailing ASCII whitespace
//!   and then escape-decoded before validation. The byte sequence
//!   validated here is the **decoded** form (after `\.` / `\:` etc.).
//! - Internal whitespace (space / tab) is allowed inside segments.
//! - `#` is allowed (single `#` has no special meaning in 0.6.0).
//! - Forbidden bytes in a decoded segment: `,`, `{`, `}`, `[`, `]`,
//!   `(`, `)`, line terminators (`LF`, `CR`). The bytes `:` and `.`
//!   are permitted in a **decoded** segment because the user expressed
//!   them via `\:` / `\.` and they have no structural meaning post-
//!   decode. Internal `\` is also permitted (originated from `\\`).
//! - Empty-after-trim → `EmptyKey`.

#[inline]
pub(super) fn is_valid_key(k: &str) -> bool {
    !k.is_empty()
        && !k.as_bytes().iter().any(|&b| {
            matches!(
                b,
                b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')' | b'\n' | b'\r'
            )
        })
}
