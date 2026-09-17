//! Direct inline-compound scanner for the thin event path.
//!
//! Scans a closed inline compound (`{ … }` / `[ … ]`) into a sequence
//! of [`Event`]s through a borrowed intermediate `Node` tree — no
//! owned `Value`, no string copies: unmodified scalars and keys are
//! borrowed from the source line, and only genuine decoding or
//! normalization allocates in the arena (escape-decoded strings,
//! canonical integer/float forms via itoa/ryu).
//!
//! Transient state per compound: one flat `Vec<Event>` scratch that
//! the scan appends to — array items and nested arrays land there
//! directly in source order, so an array never materializes a
//! per-item node list — plus one insertion-ordered key table
//! (`InlineMap`, an `IndexMap`) per object scope, which
//! duplicate/conflict detection and dotted-key merge require. An
//! array that is directly an object member's value is staged as one
//! flat bracketed event block (`Node::Array(Vec<Event>)`) until its
//! object's insertion order is final. `scan_inline_events` copies
//! the finished scratch into its sink only after the whole compound
//! validates, so a failing compound pushes no events.
//!
//! Dotted keys inside the compound are expanded with the SAME shared
//! § 6.3 outcome tables the owned parser uses (`parser::insert`), so
//! duplicate/conflict diagnostics are byte-identical. Emission order
//! matches the owned path's `Value`-tree walk exactly: insertion
//! order per object, with dotted keys merged into shared sub-objects.

mod emit;
mod node;
mod perf;
mod scan;
#[cfg(test)]
mod tests;

pub(crate) use emit::fast_plain_decimal_i64;
pub(crate) use scan::scan_inline_events;
