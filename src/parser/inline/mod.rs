//! Inline compound parser (spec 0.5.0 section 5.8).
//!
//! Parses `{ key: value, ... }` into `Value::Object` and
//! `[ v1, v2, ... ]` into `Value::Array`.
//!
//! Escape sequences (section 3.7) are processed inside inline scalar values.

mod compound;
mod escapes;
#[cfg(test)]
pub(crate) mod ix_probe;
mod keys;
mod scan_config;
mod scanner;
mod split;

pub(crate) use compound::{
    parse_float_value, parse_inline_array, parse_inline_object, MAX_INLINE_DEPTH,
};
pub(crate) use escapes::process_escapes;
pub(crate) use keys::{
    decode_key_segment, find_unescaped_colon, has_quote_bytes, key_is_single_segment,
    quoted_span_end, scan_unescaped_colon, split_key_path, ColonScan,
};
pub(crate) use split::{
    find_matching_close, find_unescaped_colon_inline, malformed_closer_not_at_end,
    scan_inline_closer, scan_inline_closer_with_bounds, split_top_level, InlineBody, InlineBounds,
    InlineCloserScan,
};
