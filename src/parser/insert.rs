//! Insert a value at a dotted path inside an object, creating intermediate
//! objects as needed. Each segment is validated as the path is descended,
//! so callers should not pre-validate.

use indexmap::map::Entry;

use crate::error::{ConflictKind, Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::validate::is_valid_key;

pub(super) fn insert_value(
    table: &mut ObjectMap,
    path: &str,
    value: Value,
    line_num: usize,
    span: Span,
) -> Result<(), Error> {
    // Fast path: non-dotted key — the vast majority of inserts. A single
    // `entry()` call collapses the old `contains_key` + `insert` into one
    // hash lookup.
    if !path.as_bytes().contains(&b'.') {
        // § 4: trim the single key segment of leading/trailing whitespace
        // before validation. Empty-after-trim → EmptyKey.
        let trimmed_key = path.trim();
        if trimmed_key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span,
            }));
        }
        if !is_valid_key(trimmed_key) {
            return Err(Error::Structured(ErrorKind::InvalidKey {
                line: line_num as u32,
                key: path.to_string(),
                span,
            }));
        }
        return match table.entry(trimmed_key.into()) {
            Entry::Occupied(e) => {
                let existing = e.get();
                if matches!(existing, Value::Object(_)) && !matches!(value, Value::Object(_))
                    || !matches!(existing, Value::Object(_)) && matches!(value, Value::Object(_))
                {
                    Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: path.to_string(),
                        kind: ConflictKind::Overwrite {
                            existing: kind_label(existing),
                            new_kind: kind_label(&value),
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
            Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        };
    }
    insert_dotted(table, path, value, line_num, span)
}

fn insert_dotted(
    mut table: &mut ObjectMap,
    full_path: &str,
    value: Value,
    line_num: usize,
    span: Span,
) -> Result<(), Error> {
    let mut rest = full_path;
    loop {
        if let Some((part, tail)) = rest.split_once('.') {
            // § 4: trim each segment of leading/trailing whitespace.
            let trimmed_part = part.trim();
            if trimmed_part.is_empty() {
                return Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }));
            }
            if !is_valid_key(trimmed_part) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: full_path.to_string(),
                    span,
                }));
            }
            // Single lookup via `entry()`: if Vacant, create an empty
            // Object; if Occupied, it must already be an Object to
            // continue descending.
            let entry = table
                .entry(trimmed_part.into())
                .or_insert_with(|| Value::Object(ObjectMap::default()));
            table = match entry {
                Value::Object(sub) => sub,
                _ => {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        path: full_path.to_string(),
                        kind: ConflictKind::BlockedByValue,
                        span,
                    }));
                }
            };
            rest = tail;
        } else {
            // Leaf insert — trim the final segment.
            let trimmed_rest = rest.trim();
            if trimmed_rest.is_empty() {
                return Err(Error::Structured(ErrorKind::EmptyKey {
                    line: line_num as u32,
                    span,
                }));
            }
            if !is_valid_key(trimmed_rest) {
                return Err(Error::Structured(ErrorKind::InvalidKey {
                    line: line_num as u32,
                    key: full_path.to_string(),
                    span,
                }));
            }
            return match table.entry(trimmed_rest.into()) {
                Entry::Occupied(_) => Err(Error::Structured(ErrorKind::DuplicateKey {
                    line: line_num as u32,
                    key: full_path.to_string(),
                    span,
                })),
                Entry::Vacant(v) => {
                    v.insert(value);
                    Ok(())
                }
            };
        }
    }
}

fn kind_label(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
