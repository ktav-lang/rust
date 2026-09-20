// ---------------------------------------------------------------------------
// § 5.9.8 Float canonical form
// ---------------------------------------------------------------------------

use crate::error::{Error, ReasonCode, Result};

/// Convert a stored float scalar (ryu shortest-decimal) to the spec § 5.9.8
/// canonical form:
/// - Use scientific notation when `abs(value) >= 1e7` or
///   `0 < abs(value) < 1e-2`.
/// - Otherwise keep the ryu decimal form unchanged.
/// - Scientific: lowercase `e`, no `+` in exponent, strip trailing `.0`
///   in mantissa (so `1.0e9` → `1e9`).
///
/// Returns [`std::borrow::Cow::Borrowed`] whenever the stored form is already canonical
/// text (zero / decimal region / passthrough); only the scientific branch
/// allocates. Thresholds, sign of zero, the shortest-roundtrip guarantee,
/// and the LossyScalar payload are unchanged.
pub(crate) fn canonical_float(s: &str) -> std::borrow::Cow<'_, str> {
    match canonical_float_inner(s) {
        Ok(text) => text,
        // Non-finite. Callers on this entry point (the parsers' § 8.1
        // `LossyScalar` rendering) only ever hand over ryu output, which
        // is always finite, so this preserves the historical
        // pass-through instead of introducing a failure they cannot
        // report.
        Err(NonFinite) => std::borrow::Cow::Borrowed(s),
    }
}

/// [`canonical_float`] for the § 5.9.0 writers: a non-finite payload is
/// a rejection ([`ReasonCode::NonFiniteFloat`]) rather than a
/// pass-through, and the single `parse::<f64>()` below answers both
/// questions — "is it finite" and "which form does § 5.9.8 want" — so
/// the writers no longer parse every Float twice (once in a
/// representability pre-pass, once here).
pub(crate) fn canonical_float_checked(s: &str) -> Result<std::borrow::Cow<'_, str>> {
    canonical_float_inner(s).map_err(|NonFinite| Error::Unrepresentable(ReasonCode::NonFiniteFloat))
}

/// Marker for the one failure `canonical_float_inner` can report.
struct NonFinite;

fn canonical_float_inner(s: &str) -> std::result::Result<std::borrow::Cow<'_, str>, NonFinite> {
    // Parse the stored ryu string back to f64.
    let val: f64 = match s.parse() {
        Ok(v) => v,
        // A payload that is not an f64 lexical form at all is outside the
        // Float domain but is not one of the seven § 5.9.0 reason codes;
        // emission keeps its historical pass-through for it.
        Err(_) => return Ok(std::borrow::Cow::Borrowed(s)),
    };

    if val.is_nan() || val.is_infinite() {
        // § 5.9.0 NonFiniteFloat. Checked here rather than downstream
        // because the scientific branch below cannot render it: `{:e}`
        // gives "NaN"/"inf", which carries no `e` for
        // `normalise_scientific` to split on.
        return Err(NonFinite);
    }

    if val == 0.0 {
        // Positive/negative zero in ryu is "0.0" or "-0.0"; keep as-is.
        return Ok(std::borrow::Cow::Borrowed(s));
    }

    let abs = val.abs();

    if !(1e-2..1e7).contains(&abs) {
        // Build scientific form.
        // Use Rust's {:e} formatter then normalise.
        let raw = format!("{:e}", val); // e.g. "1e9", "1.5e9", "-2.5e-10"
        Ok(std::borrow::Cow::Owned(normalise_scientific(&raw)))
    } else {
        // Decimal region: ryu's output is already correct.
        Ok(std::borrow::Cow::Borrowed(s))
    }
}

/// Normalise Rust's `{:e}` scientific output to the spec form:
/// - lowercase `e` (already lowercase from `{:e}`)
/// - no `+` sign in the exponent
/// - strip trailing `.0` in the mantissa  (`1.0e9` → `1e9`)
/// - strip trailing zeros after decimal point in mantissa (`1.50e9` → `1.5e9`)
fn normalise_scientific(raw: &str) -> String {
    // Rust {:e} format: "<mantissa>e<exp>" where exp may be negative.
    // Example: "1e9", "1.5e9", "-2.5e-10", "1.5e-3".
    //
    // No `e` means `raw` is not scientific output at all; splitting on a
    // position one past the end would panic, so return it untouched. The
    // only inputs that reach `{:e}` without an `e` are the non-finite
    // ones, which `canonical_float_inner` now rejects before this point —
    // this is the belt to that braces.
    let Some(e_pos) = raw.find('e') else {
        return raw.to_string();
    };
    let mantissa = &raw[..e_pos];
    let exp_part = &raw[e_pos + 1..]; // e.g. "9", "-10", "3"

    // Strip trailing zeros and unnecessary decimal point from mantissa.
    let mantissa = if mantissa.contains('.') {
        let trimmed = mantissa.trim_end_matches('0');
        trimmed.trim_end_matches('.')
    } else {
        mantissa
    };

    // Remove leading '+' from exponent (Rust never emits one, but be safe).
    let exp_str = exp_part.trim_start_matches('+');

    // Assembled by hand rather than with `format!`: both parts are
    // already slices of `raw`, so the exact length is known and this is
    // one allocation instead of the macro's.
    let mut out = String::with_capacity(mantissa.len() + 1 + exp_str.len());
    out.push_str(mantissa);
    out.push('e');
    out.push_str(exp_str);
    out
}

// ---------------------------------------------------------------------------
// § 5.9.5 / 5.9.6 / 5.9.7 — Would the parser re-classify this body?
// ---------------------------------------------------------------------------

/// Returns `true` if `body` would be classified by § 5.2 as something
/// other than a String (number, keyword, compound opener, or multi-line
/// opener). In that case the canonical writer must use the `::` raw
/// marker so the parser reads it back as a String.
///
/// Delegates to the shared `render::helpers` implementation, which
/// also covers a body starting with `(` (§ 5.2 would open a
/// multi-line block or reject it as an inline paren compound).
pub(super) fn needs_raw_marker(body: &str) -> bool {
    crate::render::helpers::needs_raw_marker(body)
}

// Number grammar matching now delegated to crate::parser::classify
// (matches_integer_grammar / matches_float_grammar).
