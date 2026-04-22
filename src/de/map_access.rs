//! `MapAccess` over a `Value::Object`.

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess};

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Scalar, Value};

use super::value_deserializer::ValueDeserializer;

pub(super) struct MapDe {
    iter: indexmap::map::IntoIter<Scalar, Value>,
    next_value: Option<Value>,
}

impl MapDe {
    pub(super) fn new(obj: ObjectMap) -> Self {
        Self {
            iter: obj.into_iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDe {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.iter.next() {
            Some((key, value)) => {
                self.next_value = Some(value);
                seed.deserialize(<String as IntoDeserializer<Error>>::into_deserializer(
                    key.into_string(),
                ))
                .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let value = self
            .next_value
            .take()
            .ok_or_else(|| <Error as de::Error>::custom("next_value called without a key"))?;
        seed.deserialize(ValueDeserializer::new(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}
