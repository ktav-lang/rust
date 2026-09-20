use std::cell::Cell;
use std::rc::Rc;

use serde::ser::{self, Serialize};

use crate::error::{Error, Result};

use super::object::{ObjectCompound, PairValueSer};
use super::seq::SeqCompound;
use super::shared::{push_f32_item, push_f64_item, push_int_item, write_indent};
use super::variants::{StructVariantItem, TupleVariantItem};

// ---------------------------------------------------------------------------
// ItemValueSer — one item inside an array. Writes `<indent>value\n`,
// `<indent>:: value\n`, `<indent>{...}\n`, `<indent>[...]\n`, or multi-line.
// ---------------------------------------------------------------------------

pub(super) struct ItemValueSer<'a> {
    pub(super) out: &'a mut String,
    pub(super) indent: usize,
    /// TRUE only for index 0 of an UNWRAPPED root Array (§ 5.9.6 /
    /// § 5.9.12): the one item position exposed to § 5.0.1 root-kind
    /// detection, where a one-line String body satisfying the
    /// pair-candidate test (or beginning with U+FEFF) must take the
    /// `:: ` raw marker. Mirrors render/array_item.rs.
    pub(super) is_root_array_first: bool,
}

impl<'a> ItemValueSer<'a> {
    fn write_scalar_line(self, v: &str) -> Result<()> {
        if v.contains('\r') {
            return Err(crate::render::helpers::cr_error());
        }
        write_indent(self.out, self.indent);
        if crate::render::helpers::string_needs_multiline(v) {
            // Mirror render/array_item.rs: prefer indented stripped form,
            // fall back to verbatim when stripped can't round-trip the
            // content; the shared chooser errors when neither form can
            // hold the body (§ 5.6.1).
            let form = crate::render::helpers::choose_multiline_form(v, true)?;

            if matches!(form, crate::render::helpers::MultilineForm::Stripped) {
                self.out.push_str("(\n");
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
                self.out.push_str("((\n");
                self.out.push_str(v);
                self.out.push('\n');
                write_indent(self.out, self.indent);
                self.out.push_str("))\n");
            }
        } else if v.is_empty() {
            // An empty-string item would otherwise render as a bare
            // indented blank line, which the parser treats as decorative
            // and drops. Force `::` so it stays a literal-string entry.
            self.out.push_str("::\n");
        } else if crate::render::helpers::item_needs_raw_marker(v)
            || (self.is_root_array_first
                && (crate::render::helpers::bare_item_is_pair_candidate(v)
                    || v.starts_with('\u{FEFF}')))
        {
            self.out.push_str(":: ");
            self.out.push_str(v);
            self.out.push('\n');
        } else {
            self.out.push_str(v);
            self.out.push('\n');
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for ItemValueSer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqCompound<'a>;
    type SerializeTuple = SeqCompound<'a>;
    type SerializeTupleStruct = SeqCompound<'a>;
    type SerializeTupleVariant = TupleVariantItem<'a>;
    type SerializeMap = ObjectCompound<'a>;
    type SerializeStruct = ObjectCompound<'a>;
    type SerializeStructVariant = StructVariantItem<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        write_indent(self.out, self.indent);
        self.out.push_str(if v { "true" } else { "false" });
        self.out.push('\n');
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        write_indent(self.out, self.indent);
        push_int_item(self.out, v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        write_indent(self.out, self.indent);
        push_f32_item(self.out, v)
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        write_indent(self.out, self.indent);
        push_f64_item(self.out, v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let s = v.to_string();
        self.write_scalar_line(&s)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_scalar_line(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        write_indent(self.out, self.indent);
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
        write_indent(self.out, self.indent);
        self.out.push_str("null\n");
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        write_indent(self.out, self.indent);
        self.out.push_str("null\n");
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
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
        write_indent(self.out, self.indent);
        self.out.push_str("{\n");
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
        write_indent(self.out, self.indent);
        if let Some(0) = len {
            self.out.push_str("[]\n");
            return Ok(SeqCompound::closed(self.out));
        }
        self.out.push_str("[\n");
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
    ) -> Result<TupleVariantItem<'a>> {
        write_indent(self.out, self.indent);
        self.out.push_str("{\n");
        write_indent(self.out, self.indent + 1);
        // Variant names are always emitted inside a `: {` wrapper, so
        // they can never land at byte offset 0 — the § 5.9.10 rule (c)
        // guard never applies here.
        crate::render::helpers::push_escaped_key_segment(variant, false, self.out);
        self.out.push_str(": [\n");
        Ok(TupleVariantItem {
            out: self.out,
            outer_indent: self.indent,
            variant_indent: self.indent + 1,
            item_indent: self.indent + 2,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<ObjectCompound<'a>> {
        write_indent(self.out, self.indent);
        if let Some(0) = len {
            self.out.push_str("{}\n");
            return Ok(ObjectCompound::closed(self.out));
        }
        self.out.push_str("{\n");
        Ok(ObjectCompound::wrapped(
            self.out,
            self.indent + 1,
            self.indent,
        ))
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<ObjectCompound<'a>> {
        write_indent(self.out, self.indent);
        if len == 0 {
            self.out.push_str("{}\n");
            return Ok(ObjectCompound::closed(self.out));
        }
        self.out.push_str("{\n");
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
    ) -> Result<StructVariantItem<'a>> {
        write_indent(self.out, self.indent);
        self.out.push_str("{\n");
        write_indent(self.out, self.indent + 1);
        // Variant names are always emitted inside a `: {` wrapper, so
        // they can never land at byte offset 0 — the § 5.9.10 rule (c)
        // guard never applies here.
        crate::render::helpers::push_escaped_key_segment(variant, false, self.out);
        self.out.push_str(": {\n");
        Ok(StructVariantItem {
            out: self.out,
            outer_indent: self.indent,
            variant_indent: self.indent + 1,
            field_indent: self.indent + 2,
        })
    }
}

// ---------------------------------------------------------------------------
// RootFirstItemSer — FIRST element of an unwrapped root Array (§ 5.9.3).
// Scalars render like ItemValueSer at indent 0 but keep the § 5.9.6 /
// § 5.9.12 first-item safeguards (`is_root_array_first = true`). If the
// element is a compound — or bytes, which the owned path stores as an
// Array — the root Array takes the lone-`[` wrap: `[\n` is emitted (the
// document is still empty here, so it lands at offset 0) and the
// element renders as a nested item at indent 1.
// ---------------------------------------------------------------------------

pub(super) struct RootFirstItemSer<'a> {
    pub(super) out: &'a mut String,
    pub(super) wrap: Rc<Cell<bool>>,
}

impl<'a> RootFirstItemSer<'a> {
    /// This element as a bare root item at indent 0, exposed to the
    /// § 5.0.1 root-kind detection safeguards.
    fn scalar_item(self) -> ItemValueSer<'a> {
        ItemValueSer {
            out: self.out,
            indent: 0,
            is_root_array_first: true,
        }
    }

    /// Switch the root Array to the § 5.9.3 wrapped form and render
    /// this element as a nested item at indent 1 (never root-detected,
    /// § 5.9.6 — the wrap's `[` is the root's first line instead).
    fn wrapped_item(self) -> ItemValueSer<'a> {
        self.wrap.set(true);
        self.out.push_str("[\n");
        ItemValueSer {
            out: self.out,
            indent: 1,
            is_root_array_first: false,
        }
    }
}

impl<'a> ser::Serializer for RootFirstItemSer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqCompound<'a>;
    type SerializeTuple = SeqCompound<'a>;
    type SerializeTupleStruct = SeqCompound<'a>;
    type SerializeTupleVariant = TupleVariantItem<'a>;
    type SerializeMap = ObjectCompound<'a>;
    type SerializeStruct = ObjectCompound<'a>;
    type SerializeStructVariant = StructVariantItem<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.scalar_item().serialize_bool(v)
    }
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.scalar_item().serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.scalar_item().serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.scalar_item().serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.scalar_item().serialize_i64(v)
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        self.scalar_item().serialize_i128(v)
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.scalar_item().serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.scalar_item().serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.scalar_item().serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.scalar_item().serialize_u64(v)
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        self.scalar_item().serialize_u128(v)
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.scalar_item().serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.scalar_item().serialize_f64(v)
    }
    fn serialize_char(self, v: char) -> Result<()> {
        self.scalar_item().serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<()> {
        self.scalar_item().serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.wrapped_item().serialize_bytes(v)
    }
    fn serialize_none(self) -> Result<()> {
        self.scalar_item().serialize_none()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        self.scalar_item().serialize_unit()
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.scalar_item().serialize_unit()
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.scalar_item().write_scalar_line(variant)
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
        // Owned path: Value::Object({variant: value}) — a compound.
        self.wrapped_item()
            .serialize_newtype_variant("", 0, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqCompound<'a>> {
        self.wrapped_item().serialize_seq(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqCompound<'a>> {
        self.wrapped_item().serialize_tuple(len)
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<SeqCompound<'a>> {
        self.wrapped_item().serialize_tuple_struct("", len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantItem<'a>> {
        self.wrapped_item()
            .serialize_tuple_variant("", 0, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<ObjectCompound<'a>> {
        self.wrapped_item().serialize_map(len)
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<ObjectCompound<'a>> {
        self.wrapped_item().serialize_struct("", len)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructVariantItem<'a>> {
        self.wrapped_item()
            .serialize_struct_variant("", 0, variant, len)
    }
}
