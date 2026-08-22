//! Rules for valid keys and dotted paths (spec § 4).
//!
//! Under spec 0.6.0:
//! - Each key segment is trimmed of leading/trailing ASCII whitespace,
//!   then validated **before** escape-decoding — this function looks
//!   at the raw, still-escaped segment text, not the decoded result.
//!   That ordering matters: § 3.7 defines ten key escapes (`\\`, `\,`,
//!   `\}`, `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:`), so a decoded
//!   segment legitimately CAN contain a `,` / `{` / `}` / `[` / `]` /
//!   LF / CR byte when it arrived via its escape. Checking the raw
//!   text instead lets those through while still rejecting the same
//!   byte when it appears bare (unescaped) in the source — which is
//!   what actually needs to be forbidden, e.g. so a raw `,` inside an
//!   inline compound can't be mistaken for anything but the compound's
//!   own pair/item separator.
//! - Internal whitespace (space / tab) is allowed inside segments.
//! - `#` is allowed (single `#` has no special meaning in 0.6.0).
//! - Forbidden RAW (unescaped) bytes: `,`, `{`, `}`, `[`, `]`, line
//!   terminators (`LF`, `CR`), and `(` / `)` — the last two have no
//!   § 3.7 escape at all, so they are forbidden even when the caller
//!   tries to escape them (`decode_key_segment` rejects `\(` / `\)`
//!   as an unrecognised escape before this distinction would matter).
//!   A byte immediately following an unescaped `\` is always skipped
//!   here — validating that it is one of the ten recognised escapes
//!   is `decode_key_segment`'s job, not this function's.
//! - Empty (or empty-after-trim, by the caller) → `EmptyKey`.

#[inline]
fn is_forbidden_raw_key_byte(b: u8) -> bool {
    matches!(
        b,
        b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')' | b'\n' | b'\r'
    )
}

#[inline]
pub(crate) fn is_valid_key(raw: &str) -> bool {
    if raw.is_empty() {
        return false;
    }
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            // Skip the escape marker and whatever follows it (even if
            // that byte would itself be forbidden raw) — decoding
            // separately validates it is one of the ten recognised
            // escapes and errors on anything else.
            i += 2;
            continue;
        }
        if is_forbidden_raw_key_byte(bytes[i]) {
            return false;
        }
        i += 1;
    }
    true
}
