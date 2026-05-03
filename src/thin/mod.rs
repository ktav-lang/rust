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

pub(crate) use event_deserializer::{EventCursor, EventDeserializer};
pub(crate) use event_parser::parse_events as parse_events_raw;

use bumpalo::Bump;

use crate::error::Result;

/// A single token emitted by [`parse_events`].
///
/// Each variant either is unit (compound brackets, `Null`, `Bool`) or
/// carries a `&'a str` slice borrowed from the input buffer. Compound
/// values are bracketed by `BeginObject` / `EndObject` (or
/// `BeginArray` / `EndArray`) pairs in the stream; an [`ParseEvent::Key`]
/// is always immediately followed by its value event (which may itself
/// open a nested compound).
///
/// The whole document is wrapped in an outer `BeginObject` / `EndObject`
/// pair, even if the source is empty — Ktav documents are an implicit
/// top-level object.
///
/// # Numeric scalars
///
/// `Integer` and `Float` carry the *textual* literal (with any leading
/// `+` already stripped). They appear when the source used the typed
/// markers `:i` / `:f`. Plain numeric-looking scalars without the marker
/// arrive as [`ParseEvent::Str`] — Ktav has no implicit numeric type
/// inference at the lexer level; the deserializer is responsible for
/// trying `&str → number` parses where the visitor wants a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseEvent<'a> {
    /// Plain `null` keyword.
    Null,
    /// Plain `true` / `false` keyword.
    Bool(bool),
    /// Typed-integer literal (from `:i`). Borrowed from input (or from
    /// the parse arena, if a leading `+` had to be stripped).
    Integer(&'a str),
    /// Typed-float literal (from `:f`).
    Float(&'a str),
    /// String scalar — single-line scalar, multi-line `( … )` block,
    /// or any untyped numeric-looking value.
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
/// The whole document is bracketed by an outer
/// `BeginObject` / `EndObject` pair (Ktav documents are an implicit
/// top-level object).
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
/// ```
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
///     ParseEvent::Str(s) => {
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
pub fn parse_events<F>(input: &str, mut callback: F) -> Result<()>
where
    F: FnMut(ParseEvent<'_>),
{
    let bump = Bump::new();
    let events = parse_events_raw(input, &bump)?;
    for e in events.iter() {
        callback(ParseEvent::from_internal(*e));
    }
    Ok(())
}
