//! Serialization: two paths.
//!
//! - `to_string` (and via `crate::to_string`): a direct text serializer
//!   that writes Ktav straight to a `String`, no `Value` intermediate.
//! - `to_value`: builds a [`crate::Value`] tree from `T: Serialize`. Useful
//!   when you want to inspect or post-process the document generically
//!   before rendering. Pair with [`crate::render`] to produce text.

mod text_serializer;
mod to_value;
mod value_serializer;

// serde's compound-serializer adapters — one file per `SerializeMap` /
// `SerializeSeq` / `SerializeStruct` / variant impl. They live in
// `compounds/` to keep this directory within its file budget, but
// `#[path]` keeps them direct children of this module, so a `super::`
// inside one of them still means `ser::` and no module path moved.
#[path = "compounds/map_serializer.rs"]
mod map_serializer;
#[path = "compounds/seq_serializer.rs"]
mod seq_serializer;
#[path = "compounds/struct_serializer.rs"]
mod struct_serializer;
#[path = "compounds/struct_variant_serializer.rs"]
mod struct_variant_serializer;
#[path = "compounds/tuple_variant_serializer.rs"]
mod tuple_variant_serializer;

pub use text_serializer::to_string;
pub use to_value::to_value;
