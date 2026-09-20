//! Direct text serializer: serializes `T: Serialize` straight to a Ktav
//! text string, skipping the `Value` intermediate that `ser::to_value` +
//! `render::render` go through. Produces byte-identical output to the
//! Value-based path, with one divergence: for scientific-notation Floats
//! this direct path emits a text literal with a `.0` mantissa (`1.0e100`),
//! while the Value path stores/renders the parser-normalized payload
//! (`1e100`); `emit_canonical` normalises both.

mod items;
mod object;
mod root;
mod seq;
mod shared;
mod variants;

pub(super) use object::serialize_key_name;
pub use root::to_string;
