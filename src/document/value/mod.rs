//! Dynamic representation of a Ktav document.

pub mod object_map;
pub mod value;

pub use object_map::ObjectMap;
pub use value::{Scalar, Value};

#[cfg(test)]
mod tests;
