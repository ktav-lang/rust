//! The dynamic `Value` enum produced by the parser and consumed by the
//! serializer.

use compact_str::CompactString;

use super::object_map::ObjectMap;

/// Scalar-text storage. `CompactString` inlines strings up to 24 bytes
/// on 64-bit (no heap allocation for typical configuration scalars — ports,
/// booleans-as-string, identifiers), and transparently falls back to a
/// heap allocation for longer content. Behaves like `String` through
/// `Deref<Target=str>`, so `.as_str()`, `AsRef<str>`, `&*s`, etc. all work.
pub type Scalar = CompactString;

/// The dynamic representation a parsed document is decoded into.
///
/// Most callers go through [`crate::from_str`] / [`crate::to_string`] and
/// never touch `Value` directly. It is exposed for advanced cases: custom
/// serializers, inspection, or building documents programmatically.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The `null` keyword. Also produced by `Option::None` during
    /// serialization. Rendered as the literal `null`.
    Null,
    /// The `true` / `false` keywords.
    Bool(bool),
    /// A typed integer scalar — emitted/parsed with the `:i` marker. Held
    /// as a text form so arbitrary precision (digits beyond any fixed-width
    /// Rust integer) round-trips byte-for-byte. Leading `+` is stripped at
    /// parse time; leading `-` is preserved.
    Integer(Scalar),
    /// A typed floating-point scalar — emitted/parsed with the `:f` marker.
    /// Held as the textual form (mantissa with a decimal point, optional
    /// scientific exponent) so precision round-trips exactly. Leading `+`
    /// on the mantissa is stripped at parse time; signs in the exponent
    /// are preserved verbatim.
    Float(Scalar),
    /// A scalar string leaf. Numbers written without a type marker are
    /// held here too; conversion to `u16` / `f64` / … happens through
    /// serde on deserialization.
    String(Scalar),
    /// A multi-line `[ ... ]` array. Items may be any variant.
    Array(Vec<Value>),
    /// A multi-line `{ ... }` object (also the top-level document).
    Object(ObjectMap),
}

impl Value {
    /// Returns `true` if this is [`Value::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns the inner `bool` if this is [`Value::Bool`], else `None`.
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Returns the inner `&str` if this is [`Value::String`], else `None`.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the digit form of a `Value::Integer` (leading `-` preserved,
    /// leading `+` already stripped). `None` for any other variant — notably
    /// a numeric `Value::String` is NOT promoted.
    pub fn as_integer(&self) -> Option<&str> {
        if let Value::Integer(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the text form of a `Value::Float` (mantissa with decimal
    /// point, optional `e±N` exponent). `None` for any other variant.
    pub fn as_float(&self) -> Option<&str> {
        if let Value::Float(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the inner slice reference if this is [`Value::Array`], else `None`.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Returns the inner map reference if this is [`Value::Object`], else `None`.
    pub fn as_object(&self) -> Option<&ObjectMap> {
        if let Value::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }
}
