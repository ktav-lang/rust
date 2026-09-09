//! Shared typed-map-key deserializer used by BOTH read paths — the owned
//! `Value` tree (`de::from_value`) and the thin event stream (`from_str`).
//!
//! Policy (R15-F1): a key name is converted to a non-string type ONLY when
//! the target type asks for it through a specific `deserialize_*` method;
//! `deserialize_any` and the string methods always yield the original text,
//! so a numeric-looking name into a `String` target stays verbatim.

use std::str::FromStr;

use serde::de::{self, DeserializeSeed, Deserializer, EnumAccess, VariantAccess, Visitor};

use crate::error::{Error, Result};

#[derive(Clone, Copy)]
enum KeyText<'key, 'de> {
    /// Event-path key, borrowed straight from the document buffer.
    Borrowed(&'de str),
    /// Owned-tree key; the scalar moves out of the map iteration, so the
    /// text is only valid for the duration of the key visit.
    Local(&'key str),
}

pub(crate) struct KeyDeserializer<'key, 'de> {
    text: KeyText<'key, 'de>,
}

impl<'key, 'de> KeyDeserializer<'key, 'de> {
    pub(crate) fn borrowed(value: &'de str) -> Self {
        Self {
            text: KeyText::Borrowed(value),
        }
    }

    pub(crate) fn local(value: &'key str) -> Self {
        Self {
            text: KeyText::Local(value),
        }
    }

    fn visit_text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.text {
            KeyText::Borrowed(s) => visitor.visit_borrowed_str(s),
            KeyText::Local(s) => visitor.visit_str(s),
        }
    }

    fn parse<T: FromStr>(&self, type_name: &'static str) -> Result<T> {
        let s = match self.text {
            KeyText::Borrowed(s) => s,
            KeyText::Local(s) => s,
        };
        s.parse::<T>().map_err(|_| {
            <Error as de::Error>::custom(format!("failed to parse map key '{s}' as {type_name}"))
        })
    }
}

impl<'de, 'key> Deserializer<'de> for KeyDeserializer<'key, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_text(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_text(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_text(visitor)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.parse("bool")?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse("i8")?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse("i16")?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse("i32")?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse("i64")?)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Exact 128-bit parse — no 64-bit/f64 hop (would lose range/precision).
        visitor.visit_i128(self.parse("i128")?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse("u8")?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse("u16")?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse("u32")?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse("u64")?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Exact 128-bit parse — no 64-bit/f64 hop (would lose range/precision).
        visitor.visit_u128(self.parse("u128")?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.parse("f32")?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.parse("f64")?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = match self.text {
            KeyText::Borrowed(s) => s,
            KeyText::Local(s) => s,
        };
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(<Error as de::Error>::custom(format!(
                "expected single character map key, got '{s}'"
            ))),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(KeyEnum { text: self.text })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_text(visitor)
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf option unit unit_struct seq tuple tuple_struct map
        struct ignored_any
    }
}

struct KeyEnum<'key, 'de> {
    text: KeyText<'key, 'de>,
}

impl<'de, 'key> EnumAccess<'de> for KeyEnum<'key, 'de> {
    type Error = Error;
    type Variant = KeyUnitVariant;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let variant = seed.deserialize(KeyDeserializer { text: self.text })?;
        Ok((variant, KeyUnitVariant))
    }
}

/// A bare key name carries no variant payload — mirrors serde's own
/// `UnitOnly` variant access used by its string deserializers.
struct KeyUnitVariant;

impl<'de> VariantAccess<'de> for KeyUnitVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value> {
        Err(<Error as de::Error>::invalid_type(
            de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(<Error as de::Error>::invalid_type(
            de::Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(<Error as de::Error>::invalid_type(
            de::Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}
