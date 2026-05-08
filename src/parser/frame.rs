//! A single nesting level on the parser's stack: either an Object being
//! filled in, or an Array being filled in.

use rustc_hash::FxBuildHasher;

use crate::error::Span;
use crate::value::{ObjectMap, Value};

pub(super) enum Frame<'a> {
    Object {
        pairs: ObjectMap,
        /// Set when the last pair opened a compound value whose body is
        /// being filled in a pushed child frame. Cleared when the child
        /// closes and its value is inserted here.
        pending_key: Option<&'a str>,
        /// Span of the key that opened the pending compound — points at
        /// the key text (e.g. `value`), not the closing `}`. Used for
        /// diagnostic ranges so a duplicate-key / conflict error
        /// highlights the offending key, not its trailing brace.
        pending_key_span: Option<Span>,
    },
    Array {
        items: Vec<Value>,
    },
}

impl<'a> Frame<'a> {
    pub(super) fn new_object() -> Self {
        // Most Ktav objects have a handful of entries; preallocating avoids
        // the first one or two rehashes. Empirically `8` covers the typical
        // 5-7-field config row without growing, and the overhead vs `4` for
        // tiny objects is negligible (one extra word group of unused slots).
        Frame::Object {
            pairs: ObjectMap::with_capacity_and_hasher(8, FxBuildHasher),
            pending_key: None,
            pending_key_span: None,
        }
    }

    pub(super) fn new_array() -> Self {
        Frame::Array {
            items: Vec::with_capacity(8),
        }
    }

    pub(super) fn into_value(self) -> Value {
        match self {
            Frame::Object { pairs, .. } => Value::Object(pairs),
            Frame::Array { items } => Value::Array(items),
        }
    }
}
