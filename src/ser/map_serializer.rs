//! Backs `serialize_map` — collects pairs where keys must serialize to
//! strings.

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
        let key_value = key.serialize(ValueSerializer)?;
        match key_value {
            Value::String(s) | Value::Integer(s) | Value::Float(s) => {
                self.next_key = Some(s);
                Ok(())
            }
            _ => Err(<Error as ser::Error>::custom(
                "map keys must serialize to strings",
            )),
        }
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
