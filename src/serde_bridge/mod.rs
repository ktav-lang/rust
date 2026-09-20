//! The serde integration: the zero-copy thin event stream and the
//! `serde::Deserializer` / `serde::Serializer` implementations that
//! consume and produce it.

mod event_deserializer;

pub(crate) use event_deserializer::{EventCursor, EventDeserializer};

pub mod de;
pub mod ser;
pub mod thin;
