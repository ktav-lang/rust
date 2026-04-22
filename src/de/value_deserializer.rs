//! The top-level `serde::Deserializer` wrapper around a [`Value`].

use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};

use crate::error::{Error, Result};
use crate::value::Value;

use super::describe::describe;
use super::enum_access::EnumDe;
use super::map_access::MapDe;
use super::seq_access::SeqDe;

pub(crate) struct ValueDeserializer {
    value: Value,
}

impl ValueDeserializer {
    pub(crate) fn new(value: Value) -> Self {
        Self { value }
    }

    fn parse_scalar<T: FromStr>(s: &str, type_name: &str) -> Result<T> {
        s.parse::<T>().map_err(|_| {
            <Error as de::Error>::custom(format!("failed to parse '{}' as {}", s, type_name))
        })
    }

    /// Returns the text content of a value that should parse as a scalar —
    /// works for `Value::String`, `Value::Integer`, and `Value::Float`. For
    /// numeric targets we accept all three: typed-marker documents produce
    /// Integer/Float, legacy / marker-less documents produce String, and
    /// both paths go through `parse_scalar::<T>`.
    fn expect_numeric_string(&self, type_name: &str) -> Result<&str> {
        match &self.value {
            Value::String(s) => Ok(s.as_str()),
            Value::Integer(s) => Ok(s.as_str()),
            Value::Float(s) => Ok(s.as_str()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected {}, got {}",
                type_name,
                describe(other)
            ))),
        }
    }
}

impl<'de> Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            // For `Integer` / `Float` under `deserialize_any`, surface the
            // typed scalar as the closest serde numeric type we can — f64
            // for Float (guaranteed precision loss is documented), i64 for
            // Integer that fits, else String to preserve the literal.
            Value::Integer(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    visitor.visit_i64(i)
                } else if let Ok(u) = s.parse::<u64>() {
                    visitor.visit_u64(u)
                } else {
                    visitor.visit_string(s.into_string())
                }
            }
            Value::Float(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    visitor.visit_f64(f)
                } else {
                    visitor.visit_string(s.into_string())
                }
            }
            Value::String(s) => visitor.visit_string(s.into_string()),
            Value::Array(items) => visitor.visit_seq(SeqDe {
                iter: items.into_iter(),
            }),
            Value::Object(obj) => visitor.visit_map(MapDe::new(obj)),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.value {
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::String(s) => visitor.visit_bool(Self::parse_scalar(s, "bool")?),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bool, got {}",
                describe(other)
            ))),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("i8")?;
        visitor.visit_i8(Self::parse_scalar(s, "i8")?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("i16")?;
        visitor.visit_i16(Self::parse_scalar(s, "i16")?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("i32")?;
        visitor.visit_i32(Self::parse_scalar(s, "i32")?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("i64")?;
        visitor.visit_i64(Self::parse_scalar(s, "i64")?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("u8")?;
        visitor.visit_u8(Self::parse_scalar(s, "u8")?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("u16")?;
        visitor.visit_u16(Self::parse_scalar(s, "u16")?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("u32")?;
        visitor.visit_u32(Self::parse_scalar(s, "u32")?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("u64")?;
        visitor.visit_u64(Self::parse_scalar(s, "u64")?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("f32")?;
        visitor.visit_f32(Self::parse_scalar(s, "f32")?)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.expect_numeric_string("f64")?;
        visitor.visit_f64(Self::parse_scalar(s, "f64")?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // `char` takes only String — typed Integer/Float would be odd here.
        let s = match &self.value {
            Value::String(s) => s.as_str(),
            other => {
                return Err(<Error as de::Error>::custom(format!(
                    "expected char, got {}",
                    describe(other)
                )))
            }
        };
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(<Error as de::Error>::custom(format!(
                "expected a single character, got '{}'",
                s
            ))),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Allow Integer/Float to deserialize as their textual form — this
        // preserves arbitrary-precision Integer values (e.g.
        // `:i 99999999999999999999` into a `String` field keeps every
        // digit) and matches the "Value holds text" invariant.
        //
        // Pass `&str` rather than an owned `String`: because the Value
        // scalar is a `CompactString`, most payloads are inline; moving
        // into `visit_string(s.into_string())` would allocate a fresh
        // `String` for every inline scalar. `visit_str(&s)` lets serde's
        // String deserializer copy once at the boundary — same result,
        // no extra allocation on the common short-string path.
        match &self.value {
            Value::String(s) | Value::Integer(s) | Value::Float(s) => visitor.visit_str(s),
            other => Err(<Error as de::Error>::custom(format!(
                "expected string, got {}",
                describe(other)
            ))),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_string(s.into_string()),
            Value::Integer(s) => visitor.visit_string(s.into_string()),
            Value::Float(s) => visitor.visit_string(s.into_string()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected string, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = match &self.value {
            Value::String(s) => s.as_str(),
            other => {
                return Err(<Error as de::Error>::custom(format!(
                    "expected bytes, got {}",
                    describe(other)
                )))
            }
        };
        visitor.visit_bytes(s.as_bytes())
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_byte_buf(s.into_string().into_bytes()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bytes, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            other => Err(<Error as de::Error>::custom(format!(
                "expected unit, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Array(items) => visitor.visit_seq(SeqDe {
                iter: items.into_iter(),
            }),
            other => Err(<Error as de::Error>::custom(format!(
                "expected an array, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Object(obj) => visitor.visit_map(MapDe::new(obj)),
            other => Err(<Error as de::Error>::custom(format!(
                "expected an object, got {}",
                describe(&other)
            ))),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(EnumDe { value: self.value })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}
