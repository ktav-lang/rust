//! Zero-copy event-stream parser.
//!
//! Tokenize a Ktav document into a linear sequence of [`ParseEvent`]s
//! delivered to a user callback — no intermediate tree, no per-compound
//! allocation. Object keys and single-line scalars are borrowed straight
//! from the input string; the events you receive carry `&'a str` slices
//! into the original buffer.
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

mod event;
mod event_deserializer;
mod event_parser;
mod fast_num;
mod merge;

pub(crate) use event_deserializer::{EventCursor, EventDeserializer};
pub(crate) use event_parser::parse_events as parse_events_raw;

use bumpalo::Bump;

use crate::error::Result;

use crate::thin::event::EventStream;

/// Parse and, when the parser reports re-opened dotted-key prefixes
/// (spec 0.7 § 5.3.2), fold each re-opened block into the buffer of its
/// first appearance so the one-pass serde `MapAccess` sees a single
/// object per key path. Reopen-free documents take the zero-copy fast
/// path: the raw stream is returned untouched (this is the `from_str`
/// hot route).
pub(crate) fn parse_events_merged<'a>(text: &'a str, bump: &'a Bump) -> Result<EventStream<'a>> {
    let (stream, reopens) = parse_events_raw(text, bump)?;
    if reopens == 0 {
        Ok(stream)
    } else {
        Ok(merge::merge_reopened(&stream, bump))
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
/// under the same key path (the crate's own [`from_str`] does).
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
    fn from_internal(e: event::Event<'a>) -> ParseEvent<'a> {
        match e {
            event::Event::Null => ParseEvent::Null,
            event::Event::Bool(b) => ParseEvent::Bool(b),
            event::Event::Integer(s) => ParseEvent::Integer(s),
            event::Event::Float(s) => ParseEvent::Float(s),
            event::Event::Str(s) => ParseEvent::Str(s),
            event::Event::Key(s) => ParseEvent::Key(s),
            event::Event::BeginObject => ParseEvent::BeginObject,
            event::Event::EndObject => ParseEvent::EndObject,
            event::Event::BeginArray => ParseEvent::BeginArray,
            event::Event::EndArray => ParseEvent::EndArray,
        }
    }
}

/// Tokenize `input` and invoke `callback` with each [`ParseEvent`] in
/// document order.
///
/// Events borrow `&str` slices from `input` where possible (object
/// keys, plain scalars). Multi-line scalars and `+`-stripped typed
/// numbers are allocated in a temporary bump arena owned by this call;
/// their slices live as long as the call itself, which is sufficient
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
/// (See `tests/thin_public.rs::flat_pairs_emit_expected_sequence` for the
/// executed test.)
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
