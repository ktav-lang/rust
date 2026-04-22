//! `VariantAccess` for one enum variant's payload — unit / newtype / tuple /
//! struct. Handed out by [`super::enum_access::EnumDe`].

use serde::de::{self, DeserializeSeed, Deserializer, VariantAccess, Visitor};

use crate::error::{Error, Result};
use crate::value::Value;

use super::value_deserializer::ValueDeserializer;

pub(super) struct VariantDe {
    pub(super) payload: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDe {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.payload {
            None => Ok(()),
            Some(_) => Err(<Error as de::Error>::custom("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let payload = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected newtype variant"))?;
        seed.deserialize(ValueDeserializer::new(payload))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let payload = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected tuple variant"))?;
        ValueDeserializer::new(payload).deserialize_seq(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let payload = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected struct variant"))?;
        ValueDeserializer::new(payload).deserialize_map(visitor)
    }
}
