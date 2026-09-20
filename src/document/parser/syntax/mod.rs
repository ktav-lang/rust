//! What the parser reads before it builds anything: the classification
//! of the text after a `:` (or of a bare array line), the shape that
//! classification produces, the bracket kind a compound is waiting to
//! close, and the state of a multi-line string being collected.
//!
//! Nothing here touches the `Value` tree — that is [`super::build`]'s
//! half. Splitting the two is what lets the formatter's trivia-preserving
//! fork reuse this side verbatim while building something else entirely.

pub(super) mod bracket;
pub(crate) mod classify;
pub(super) mod collecting;
pub(super) mod value_start;
