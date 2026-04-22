//! Ordered, Fx-hashed `Scalar → Value` map.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use super::value::{Scalar, Value};

/// Ordered `Scalar → Value` map used for Ktav objects. Insertion order is
/// preserved (so serialization keeps struct field order), and hashing uses
/// FxHash — fast and deterministic, not hash-flood-resistant (which a
/// config parser does not need). Keys use `Scalar` (`CompactString`) so
/// short identifiers — the common case for config keys — live inline
/// without heap allocation.
pub type ObjectMap = IndexMap<Scalar, Value, FxBuildHasher>;
