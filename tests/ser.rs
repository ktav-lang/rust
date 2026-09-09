//! Serialization integration tests, organized by feature.

#[path = "ser/arrays.rs"]
mod arrays;
#[path = "ser/enums.rs"]
mod enums;
#[path = "ser/key_name_consistency.rs"]
mod key_name_consistency;
#[path = "ser/keywords.rs"]
mod keywords;
#[path = "ser/multiline.rs"]
mod multiline;
#[path = "ser/objects.rs"]
mod objects;
#[path = "ser/raw_marker.rs"]
mod raw_marker;
#[path = "ser/root_arrays.rs"]
mod root_arrays;
#[path = "ser/scalars.rs"]
mod scalars;
#[path = "ser/typed.rs"]
mod typed;
