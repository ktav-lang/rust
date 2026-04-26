//! Hand-rolled integer parsers for the typed-deserialization hot path.
//!
//! `<T as FromStr>` is generic and the monomorphisations carry overflow
//! handling that the compiler often can't fully elide. The parser has
//! already validated the digit sequence (no whitespace, no garbage),
//! so we drop into a tight byte loop with `checked_*` for overflow.
//!
//! Float parsing is *not* re-implemented — `f64::from_str` is hard to
//! beat without a dedicated library and the extra precision pitfalls
//! aren't worth a few percent.

#[inline]
pub(crate) fn parse_u64(s: &str) -> Option<u64> {
    let bytes = s.as_bytes();
    // Tolerate a leading `+` — the parser already strips them, but a
    // user-supplied raw scalar (no `:i` marker) can still carry one.
    let digits = if !bytes.is_empty() && bytes[0] == b'+' {
        &bytes[1..]
    } else {
        bytes
    };
    if digits.is_empty() {
        return None;
    }
    let mut acc: u64 = 0;
    for &b in digits {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(d as u64)?;
    }
    Some(acc)
}

#[inline]
pub(crate) fn parse_i64(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (negative, digits) = match bytes[0] {
        b'-' => (true, &bytes[1..]),
        b'+' => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return None;
    }
    let mut acc: i64 = 0;
    if negative {
        // Build as negative so i64::MIN is representable without
        // overflowing the positive accumulator.
        for &b in digits {
            let d = b.wrapping_sub(b'0');
            if d > 9 {
                return None;
            }
            acc = acc.checked_mul(10)?.checked_sub(d as i64)?;
        }
    } else {
        for &b in digits {
            let d = b.wrapping_sub(b'0');
            if d > 9 {
                return None;
            }
            acc = acc.checked_mul(10)?.checked_add(d as i64)?;
        }
    }
    Some(acc)
}

// Bounded width helpers — let the caller decide the target width and
// return None on overflow rather than wrap.
#[inline]
pub(crate) fn parse_u_bounded(s: &str, max: u64) -> Option<u64> {
    parse_u64(s).filter(|&v| v <= max)
}

#[inline]
pub(crate) fn parse_i_bounded(s: &str, min: i64, max: i64) -> Option<i64> {
    parse_i64(s).filter(|&v| v >= min && v <= max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_basic() {
        assert_eq!(parse_u64("0"), Some(0));
        assert_eq!(parse_u64("42"), Some(42));
        assert_eq!(parse_u64("+42"), Some(42));
        assert_eq!(parse_u64("18446744073709551615"), Some(u64::MAX));
        assert_eq!(parse_u64("18446744073709551616"), None); // overflow
        assert_eq!(parse_u64(""), None);
        assert_eq!(parse_u64("-1"), None);
        assert_eq!(parse_u64("4a"), None);
        assert_eq!(parse_u64(" 1"), None);
    }

    #[test]
    fn signed_basic() {
        assert_eq!(parse_i64("0"), Some(0));
        assert_eq!(parse_i64("42"), Some(42));
        assert_eq!(parse_i64("+42"), Some(42));
        assert_eq!(parse_i64("-42"), Some(-42));
        assert_eq!(parse_i64("9223372036854775807"), Some(i64::MAX));
        assert_eq!(parse_i64("-9223372036854775808"), Some(i64::MIN));
        assert_eq!(parse_i64("9223372036854775808"), None); // overflow
        assert_eq!(parse_i64("-9223372036854775809"), None); // overflow
        assert_eq!(parse_i64(""), None);
        assert_eq!(parse_i64("--1"), None);
        assert_eq!(parse_i64("4a"), None);
    }

    #[test]
    fn bounded() {
        assert_eq!(parse_u_bounded("255", u8::MAX as u64), Some(255));
        assert_eq!(parse_u_bounded("256", u8::MAX as u64), None);
        assert_eq!(parse_i_bounded("127", i8::MIN as i64, i8::MAX as i64), Some(127));
        assert_eq!(parse_i_bounded("-128", i8::MIN as i64, i8::MAX as i64), Some(-128));
        assert_eq!(parse_i_bounded("128", i8::MIN as i64, i8::MAX as i64), None);
        assert_eq!(parse_i_bounded("-129", i8::MIN as i64, i8::MAX as i64), None);
    }
}
