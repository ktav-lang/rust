//! Backs the `Enum::Variant { field: T, ... }` case — externally tagged as
//! a single-key object whose value is the variant's struct body.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use serde::ser::{Serialize, SerializeStructVariant};

use crate::error::{Error, Result};
use crate::value::{ObjectMap, Value};

use super::value_serializer::ValueSerializer;

pub(crate) struct StructVariantSerializer {
    pub(super) variant: &'static str,
    pub(super) entries: ObjectMap,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, name: &'static str, v: &T) -> Result<()> {
        self.entries
            .insert(name.into(), v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = IndexMap::with_capacity_and_hasher(1, FxBuildHasher);
        map.insert(self.variant.into(), Value::Object(self.entries));
        Ok(Value::Object(map))
    }
}
