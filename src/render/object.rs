//! Render the pairs of an object (without surrounding braces).

use crate::error::Result;
use crate::value::ObjectMap;

use super::pair::render_pair;

pub(super) fn render_object_body(obj: &ObjectMap, indent: usize, out: &mut String) -> Result<()> {
    for (key, value) in obj {
        render_pair(key, value, indent, out)?;
    }
    Ok(())
}
