//! The top-level `serde::Serializer` that builds a [`Value`] from any
//! `T: Serialize`.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use serde::ser::Serialize;

use crate::error::{Error, Result};
use crate::parser::classify::try_parse_integer;
use crate::value::{Scalar, Value};

use super::map_serializer::MapSerializer;
use super::seq_serializer::SeqSerializer;
use super::struct_serializer::StructSerializer;
use super::struct_variant_serializer::StructVariantSerializer;
use super::tuple_variant_serializer::TupleVariantSerializer;

/// Format an integer into a `Scalar` via `itoa` — same fast path as
/// `text_serializer::push_int_pair`, but producing a stored Value rather
/// than writing into a text buffer.
fn int_scalar<I: itoa::Integer>(v: I) -> Scalar {
    let mut buf = itoa::Buffer::new();
    buf.format(v).into()
}

/// WHY (R14-F2): the parser's normative Integer domain (§ 5, § 5.2 rule 13)
/// is i64 — `try_parse_integer` rejects wider decimal literals, and
/// `parse()` of such a literal stores it as `Value::String` (fixtures
/// `i64_overflow_to_string` / `big_overflow_to_string`). `to_value` must
/// produce the same `Value` that `parse()` of the emitted text would, or
/// `to_value -> writer -> parse` silently changes both the variant and the
/// canonical bytes. In-domain stays `Value::Integer`; beyond it becomes
/// `Value::String` — infallible, no new error kind.
fn int_value_in_i64_domain<I: itoa::Integer>(v: I) -> Value {
    let mut buf = itoa::Buffer::new();
    let text = buf.format(v);
    if try_parse_integer(text).is_some() {
        Value::Integer(text.into())
    } else {
        Value::String(text.into())
    }
}

/// WHY (R13-F1): normalize through the same path the parser uses — parse the
/// shortest ryu-f32 decimal string back as f64 (see `parse_float_value` in
/// `src/parser/inline.rs`), then re-emit with ryu-f64 — so the stored payload
/// matches `parse()` of this number. ryu format32/format64 pick
/// decimal-vs-scientific notation at different thresholds, so the raw f32
/// output can differ. Do NOT use `v as f64` + ryu-f64: widening changes the
/// shortest digit set (e.g. 0.1_f32). The f64 parse cannot fail: ryu always
/// yields a valid finite decimal literal within f64 range.
fn float_scalar_f32(v: f32) -> Result<Scalar> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf32 = ryu::Buffer::new();
    let short = buf32.format(v);
    let as_f64: f64 = short
        .parse()
        .expect("ryu f32 output is a valid finite f64 literal");
    let mut buf64 = ryu::Buffer::new();
    Ok(buf64.format(as_f64).into())
}

/// Same as [`float_scalar_f32`], for `f64`.
fn float_scalar_f64(v: f64) -> Result<Scalar> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf = ryu::Buffer::new();
    Ok(buf.format(v).into())
}

pub(crate) struct ValueSerializer;

impl serde::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_i128(self, v: i128) -> Result<Value> {
        Ok(int_value_in_i64_domain(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::Integer(int_scalar(v)))
    }
    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(int_value_in_i64_domain(v))
    }
    fn serialize_u128(self, v: u128) -> Result<Value> {
        Ok(int_value_in_i64_domain(v))
    }
    fn serialize_f32(self, v: f32) -> Result<Value> {
        Ok(Value::Float(float_scalar_f32(v)?))
    }
    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Float(float_scalar_f64(v)?))
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        let mut s = crate::value::Scalar::new("");
        s.push(v);
        Ok(Value::String(s))
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        // Mirrors serde_json: array of integers. Each element is a typed
        // integer now.
        let items = v.iter().map(|&b| Value::Integer(int_scalar(b))).collect();
        Ok(Value::Array(items))
    }

    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<Value> {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        Ok(Value::String(variant.into()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value> {
        let mut map = IndexMap::with_capacity_and_hasher(1, FxBuildHasher);
        map.insert(variant.into(), value.serialize(ValueSerializer)?);
        Ok(Value::Object(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SeqSerializer> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantSerializer> {
        Ok(TupleVariantSerializer {
            variant,
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<MapSerializer> {
        Ok(MapSerializer {
            entries: IndexMap::with_capacity_and_hasher(len.unwrap_or(0), FxBuildHasher),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<StructSerializer> {
        Ok(StructSerializer {
            entries: IndexMap::with_capacity_and_hasher(len, FxBuildHasher),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructVariantSerializer> {
        Ok(StructVariantSerializer {
            variant,
            entries: IndexMap::with_capacity_and_hasher(len, FxBuildHasher),
        })
    }
}
