//! `serde::Deserializer` over a flat [`Event`] stream.
//!
//! The driver is a single shared cursor (`EventCursor`) into the events
//! slice; sub-deserializers (`EventDeserializer`) and access objects
//! (`EventMap`, `EventSeq`) borrow `&mut` to it and advance it as they
//! consume tokens. There is no per-compound allocation, no recursion
//! into a tree — just `cursor.next()` walking forward.
//!
//! Number deserialization re-uses [`super::fast_num`] for the byte-loop
//! atoi; floats stay on `<f64 as FromStr>::from_str`.

use std::str::FromStr;

use serde::de::{
    self, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};

use crate::error::{Error, Result};

use super::event::Event;
use super::fast_num;

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

pub(crate) struct EventCursor<'a, 'e> {
    events: &'e [Event<'a>],
    pos: usize,
}

impl<'a, 'e> EventCursor<'a, 'e> {
    pub(crate) fn new(events: &'e [Event<'a>]) -> Self {
        Self { events, pos: 0 }
    }

    #[inline]
    fn peek(&self) -> Option<&Event<'a>> {
        self.events.get(self.pos)
    }

    #[inline]
    fn next(&mut self) -> Option<Event<'a>> {
        let e = self.events.get(self.pos).copied();
        if e.is_some() {
            self.pos += 1;
        }
        e
    }

    /// Consume one full value starting at the cursor — for
    /// `IgnoredAny` / unknown-field skipping.
    fn skip_value(&mut self) -> Result<()> {
        let mut depth: usize = 0;
        loop {
            let ev = self.next().ok_or_else(|| {
                <Error as de::Error>::custom("unexpected end of event stream while skipping")
            })?;
            match ev {
                Event::BeginObject | Event::BeginArray => depth += 1,
                Event::EndObject | Event::EndArray => {
                    if depth == 0 {
                        return Err(<Error as de::Error>::custom(
                            "unexpected close while skipping value",
                        ));
                    }
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                // Key is part of an object body — don't terminate on it.
                Event::Key(_) => {}
                _ => {
                    if depth == 0 {
                        return Ok(());
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Deserializer
// ---------------------------------------------------------------------------

pub(crate) struct EventDeserializer<'a, 'e, 'c> {
    cursor: &'c mut EventCursor<'a, 'e>,
}

impl<'a, 'e, 'c> EventDeserializer<'a, 'e, 'c> {
    pub(crate) fn new(cursor: &'c mut EventCursor<'a, 'e>) -> Self {
        Self { cursor }
    }

    #[inline(never)]
    fn parse_error(s: &str, type_name: &'static str) -> Error {
        <Error as de::Error>::custom(format!("failed to parse '{}' as {}", s, type_name))
    }

    /// Pull the next event and require it to be a numeric/string scalar
    /// — return its inner text. Anything else is a type error.
    #[inline]
    fn next_numeric_text(&mut self, type_name: &'static str) -> Result<&'a str> {
        match self.cursor.next() {
            Some(Event::Integer(s)) | Some(Event::Float(s)) | Some(Event::Str(s)) => Ok(s),
            other => Err(<Error as de::Error>::custom(format!(
                "expected {}, got {:?}",
                type_name, other
            ))),
        }
    }
}

impl<'de, 'e, 'c> Deserializer<'de> for EventDeserializer<'de, 'e, 'c> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Null) => visitor.visit_unit(),
            Some(Event::Bool(b)) => visitor.visit_bool(b),
            Some(Event::Integer(s)) => {
                if let Some(i) = fast_num::parse_i64(s) {
                    visitor.visit_i64(i)
                } else if let Some(u) = fast_num::parse_u64(s) {
                    visitor.visit_u64(u)
                } else {
                    visitor.visit_borrowed_str(s)
                }
            }
            Some(Event::Float(s)) => match s.parse::<f64>() {
                Ok(f) => visitor.visit_f64(f),
                Err(_) => visitor.visit_borrowed_str(s),
            },
            Some(Event::Str(s)) => visitor.visit_borrowed_str(s),
            Some(Event::BeginObject) => visitor.visit_map(EventMap { cursor: self.cursor }),
            Some(Event::BeginArray) => visitor.visit_seq(EventSeq { cursor: self.cursor }),
            other => Err(<Error as de::Error>::custom(format!(
                "deserialize_any: unexpected event {:?}",
                other
            ))),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Bool(b)) => visitor.visit_bool(b),
            Some(Event::Str(s)) => match s {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(Self::parse_error(s, "bool")),
            },
            other => Err(<Error as de::Error>::custom(format!(
                "expected bool, got {:?}",
                other
            ))),
        }
    }

    #[inline]
    fn deserialize_i8<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("i8")?;
        let v = fast_num::parse_i_bounded(s, i8::MIN as i64, i8::MAX as i64)
            .ok_or_else(|| Self::parse_error(s, "i8"))?;
        visitor.visit_i8(v as i8)
    }
    #[inline]
    fn deserialize_i16<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("i16")?;
        let v = fast_num::parse_i_bounded(s, i16::MIN as i64, i16::MAX as i64)
            .ok_or_else(|| Self::parse_error(s, "i16"))?;
        visitor.visit_i16(v as i16)
    }
    #[inline]
    fn deserialize_i32<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("i32")?;
        let v = fast_num::parse_i_bounded(s, i32::MIN as i64, i32::MAX as i64)
            .ok_or_else(|| Self::parse_error(s, "i32"))?;
        visitor.visit_i32(v as i32)
    }
    #[inline]
    fn deserialize_i64<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("i64")?;
        let v = fast_num::parse_i64(s).ok_or_else(|| Self::parse_error(s, "i64"))?;
        visitor.visit_i64(v)
    }
    #[inline]
    fn deserialize_u8<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("u8")?;
        let v = fast_num::parse_u_bounded(s, u8::MAX as u64)
            .ok_or_else(|| Self::parse_error(s, "u8"))?;
        visitor.visit_u8(v as u8)
    }
    #[inline]
    fn deserialize_u16<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("u16")?;
        let v = fast_num::parse_u_bounded(s, u16::MAX as u64)
            .ok_or_else(|| Self::parse_error(s, "u16"))?;
        visitor.visit_u16(v as u16)
    }
    #[inline]
    fn deserialize_u32<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("u32")?;
        let v = fast_num::parse_u_bounded(s, u32::MAX as u64)
            .ok_or_else(|| Self::parse_error(s, "u32"))?;
        visitor.visit_u32(v as u32)
    }
    #[inline]
    fn deserialize_u64<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("u64")?;
        let v = fast_num::parse_u64(s).ok_or_else(|| Self::parse_error(s, "u64"))?;
        visitor.visit_u64(v)
    }
    fn deserialize_f32<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("f32")?;
        visitor.visit_f32(f32::from_str(s).map_err(|_| Self::parse_error(s, "f32"))?)
    }
    fn deserialize_f64<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        let s = self.next_numeric_text("f64")?;
        visitor.visit_f64(f64::from_str(s).map_err(|_| Self::parse_error(s, "f64"))?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Str(s)) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => visitor.visit_char(c),
                    _ => Err(<Error as de::Error>::custom(format!(
                        "expected single character, got '{}'",
                        s
                    ))),
                }
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected char, got {:?}",
                other
            ))),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Str(s)) | Some(Event::Integer(s)) | Some(Event::Float(s)) => {
                visitor.visit_borrowed_str(s)
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected string, got {:?}",
                other
            ))),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Str(s)) => visitor.visit_borrowed_bytes(s.as_bytes()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bytes, got {:?}",
                other
            ))),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Str(s)) => visitor.visit_byte_buf(s.as_bytes().to_vec()),
            other => Err(<Error as de::Error>::custom(format!(
                "expected bytes, got {:?}",
                other
            ))),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.peek() {
            Some(Event::Null) => {
                self.cursor.next();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.cursor.next() {
            Some(Event::Null) => visitor.visit_unit(),
            other => Err(<Error as de::Error>::custom(format!(
                "expected unit, got {:?}",
                other
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
        match self.cursor.next() {
            Some(Event::BeginArray) => visitor.visit_seq(EventSeq { cursor: self.cursor }),
            other => Err(<Error as de::Error>::custom(format!(
                "expected array, got {:?}",
                other
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
        match self.cursor.next() {
            Some(Event::BeginObject) => visitor.visit_map(EventMap { cursor: self.cursor }),
            other => Err(<Error as de::Error>::custom(format!(
                "expected object, got {:?}",
                other
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
        // Two shapes: bare string variant, or `{ name: payload }` object.
        match self.cursor.peek() {
            Some(Event::Str(_)) => visitor.visit_enum(EventEnum {
                cursor: self.cursor,
                from_object: false,
            }),
            Some(Event::BeginObject) => {
                self.cursor.next(); // consume BeginObject
                visitor.visit_enum(EventEnum {
                    cursor: self.cursor,
                    from_object: true,
                })
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected enum representation, got {:?}",
                other
            ))),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // Skip however many events make up the next value, then visit
        // unit so the caller sees something.
        self.cursor.skip_value()?;
        visitor.visit_unit()
    }
}

// ---------------------------------------------------------------------------
// MapAccess
// ---------------------------------------------------------------------------

struct EventMap<'a, 'e, 'c> {
    cursor: &'c mut EventCursor<'a, 'e>,
}

impl<'de, 'e, 'c> MapAccess<'de> for EventMap<'de, 'e, 'c> {
    type Error = Error;

    #[inline]
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.cursor.peek() {
            Some(Event::EndObject) => {
                self.cursor.next();
                Ok(None)
            }
            Some(Event::Key(_)) => {
                // Consume the Key event and feed the borrowed slice to
                // the seed via a borrowed-str deserializer.
                let k = match self.cursor.next() {
                    Some(Event::Key(k)) => k,
                    _ => unreachable!(),
                };
                seed.deserialize(BorrowedStrDeserializer { value: k }).map(Some)
            }
            other => Err(<Error as de::Error>::custom(format!(
                "expected Key or EndObject in map, got {:?}",
                other
            ))),
        }
    }

    #[inline]
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(EventDeserializer::new(self.cursor))
    }
}

// ---------------------------------------------------------------------------
// SeqAccess
// ---------------------------------------------------------------------------

struct EventSeq<'a, 'e, 'c> {
    cursor: &'c mut EventCursor<'a, 'e>,
}

impl<'de, 'e, 'c> SeqAccess<'de> for EventSeq<'de, 'e, 'c> {
    type Error = Error;

    #[inline]
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.cursor.peek() {
            Some(Event::EndArray) => {
                self.cursor.next();
                Ok(None)
            }
            _ => seed.deserialize(EventDeserializer::new(self.cursor)).map(Some),
        }
    }
}

// ---------------------------------------------------------------------------
// EnumAccess
// ---------------------------------------------------------------------------

struct EventEnum<'a, 'e, 'c> {
    cursor: &'c mut EventCursor<'a, 'e>,
    from_object: bool,
}

impl<'de, 'e, 'c> EnumAccess<'de> for EventEnum<'de, 'e, 'c> {
    type Error = Error;
    type Variant = EventVariant<'de, 'e, 'c>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        if self.from_object {
            // Inside object: next event MUST be Key.
            let name = match self.cursor.next() {
                Some(Event::Key(k)) => k,
                other => {
                    return Err(<Error as de::Error>::custom(format!(
                        "expected enum variant key, got {:?}",
                        other
                    )))
                }
            };
            let v = seed.deserialize(BorrowedStrDeserializer { value: name })?;
            Ok((
                v,
                EventVariant {
                    cursor: self.cursor,
                    has_payload: true,
                },
            ))
        } else {
            // Bare string: this is the variant name; no payload follows.
            let name = match self.cursor.next() {
                Some(Event::Str(s)) => s,
                other => {
                    return Err(<Error as de::Error>::custom(format!(
                        "expected enum string, got {:?}",
                        other
                    )))
                }
            };
            let v = seed.deserialize(BorrowedStrDeserializer { value: name })?;
            Ok((
                v,
                EventVariant {
                    cursor: self.cursor,
                    has_payload: false,
                },
            ))
        }
    }
}

struct EventVariant<'a, 'e, 'c> {
    cursor: &'c mut EventCursor<'a, 'e>,
    has_payload: bool,
}

impl<'de, 'e, 'c> VariantAccess<'de> for EventVariant<'de, 'e, 'c> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if !self.has_payload {
            Ok(())
        } else {
            // Object-form unit variant: consume the value+EndObject.
            self.cursor.skip_value()?;
            // Then the closing EndObject of the wrapping enum object.
            match self.cursor.next() {
                Some(Event::EndObject) => Ok(()),
                other => Err(<Error as de::Error>::custom(format!(
                    "expected EndObject after enum variant, got {:?}",
                    other
                ))),
            }
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        if !self.has_payload {
            return Err(<Error as de::Error>::custom(
                "newtype variant requires a payload",
            ));
        }
        let value = seed.deserialize(EventDeserializer::new(self.cursor))?;
        // Consume the closing EndObject of the wrapping enum object.
        match self.cursor.next() {
            Some(Event::EndObject) => Ok(value),
            other => Err(<Error as de::Error>::custom(format!(
                "expected EndObject after enum variant, got {:?}",
                other
            ))),
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        if !self.has_payload {
            return Err(<Error as de::Error>::custom(
                "tuple variant requires a payload",
            ));
        }
        let value = EventDeserializer::new(self.cursor).deserialize_seq(visitor)?;
        match self.cursor.next() {
            Some(Event::EndObject) => Ok(value),
            other => Err(<Error as de::Error>::custom(format!(
                "expected EndObject after enum variant, got {:?}",
                other
            ))),
        }
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if !self.has_payload {
            return Err(<Error as de::Error>::custom(
                "struct variant requires a payload",
            ));
        }
        let value = EventDeserializer::new(self.cursor).deserialize_map(visitor)?;
        match self.cursor.next() {
            Some(Event::EndObject) => Ok(value),
            other => Err(<Error as de::Error>::custom(format!(
                "expected EndObject after enum variant, got {:?}",
                other
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// BorrowedStrDeserializer — exact same shape as the tree-builder's, kept
// local so the modules don't cross-depend.
// ---------------------------------------------------------------------------

struct BorrowedStrDeserializer<'de> {
    value: &'de str,
}

impl<'de> BorrowedStrDeserializer<'de> {
    fn parse<T: FromStr>(&self, ty: &str) -> Result<T> {
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
