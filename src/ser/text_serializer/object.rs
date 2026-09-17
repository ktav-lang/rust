use std::fmt::{self, Write as _};

use serde::ser::{self, Serialize, SerializeMap, SerializeStruct};

use crate::error::{Error, Result};
use crate::value::Scalar;

use super::seq::SeqCompound;
use super::shared::{
    key_err, needs_raw_marker, push_f32_pair, push_f64_pair, push_int_item, push_int_pair,
    write_indent,
};
use super::variants::{StructVariantPair, TupleVariantPair, UnreachableCompound};

// ---------------------------------------------------------------------------
// ObjectCompound — SerializeStruct + SerializeMap
//
// Two shapes:
// - Root { indent = 0 }: no surrounding `{}`, no closing.
// - Wrapped { field_indent, close_indent }: caller has already written
//   `: {\n`; we write each field at `field_indent`, at end we write
//   `<close_indent>}\n`.
// ---------------------------------------------------------------------------

pub(super) struct ObjectCompound<'a> {
    out: &'a mut String,
    field_indent: usize,
    close: Option<usize>, // Some(outer_indent) → write `<outer>}\n` at end.
    pending_key: Option<Scalar>, // used by SerializeMap
    empty_so_far: bool,
    /// True only for the document-root Object: its FIRST serialized
    /// key is the only one that can land at byte offset 0, so it is
    /// the only one eligible for the § 5.9.10 rule (c) U+FEFF guard.
    is_root: bool,
}

impl<'a> ObjectCompound<'a> {
    pub(super) fn root(out: &'a mut String) -> Self {
        Self {
            out,
            field_indent: 0,
            close: None,
            pending_key: None,
            empty_so_far: true,
            is_root: true,
        }
    }

    pub(super) fn wrapped(out: &'a mut String, field_indent: usize, close_indent: usize) -> Self {
        Self {
            out,
            field_indent,
            close: Some(close_indent),
            pending_key: None,
            empty_so_far: true,
            is_root: false,
        }
    }
}

impl<'a> SerializeStruct for ObjectCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        if key.is_empty() {
            return Err(Error::Unrepresentable(
                crate::error::ReasonCode::EmptyKeyName,
            ));
        }
        // Capture BEFORE clearing `empty_so_far`: the § 5.9.10 rule
        // (c) U+FEFF guard applies only to the root Object's first
        // serialized key, which is the only one at byte offset 0.
        let root_first_key = self.is_root && self.empty_so_far;
        self.empty_so_far = false;
        write_indent(self.out, self.field_indent);
        crate::render::helpers::push_escaped_key_segment(key, root_first_key, self.out);
        value.serialize(PairValueSer {
            out: self.out,
            indent: self.field_indent,
        })
    }

    fn end(self) -> Result<()> {
        if let Some(outer) = self.close {
            write_indent(self.out, outer);
            self.out.push_str("}\n");
        }
        Ok(())
    }
}

impl<'a> SerializeMap for ObjectCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.pending_key = Some(serialize_key_name(key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.pending_key.take().ok_or_else(|| {
            <Error as ser::Error>::custom("serialize_value without preceding key")
        })?;
        if key.is_empty() {
            return Err(Error::Unrepresentable(
                crate::error::ReasonCode::EmptyKeyName,
            ));
        }
        // See serialize_field: only the root Object's first serialized
        // key takes the § 5.9.10 rule (c) guard.
        let root_first_key = self.is_root && self.empty_so_far;
        self.empty_so_far = false;
        write_indent(self.out, self.field_indent);
        crate::render::helpers::push_escaped_key_segment(&key, root_first_key, self.out);
        value.serialize(PairValueSer {
            out: self.out,
            indent: self.field_indent,
        })
    }

    fn end(self) -> Result<()> {
        if let Some(outer) = self.close {
            write_indent(self.out, outer);
            self.out.push_str("}\n");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// KeyOnlySer — THE shared map key-name policy (R15-F2). This serializer
// defines how a map key becomes its name text; it is the single policy
// shared by the direct text writer (`ObjectCompound::serialize_key`) and
// `ser::to_value` (`MapSerializer::serialize_key`), so one key value always
// produces byte-identical name text regardless of which writer API is used.
//
// The name is written directly into the inline-capable `Scalar`
// (R16-F2), and `collect_str` is overridden to format `Display` names
// (and `fmt::Arguments`, which serde routes through it) into the same
// `Scalar` (R17-F2): serde's default `collect_str` materializes the
// formatted name as a temporary heap `String` before `serialize_str`
// runs. With the override, a short name (up to the 24-byte inline
// capacity on 64-bit) never allocates a heap `String`, and a long name
// pays only its single `Scalar` spill.
//
// Accepted scalar-like key types: `str`; `bool` (`true`/`false`); all
// native ints i8–i64 plus i128/u128 (plain decimal); `f32`/`f64`; `char`;
// unit-enum variant name; newtype-struct and `Some` (delegated). Everything
// else errors.
//
// WHY: float key names intentionally use Rust `Display`, NOT the `Value`
// -float ryu normalization — a key name is a string and must be
// byte-identical regardless of which writer API produced it; ryu
// normalization exists only so float VALUES re-parse identically (R15-F2).
// ---------------------------------------------------------------------------

struct KeyOnlySer<'a> {
    out: &'a mut Scalar,
}

/// The single map key-name policy (R15-F2): both the direct text writer
/// and `ser::to_value` route key names through here so one key value
/// always produces byte-identical name text on both paths.
pub fn serialize_key_name<T: ?Sized + Serialize>(key: &T) -> Result<Scalar> {
    let mut buf = Scalar::default();
    key.serialize(KeyOnlySer { out: &mut buf })?;
    Ok(buf)
}

impl<'a> ser::Serializer for KeyOnlySer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = UnreachableCompound;
    type SerializeTuple = UnreachableCompound;
    type SerializeTupleStruct = UnreachableCompound;
    type SerializeTupleVariant = UnreachableCompound;
    type SerializeMap = UnreachableCompound;
    type SerializeStruct = UnreachableCompound;
    type SerializeStructVariant = UnreachableCompound;

    fn serialize_str(self, v: &str) -> Result<()> {
        self.out.push_str(v);
        Ok(())
    }

    fn collect_str<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + fmt::Display,
    {
        write!(self.out, "{value}").map_err(|_| Error::Message("fmt error".into()))
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.out.push_str(if v { "true" } else { "false" });
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        write!(self.out, "{v}").map_err(|_| Error::Message("fmt error".into()))
    }
    fn serialize_char(self, v: char) -> Result<()> {
        self.out.push(v);
        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        Err(key_err())
    }
    fn serialize_none(self) -> Result<()> {
        Err(key_err())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        Err(key_err())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(key_err())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.out.push_str(variant);
        Ok(())
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
        Err(key_err())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_err())
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Err(key_err())
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_err())
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_err())
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_err())
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Err(key_err())
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_err())
    }
}

// ---------------------------------------------------------------------------
// PairValueSer — the "value half" of a pair (after the key has been written).
// Writes `: value\n`, `:: value\n`, `: {...}\n`, `: [...]\n`, or the
// multi-line-string form.
// ---------------------------------------------------------------------------

pub(super) struct PairValueSer<'a> {
    pub(super) out: &'a mut String,
    pub(super) indent: usize, // indent of the KEY line
}

impl<'a> PairValueSer<'a> {
    fn write_scalar_line(self, v: &str) -> Result<()> {
        if v.contains('\r') {
            return Err(crate::render::helpers::cr_error());
        }
        if crate::render::helpers::string_needs_multiline(v) {
            // Mirror render/pair.rs choice: prefer indented stripped form
            // for readability; fall back to verbatim when stripped can't
            // round-trip the content. The shared chooser errors when
            // neither form can hold the body (§ 5.6.1).
            let form = crate::render::helpers::choose_multiline_form(v, true)?;

            if matches!(form, crate::render::helpers::MultilineForm::Stripped) {
                // Stripped form (default). content_indent = key_indent + 1.
                self.out.push_str(": (\n");
                let content_indent = self.indent + 1;
                for line in v.split('\n') {
                    if !line.is_empty() {
                        write_indent(self.out, content_indent);
                        self.out.push_str(line);
                    }
                    self.out.push('\n');
                }
                write_indent(self.out, self.indent);
                self.out.push_str(")\n");
            } else {
                // Verbatim form (fallback). One `\n` after content covers
                // both cases: no-trailing-\n in v → separator before `))`;
                // trailing \n in v → blank-line marker that survives the
                // round-trip.
                self.out.push_str(": ((\n");
                self.out.push_str(v);
                self.out.push('\n');
                write_indent(self.out, self.indent);
                self.out.push_str("))\n");
            }
        } else if needs_raw_marker(v) {
            self.out.push_str(":: ");
            self.out.push_str(v);
            self.out.push('\n');
        } else {
            self.out.push_str(": ");
            self.out.push_str(v);
            self.out.push('\n');
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for PairValueSer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqCompound<'a>;
    type SerializeTuple = SeqCompound<'a>;
    type SerializeTupleStruct = SeqCompound<'a>;
    type SerializeTupleVariant = TupleVariantPair<'a>;
    type SerializeMap = ObjectCompound<'a>;
    type SerializeStruct = ObjectCompound<'a>;
    type SerializeStructVariant = StructVariantPair<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.out.push_str(": ");
        self.out.push_str(if v { "true" } else { "false" });
        self.out.push('\n');
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        push_int_pair(self.out, v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        push_f32_pair(self.out, v)
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        push_f64_pair(self.out, v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let s = v.to_string();
        self.write_scalar_line(&s)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_scalar_line(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.out.push_str(": ");
        if v.is_empty() {
            self.out.push_str("[]\n");
        } else {
            self.out.push_str("[\n");
            for &b in v {
                write_indent(self.out, self.indent + 1);
                push_int_item(self.out, b);
            }
            write_indent(self.out, self.indent);
            self.out.push_str("]\n");
        }
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.out.push_str(": null\n");
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.out.push_str(": null\n");
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.out.push_str(": null\n");
        Ok(())
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.write_scalar_line(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, v: &T) -> Result<()> {
        v.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        // Externally-tagged: `: {\n    VariantName: value\n}\n`
        self.out.push_str(": {\n");
        write_indent(self.out, self.indent + 1);
        // Variant names are always emitted inside a `: {` wrapper, so
        // they can never land at byte offset 0 — the § 5.9.10 rule (c)
        // guard never applies here.
        crate::render::helpers::push_escaped_key_segment(variant, false, self.out);
        value.serialize(PairValueSer {
            out: self.out,
            indent: self.indent + 1,
        })?;
        write_indent(self.out, self.indent);
        self.out.push_str("}\n");
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqCompound<'a>> {
        if let Some(0) = len {
            self.out.push_str(": []\n");
            return Ok(SeqCompound::closed(self.out));
        }
        self.out.push_str(": [\n");
        Ok(SeqCompound::wrapped(self.out, self.indent + 1, self.indent))
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqCompound<'a>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<SeqCompound<'a>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<TupleVariantPair<'a>> {
        self.out.push_str(": {\n");
        write_indent(self.out, self.indent + 1);
        // Variant names are always emitted inside a `: {` wrapper, so
        // they can never land at byte offset 0 — the § 5.9.10 rule (c)
        // guard never applies here.
        crate::render::helpers::push_escaped_key_segment(variant, false, self.out);
        self.out.push_str(": [\n");
        Ok(TupleVariantPair {
            out: self.out,
            outer_indent: self.indent,
            variant_indent: self.indent + 1,
            item_indent: self.indent + 2,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<ObjectCompound<'a>> {
        if let Some(0) = len {
            self.out.push_str(": {}\n");
            return Ok(ObjectCompound::closed(self.out));
        }
        self.out.push_str(": {\n");
        Ok(ObjectCompound::wrapped(
            self.out,
            self.indent + 1,
            self.indent,
        ))
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<ObjectCompound<'a>> {
        if len == 0 {
            self.out.push_str(": {}\n");
            return Ok(ObjectCompound::closed(self.out));
        }
        self.out.push_str(": {\n");
        Ok(ObjectCompound::wrapped(
            self.out,
            self.indent + 1,
            self.indent,
        ))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<StructVariantPair<'a>> {
        self.out.push_str(": {\n");
        write_indent(self.out, self.indent + 1);
        // Variant names are always emitted inside a `: {` wrapper, so
        // they can never land at byte offset 0 — the § 5.9.10 rule (c)
        // guard never applies here.
        crate::render::helpers::push_escaped_key_segment(variant, false, self.out);
        self.out.push_str(": {\n");
        Ok(StructVariantPair {
            out: self.out,
            outer_indent: self.indent,
            variant_indent: self.indent + 1,
            field_indent: self.indent + 2,
        })
    }
}

// "Already closed" helper for empty compounds.
impl<'a> ObjectCompound<'a> {
    pub(super) fn closed(out: &'a mut String) -> Self {
        Self {
            out,
            field_indent: 0,
            close: None,
            pending_key: None,
            empty_so_far: true,
            is_root: false,
        }
    }
}
