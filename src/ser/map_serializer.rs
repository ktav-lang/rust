//! Backs `serialize_map` — collects pairs. Keys go through the shared
//! `text_serializer::serialize_key_name` policy (R15-F2) so both writer
//! paths accept the same key types and produce identical name text.

use serde::ser::{self, Serialize, SerializeMap};

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Scalar, Value};

use super::value_serializer::ValueSerializer;

pub(crate) struct MapSerializer {
    pub(super) entries: ObjectMap,
    pub(super) next_key: Option<Scalar>,
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.next_key = Some(super::text_serializer::serialize_key_name(key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        let key = self.next_key.take().ok_or_else(|| {
            <Error as ser::Error>::custom("serialize_value called without a preceding key")
        })?;
        self.entries.insert(key, v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.entries))
    }
}
