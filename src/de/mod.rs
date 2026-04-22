//! `serde::Deserializer` consuming a [`crate::Value`] into any
//! `T: Deserialize`.

mod describe;
mod enum_access;
mod from_value;
mod map_access;
mod seq_access;
mod value_deserializer;
mod variant_access;

pub use from_value::from_value;
