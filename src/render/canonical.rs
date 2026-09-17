//! Canonical writer — emits a deterministic byte sequence for any [`Value`]
//! per spec § 5.9.
//!
//! The canonical form is:
//! - LF-only line endings (no `CR`).
//! - 4-space indent per nesting level.
//! - Trailing `LF` at end of document (empty Object root → zero bytes).
//! - No comments.
//! - No inline compounds (except empty `{}` / `[]`).
//! - Numbers in canonical form (Integer: base-10; Float: shortest decimal).
//! - Multi-line strings prefer verbatim `((…))`.
//!
//! A non-representable Value — a scalar root, an empty key name, a
//! non-finite Float, a `CR` byte, or one of the three multi-line
//! collision cases — is rejected per § 5.9.0 with a
//! [`crate::error::ReasonCode`]-coded `Error::Unrepresentable` before
//! any bytes are emitted; partial output followed by failure never
//! happens.
//!
//! Two writer-conforming implementations fed the same Value MUST produce
//! identical output (§ 8.2).

mod num;
mod shared;
mod strings;
mod walk;

pub(super) use super::representable;
pub(crate) use num::canonical_float;
pub(crate) use strings::{emit_string_as_item, emit_string_in_pair};
pub use walk::emit_canonical;

#[cfg(test)]
mod tests;
