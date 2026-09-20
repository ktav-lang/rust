use std::borrow::Cow;

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::parser::insert::{InsertShape, InsertTable, OccupiedShape};

use super::super::Event;

/// A lightweight inline value staged before emission. Scalars are
/// ready-made leaf events; an array is a flat, complete event block
/// (BeginArray .. EndArray) — arrays impose no ordering or duplicate
/// rules of their own, so source order is emission order and no
/// per-item nodes are staged; an object keeps the insertion-ordered
/// key table that duplicate/conflict detection and dotted-key merge
/// require.
pub(crate) enum Node<'a> {
    /// A scalar leaf (Null / Bool / Integer / Float / Str).
    Leaf(Event<'a>),
    /// Flat event block of an array value, brackets included.
    Array(Vec<Event<'a>>),
    Object(InlineMap<'a>),
}

/// Insertion-ordered table backing one inline object scope. Keys are
/// borrowed from the source when unescaped; escape-decoded keys stay
/// in a `Cow::Owned` until emission copies them into the arena.
pub(crate) type InlineMap<'a> = IndexMap<Cow<'a, str>, Node<'a>, FxBuildHasher>;

impl InsertShape for Node<'_> {
    fn is_object(&self) -> bool {
        matches!(self, Node::Object(_))
    }
    fn kind_label(&self) -> &'static str {
        match self {
            Node::Leaf(Event::Null) => "null",
            Node::Leaf(Event::Bool(_)) => "bool",
            Node::Leaf(Event::Integer(_)) => "integer",
            Node::Leaf(Event::Float(_)) => "float",
            Node::Leaf(Event::Str(_)) => "string",
            Node::Leaf(_) => unreachable!("not a value-start event"),
            Node::Array(_) => "array",
            Node::Object(_) => "object",
        }
    }
}

impl<'a> InsertTable<'a> for InlineMap<'a> {
    type Value = Node<'a>;

    fn insert_leaf(
        &mut self,
        key: Cow<'a, str>,
        value: Node<'a>,
    ) -> Result<(), (OccupiedShape, Node<'a>)> {
        match self.entry(key) {
            indexmap::map::Entry::Occupied(e) => Err((
                OccupiedShape {
                    is_object: e.get().is_object(),
                    label: e.get().kind_label(),
                },
                value,
            )),
            indexmap::map::Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        }
    }

    fn descend(&mut self, key: Cow<'a, str>) -> Result<&mut Self, ()> {
        match self.entry(key) {
            indexmap::map::Entry::Occupied(e) => match e.into_mut() {
                Node::Object(sub) => Ok(sub),
                _ => Err(()),
            },
            indexmap::map::Entry::Vacant(e) => {
                Ok(match e.insert(Node::Object(InlineMap::default())) {
                    Node::Object(sub) => sub,
                    _ => unreachable!("just inserted an object"),
                })
            }
        }
    }
}
