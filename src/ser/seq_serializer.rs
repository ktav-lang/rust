//! Backs `serialize_seq` / `serialize_tuple` / `serialize_tuple_struct` —
//! all three collect homogeneous elements into a `Value::Array`.

use serde::ser::{Serialize, SerializeSeq, SerializeTuple, SerializeTupleStruct};

use crate::error::{Error, Result};
use crate::value::Value;

use super::value_serializer::ValueSerializer;

pub(crate) struct SeqSerializer {
    pub(super) items: Vec<Value>,
}

impl SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.items.push(v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.items))
    }
}

impl SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.items.push(v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.items))
    }
}

impl SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.items.push(v.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.items))
    }
}
