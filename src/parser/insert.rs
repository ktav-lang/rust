//! Insert a value at a dotted path inside an object, creating intermediate
//! objects as needed. Each segment is validated as the path is descended,
//! so callers should not pre-validate.

use indexmap::map::Entry;

use crate::error::Error;
use crate::value::{ObjectMap, Value};

use super::validate::is_valid_key;

pub(super) fn insert_value(
    table: &mut ObjectMap,
    path: &str,
    value: Value,
    line_num: usize,
) -> Result<(), Error> {
    // Fast path: non-dotted key — the vast majority of inserts. A single
    // `entry()` call collapses the old `contains_key` + `insert` into one
    // hash lookup.
    if !path.as_bytes().contains(&b'.') {
        if !is_valid_key(path) {
            return Err(Error::Syntax(format!(
                "Invalid key at line {}: '{}'",
                line_num, path
            )));
        }
        return match table.entry(path.into()) {
            Entry::Occupied(e) => {
                let existing = e.get();
                if matches!(existing, Value::Object(_)) && !matches!(value, Value::Object(_))
                    || !matches!(existing, Value::Object(_)) && matches!(value, Value::Object(_))
                {
                    Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — cannot overwrite {} with {}",
                        line_num,
                        path,
                        kind_label(existing),
                        kind_label(&value)
                    )))
                } else {
                    Err(Error::Syntax(format!(
                        "Line {}: duplicate key '{}'",
                        line_num, path
                    )))
                }
            }
            Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        };
    }
    insert_dotted(table, path, value, line_num)
}

fn insert_dotted(
    mut table: &mut ObjectMap,
    full_path: &str,
    value: Value,
    line_num: usize,
) -> Result<(), Error> {
    let mut rest = full_path;
    loop {
        if let Some((part, tail)) = rest.split_once('.') {
            if !is_valid_key(part) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, full_path
                )));
            }
            // Single lookup via `entry()`: if Vacant, create an empty
            // Object; if Occupied, it must already be an Object to
            // continue descending.
            let entry = table
                .entry(part.into())
                .or_insert_with(|| Value::Object(ObjectMap::default()));
            table = match entry {
                Value::Object(sub) => sub,
                _ => {
                    return Err(Error::Syntax(format!(
                        "Line {}: conflict at '{}' — an existing value blocks the path",
                        line_num, full_path
                    )));
                }
            };
            rest = tail;
        } else {
            // Leaf insert.
            if !is_valid_key(rest) {
                return Err(Error::Syntax(format!(
                    "Invalid key at line {}: '{}'",
                    line_num, full_path
                )));
            }
            return match table.entry(rest.into()) {
                Entry::Occupied(_) => Err(Error::Syntax(format!(
                    "Line {}: duplicate key '{}'",
                    line_num, full_path
                ))),
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
