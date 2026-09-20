//! Assembling the `Value` tree: one nesting level of the parser's stack,
//! the dotted-path insert that creates intermediate objects as it
//! descends, and the key and path rules of spec § 4 that insert applies
//! on the way down.
//!
//! The counterpart to [`super::syntax`], which decides what a line says
//! without deciding where it goes.

pub(super) mod frame;
pub(crate) mod insert;
pub(crate) mod validate;
