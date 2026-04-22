//! `EnumAccess` — distinguishes unit, newtype, tuple, and struct variants
//! based on whether the `Value` is a string or a single-entry object.

use serde::de::{self, DeserializeSeed, EnumAccess, IntoDeserializer};

use crate::error::{Error, Result};
use crate::value::Value;

use super::describe::describe;
use super::variant_access::VariantDe;

pub(super) struct EnumDe {
    pub(super) value: Value,
}

impl<'de> EnumAccess<'de> for EnumDe {
    type Error = Error;
    type Variant = VariantDe;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        match self.value {
            // Unit variant — the value is just the variant name.
            Value::String(name) => {
                let variant = seed.deserialize(
                    <String as IntoDeserializer<Error>>::into_deserializer(name.into_string()),
                )?;
                Ok((variant, VariantDe { payload: None }))
            }
            // Externally-tagged non-unit variant — single-entry object.
            Value::Object(obj) => {
                let mut iter = obj.into_iter();
                let (name, payload) = iter.next().ok_or_else(|| {
                    <Error as de::Error>::custom("enum variant object must contain one entry")
                })?;
                if iter.next().is_some() {
                    return Err(<Error as de::Error>::custom(
                        "enum variant object must contain exactly one entry",
                    ));
                }
                let variant = seed.deserialize(
                    <String as IntoDeserializer<Error>>::into_deserializer(name.into_string()),
                )?;
                Ok((
                    variant,
                    VariantDe {
                        payload: Some(payload),
                    },
                ))
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected enum representation, got {}",
                describe(&other)
            ))),
        }
    }
}
