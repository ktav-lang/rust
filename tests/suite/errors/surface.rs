//! Error surfaces: accessors, spans, formatting, structured
//! kinds, strict mode, non-exhaustive guarantees, and UTF-8
//! validation on from_file.
#[path = "surface/error_accessors.rs"]
mod error_accessors;
#[path = "surface/error_format.rs"]
mod error_format;
#[path = "surface/error_spans.rs"]
mod error_spans;
#[path = "surface/invalid_utf8.rs"]
mod invalid_utf8;
#[path = "surface/non_exhaustive.rs"]
mod non_exhaustive;
#[path = "surface/strict_mode.rs"]
mod strict_mode;
#[path = "surface/structured_errors.rs"]
mod structured_errors;
