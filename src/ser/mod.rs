//! Serialization: two paths.
//!
//! - `to_string` (and via `crate::to_string`): a direct text serializer
//!   that writes Ktav straight to a `String`, no `Value` intermediate.
//! - `to_value`: builds a [`crate::Value`] tree from `T: Serialize`. Useful
//!   when you want to inspect or post-process the document generically
//!   before rendering. Pair with [`crate::render`] to produce text.

mod map_serializer;
mod seq_serializer;
mod struct_serializer;
mod struct_variant_serializer;
mod text_serializer;
mod to_value;
mod tuple_variant_serializer;
mod value_serializer;

pub use text_serializer::to_string;
pub use to_value::to_value;
