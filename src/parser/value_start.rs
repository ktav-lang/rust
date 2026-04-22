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
    EmptyObject,
    EmptyArray,
    OpenObject,
    OpenArray,
    /// Opens a multi-line string; common leading whitespace is stripped
    /// from the collected lines (`(` ... `)`).
    OpenMultilineStripped,
    /// Opens a multi-line string preserved verbatim (`((` ... `))`).
    OpenMultilineVerbatim,
}
