//! Convenience wrapper `Value → T`.

use serde::de::DeserializeOwned;

use crate::error::Result;
use crate::value::Value;

use super::value_deserializer::ValueDeserializer;

/// Convert a parsed [`Value`] into `T`.
pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    T::deserialize(ValueDeserializer::new(value))
}
