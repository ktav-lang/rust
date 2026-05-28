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
    /// Integer literal inferred from the scalar body's lexical form
    /// (§ 3.6, § 5.2 rule 13). Carries the canonical base-10 decimal
    /// form (via `itoa`). Under 0.5.0, `:i` markers are removed;
    /// integer detection is automatic.
    Integer(&'a str),
    /// Float literal inferred from the scalar body's lexical form
    /// (§ 3.6, § 5.2 rule 14). Carries the canonical shortest-decimal
    /// form (via `ryu`). Under 0.5.0, `:f` markers are removed;
    /// float detection is automatic.
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

/// Abstracts where a parser state machine emits events.
///
/// Two real implementations: `BumpVec<Event<'a>>` for the callback-
/// style public API (full document tokenized into one slab, then
/// iterated), and `Vec<Event<'a>>` for the streaming deserializer
/// (small reusable per-line queue, no whole-document buffer).
///
/// Generifying the parser over this trait keeps a single state-machine
/// implementation serving both modes. Monomorphisation specialises
/// each call site, so dispatch is free.
pub(crate) trait EventSink<'a> {
    fn push(&mut self, event: Event<'a>);
}

impl<'a> EventSink<'a> for BumpVec<'a, Event<'a>> {
    #[inline(always)]
    fn push(&mut self, event: Event<'a>) {
        BumpVec::push(self, event);
    }
}

impl<'a> EventSink<'a> for Vec<Event<'a>> {
    #[inline(always)]
    fn push(&mut self, event: Event<'a>) {
        Vec::push(self, event);
    }
}
