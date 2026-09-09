//! Direct text serializer: serializes `T: Serialize` straight to a Ktav
//! text string, skipping the `Value` intermediate that `ser::to_value` +
//! `render::render` go through. Produces byte-identical output to the
//! Value-based path, with one divergence: for scientific-notation Floats
//! this direct path emits a text literal with a `.0` mantissa (`1.0e100`),
//! while the Value path stores/renders the parser-normalized payload
//! (`1e100`); `emit_canonical` normalises both.

use std::cell::Cell;
use std::fmt::Write as _;
use std::rc::Rc;

use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::error::{Error, Result};
use crate::value::Scalar;

const INDENT: &str = "    ";

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
    crate::render::helpers::needs_raw_marker(s)
}

fn top_err() -> Error {
    <Error as ser::Error>::custom("top-level value must be an object")
}

/// A bare scalar root is exactly the § 5.9.0 `ScalarRoot` case.
fn scalar_root_err() -> Error {
    Error::Unrepresentable(crate::error::ReasonCode::ScalarRoot)
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

/// Fast path for pair-position integer emission: `: <digits>\n`.
/// Under spec 0.5.0, integers use plain `:` (no `:i` marker).
/// `itoa` avoids `fmt::Formatter` overhead.
fn push_int_pair<I: itoa::Integer>(out: &mut String, v: I) {
    out.push_str(": ");
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Same, but for array-item position: bare `<digits>\n`.
/// Under spec 0.5.0, integer items are inferred from the lexical form.
fn push_int_item<I: itoa::Integer>(out: &mut String, v: I) {
    let mut buf = itoa::Buffer::new();
    out.push_str(buf.format(v));
    out.push('\n');
}

/// Fast path for pair-position float emission via `ryu`.
/// Under spec 0.5.0, floats use plain `: ` (no `:f` marker).
fn push_f64_pair(out: &mut String, v: f64) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    out.push_str(": ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

fn push_f32_pair(out: &mut String, v: f32) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    out.push_str(": ");
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

/// Bare float item emission (no pair prefix). Under spec 0.5.0,
/// float items are inferred from the lexical form.
fn push_f64_item(out: &mut String, v: f64) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

fn push_f32_item(out: &mut String, v: f32) -> Result<()> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::Unrepresentable(
            crate::error::ReasonCode::NonFiniteFloat,
        ));
    }
    let mut buf = ryu::Buffer::new();
    push_float_body(out, buf.format(v));
    out.push('\n');
    Ok(())
}

// ---------------------------------------------------------------------------
// RootSer — top-level document. Accepts Object (struct / map) and Array (seq / tuple / tuple struct) roots; root enum tuple variants serialize as a single-pair Object.
// ---------------------------------------------------------------------------

struct RootSer<'a> {
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
    pending_key: Option<Scalar>, // used by SerializeMap
    empty_so_far: bool,
    /// True only for the document-root Object: its FIRST serialized
    /// key is the only one that can land at byte offset 0, so it is
    /// the only one eligible for the § 5.9.10 rule (c) U+FEFF guard.
    is_root: bool,
}

impl<'a> ObjectCompound<'a> {
    fn root(out: &'a mut String) -> Self {
        Self {
            out,
            field_indent: 0,
            close: None,
            pending_key: None,
            empty_so_far: true,
            is_root: true,
        }
    }

    fn wrapped(out: &'a mut String, field_indent: usize, close_indent: usize) -> Self {
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
    out: &'a mut String,
}

/// The single map key-name policy (R15-F2): both the direct text writer
/// and `ser::to_value` route key names through here so one key value
/// always produces byte-identical name text on both paths.
pub(super) fn serialize_key_name<T: ?Sized + Serialize>(key: &T) -> Result<Scalar> {
    let mut buf = String::new();
    key.serialize(KeyOnlySer { out: &mut buf })?;
    Ok(buf.into())
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

struct PairValueSer<'a> {
    out: &'a mut String,
    indent: usize, // indent of the KEY line
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
    fn closed(out: &'a mut String) -> Self {
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

// ---------------------------------------------------------------------------
// ItemValueSer — one item inside an array. Writes `<indent>value\n`,
// `<indent>:: value\n`, `<indent>{...}\n`, `<indent>[...]\n`, or multi-line.
// ---------------------------------------------------------------------------

struct ItemValueSer<'a> {
    out: &'a mut String,
    indent: usize,
    /// TRUE only for index 0 of an UNWRAPPED root Array (§ 5.9.6 /
    /// § 5.9.12): the one item position exposed to § 5.0.1 root-kind
    /// detection, where a one-line String body satisfying the
    /// pair-candidate test (or beginning with U+FEFF) must take the
    /// `:: ` raw marker. Mirrors render/array_item.rs.
    is_root_array_first: bool,
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

struct RootFirstItemSer<'a> {
    out: &'a mut String,
    wrap: Rc<Cell<bool>>,
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

// ---------------------------------------------------------------------------
// SeqCompound — SerializeSeq / Tuple / TupleStruct.
// ---------------------------------------------------------------------------

struct SeqCompound<'a> {
    out: &'a mut String,
    item_indent: usize,
    close: Option<usize>, // Some(outer_indent) → write `<outer>]\n`; None → already closed inline.
    /// True only for the document-root Array (§ 5.9.3): the first
    /// element decides the wrapped/unwrapped form, and `end` emits
    /// `[]\n` if no element ever wrote anything (length-`None` seq
    /// that yielded nothing).
    is_root_array: bool,
    /// Root mode only: the § 5.9.3 wrap decision for the first
    /// element, shared with its `RootFirstItemSer`. `None` once the
    /// decision is made.
    root_first: Option<Rc<Cell<bool>>>,
}

impl<'a> SeqCompound<'a> {
    /// Document-root Array (§ 5.9.3): items start bare at indent 0
    /// with no closing `]`; the first element's serializer sets the
    /// shared `wrap` cell if the lone-`[` wrap is required.
    fn root(out: &'a mut String) -> Self {
        Self {
            out,
            item_indent: 0,
            close: None,
            is_root_array: true,
            root_first: Some(Rc::new(Cell::new(false))),
        }
    }

    fn wrapped(out: &'a mut String, item_indent: usize, close_indent: usize) -> Self {
        Self {
            out,
            item_indent,
            close: Some(close_indent),
            is_root_array: false,
            root_first: None,
        }
    }
    fn closed(out: &'a mut String) -> Self {
        Self {
            out,
            item_indent: 0,
            close: None,
            is_root_array: false,
            root_first: None,
        }
    }
}

impl<'a> SerializeSeq for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        if let Some(wrap) = self.root_first.take() {
            // First element of a root Array: it alone is exposed to
            // § 5.0.1 root-kind detection (§ 5.9.6 / § 5.9.12), and if
            // it is a compound the whole root Array takes the § 5.9.3
            // lone-`[`/`{` wrap.
            v.serialize(RootFirstItemSer {
                out: self.out,
                wrap: wrap.clone(),
            })?;
            if wrap.get() {
                self.item_indent = 1;
                self.close = Some(0);
            }
            return Ok(());
        }
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
            is_root_array_first: false,
        })
    }
    fn end(self) -> Result<()> {
        if let Some(outer) = self.close {
            write_indent(self.out, outer);
            self.out.push_str("]\n");
        } else if self.is_root_array && self.out.is_empty() {
            // Root Array with a `None` length hint that yielded no
            // elements: § 5.9.3 empty Array root. (Root position means
            // `out` is the whole document; any element would have
            // written at least one byte.)
            self.out.push_str("[]\n");
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

impl<'a> SerializeTupleVariant for SeqCompound<'a> {
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
