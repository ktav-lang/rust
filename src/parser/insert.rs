//! Insert a value at a dotted path inside an object, creating intermediate
//! objects as needed. Each segment is validated as the path is descended,
//! so callers should not pre-validate.
//!
//! Under spec 0.6.0, keys process the § 3.7 escape set; the dotted path
//! splits only on **unescaped** `.` bytes, and `\.` / `\:` decode to a
//! literal `.` / `:` inside a segment.

use indexmap::map::Entry;

use crate::error::{ConflictKind, Error, ErrorKind, Span};
use crate::value::{ObjectMap, Value};

use super::inline::{decode_key_segment, key_is_single_segment, split_key_path};
use super::validate::is_valid_key;

pub(super) fn insert_value(
    table: &mut ObjectMap,
    path: &str,
    value: Value,
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
        // Decode escapes BEFORE validation — the decoded bytes are the
        // actual key (e.g. `a\.b` decodes to `a.b`, which is a valid
        // single segment now).
        let decoded = decode_key_segment(trimmed_key, line_num, span)?;
        if !is_valid_key(&decoded) {
            return Err(Error::Structured(ErrorKind::InvalidKey {
                line: line_num as u32,
                key: path.to_string(),
                span,
            }));
        }
        return match table.entry(decoded.as_str().into()) {
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
        let decoded = decode_key_segment(trimmed, line_num, span)?;
        if !is_valid_key(&decoded) {
            return Err(Error::Structured(ErrorKind::InvalidKey {
                line: line_num as u32,
                key: full_path.to_string(),
                span,
            }));
        }
        let is_leaf = idx + 1 == n;
        if !is_leaf {
            let entry = table
                .entry(decoded.as_str().into())
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
        } else {
            return match table.entry(decoded.as_str().into()) {
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
    unreachable!("loop returns when idx == n - 1")
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
