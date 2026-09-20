//! Backs the `Enum::Variant(T1, T2, ...)` case — externally tagged as a
//! single-key object whose value is the variant's tuple body.

use serde::ser::{Serialize, SerializeTupleVariant};

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

use super::value_serializer::ValueSerializer;

pub(crate) struct TupleVariantSerializer {
    pub(super) variant: &'static str,
    pub(super) items: Vec<Value>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.items.push(v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = ObjectMap::default();
        map.insert(self.variant.into(), Value::Array(self.items));
        Ok(Value::Object(map))
    }
}
