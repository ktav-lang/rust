//! `SeqAccess` over a `Value::Array` — hands each element to the visitor.

use serde::de::{DeserializeSeed, SeqAccess};

use crate::error::{Error, Result};
use crate::value::Value;

use super::value_deserializer::ValueDeserializer;

pub(super) struct SeqDe {
    pub(super) iter: std::vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for SeqDe {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.iter.next() {
            Some(value) => seed.deserialize(ValueDeserializer::new(value)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}
