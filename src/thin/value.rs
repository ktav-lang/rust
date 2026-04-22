//! Zero-copy intermediate representation of a parsed Ktav document.
//! Strings and object keys borrow slices out of the input buffer whenever
//! possible; only multi-line strings whose dedented form cannot be a
//! single input slice get copied — and that copy goes into a bump arena,
//! so the whole parse lives inside one arena and frees in one go.

use bumpalo::collections::Vec as BumpVec;

/// `ThinValue` is parameterized by the lifetime of the arena (which in
/// turn borrows from the source buffer).
///
/// - `Str` / `Integer` / `Float` hold a `&'a str`. When the content is a
///   contiguous substring of the input, the slice points directly back
///   into it; when the parser had to normalize (strip a leading `+`,
///   dedent a multi-line block), the normalized text is copied into the
///   arena via `Bump::alloc_str` and the slice points there.
/// - `Array` / `Object` use `bumpalo::collections::Vec`, so the backing
///   buffer lives in the arena — one arena allocation serves dozens of
///   nested compounds, replacing a heap allocation per compound with a
///   single bump per compound grow-step.
#[derive(Debug, PartialEq)]
pub enum ThinValue<'a> {
    Null,
    Bool(bool),
    Integer(&'a str),
    Float(&'a str),
    Str(&'a str),
    Array(BumpVec<'a, ThinValue<'a>>),
    Object(BumpVec<'a, (&'a str, ThinValue<'a>)>),
}
