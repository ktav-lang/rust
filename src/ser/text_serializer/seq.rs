use std::cell::Cell;
use std::rc::Rc;

use serde::ser::{
    Serialize, SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::error::{Error, Result};

use super::items::{ItemValueSer, RootFirstItemSer};
use super::shared::write_indent;

// ---------------------------------------------------------------------------
// SeqCompound — SerializeSeq / Tuple / TupleStruct.
// ---------------------------------------------------------------------------

pub(super) struct SeqCompound<'a> {
    out: &'a mut String,
    item_indent: usize,
    close: Option<usize>, // Some(outer_indent) → write `<outer>]\n`; None → already closed inline.
    /// True only for the document-root Array (§ 5.9.3): the first
    /// element decides the wrapped/unwrapped form, and `end` emits
    /// `[]\n` if no element ever wrote anything (length-`None` seq
    /// that yielded nothing).
    is_root_array: bool,
    /// Root mode only: the § 5.9.3 wrap decision for the first
    /// element, shared with its `RootFirstItemSer`. `None` once the
    /// decision is made.
    root_first: Option<Rc<Cell<bool>>>,
}

impl<'a> SeqCompound<'a> {
    /// Document-root Array (§ 5.9.3): items start bare at indent 0
    /// with no closing `]`; the first element's serializer sets the
    /// shared `wrap` cell if the lone-`[` wrap is required.
    pub(super) fn root(out: &'a mut String) -> Self {
        Self {
            out,
            item_indent: 0,
            close: None,
            is_root_array: true,
            root_first: Some(Rc::new(Cell::new(false))),
        }
    }

    pub(super) fn wrapped(out: &'a mut String, item_indent: usize, close_indent: usize) -> Self {
        Self {
            out,
            item_indent,
            close: Some(close_indent),
            is_root_array: false,
            root_first: None,
        }
    }
    pub(super) fn closed(out: &'a mut String) -> Self {
        Self {
            out,
            item_indent: 0,
            close: None,
            is_root_array: false,
            root_first: None,
        }
    }
}

impl<'a> SerializeSeq for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        if let Some(wrap) = self.root_first.take() {
            // First element of a root Array: it alone is exposed to
            // § 5.0.1 root-kind detection (§ 5.9.6 / § 5.9.12), and if
            // it is a compound the whole root Array takes the § 5.9.3
            // lone-`[`/`{` wrap.
            v.serialize(RootFirstItemSer {
                out: self.out,
                wrap: wrap.clone(),
            })?;
            if wrap.get() {
                self.item_indent = 1;
                self.close = Some(0);
            }
            return Ok(());
        }
        v.serialize(ItemValueSer {
            out: self.out,
            indent: self.item_indent,
            is_root_array_first: false,
        })
    }
    fn end(self) -> Result<()> {
        if let Some(outer) = self.close {
            write_indent(self.out, outer);
            self.out.push_str("]\n");
        } else if self.is_root_array && self.out.is_empty() {
            // Root Array with a `None` length hint that yielded no
            // elements: § 5.9.3 empty Array root. (Root position means
            // `out` is the whole document; any element would have
            // written at least one byte.)
            self.out.push_str("[]\n");
        }
        Ok(())
    }
}

impl<'a> SerializeTuple for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        <Self as SerializeSeq>::serialize_element(self, v)
    }
    fn end(self) -> Result<()> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a> SerializeTupleStruct for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        <Self as SerializeSeq>::serialize_element(self, v)
    }
    fn end(self) -> Result<()> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a> SerializeTupleVariant for SeqCompound<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        <Self as SerializeSeq>::serialize_element(self, v)
    }
    fn end(self) -> Result<()> {
        <Self as SerializeSeq>::end(self)
    }
}
