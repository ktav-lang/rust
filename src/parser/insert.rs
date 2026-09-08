//! Insert a value at a dotted path inside an object, creating intermediate
//! objects as needed. Each segment is validated as the path is descended,
//! so callers should not pre-validate.
//!
//! Under spec 0.6.0, keys process the § 3.7 escape set; the dotted path
//! splits only on **unescaped** `.` bytes, and `\.` / `\:` decode to a
//! literal `.` / `:` inside a segment. The § 6.3 duplicate/conflict
//! outcome tables below are shared with the thin path.

use std::borrow::Cow;

use indexmap::map::Entry;

use crate::error::{ConflictKind, Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::inline::{decode_key_segment, key_is_single_segment, split_key_path};
use super::validate::{check_key, KeyValidity};

/// How the § 6.3 outcome tables observe a stored value's shape. The
/// labels are the same kind strings the old free `kind_label` produced
/// and appear verbatim in `ConflictKind::Overwrite` diagnostics.
pub(crate) trait InsertShape {
    fn is_object(&self) -> bool;
    fn kind_label(&self) -> &'static str;
}

/// Shape of an entry already occupying a slot, reported by
/// [`InsertTable::insert_leaf`] so `insert_value` can classify the
/// conflict without cloning the existing value.
pub(crate) struct OccupiedShape {
    pub(crate) is_object: bool,
    pub(crate) label: &'static str,
}

/// A keyed table the § 6.3 insert tables can operate on. `'k` is the
/// lifetime of the key text being inserted (the table decides how to
/// store it — `ObjectMap` copies into `Scalar`, the thin path's table
/// stores borrowed slices).
pub(crate) trait InsertTable<'k>: Sized {
    type Value: InsertShape;

    /// Insert `value` at an already-validated, already-decoded single
    /// segment. `Err((existing_shape, value))` — slot left untouched,
    /// `value` handed back so conflict diagnostics can inspect it
    /// lazily — when occupied; `Ok(())` when inserted.
    fn insert_leaf(
        &mut self,
        key: Cow<'k, str>,
        value: Self::Value,
    ) -> Result<(), (OccupiedShape, Self::Value)>;

    /// Descend one dotted-key segment, inserting an empty object when
    /// absent; `Err(())` when a non-object value blocks the path (§ 6.3
    /// `BlockedByValue`).
    fn descend(&mut self, key: Cow<'k, str>) -> Result<&mut Self, ()>;
}

impl InsertShape for Value {
    fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
    fn kind_label(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

impl<'k> InsertTable<'k> for ObjectMap {
    type Value = Value;

    fn insert_leaf(
        &mut self,
        key: Cow<'k, str>,
        value: Value,
    ) -> Result<(), (OccupiedShape, Value)> {
        match self.entry(key.as_ref().into()) {
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

    fn descend(&mut self, key: Cow<'k, str>) -> Result<&mut Self, ()> {
        match self.entry(key.as_ref().into()) {
            Entry::Occupied(e) => match e.into_mut() {
                Value::Object(sub) => Ok(sub),
                _ => Err(()),
            },
            Entry::Vacant(e) => Ok(match e.insert(Value::Object(ObjectMap::default())) {
                Value::Object(sub) => sub,
                _ => unreachable!("just inserted an object"),
            }),
        }
    }
}

/// Insert `value` at `path` inside `table`, creating intermediate objects
/// as needed. Each segment is validated as the path is descended, so
/// callers should not pre-validate.
///
/// Under spec 0.6.0, keys process the § 3.7 escape set; the dotted path
/// splits only on **unescaped** `.` bytes, and `\.` / `\:` decode to a
/// literal `.` / `:` inside a segment.
///
/// § 6.3 outcomes: an empty or invalid segment yields `EmptyKey` /
/// `InvalidKey`; a single-segment key colliding with a differently
/// shaped value yields `KeyPathConflict::Overwrite`, with a same-shaped
/// value yields `DuplicateKey`; a dotted key blocked mid-path by a
/// non-object value yields `KeyPathConflict::BlockedByValue`; an
/// occupied leaf on a dotted key yields `DuplicateKey`.
pub(crate) fn insert_value<'k, T: InsertTable<'k>>(
    table: &mut T,
    path: &'k str,
    value: T::Value,
    line_num: usize,
    span: Span,
) -> Result<(), Error> {
    // Fast path: non-dotted key (no UNescaped `.`) — the vast majority
    // of inserts. A single `entry()` call collapses the old
    // `contains_key` + `insert` into one hash lookup.
    if key_is_single_segment(path) {
        let trimmed_key = path.trim();
        if trimmed_key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }
        // Validate the RAW segment (forbidden bytes must be escaped,
        // quoted segments checked against their own class) before
        // decoding — `check_key` needs to see which bytes were
        // escaped, which the decoded form has already erased.
        match check_key(trimmed_key) {
            KeyValidity::Valid => {}
            KeyValidity::Empty => {
                return Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }));
            }
            KeyValidity::Invalid => {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: path.to_string(),
                    span,
                }));
            }
        }
        let decoded = decode_key_segment(trimmed_key, line_num, span)?;
        return match table.insert_leaf(decoded, value) {
            Err((existing, value)) => {
                let new_label = value.kind_label();
                if existing.is_object != value.is_object() {
                    Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: path.to_string(),
                        kind: ConflictKind::Overwrite {
                            existing: existing.label,
                            new_kind: new_label,
                        },
                        span,
                    }))
                } else {
                    Err(Error::Structured(ErrorKind::DuplicateKey {
                        line: line_num as u32,
                        key: path.to_string(),
                        span,
                    }))
                }
            }
            Ok(()) => Ok(()),
        };
    }
    insert_dotted(table, path, value, line_num, span)
}

fn insert_dotted<'k, T: InsertTable<'k>>(
    mut table: &mut T,
    full_path: &'k str,
    value: T::Value,
    line_num: usize,
    span: Span,
) -> Result<(), Error> {
    let segments = split_key_path(full_path);
    let n = segments.len();
    debug_assert!(n >= 2);
    for (idx, seg) in segments.iter().enumerate() {
        // § 4: trim each segment of leading/trailing whitespace.
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }
        match check_key(trimmed) {
            KeyValidity::Valid => {}
            KeyValidity::Empty => {
                return Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }));
            }
            KeyValidity::Invalid => {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: full_path.to_string(),
                    span,
                }));
            }
        }
        let decoded = decode_key_segment(trimmed, line_num, span)?;
        let is_leaf = idx + 1 == n;
        if !is_leaf {
            table = table.descend(decoded).map_err(|()| {
                Error::Structured(ErrorKind::KeyPathConflict {
                    line: line_num as u32,
                    path: full_path.to_string(),
                    kind: ConflictKind::BlockedByValue,
                    span,
                })
            })?;
        } else {
            return match table.insert_leaf(decoded, value) {
                Err(_) => Err(Error::Structured(ErrorKind::DuplicateKey {
                    line: line_num as u32,
                    key: full_path.to_string(),
                    span,
                })),
                Ok(()) => Ok(()),
            };
        }
    }
    unreachable!("loop returns when idx == n - 1")
}
