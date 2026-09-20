//! The trivia-carrying value tree and its conversion to a plain [`crate::Value`].

use std::borrow::Cow;

use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::value::{Scalar, Value};

use crate::parser::insert::{InsertShape, InsertTable, OccupiedShape};

// ---------------------------------------------------------------------------
// Trivia + trivia-carrying value tree
// ---------------------------------------------------------------------------

/// One preserved line of trivia: a blank line or a whole-line comment
/// (spec § 3.4 — every comment owns a full line). `Comment` stores the
/// trimmed source text verbatim, starting at the first `#`; the
/// formatter re-indents comments to their attachment point's depth but
/// never rewrites their body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TriviaLine {
    Blank,
    Comment(String),
}

/// A trivia-carrying Object: each pair keeps the trivia lines that
/// preceded it in the source, plus `trailing` trivia that appeared
/// after the last pair and before the compound's close (or, for the
/// document root, at end-of-file).
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PObject {
    pub(crate) pairs: IndexMap<Scalar, (Vec<TriviaLine>, PValue), FxBuildHasher>,
    pub(crate) trailing: Vec<TriviaLine>,
}

/// A trivia-carrying Array: same shape as [`PObject`], keyed by
/// position instead of by key.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PArray {
    pub(crate) items: Vec<(Vec<TriviaLine>, PValue)>,
    pub(crate) trailing: Vec<TriviaLine>,
}

/// [`Value`], plus trivia. Scalars carry no trivia of their own — only
/// the pair / item slot that holds them does (see [`PObject`] /
/// [`PArray`]).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PValue {
    Null,
    Bool(bool),
    Integer(Scalar),
    Float(Scalar),
    String(Scalar),
    Array(PArray),
    Object(PObject),
}

/// A parsed document, plus trivia that precedes the very first content
/// line. There is no tree node to attach that trivia to — the root has
/// no parent — so it lives here instead. Trivia after the last content
/// line lives on the root value's own `trailing` field (root is always
/// Object or Array, per § 5.0.1).
pub(crate) struct FmtDoc {
    pub(crate) leading: Vec<TriviaLine>,
    pub(crate) root: PValue,
}

/// Flatten a `PValue` tree back to a plain [`Value`], dropping all
/// trivia. Used to reuse `render::representable::check_representable`
/// unchanged, and to let tests compare formatter structure against
/// [`super::parse_str()`]'s oracle.
pub(crate) fn to_plain_value(v: &PValue) -> Value {
    match v {
        PValue::Null => Value::Null,
        PValue::Bool(b) => Value::Bool(*b),
        PValue::Integer(s) => Value::Integer(s.clone()),
        PValue::Float(s) => Value::Float(s.clone()),
        PValue::String(s) => Value::String(s.clone()),
        PValue::Array(a) => Value::Array(a.items.iter().map(|(_, v)| to_plain_value(v)).collect()),
        PValue::Object(o) => Value::Object(
            o.pairs
                .iter()
                .map(|(k, (_, v))| (k.clone(), to_plain_value(v)))
                .collect(),
        ),
    }
}

/// Wrap a plain `Value` (returned by the shared inline-compound /
/// root-kind-detection helpers) into a `PValue` with empty trivia
/// throughout. Sound because a comment can never occur inside an
/// inline compound (§ 3.4 — comments own a whole physical line, inline
/// compounds live on one).
pub(super) fn stamp(value: Value) -> PValue {
    match value {
        Value::Null => PValue::Null,
        Value::Bool(b) => PValue::Bool(b),
        Value::Integer(s) => PValue::Integer(s),
        Value::Float(s) => PValue::Float(s),
        Value::String(s) => PValue::String(s),
        Value::Array(items) => PValue::Array(PArray {
            items: items.into_iter().map(|v| (Vec::new(), stamp(v))).collect(),
            trailing: Vec::new(),
        }),
        Value::Object(obj) => PValue::Object(PObject {
            pairs: obj
                .into_iter()
                .map(|(k, v)| (k, (Vec::new(), stamp(v))))
                .collect(),
            trailing: Vec::new(),
        }),
    }
}

// ---------------------------------------------------------------------------
// `InsertTable` / `InsertShape` for `PObject` — reuses `insert_value`'s
// dotted-key descent, duplicate/conflict outcome tables, and reopen
// handling unchanged.
// ---------------------------------------------------------------------------

impl InsertShape for (Vec<TriviaLine>, PValue) {
    fn is_object(&self) -> bool {
        matches!(self.1, PValue::Object(_))
    }
    fn kind_label(&self) -> &'static str {
        match &self.1 {
            PValue::Null => "null",
            PValue::Bool(_) => "bool",
            PValue::Integer(_) => "integer",
            PValue::Float(_) => "float",
            PValue::String(_) => "string",
            PValue::Array(_) => "array",
            PValue::Object(_) => "object",
        }
    }
}

impl<'k> InsertTable<'k> for PObject {
    type Value = (Vec<TriviaLine>, PValue);

    fn insert_leaf(
        &mut self,
        key: Cow<'k, str>,
        value: (Vec<TriviaLine>, PValue),
    ) -> std::result::Result<(), (OccupiedShape, (Vec<TriviaLine>, PValue))> {
        match self.pairs.entry(key.as_ref().into()) {
            Entry::Occupied(e) => {
                let existing = e.get();
                Err((
                    OccupiedShape {
                        is_object: existing.is_object(),
                        label: existing.kind_label(),
                    },
                    value,
                ))
            }
            Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        }
    }

    fn descend(&mut self, key: Cow<'k, str>) -> std::result::Result<&mut Self, ()> {
        match self.pairs.entry(key.as_ref().into()) {
            Entry::Occupied(e) => match &mut e.into_mut().1 {
                PValue::Object(sub) => Ok(sub),
                _ => Err(()),
            },
            Entry::Vacant(e) => {
                let inserted = e.insert((Vec::new(), PValue::Object(PObject::default())));
                match &mut inserted.1 {
                    PValue::Object(sub) => Ok(sub),
                    _ => unreachable!("just inserted an object"),
                }
            }
        }
    }
}

/// Drop trailing `Blank` entries — used whenever a trivia run becomes a
/// `trailing` bucket (right before a close bracket, or at EOF): padding
/// with nothing after it carries no meaning.
pub(super) fn trim_trailing_blanks(mut trivia: Vec<TriviaLine>) -> Vec<TriviaLine> {
    while matches!(trivia.last(), Some(TriviaLine::Blank)) {
        trivia.pop();
    }
    trivia
}
