//! Rules for valid keys and dotted paths (spec § 4).
//!
//! Under spec 0.7:
//! - Each key segment is trimmed of leading/trailing ASCII whitespace
//!   (the OUTER trim only), then validated **before** escape-decoding —
//!   these functions look at the raw, still-escaped segment text, not
//!   the decoded result. That ordering matters: § 3.7 defines thirteen
//!   key-context escapes (`\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`,
//!   `\r`, `\.`, `\:`, `\"`, `\'`, `` \` ``) plus `\uXXXX`, so a
//!   decoded segment legitimately CAN contain a `,` / `{` / `}` / `[` /
//!   `]` / LF / CR / quote byte when it arrived via its escape.
//!   Checking the raw text instead lets those through while still
//!   rejecting the same byte when it appears bare (unescaped) in the
//!   source — which is what actually needs to be forbidden, e.g. so a
//!   raw `,` inside an inline compound can't be mistaken for anything
//!   but the compound's own pair/item separator.
//! - A segment is either a `<bare-segment>` or a `<quoted-segment>`
//!   (§ 5.3.1 / § 5.3.3), identified POSITIONALLY: a segment whose
//!   first byte is `"`, `'`, or `` ` `` is quoted; quotes elsewhere
//!   are ordinary key chars. The two forms are validated against
//!   different character classes (see [`check_key`]).
//! - Internal whitespace (space / tab) is allowed inside segments.
//! - `#` is allowed (single `#` has no special meaning in 0.7).
//! - Forbidden RAW (unescaped) bytes in a bare segment: `,`, `{`, `}`,
//!   `[`, `]`, `(`, `)`, raw control bytes, and DEL (spec 0.7 § 4
//!   `<key-char>`). The control-byte rule subsumes the LF/CR arms.
//!   `:` and `.` stay unlisted: callers slice keys at the first
//!   unescaped `:` and split at unescaped `.` before validation, so
//!   raw ones cannot reach here — same reason as in 0.6.0. `(` / `)`
//!   have no § 3.7 escape at all, so they are forbidden even when the
//!   caller tries to escape them (`decode_key_segment` rejects
//!   `\(` / `\)` as an unrecognised escape before this distinction
//!   would matter). After an unescaped `\`, the scanner skips the
//!   escape's FULL byte length (2 for the thirteen named forms, 6 for
//!   `\uXXXX`, 12 for a `\uD800`..`\uDBFF` + `\uDC00`..`\uDFFF`
//!   surrogate pair — see [`escaped_len`]); for an unrecognized form
//!   it falls back to skipping 2 bytes. Validating that an escape is
//!   actually one of the recognised sequences is `decode_key_segment`'s
//!   job, not this module's.
//! - Empty (or empty-after-trim, by the caller) → `EmptyKey`.

/// Raw control byte / DEL forbidden by spec 0.7 § 4 `<key-char>` /
/// `<dq-char>` / `<sq-char>` / `<bt-char>`: any byte < 0x20 except tab
/// (0x09), VT (0x0B), and FF (0x0C), plus DEL (0x7F). (This subsumes
/// the 0.6.0-era explicit LF/CR arms.)
#[inline]
fn is_forbidden_raw_control_byte(b: u8) -> bool {
    (b < 0x20 && b != b'\t' && b != 0x0B && b != 0x0C) || b == 0x7F
}

/// Byte length of the escape sequence starting at `bytes[i]`
/// (precondition: `bytes[i] == b'\\'`): 2 for a named form, 6 for
/// `\uXXXX`, 12 for a surrogate pair, 1 for a dangling backslash, and
/// 2 as the fallback for any unrecognized/malformed form (decoding
/// reports those; skipping 2 keeps the old scan behavior). Sequence
/// VALIDITY is `decode_key_segment`'s job — this only measures length.
#[inline]
fn escaped_len(bytes: &[u8], i: usize) -> usize {
    if i + 1 >= bytes.len() {
        // Dangling backslash; decoding reports it.
        return 1;
    }
    if bytes[i + 1] != b'u' || i + 6 > bytes.len() {
        return 2;
    }
    let hex = &bytes[i + 2..i + 6];
    if !hex.iter().all(u8::is_ascii_hexdigit) {
        return 2;
    }
    // All four bytes are verified ASCII hex, so `from_utf8` cannot fail.
    let value = u32::from_str_radix(std::str::from_utf8(hex).expect("ASCII hex"), 16)
        .expect("4 ASCII hex digits");
    if !(0xD800..=0xDBFF).contains(&value) {
        return 6;
    }
    if i + 12 > bytes.len() || bytes[i + 6] != b'\\' || bytes[i + 7] != b'u' {
        return 6;
    }
    let low_hex = &bytes[i + 8..i + 12];
    if !low_hex.iter().all(u8::is_ascii_hexdigit) {
        return 6;
    }
    let low = u32::from_str_radix(std::str::from_utf8(low_hex).expect("ASCII hex"), 16)
        .expect("4 ASCII hex digits");
    if (0xDC00..=0xDFFF).contains(&low) {
        12
    } else {
        6
    }
}

#[inline]
fn is_forbidden_raw_key_byte(b: u8) -> bool {
    // `:` and `.` are unlisted: callers slice/split on the first
    // unescaped occurrence before validation, so raw ones cannot reach
    // here (same reason as in 0.6.0).
    matches!(b, b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')') || is_forbidden_raw_control_byte(b)
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
            // Skip the escape's full byte length (even if the escaped
            // byte would itself be forbidden raw) — decoding separately
            // validates it is a recognised escape.
            i += escaped_len(bytes, i);
            continue;
        }
        if is_forbidden_raw_key_byte(bytes[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Outcome of validating one raw (already-outer-trimmed) key segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyValidity {
    Valid,
    /// Empty after trimming (bare) or empty quoted content (`""`) —
    /// `EmptyKey` per spec 0.7 § 6.5.
    Empty,
    /// A forbidden raw byte, a quoted segment with content after its
    /// closer (§ 6.4), or a quoted segment with no closer at all.
    Invalid,
}

/// Validate one raw, already-outer-trimmed key segment (spec 0.7
/// § 5.3.1). A segment whose first byte is `"`, `'`, or `` ` `` is a
/// `<quoted-segment>` (§ 5.3.3's positional rule) and is validated
/// against `<dq-char>`/`<sq-char>`/`<bt-char>`; anything else is a
/// `<bare-segment>` validated against `<key-char>`.
pub(crate) fn check_key(raw: &str) -> KeyValidity {
    if raw.is_empty() {
        return KeyValidity::Empty;
    }
    match raw.as_bytes()[0] {
        b'"' | b'\'' | b'`' => check_quoted_key(raw),
        _ => {
            if is_valid_key(raw) {
                KeyValidity::Valid
            } else {
                KeyValidity::Invalid
            }
        }
    }
}

fn check_quoted_key(raw: &str) -> KeyValidity {
    let bytes = raw.as_bytes();
    // The segment's own first unescaped delimiter closes it; anything
    // other than whitespace-trimming leftovers after the closer (i.e.
    // any byte at all — callers pre-trim) is InvalidKey per § 6.4.
    let closer = match super::inline::quoted_span_end(bytes, 0) {
        Some(c) => c,
        None => return KeyValidity::Invalid, // unterminated; normally diagnosed earlier
    };
    if closer != bytes.len() - 1 {
        return KeyValidity::Invalid;
    }
    // Interior: raw control bytes / DEL are InvalidKey (§ 6.4); every
    // structural byte other than the segment's own delimiter — `.` `:`
    // `,` `{` `}` `[` `]` `(` `)` and the two other quote chars — is
    // ordinary content. `\` skips the escape's full byte length; escape
    // SEQUENCE validity is decode_key_segment's job, as for bare segments.
    let mut i = 1;
    while i < closer {
        if bytes[i] == b'\\' {
            i += escaped_len(bytes, i);
            continue;
        }
        if is_forbidden_raw_control_byte(bytes[i]) {
            return KeyValidity::Invalid;
        }
        i += 1;
    }
    if closer == 1 {
        // `""` / `''` / `` `` `` — empty quoted content is EmptyKey (§ 6.5),
        // NOT valid, even though bare whitespace-only trims to Empty too.
        return KeyValidity::Empty;
    }
    KeyValidity::Valid
}
