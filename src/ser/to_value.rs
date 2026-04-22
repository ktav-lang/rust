//! Convenience wrapper `T → Value`.

use serde::Serialize;

use crate::error::Result;
use crate::value::Value;

use super::value_serializer::ValueSerializer;

/// Convert a `T: Serialize` into a [`Value`]. Normally invoked indirectly
/// via [`crate::to_string`] / [`crate::to_file`].
pub fn to_value<T: ?Sized + Serialize>(value: &T) -> Result<Value> {
    value.serialize(ValueSerializer)
}
