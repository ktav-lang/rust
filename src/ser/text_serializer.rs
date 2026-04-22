//! Direct text serializer: serializes `T: Serialize` straight to a Ktav
//! text string, skipping the `Value` intermediate that `ser::to_value` +
//! `render::render` go through. Produces byte-identical output to the
//! Value-based path.

use std::fmt::Write as _;

use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::error::{Error, Result};

const INDENT: &str = "    ";

/// Render `value` directly to a Ktav string. Top-level value MUST be an
/// object (struct or map); anything else returns an error.
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
// Helpers
// ---------------------------------------------------------------------------

fn write_indent(out: &mut String, level: usize) {
    // Push `level * INDENT.len()` spaces via slice copies of a static string.
    // Using a precomputed all-spaces slice lets the hot path be a single
    // `push_str` (vectorised memcpy) instead of a byte-by-byte loop —
    // the equivalent of the old `unsafe { as_mut_vec }.extend(repeat)`,
    // but with no unsafe and usually faster, since memcpy is SIMD-friendly.
    const SPACES: &str = "                                                                "; // 64
    let mut remaining = level * INDENT.len();
    if remaining == 0 {
        return;
    }
    out.reserve(remaining);
    while remaining > 0 {
        let chunk = remaining.min(SPACES.len());
        out.push_str(&SPACES[..chunk]);
        remaining -= chunk;
    }
}

fn needs_raw_marker(s: &str) -> bool {
    // Fast path: most scalars don't start with whitespace, so we can check
    // the first byte directly and skip `trim_start`'s whole-string scan.
    match s.as_bytes().first() {
        None => false,
        Some(&b' ') | Some(&b'\t') => needs_raw_marker_slow(s.trim_start()),
        Some(&b'{') | Some(&b'[') => true,
        Some(_) => {
            matches!(s, "null" | "true" | "false" | "(" | "((" | "()" | "(())")
        }
    }
}

#[cold]
#[inline(never)]
fn needs_raw_marker_slow(t: &str) -> bool {
    t.starts_with('{')
        || t.starts_with('[')
        || matches!(t, "null" | "true" | "false" | "(" | "((" | "()" | "(())")
}

fn top_err() -> Error {
    <Error as ser::Error>::custom("top-level value must be an object")
}

fn key_err() -> Error {
    <Error as ser::Error>::custom("map keys must serialize to strings")
}

/// Append `s` (a ryu-formatted float) to `out`, ensuring a decimal point
/// is present in the mantissa — Ktav's Float grammar requires `N.N` at a
/// minimum. `ryu` emits `1.0` for `1.0f64` (good) but `1e100` without a
/// decimal point for very large values; insert `.0` before the exponent
/// in that case so the parser accepts the literal.
fn push_float_body(out: &mut String, s: &str) {
    // Single pass over the mantissa bytes: ryu emits either `N.N`,
    // `N.Ne±E`, or `NeE` (no dot, exponent only) — the exponent always
    // comes after any dot, so the first `e`/`E` ends the scan.
    let bytes = s.as_bytes();
    let mut e_pos: Option<usize> = None;
    let mut has_dot = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'.' {
            has_dot = true;
        } else if b == b'e' || b == b'E' {
            e_pos = Some(i);
            break;
        }
    }
    match (e_pos, has_dot) {
        (_, true) => out.push_str(s),
        (Some(pos), false) => {
            out.push_str(&s[..pos]);
            out.push_str(".0");
            out.push_str(&s[pos..]);
        }
        (None, false) => {
            out.push_str(s);
            out.push_str(".0");
        }
    }
}

/// Produce the textual form of an `f64` value suitable for the `:f`
/// marker: always contains a decimal point. NaN / ±Infinity return an
/// error — Ktav 0.1.0 does not represent them.
pub(crate) fn format_f64(v: f64) -> Result<String> {
    if v.is_nan() || v.is_infinite() {
        return Err(<Error as ser::Error>::custom(
            "NaN / Infinity is not representable in Ktav 0.1.0",
        ));
    }
    let mut buf = ryu::Buffer::new();
    let mut out = String::with_capacity(24);
    push_float_body(&mut out, buf.format(v));
    Ok(out)
}

/// Produce the textual form of an `f32` value suitable for the `:f` marker.
pub(crate) fn format_f32(v: f32) -> Result<String> {
    if v.is_nan() || v.is_infinite() {
        return Err(<Error as ser::Error>::custom(
            "NaN / Infinity is not representable in Ktav 0.1.0",
        ));
    }
    let mut buf = ryu::Buffer::new();
    let mut out = String::with_capacity(16);
    push_float_body(&mut out, buf.format(v));
    Ok(out)
}

/// Fast path for pair-position integer emission: `:i <digits>\n`.
/// `itoa` avoids `fmt::Formatter` overhead.
fn push_int_pair<I: itoa::Integer>(out: &mut String, v: I) {
    out.push_str(":i ");
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Same, but for array-item position (no leading `: ` since that's only
/// in pair position; this helper is called after the indent prefix is
/// already written).
fn push_int_item<I: itoa::Integer>(out: &mut String, v: I) {
    out.push_str(":i ");
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Fast path for pair-position float emission via `ryu`.
fn push_f64_pair(out: &mut String, v: f64) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(<Error as ser::Error>::custom(
            "NaN / Infinity is not representable in Ktav 0.1.0",
        ));
    }
    out.push_str(":f ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

fn push_f32_pair(out: &mut String, v: f32) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(<Error as ser::Error>::custom(
            "NaN / Infinity is not representable in Ktav 0.1.0",
        ));
    }
    out.push_str(":f ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

// ---------------------------------------------------------------------------
// RootSer — top-level document. Accepts only struct / map.
// ---------------------------------------------------------------------------

struct RootSer<'a> {
    out: &'a mut String,
}

impl<'a> ser::Serializer for RootSer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = UnreachableCompound;
    type SerializeTuple = UnreachableCompound;
    type SerializeTupleStruct = UnreachableCompound;
    type SerializeTupleVariant = UnreachableCompound;
    type SerializeMap = ObjectCompound<'a>;
    type SerializeStruct = ObjectCompound<'a>;
    type SerializeStructVariant = UnreachableCompound;

    fn serialize_bool(self, _: bool) -> Result<()> {
        Err(top_err())
    }
    fn serialize_i8(self, _: i8) -> Result<()> {
        Err(top_err())
    }
    fn serialize_i16(self, _: i16) -> Result<()> {
        Err(top_err())
    }
    fn serialize_i32(self, _: i32) -> Result<()> {
        Err(top_err())
    }
    fn serialize_i64(self, _: i64) -> Result<()> {
        Err(top_err())
    }
    fn serialize_u8(self, _: u8) -> Result<()> {
        Err(top_err())
    }
    fn serialize_u16(self, _: u16) -> Result<()> {
        Err(top_err())
    }
    fn serialize_u32(self, _: u32) -> Result<()> {
        Err(top_err())
    }
    fn serialize_u64(self, _: u64) -> Result<()> {
        Err(top_err())
    }
    fn serialize_f32(self, _: f32) -> Result<()> {
        Err(top_err())
    }
    fn serialize_f64(self, _: f64) -> Result<()> {
        Err(top_err())
    }
    fn serialize_char(self, _: char) -> Result<()> {
        Err(top_err())
    }
    fn serialize_str(self, _: &str) -> Result<()> {
        Err(top_err())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        Err(top_err())
    }
    fn serialize_none(self) -> Result<()> {
        Err(top_err())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        Err(top_err())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(top_err())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        Err(top_err())
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

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(top_err())
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(top_err())
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(top_err())
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(top_err())
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

// ---------------------------------------------------------------------------
// ObjectCompound — SerializeStruct + SerializeMap
//
// Two shapes:
// - Root { indent = 0 }: no surrounding `{}`, no closing.
// - Wrapped { field_indent, close_indent }: caller has already written
//   `: {\n`; we write each field at `field_indent`, at end we write
//   `<close_indent>}\n`.
// ---------------------------------------------------------------------------

struct ObjectCompound<'a> {
    out: &'a mut String,
    field_indent: usize,
    close: Option<usize>, // Some(outer_indent) → write `<outer>}\n` at end.
    pending_key: Option<String>, // used by SerializeMap
    empty_so_far: bool,
}

impl<'a> ObjectCompound<'a> {
    fn root(out: &'a mut String) -> Self {
        Self {
            out,
            field_indent: 0,
            close: None,
            pending_key: None,
            empty_so_far: true,
        }
    }

    fn wrapped(out: &'a mut String, field_indent: usize, close_indent: usize) -> Self {
        Self {
            out,
            field_indent,
            close: Some(close_indent),
            pending_key: None,
            empty_so_far: true,
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
        self.empty_so_far = false;
        write_indent(self.out, self.field_indent);
        self.out.push_str(key);
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
        let mut buf = String::new();
        key.serialize(KeyOnlySer { out: &mut buf })?;
        self.pending_key = Some(buf);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.pending_key.take().ok_or_else(|| {
            <Error as ser::Error>::custom("serialize_value without preceding key")
        })?;
        self.empty_so_far = false;
        write_indent(self.out, self.field_indent);
        self.out.push_str(&key);
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
// KeyOnlySer — serializes a map key into a plain string. Only scalar-like
// serializations are allowed; compounds error.
// ---------------------------------------------------------------------------

struct KeyOnlySer<'a> {
    out: &'a mut String,
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

struct PairValueSer<'a> {
    out: &'a mut String,
    indent: usize, // indent of the KEY line
}

impl<'a> PairValueSer<'a> {
    fn write_scalar_line(self, v: &str) {
        if v.contains('\n') {
            // Multi-line verbatim form. One `\n` after content covers both
            // cases: no-trailing-\n in v → separator before `))`; trailing
            // \n in v → blank-line marker that survives the round-trip.
            self.out.push_str(": ((\n");
            self.out.push_str(v);
            self.out.push('\n');
            write_indent(self.out, self.indent);
            self.out.push_str("))\n");
        } else if needs_raw_marker(v) {
            self.out.push_str(":: ");
            self.out.push_str(v);
            self.out.push('\n');
        } else {
            self.out.push_str(": ");
            self.out.push_str(v);
            self.out.push('\n');
        }
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
        self.write_scalar_line(&s);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_scalar_line(v);
        Ok(())
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
        self.write_scalar_line(variant);
        Ok(())
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
        self.out.push_str(variant);
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
        self.out.push_str(variant);
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
        self.out.push_str(variant);
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
    fn closed(out: &'a mut String) -> Self {
        Self {
            out,
            field_indent: 0,
            close: None,
            pending_key: None,
            empty_so_far: true,
        }
    }
}

// ---------------------------------------------------------------------------
// ItemValueSer — one item inside an array. Writes `<indent>value\n`,
// `<indent>:: value\n`, `<indent>{...}\n`, `<indent>[...]\n`, or multi-line.
// ---------------------------------------------------------------------------

struct ItemValueSer<'a> {
    out: &'a mut String,
    indent: usize,
}

impl<'a> ItemValueSer<'a> {
    fn write_scalar_line(self, v: &str) {
        write_indent(self.out, self.indent);
        if v.contains('\n') {
            self.out.push_str("((\n");
            self.out.push_str(v);
            self.out.push('\n');
            write_indent(self.out, self.indent);
            self.out.push_str("))\n");
        } else if v.is_empty() {
            // An empty-string item would otherwise render as a bare
            // indented blank line, which the parser treats as decorative
            // and drops. Force `::` so it stays a literal-string entry.
            self.out.push_str("::\n");
        } else if needs_raw_marker(v) {
            self.out.push_str(":: ");
            self.out.push_str(v);
            self.out.push('\n');
        } else {
            self.out.push_str(v);
            self.out.push('\n');
        }
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
        push_f32_pair(self.out, v)
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        write_indent(self.out, self.indent);
        push_f64_pair(self.out, v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let s = v.to_string();
        self.write_scalar_line(&s);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_scalar_line(v);
        Ok(())
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
        self.write_scalar_line(variant);
        Ok(())
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
        self.out.push_str(variant);
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
        self.out.push_str(variant);
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
        self.out.push_str(variant);
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
// SeqCompound — SerializeSeq / Tuple / TupleStruct.
// ---------------------------------------------------------------------------

struct SeqCompound<'a> {
    out: &'a mut String,
    item_indent: usize,
    close: Option<usize>, // Some(outer_indent) → write `<outer>]\n`; None → already closed inline.
}

impl<'a> SeqCompound<'a> {
    fn wrapped(out: &'a mut String, item_indent: usize, close_indent: usize) -> Self {
        Self {
            out,
            item_indent,
            close: Some(close_indent),
        }
    }
    fn closed(out: &'a mut String) -> Self {
        Self {
            out,
            item_indent: 0,
            close: None,
        }
    }
}

impl<'a> SerializeSeq for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
        })
    }
    fn end(self) -> Result<()> {
        if let Some(outer) = self.close {
            write_indent(self.out, outer);
            self.out.push_str("]\n");
        }
        Ok(())
    }
}

impl<'a> SerializeTuple for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        <Self as SerializeSeq>::serialize_element(self, v)
    }
    fn end(self) -> Result<()> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a> SerializeTupleStruct for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        <Self as SerializeSeq>::serialize_element(self, v)
    }
    fn end(self) -> Result<()> {
        <Self as SerializeSeq>::end(self)
    }
}

// ---------------------------------------------------------------------------
// TupleVariantPair / TupleVariantItem — Enum::Variant(T1, T2, ...)
// wrapping form: `{\n<vi>Variant: [\n<ii>...\n<vi>]\n<oi>}\n`
// "Pair" variant: caller has already written the key; we include the `: `
// prefix. "Item" variant: caller wrote indent; we need to emit without `: `.
// ---------------------------------------------------------------------------

struct TupleVariantPair<'a> {
    out: &'a mut String,
    outer_indent: usize,
    variant_indent: usize,
    item_indent: usize,
}

impl<'a> SerializeTupleVariant for TupleVariantPair<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
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

struct TupleVariantItem<'a> {
    out: &'a mut String,
    outer_indent: usize,
    variant_indent: usize,
    item_indent: usize,
}

impl<'a> SerializeTupleVariant for TupleVariantItem<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
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

struct StructVariantPair<'a> {
    out: &'a mut String,
    outer_indent: usize,
    variant_indent: usize,
    field_indent: usize,
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
        self.out.push_str(name);
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

struct StructVariantItem<'a> {
    out: &'a mut String,
    outer_indent: usize,
    variant_indent: usize,
    field_indent: usize,
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
        self.out.push_str(name);
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
// paths that are disallowed (e.g. seq at top level, map keys). The actual
// method calls all return errors before reaching here.
// ---------------------------------------------------------------------------

struct UnreachableCompound;

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
