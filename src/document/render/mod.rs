//! Render a [`crate::Value`] as a Ktav text document.
//!
//! The per-compound emitters live in `emitters/` so this directory stays
//! within its file budget, but they are declared here with `#[path]` and
//! so remain direct children of this module: a `super::helpers` inside
//! one of them still means `render::helpers`, and no module path changed
//! when the files moved.

pub mod canonical;
pub(crate) mod formatted;
pub(crate) mod helpers;
mod representable;

#[path = "emitters/array_item.rs"]
mod array_item;
#[path = "emitters/object.rs"]
mod object;
#[path = "emitters/pair.rs"]
mod pair;
#[path = "emitters/render.rs"]
mod render;

pub use canonical::emit_canonical;
pub use render::{render, to_string_force_strings};

#[cfg(test)]
#[path = "emitters/tests.rs"]
mod tests;
