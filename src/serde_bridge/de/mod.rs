//! `serde::Deserializer` consuming a [`crate::Value`] into any
//! `T: Deserialize`.

mod describe;
mod from_value;
mod key_deserializer;
mod value_deserializer;

// serde's `*Access` adapters, grouped into `access/` to keep this
// directory within its file budget. `#[path]` keeps them direct children
// of this module, so their `super::` references are unchanged.
#[path = "access/enum_access.rs"]
mod enum_access;
#[path = "access/map_access.rs"]
mod map_access;
#[path = "access/seq_access.rs"]
mod seq_access;
#[path = "access/variant_access.rs"]
mod variant_access;

pub(crate) use key_deserializer::KeyDeserializer;

pub use from_value::from_value;
