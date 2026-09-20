use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};

use crate::error::{Error, Result};

use super::items::ItemValueSer;
use super::object::PairValueSer;
use super::shared::write_indent;

// ---------------------------------------------------------------------------
// TupleVariantPair / TupleVariantItem — Enum::Variant(T1, T2, ...)
// wrapping form: `{\n<vi>Variant: [\n<ii>...\n<vi>]\n<oi>}\n`
// "Pair" variant: caller has already written the key; we include the `: `
// prefix. "Item" variant: caller wrote indent; we need to emit without `: `.
// ---------------------------------------------------------------------------

pub(super) struct TupleVariantPair<'a> {
    pub(super) out: &'a mut String,
    pub(super) outer_indent: usize,
    pub(super) variant_indent: usize,
    pub(super) item_indent: usize,
}

impl<'a> SerializeTupleVariant for TupleVariantPair<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
            is_root_array_first: false,
        })
    }
    fn end(self) -> Result<()> {
        write_indent(self.out, self.variant_indent);
        self.out.push_str("]\n");
        write_indent(self.out, self.outer_indent);
        self.out.push_str("}\n");
        Ok(())
    }
}

pub(super) struct TupleVariantItem<'a> {
    pub(super) out: &'a mut String,
    pub(super) outer_indent: usize,
    pub(super) variant_indent: usize,
    pub(super) item_indent: usize,
}

impl<'a> SerializeTupleVariant for TupleVariantItem<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
            is_root_array_first: false,
        })
    }
    fn end(self) -> Result<()> {
        write_indent(self.out, self.variant_indent);
        self.out.push_str("]\n");
        write_indent(self.out, self.outer_indent);
        self.out.push_str("}\n");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// StructVariantPair / StructVariantItem — Enum::Variant { fields... }
// wrapping form: `{\n<vi>Variant: {\n<fi>k: v\n<vi>}\n<oi>}\n`
// ---------------------------------------------------------------------------

pub(super) struct StructVariantPair<'a> {
    pub(super) out: &'a mut String,
    pub(super) outer_indent: usize,
    pub(super) variant_indent: usize,
    pub(super) field_indent: usize,
}

impl<'a> SerializeStructVariant for StructVariantPair<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> Result<()> {
        write_indent(self.out, self.field_indent);
        // Variant names are always emitted inside a `: {` wrapper — the
        // § 5.9.10 rule (c) guard never applies here.
        crate::render::helpers::push_escaped_key_segment(name, false, self.out);
        value.serialize(PairValueSer {
            out: self.out,
            indent: self.field_indent,
        })
    }
    fn end(self) -> Result<()> {
        write_indent(self.out, self.variant_indent);
        self.out.push_str("}\n");
        write_indent(self.out, self.outer_indent);
        self.out.push_str("}\n");
        Ok(())
    }
}

pub(super) struct StructVariantItem<'a> {
    pub(super) out: &'a mut String,
    pub(super) outer_indent: usize,
    pub(super) variant_indent: usize,
    pub(super) field_indent: usize,
}

impl<'a> SerializeStructVariant for StructVariantItem<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> Result<()> {
        write_indent(self.out, self.field_indent);
        // Variant names are always emitted inside a `: {` wrapper — the
        // § 5.9.10 rule (c) guard never applies here.
        crate::render::helpers::push_escaped_key_segment(name, false, self.out);
        value.serialize(PairValueSer {
            out: self.out,
            indent: self.field_indent,
        })
    }
    fn end(self) -> Result<()> {
        write_indent(self.out, self.variant_indent);
        self.out.push_str("}\n");
        write_indent(self.out, self.outer_indent);
        self.out.push_str("}\n");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// UnreachableCompound — satisfies the associated type requirements for
// paths that are disallowed (e.g. struct variants at top level, map keys). The actual
// method calls all return errors before reaching here.
// ---------------------------------------------------------------------------

pub(super) struct UnreachableCompound;

impl SerializeSeq for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeTuple for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeTupleStruct for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeTupleVariant for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeMap for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeStruct for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &'static str, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
impl SerializeStructVariant for UnreachableCompound {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &'static str, _: &T) -> Result<()> {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}
