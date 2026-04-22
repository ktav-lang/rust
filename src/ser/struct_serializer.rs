//! Backs `serialize_struct` — named fields with compile-time-known names.

use serde::ser::{Serialize, SerializeStruct};

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

use super::value_serializer::ValueSerializer;

pub(crate) struct StructSerializer {
    pub(super) entries: ObjectMap,
}

impl SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, name: &'static str, v: &T) -> Result<()> {
        self.entries
            .insert(name.into(), v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.entries))
    }
}
