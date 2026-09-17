// ---------------------------------------------------------------------------
// § 5.9.8 Float canonical form
// ---------------------------------------------------------------------------

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
    // Parse the stored ryu string back to f64.
    let val: f64 = match s.parse() {
        Ok(v) => v,
        Err(_) => return std::borrow::Cow::Borrowed(s), // shouldn't happen; pass through
    };

    if val == 0.0 {
        // Positive/negative zero in ryu is "0.0" or "-0.0"; keep as-is.
        return std::borrow::Cow::Borrowed(s);
    }

    let abs = val.abs();

    if !(1e-2..1e7).contains(&abs) {
        // Build scientific form.
        // Use Rust's {:e} formatter then normalise.
        let raw = format!("{:e}", val); // e.g. "1e9", "1.5e9", "-2.5e-10"
        std::borrow::Cow::Owned(normalise_scientific(&raw))
    } else {
        // Decimal region: ryu's output is already correct.
        std::borrow::Cow::Borrowed(s)
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
    let e_pos = raw.find('e').unwrap_or(raw.len());
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

    format!("{}e{}", mantissa, exp_str)
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
