//! The byte scanner shared by every inline-compound operation.
//!
//! One state machine, three faces: [`scan_config`] declares the knobs a
//! caller sets (where to stop, whether quotes are possible, which fast
//! path applies), [`scanner`] is the machine itself, and [`split`] is the
//! set of operations built on it — finding a closer, finding an unescaped
//! colon, splitting a body at top level.
//!
//! These three were one file's worth of coupling before they were three
//! files; keeping them in their own directory says so.

pub(super) mod scan_config;
pub(super) mod scanner;
pub(super) mod split;
