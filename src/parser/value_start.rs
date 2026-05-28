//! Classification of what follows a `:` on a pair line, or a bare line
//! inside an array.

use crate::value::Scalar;

pub(super) enum ValueStart {
    /// A scalar string (anything that isn't a keyword, empty compound, or
    /// opening `{` / `[` / `(` / `((`).
    Scalar(Scalar),
    /// The `null` keyword.
    Null,
    /// The `true` / `false` keywords.
    Bool(bool),
    /// An Integer value (§ 5.2 rule 13). Carries the canonical base-10
    /// decimal form (via `itoa`).
    Integer(Scalar),
    /// A Float value (§ 5.2 rule 14). Carries the canonical form (via `ryu`).
    Float(Scalar),
    EmptyObject,
    EmptyArray,
    OpenObject,
    OpenArray,
    /// Opens a multi-line string; common leading whitespace is stripped
    /// from the collected lines (`(` ... `)`).
    OpenMultilineStripped,
    /// Opens a multi-line string preserved verbatim (`((` ... `))`).
    OpenMultilineVerbatim,
    /// A closed inline object `{key: value, ...}` parsed into a Value.
    /// Phase 4 (section 5.8).
    InlineValue(crate::value::Value),
}
