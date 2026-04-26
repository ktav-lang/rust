//! Zero-copy event-stream deserialization path.
//!
//! `from_str` tokenizes the document into a flat `Vec<Event>` (object
//! keys and single-line scalars are borrowed straight from the input),
//! then `EventDeserializer` walks that vec linearly and drives the
//! serde `Visitor` callbacks. There is no per-compound allocation, no
//! tree-shaped intermediate.
//!
//! Dotted keys are resolved at tokenize time via a per-frame stack of
//! synthetic prefixes (LCP fold). Lines using the same prefix must be
//! grouped contiguously; interleaving (e.g. `a.x` … `b.y` … `a.z`)
//! surfaces as a clear conflict error rather than buffering the whole
//! document to merge.

mod event;
mod event_deserializer;
mod event_parser;
mod fast_num;

pub(crate) use event_deserializer::{EventCursor, EventDeserializer};
pub(crate) use event_parser::parse_events;
