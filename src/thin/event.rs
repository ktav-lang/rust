//! Flat event representation of a parsed Ktav document.
//!
//! Replaces the recursive [`super::value::ThinValue`] tree on the typed
//! deserialization hot path. Each compound (`{...}`, `[...]`) is bracketed
//! by a `BeginObject`/`EndObject` or `BeginArray`/`EndArray` pair instead
//! of being its own boxed `BumpVec`.
//!
//! Why: a flat `BumpVec<Event<'a>>` lives in one contiguous slab — the
//! deserializer walks it with a single cursor, no per-compound bumpalo
//! allocation, no per-node enum-discriminant load behind a `Box`-style
//! indirection. Cache-friendly linear iteration vs. tree-pointer chasing.
//!
//! Dotted keys are *resolved at tokenize time* into synthetic
//! `Key`+`BeginObject`/.../`EndObject` triples, so the deserializer never
//! has to know they existed.

use bumpalo::collections::Vec as BumpVec;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Event<'a> {
    Null,
    Bool(bool),
    /// Typed-integer marker (`:i`) text, or an untyped scalar that may
    /// happen to look like a number — both reach the deserializer the
    /// same way and get fed to `fast_num::parse_*`.
    Integer(&'a str),
    /// Typed-float marker (`:f`) text.
    Float(&'a str),
    /// Plain string scalar.
    Str(&'a str),
    /// An object key. The next event is its value (which may itself be
    /// a `BeginObject` / `BeginArray` opening a nested compound).
    Key(&'a str),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

pub(crate) type EventStream<'a> = BumpVec<'a, Event<'a>>;
