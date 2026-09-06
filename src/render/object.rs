//! Render the pairs of an object (without surrounding braces).

use crate::error::Result;
use crate::value::ObjectMap;

use super::pair::render_pair;

/// Render an Object's pairs at the given indent level.
///
/// `is_root` is true ONLY for the document-root Object; it lets the
/// first pair's key take the § 5.9.10 rule (c) U+FEFF guard, which
/// applies only at byte offset 0 of the document (§ 5.9.12). Every
/// nested call passes `false`.
pub(super) fn render_object_body(
    obj: &ObjectMap,
    indent: usize,
    is_root: bool,
    out: &mut String,
) -> Result<()> {
    for (index, (key, value)) in obj.iter().enumerate() {
        render_pair(key, value, indent, is_root && index == 0, out)?;
    }
    Ok(())
}
