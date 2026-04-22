//! A single nesting level on the parser's stack: either an Object being
//! filled in, or an Array being filled in.

use rustc_hash::FxBuildHasher;

use crate::value::{ObjectMap, Value};

pub(super) enum Frame<'a> {
    Object {
        pairs: ObjectMap,
        /// Set when the last pair opened a compound value whose body is
        /// being filled in a pushed child frame. Cleared when the child
        /// closes and its value is inserted here.
        pending_key: Option<&'a str>,
    },
    Array {
        items: Vec<Value>,
    },
}

impl<'a> Frame<'a> {
    pub(super) fn new_object() -> Self {
        // Most Ktav objects have a handful of entries; preallocating avoids
        // the first one or two rehashes.
        Frame::Object {
            pairs: ObjectMap::with_capacity_and_hasher(4, FxBuildHasher),
            pending_key: None,
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
