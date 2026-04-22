//! `serde::Deserializer` over a [`ThinValue<'a>`]. Prefers
//! `visit_borrowed_str` when the content is still borrowed from the input
//! buffer — destination types that accept `&'de str` can avoid any
//! allocation.

use std::str::FromStr;

use bumpalo::collections::Vec as BumpVec;
use serde::de::{
    self, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};

use crate::error::{Error, Result};

use super::value::ThinValue;

pub(crate) struct ThinDeserializer<'a> {
    value: ThinValue<'a>,
}

impl<'a> ThinDeserializer<'a> {
    pub(crate) fn new(value: ThinValue<'a>) -> Self {
        Self { value }
    }

    fn parse_scalar<T: FromStr>(s: &str, type_name: &str) -> Result<T> {
        s.parse::<T>().map_err(|_| {
            <Error as de::Error>::custom(format!("failed to parse '{}' as {}", s, type_name))
        })
    }

    fn expect_string(&self, type_name: &str) -> Result<&'a str> {
        match &self.value {
            ThinValue::Str(s) => Ok(*s),
            other => Err(<Error as de::Error>::custom(format!(
                "expected {}, got {}",
                type_name,
                describe(other)
            ))),
        }
    }

    /// Numeric targets accept all three scalar-text variants — Integer,
    /// Float, and Str. Documents with typed markers produce the typed
    /// variants; legacy marker-less documents produce Str, and both parse
    /// into the target through `FromStr`.
    fn expect_numeric_string(&self, type_name: &str) -> Result<&'a str> {
        match &self.value {
            ThinValue::Str(s) => Ok(*s),
            ThinValue::Integer(s) => Ok(*s),
            ThinValue::Float(s) => Ok(*s),
            other => Err(<Error as de::Error>::custom(format!(
                "expected {}, got {}",
                type_name,
                describe(other)
            ))),
        }
    }
}

impl<'de> Deserializer<'de> for ThinDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Null => visitor.visit_unit(),
            ThinValue::Bool(b) => visitor.visit_bool(b),
            ThinValue::Integer(s) => {
                // `deserialize_any` needs a concrete numeric callback when
                // possible; else fall back to surfacing the string.
                if let Ok(i) = s.parse::<i64>() {
                    visitor.visit_i64(i)
                } else if let Ok(u) = s.parse::<u64>() {
                    visitor.visit_u64(u)
                } else {
                    visitor.visit_borrowed_str(s)
                }
            }
            ThinValue::Float(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    visitor.visit_f64(f)
                } else {
                    visitor.visit_borrowed_str(s)
                }
            }
            ThinValue::Str(s) => visitor.visit_borrowed_str(s),
            ThinValue::Array(items) => visitor.visit_seq(ThinSeq {
                iter: items.into_iter(),
            }),
            ThinValue::Object(pairs) => visitor.visit_map(ThinMap::new(pairs)),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.value {
            ThinValue::Bool(b) => visitor.visit_bool(*b),
            ThinValue::Str(s) => visitor.visit_bool(Self::parse_scalar(s, "bool")?),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bool, got {}",
                describe(other)
            ))),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(Self::parse_scalar(self.expect_numeric_string("i8")?, "i8")?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(Self::parse_scalar(
            self.expect_numeric_string("i16")?,
            "i16",
        )?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(Self::parse_scalar(
            self.expect_numeric_string("i32")?,
            "i32",
        )?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(Self::parse_scalar(
            self.expect_numeric_string("i64")?,
            "i64",
        )?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(Self::parse_scalar(self.expect_numeric_string("u8")?, "u8")?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(Self::parse_scalar(
            self.expect_numeric_string("u16")?,
            "u16",
        )?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(Self::parse_scalar(
            self.expect_numeric_string("u32")?,
            "u32",
        )?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(Self::parse_scalar(
            self.expect_numeric_string("u64")?,
            "u64",
        )?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(Self::parse_scalar(
            self.expect_numeric_string("f32")?,
            "f32",
        )?)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(Self::parse_scalar(
            self.expect_numeric_string("f64")?,
            "f64",
        )?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_string("char")?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(<Error as de::Error>::custom(format!(
                "expected single character, got '{}'",
                s
            ))),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Accept Integer/Float as their textual form — preserves arbitrary-
        // precision integer content and matches the owned-path behavior.
        match self.value {
            ThinValue::Str(s) | ThinValue::Integer(s) | ThinValue::Float(s) => {
                visitor.visit_borrowed_str(s)
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected string, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.value {
            ThinValue::Str(s) => visitor.visit_borrowed_bytes(s.as_bytes()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bytes, got {}",
                describe(other)
            ))),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Str(s) => visitor.visit_byte_buf(s.as_bytes().to_vec()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bytes, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Null => visitor.visit_unit(),
            other => Err(<Error as de::Error>::custom(format!(
                "expected unit, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Array(items) => visitor.visit_seq(ThinSeq {
                iter: items.into_iter(),
            }),
            other => Err(<Error as de::Error>::custom(format!(
                "expected array, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            ThinValue::Object(pairs) => visitor.visit_map(ThinMap::new(pairs)),
            other => Err(<Error as de::Error>::custom(format!(
                "expected object, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(ThinEnum { value: self.value })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}

// ---------------------------------------------------------------------------
// Seq / Map / Enum / Variant accesses
// ---------------------------------------------------------------------------

struct ThinSeq<'a> {
    iter: bumpalo::collections::vec::IntoIter<'a, ThinValue<'a>>,
}

impl<'de> SeqAccess<'de> for ThinSeq<'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.iter.next() {
            Some(v) => seed.deserialize(ThinDeserializer::new(v)).map(Some),
            None => Ok(None),
        }
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct ThinMap<'a> {
    iter: bumpalo::collections::vec::IntoIter<'a, (&'a str, ThinValue<'a>)>,
    next_value: Option<ThinValue<'a>>,
}

impl<'a> ThinMap<'a> {
    fn new(pairs: BumpVec<'a, (&'a str, ThinValue<'a>)>) -> Self {
        Self {
            iter: pairs.into_iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ThinMap<'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.iter.next() {
            Some((k, v)) => {
                self.next_value = Some(v);
                // Borrowed key directly — struct fields match by str.
                seed.deserialize(BorrowedStrDeserializer { value: k })
                    .map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let v = self
            .next_value
            .take()
            .ok_or_else(|| <Error as de::Error>::custom("next_value without key"))?;
        seed.deserialize(ThinDeserializer::new(v))
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct ThinEnum<'a> {
    value: ThinValue<'a>,
}

impl<'de> EnumAccess<'de> for ThinEnum<'de> {
    type Error = Error;
    type Variant = ThinVariant<'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        match self.value {
            ThinValue::Str(name) => {
                let v = seed.deserialize(BorrowedStrDeserializer { value: name })?;
                Ok((v, ThinVariant { payload: None }))
            }
            ThinValue::Object(pairs) => {
                let mut iter = pairs.into_iter();
                let (name, payload) = iter.next().ok_or_else(|| {
                    <Error as de::Error>::custom("enum variant object must contain one entry")
                })?;
                if iter.next().is_some() {
                    return Err(<Error as de::Error>::custom(
                        "enum variant object must contain exactly one entry",
                    ));
                }
                let v = seed.deserialize(BorrowedStrDeserializer { value: name })?;
                Ok((
                    v,
                    ThinVariant {
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

struct ThinVariant<'a> {
    payload: Option<ThinValue<'a>>,
}

impl<'de> VariantAccess<'de> for ThinVariant<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.payload {
            None => Ok(()),
            Some(_) => Err(<Error as de::Error>::custom("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let p = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected newtype variant"))?;
        seed.deserialize(ThinDeserializer::new(p))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        let p = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected tuple variant"))?;
        ThinDeserializer::new(p).deserialize_seq(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let p = self
            .payload
            .ok_or_else(|| <Error as de::Error>::custom("expected struct variant"))?;
        ThinDeserializer::new(p).deserialize_map(visitor)
    }
}

// ---------------------------------------------------------------------------
// BorrowedStrDeserializer — wraps a `&'de str` key for seed.deserialize
// without forcing an allocation (as `IntoDeserializer<'_, str>` would).
// ---------------------------------------------------------------------------

struct BorrowedStrDeserializer<'de> {
    value: &'de str,
}

impl<'de> BorrowedStrDeserializer<'de> {
    fn parse<T: std::str::FromStr>(&self, ty: &str) -> Result<T> {
        self.value.parse::<T>().map_err(|_| {
            <Error as de::Error>::custom(format!(
                "failed to parse map key '{}' as {}",
                self.value, ty
            ))
        })
    }
}

impl<'de> Deserializer<'de> for BorrowedStrDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_borrowed_str(self.value)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_borrowed_str(self.value)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_str(self.value)
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_borrowed_str(self.value)
    }

    // Numeric / bool / char map keys: parse the borrowed slice into the
    // target scalar and visit with the typed method. This is the canonical
    // serde pattern for text formats used as map keys.
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.parse::<bool>("bool")?)
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse::<i8>("i8")?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse::<i16>("i16")?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse::<i32>("i32")?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse::<i64>("i64")?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse::<u8>("u8")?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse::<u16>("u16")?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse::<u32>("u32")?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse::<u64>("u64")?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.parse::<f32>("f32")?)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.parse::<f64>("f64")?)
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut chars = self.value.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(<Error as de::Error>::custom(format!(
                "expected single character map key, got '{}'",
                self.value
            ))),
        }
    }

    serde::forward_to_deserialize_any! {
        i128 u128 bytes byte_buf option unit unit_struct newtype_struct
        seq tuple tuple_struct map struct enum ignored_any
    }
}

// ---------------------------------------------------------------------------
// describe helper
// ---------------------------------------------------------------------------

fn describe(v: &ThinValue) -> &'static str {
    match v {
        ThinValue::Null => "null",
        ThinValue::Bool(_) => "a bool",
        ThinValue::Integer(_) => "a typed integer",
        ThinValue::Float(_) => "a typed float",
        ThinValue::Str(_) => "a string",
        ThinValue::Array(_) => "an array",
        ThinValue::Object(_) => "an object",
    }
}
