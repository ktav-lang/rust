//! Value-level semantics and wire representation: number consistency,
//! writer losslessness and representability, root shape, and the C ABI
//! wire round-trips.

#[path = "value/cabi_corpus.rs"]
mod cabi_corpus;
#[path = "value/cabi_load.rs"]
mod cabi_load;
#[path = "value/float_value_consistency.rs"]
mod float_value_consistency;
#[path = "value/integer_value_consistency.rs"]
mod integer_value_consistency;
#[path = "value/render_lossless.rs"]
mod render_lossless;
#[path = "value/top_level_array.rs"]
mod top_level_array;
#[path = "value/unrepresentable.rs"]
mod unrepresentable;
