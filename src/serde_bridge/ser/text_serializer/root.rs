use serde::ser::{self, Serialize};

use crate::error::{Error, Result};

use super::object::ObjectCompound;
use super::seq::SeqCompound;
use super::shared::{scalar_root_err, top_err};
use super::variants::UnreachableCompound;

/// Render `value` directly to a Ktav string. The top-level value must
/// be an Object (struct or map) or an Array (seq / tuple / tuple
/// struct) per § 5.0.1. A root enum tuple variant serializes as the
/// same single-pair Object document the owned `to_value` path
/// produces (§ 8.2 byte-identity). Scalar-like roots are rejected
/// (§ 5.9.0 `ScalarRoot`).
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    // 2048 is a reasonable default: tiny documents pay a negligible heap
    // cost (the allocator returns it to its arena on drop), while medium
    // and large documents avoid several doubling reallocations (a 22 KB
    // output would otherwise go 256 → 512 → … → 32768 = 7 reallocs).
    let mut out = String::with_capacity(2048);
    value.serialize(RootSer { out: &mut out })?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// RootSer — top-level document. Accepts Object (struct / map) and Array (seq / tuple / tuple struct) roots; root enum tuple variants serialize as a single-pair Object.
// ---------------------------------------------------------------------------

pub(super) struct RootSer<'a> {
    out: &'a mut String,
}

impl<'a> ser::Serializer for RootSer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqCompound<'a>;
    type SerializeTuple = SeqCompound<'a>;
    type SerializeTupleStruct = SeqCompound<'a>;
    type SerializeTupleVariant = SeqCompound<'a>;
    type SerializeMap = ObjectCompound<'a>;
    type SerializeStruct = ObjectCompound<'a>;
    type SerializeStructVariant = UnreachableCompound;

    fn serialize_bool(self, _: bool) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_i8(self, _: i8) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_i16(self, _: i16) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_i32(self, _: i32) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_i64(self, _: i64) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_u8(self, _: u8) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_u16(self, _: u16) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_u32(self, _: u32) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_u64(self, _: u64) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_f32(self, _: f32) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_f64(self, _: f64) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_char(self, _: char) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_str(self, _: &str) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        Err(top_err())
    }
    fn serialize_none(self) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(scalar_root_err())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        // A unit variant root serializes as a bare string scalar,
        // i.e. exactly the ScalarRoot case (spec § 5.9.0).
        Err(scalar_root_err())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()> {
        Err(top_err())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqCompound<'a>> {
        if let Some(0) = len {
            // § 5.9.3: empty Array root → `[]\n`.
            self.out.push_str("[]\n");
            return Ok(SeqCompound::closed(self.out));
        }
        // Items start bare at indent 0; if the FIRST element turns out
        // to be a compound, SeqCompound switches the whole root Array
        // to the § 5.9.3 lone-`[` wrap on that element.
        Ok(SeqCompound::root(self.out))
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqCompound<'a>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SeqCompound<'a>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SeqCompound<'a>> {
        // Byte-identity with the owned path (§ 8.2): `to_value` maps a
        // root tuple variant to `Value::Object({variant: items})`, i.e.
        // a single-pair root Object whose value is an Array. The
        // variant name IS the root Object's first serialized key at
        // byte offset 0, so the § 5.9.10 rule (c) guard applies.
        if variant.is_empty() {
            return Err(Error::Unrepresentable(
                crate::error::ReasonCode::EmptyKeyName,
            ));
        }
        crate::render::helpers::push_escaped_key_segment(variant, true, self.out);
        if len == 0 {
            self.out.push_str(": []\n");
            return Ok(SeqCompound::closed(self.out));
        }
        self.out.push_str(": [\n");
        Ok(SeqCompound::wrapped(self.out, 1, 0))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<ObjectCompound<'a>> {
        Ok(ObjectCompound::root(self.out))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<ObjectCompound<'a>> {
        Ok(ObjectCompound::root(self.out))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(top_err())
    }
}
