//! Zero-copy event-stream parser.
//!
//! Tokenize a Ktav document into a linear sequence of [`ParseEvent`]s
//! delivered to a user callback. Object keys and unmodified scalars
//! are borrowed straight from the input string; the events you receive
//! carry `&'a str` slices into the original buffer (only
//! escape-decoded strings and canonical numeric forms are allocated,
//! in a temporary arena).
//!
//! Transient state differs by compound shape. The line-oriented state
//! machine writes multi-line compounds straight into the flat event
//! list; its transient state is one scope frame per open bracket plus
//! the parse-wide dotted-key path table. Inline compounds (`a: {x:
//! 1}`) are staged before publication: the scanner appends their
//! events to a flat per-compound `Vec<Event>` scratch (array items —
//! nested arrays included — land there in source order, each exactly
//! once), keeps one
//! insertion-ordered key table (`IndexMap`) per object scope for
//! duplicate/conflict detection and dotted-key merge (§ 6.3), and
//! stages one flat event block per array that is directly an object
//! member's value (the only array staging there is: dotted re-entry,
//! § 5.3.2, can add merge-pairs to an already-seen key, so member
//! values are emitted from the object's final walk, never at arrival
//! time). Only after the whole compound validates are its
//! events appended — through the parser's reusable per-parse staging
//! buffer when the compound is a key's value. No owned `Value` is
//! built, and events are never published from a half-scanned
//! compound.
//!
//! Internally this is the same path that powers [`crate::from_str`]: the
//! flat-event stream is the hot deserialization route. Exposing it
//! publicly lets callers build their own consumers (custom DOMs,
//! streaming validators, partial extractors) without paying for serde.
//!
//! Dotted keys (`a.b.c: 10`) are resolved at tokenize time into
//! synthetic `Key` + `BeginObject` / … / `EndObject` triples, so a
//! callback never has to know they existed — it sees the same shape as
//! a fully-spelled nested object.
//!
//! ## Example
//!
//! See [`parse_events`] for a runnable example.

pub(crate) mod event_parser;
mod inline_emit;
mod merge;

pub(crate) use event_parser::parse_events as parse_events_raw;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::error::Result;

/// Parse and, when the parser reports re-opened dotted-key prefixes
/// (spec 0.7 § 5.3.2), fold each re-opened block into the buffer of its
/// first appearance so the one-pass serde `MapAccess` sees a single
/// object per key path. Reopen-free documents take the zero-copy fast
/// path: the raw stream is returned untouched (this is the `from_str`
/// hot route). The trigger is document-global: any reopen count above
/// zero — even one in a branch unrelated to the rest of the document —
/// routes the whole stream through the full rebuild in the `merge`
/// module; the fast path is all-or-nothing.
pub(crate) fn parse_events_merged<'a>(text: &'a str, bump: &'a Bump) -> Result<EventStream<'a>> {
    let (stream, reopens) = parse_events_raw(text, bump)?;
    if reopens == 0 {
        Ok(stream)
    } else {
        Ok(merge::merge_reopened(&stream, bump))
    }
}

/// Flat event representation of a parsed Ktav document.
///
/// Replaces the recursive `ThinValue` tree (since removed) on the typed
/// deserialization hot path. Each compound (`{...}`, `[...]`) is bracketed
/// by a `BeginObject`/`EndObject` or `BeginArray`/`EndArray` pair instead
/// of being its own boxed `BumpVec`.
///
/// Why: a flat `BumpVec<Event<'a>>` lives in one contiguous slab — the
/// deserializer walks it with a single cursor, no per-compound bumpalo
/// allocation, no per-node enum-discriminant load behind a `Box`-style
/// indirection. Cache-friendly linear iteration vs. tree-pointer chasing.
///
/// Dotted keys are *resolved at tokenize time* into synthetic
/// `Key`+`BeginObject`/.../`EndObject` triples, so the deserializer never
/// has to know they existed.
#[derive(Debug, Clone, Copy, PartialEq)]
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
/// Two real implementations: `BumpVec<Event<'a>>` for the whole-
/// document stream (shared by `from_str` and the public `parse_events`),
/// and `Vec<Event<'a>>` for the parser's per-parse reusable inline
/// staging buffer (`EventParser::staging`).
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

/// A single token emitted by [`parse_events`].
///
/// Each variant either is unit (compound brackets, `Null`, `Bool`) or
/// carries a `&'a str` slice borrowed from the input buffer. Compound
/// values are bracketed by `BeginObject` / `EndObject` (or
/// `BeginArray` / `EndArray`) pairs in the stream; an [`ParseEvent::Key`]
/// is always immediately followed by its value event (which may itself
/// open a nested compound).
///
/// The event stream mirrors the document's actual root shape (spec
/// § 5.0.1): an implicit Object/Array root is bracketed by its own
/// `BeginObject`/`EndObject` (or array) pair; a whole-document inline
/// compound (§ 5.0.1 rules 2/3) emits exactly its own bracket pair with
/// no extra wrapping; a lone-`{`/`[`-opened multi-line root (rules 4/5)
/// emits its real `Begin` when opened and its real `End` at the matching
/// close. Empty / comments-only documents default to an empty implicit
/// Object root (`BeginObject` / `EndObject`).
///
/// # Dotted-key re-entry (spec 0.7 § 5.3.2)
///
/// A dotted-key Object re-opened after an intervening sibling pair (or
/// explicitly created earlier as `a: { … }`) emits a separate
/// `Key` + `BeginObject` … `EndObject` block at its own document
/// position — the stream never re-opens an already-closed compound.
/// Consumers that build values from the stream MUST merge such blocks
/// under the same key path (the crate's own [`crate::from_str`] does).
///
/// # Numeric scalars (spec 0.5.0)
///
/// `Integer` and `Float` carry the *canonical* textual form of the
/// number literal. Under spec 0.5.0, numeric types are inferred from
/// the scalar body's lexical form (§ 3.6, § 5.2 rules 13-14). The
/// old `:i` / `:f` typed markers are removed. `Integer` holds the
/// canonical base-10 decimal form (via `itoa`), `Float` holds the
/// shortest decimal form (via `ryu`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseEvent<'a> {
    /// Plain `null` keyword.
    Null,
    /// Plain `true` / `false` keyword.
    Bool(bool),
    /// Integer literal inferred from the scalar body (§ 5.2 rule 13).
    /// Carries the canonical base-10 decimal form.
    Integer(&'a str),
    /// Float literal inferred from the scalar body (§ 5.2 rule 14).
    /// Carries the canonical shortest-decimal form.
    Float(&'a str),
    /// String scalar — single-line scalar, multi-line `( … )` block,
    /// or a value forced to String by `::`.
    Str(&'a str),
    /// Object key. The next event is the corresponding value.
    Key(&'a str),
    /// Opens an object compound (the implicit document object, or an
    /// inline `key: { …` opener).
    BeginObject,
    /// Closes the most recently opened object compound.
    EndObject,
    /// Opens an array compound.
    BeginArray,
    /// Closes the most recently opened array compound.
    EndArray,
}

impl<'a> ParseEvent<'a> {
    #[inline]
    fn from_internal(e: Event<'a>) -> ParseEvent<'a> {
        match e {
            Event::Null => ParseEvent::Null,
            Event::Bool(b) => ParseEvent::Bool(b),
            Event::Integer(s) => ParseEvent::Integer(s),
            Event::Float(s) => ParseEvent::Float(s),
            Event::Str(s) => ParseEvent::Str(s),
            Event::Key(s) => ParseEvent::Key(s),
            Event::BeginObject => ParseEvent::BeginObject,
            Event::EndObject => ParseEvent::EndObject,
            Event::BeginArray => ParseEvent::BeginArray,
            Event::EndArray => ParseEvent::EndArray,
        }
    }
}

/// Tokenize `input` and invoke `callback` with each [`ParseEvent`] in
/// document order.
///
/// Events borrow `&str` slices from `input` where possible (object
/// keys, plain scalars). Multi-line scalar bodies and canonicalized
/// scalars (escape decoding, itoa/ryu numeric reformatting) are
/// allocated in a temporary bump arena owned by this call; their
/// slices live as long as the call itself, which is sufficient
/// because the callback only sees them by reference through
/// `ParseEvent<'_>`.
///
/// The event stream mirrors the document's actual root shape (spec
/// § 5.0.1): implicit roots are bracketed by their Begin/End pair,
/// whole-document inline compounds (§ 5.0.1 rules 2/3) emit exactly
/// their own bracket pair, lone-`{`/`[`-opened roots (rules 4/5) emit
/// their real Begin/End, and empty / comments-only documents default to
/// an empty implicit Object root.
///
/// # Errors
///
/// Returns the same [`crate::Error::Structured`] kinds as
/// [`crate::parse`] / [`crate::from_str`] — invalid keys, duplicate
/// keys, dotted-key conflicts, unbalanced brackets, etc. The callback
/// is not invoked for events past the failure point.
///
/// # Examples
///
/// ```text
/// use ktav::thin::{parse_events, ParseEvent};
/// use std::collections::HashMap;
///
/// let src = "port: 8080\nhost: example.com\n";
///
/// let mut depth = 0_usize;
/// let mut flat: HashMap<String, String> = HashMap::new();
/// let mut last_key: Option<String> = None;
///
/// parse_events(src, |ev| match ev {
///     ParseEvent::BeginObject => depth += 1,
///     ParseEvent::EndObject => depth -= 1,
///     ParseEvent::Key(k) if depth == 1 => last_key = Some(k.to_string()),
///     ParseEvent::Str(s) | ParseEvent::Integer(s) | ParseEvent::Float(s) => {
///         if let Some(k) = last_key.take() {
///             flat.insert(k, s.to_string());
///         }
///     }
///     _ => {}
/// })
/// .unwrap();
///
/// assert_eq!(flat.get("port").map(String::as_str), Some("8080"));
/// assert_eq!(flat.get("host").map(String::as_str), Some("example.com"));
/// ```
/// (See
/// `tests/suite/api/thin/surface/thin_public.rs::flat_pairs_emit_expected_sequence`
/// for the executed test.)
pub fn parse_events<F>(input: &str, mut callback: F) -> Result<()>
where
    F: FnMut(ParseEvent<'_>),
{
    let arena_bytes = (input.len() / 4).saturating_mul(24) + 4096;
    let bump = Bump::with_capacity(arena_bytes);
    let (events, _) = parse_events_raw(input, &bump)?;
    for e in events.iter() {
        callback(ParseEvent::from_internal(*e));
    }
    Ok(())
}
