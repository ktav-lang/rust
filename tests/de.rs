//! Deserialization integration tests, organized by feature.

#[path = "common/mod.rs"]
mod common;

#[path = "de/arrays.rs"]
mod arrays;
#[path = "de/enums.rs"]
mod enums;
#[path = "de/errors.rs"]
mod errors;
#[path = "de/keywords.rs"]
mod keywords;
#[path = "de/multiline.rs"]
mod multiline;
#[path = "de/objects.rs"]
mod objects;
#[path = "de/option_map_keys.rs"]
mod option_map_keys;
#[path = "de/owned_key_names.rs"]
mod owned_key_names;
#[path = "de/raw_marker.rs"]
mod raw_marker;
#[path = "de/scalars.rs"]
mod scalars;
#[path = "de/typed.rs"]
mod typed;
