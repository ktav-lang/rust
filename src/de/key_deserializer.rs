//! Shared typed-map-key deserializer used by BOTH read paths — the owned
//! `Value` tree (`de::from_value`) and the thin event stream (`from_str`).
//!
//! Policy (R15-F1): a key name is converted to a non-string type ONLY when
//! the target type asks for it through a specific `deserialize_*` method;
//! `deserialize_any` and the string methods always yield the original text,
//! so a numeric-looking name into a `String` target stays verbatim.
//!
//! Option keys (R16-F1): `deserialize_option` always yields `visit_some` — a
//! key name always exists (§5), so the literal name `null` stays a string and
//! never becomes `None`; the writer's rejection of `None`/`Some(None)` keys is
//! unchanged.
//!
//! Owned buffers (R16-F3): in the owned branch the adapter owns the key
//! Scalar, so the string methods transfer its heap buffer via `visit_string`
//! while numeric methods keep parsing the borrowed slice; the borrowed event
//! branch keeps `visit_borrowed_str` and never lends a locally owned buffer as
//! a long-lived borrow.

use std::str::FromStr;

use serde::de::{self, DeserializeSeed, Deserializer, EnumAccess, VariantAccess, Visitor};

use crate::error::{Error, Result};
use crate::value::Scalar;

#[derive(Clone)]
enum KeyText<'de> {
    /// Event-path key, borrowed straight from the document buffer.
    Borrowed(&'de str),
    /// Owned-tree key; the scalar moves out of the map iteration and is
    /// owned here so its heap buffer can be handed to a String target
    /// (R16-F3) instead of being copied.
    Local(Scalar),
}

pub(crate) struct KeyDeserializer<'de> {
    text: KeyText<'de>,
}

impl<'de> KeyDeserializer<'de> {
    pub(crate) fn borrowed(value: &'de str) -> Self {
        Self {
            text: KeyText::Borrowed(value),
        }
    }

    pub(crate) fn local(value: Scalar) -> Self {
        Self {
            text: KeyText::Local(value),
        }
    }

    fn text(&self) -> &str {
        match &self.text {
            KeyText::Borrowed(s) => s,
            KeyText::Local(s) => s.as_str(),
        }
    }

    fn visit_text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.text {
            KeyText::Borrowed(s) => visitor.visit_borrowed_str(s),
            KeyText::Local(s) => visitor.visit_str(&s),
        }
    }

    fn visit_owned_text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.text {
            KeyText::Borrowed(s) => visitor.visit_borrowed_str(s),
            // R16-F3: hand the buffer to the target String — `into_string`
            // moves a heap-backed Scalar's allocation without copying; an
            // inline Scalar allocates here, but a String target needed that
            // allocation anyway. Numeric targets below still parse the
            // borrowed slice untouched.
            KeyText::Local(s) => visitor.visit_string(s.into_string()),
        }
    }

    fn parse<T: FromStr>(&self, type_name: &'static str) -> Result<T> {
        let s = self.text();
        s.parse::<T>().map_err(|_| {
            <Error as de::Error>::custom(format!("failed to parse map key '{s}' as {type_name}"))
        })
    }
}

impl<'de> Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_text(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_owned_text(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_owned_text(visitor)
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
        let s = self.text();
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

    // A map key name always exists (§5: Object names are strings, not scalar
    // values), so an Option key is always `Some`: the inner type deserializes
    // from the same name, and a literal name `null` stays the string "null"
    // rather than becoming `None` (R16-F1).
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct seq tuple tuple_struct map
        struct ignored_any
    }
}

struct KeyEnum<'de> {
    text: KeyText<'de>,
}

impl<'de> EnumAccess<'de> for KeyEnum<'de> {
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
