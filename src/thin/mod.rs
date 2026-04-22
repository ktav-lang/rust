//! Zero-copy deserialization path.
//!
//! A lightweight intermediate `ThinValue<'a>` borrows scalars and object
//! keys out of the input buffer. `from_str` uses this path directly —
//! `Value` (owned) is only built on demand via `crate::parse`.

mod deserializer;
mod parser;
mod value;

pub(crate) use deserializer::ThinDeserializer;
pub(crate) use parser::parse_thin;
