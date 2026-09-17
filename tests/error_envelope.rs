//! The unified ten-field error envelope (issue rust#12): one JSON
//! object for every structured error, parse-time and writer-time
//! alike. These tests pin the wire contract — field set and order,
//! path-is-an-array semantics, byte-offset spans, RFC 8259 escaping,
//! spec-section mapping, and the parse-trigger → envelope mapping for
//! every `ErrorKind` and `ReasonCode`.

#[path = "error_envelope/escapes.rs"]
mod escapes;
#[path = "error_envelope/helpers.rs"]
mod helpers;
#[path = "error_envelope/message.rs"]
mod message;
#[path = "error_envelope/paths.rs"]
mod paths;
#[path = "error_envelope/shape.rs"]
mod shape;
#[path = "error_envelope/spans.rs"]
mod spans;
