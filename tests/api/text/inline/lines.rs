//! Scalar-line and multiline-block edge grammar.

#[path = "lines/inline_colon_value.rs"]
mod inline_colon_value;
#[path = "lines/inline_comma_key_context.rs"]
mod inline_comma_key_context;
#[path = "lines/inline_scalar_edges.rs"]
mod inline_scalar_edges;
#[path = "lines/inline_trailing_whitespace.rs"]
mod inline_trailing_whitespace;
#[path = "lines/midvalue_braces.rs"]
mod midvalue_braces;
#[path = "lines/multiline_codepoint_dedent.rs"]
mod multiline_codepoint_dedent;
#[path = "lines/multiline_vt_dedent.rs"]
mod multiline_vt_dedent;
