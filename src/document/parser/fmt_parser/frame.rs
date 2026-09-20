//! One nesting level of the trivia-preserving parser's stack.
//!
//! The counterpart to `super::super::build::frame::Frame`: same two
//! shapes, but each carries the comment and blank-line trivia that the
//! `Value` parser throws away, plus the pending key's span so a later
//! error can point at it.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::error::Span;

use super::doc::{PArray, PObject, PValue, TriviaLine};

pub(super) enum PFrame<'a> {
    Object {
        table: PObject,
        pending_key: Option<&'a str>,
        pending_key_span: Option<Span>,
        pending_key_trivia: Vec<TriviaLine>,
        at_start: bool,
    },
    Array {
        table: PArray,
        pending_item_trivia: Vec<TriviaLine>,
        at_start: bool,
    },
}

impl<'a> PFrame<'a> {
    pub(super) fn new_object() -> Self {
        PFrame::Object {
            table: PObject {
                pairs: IndexMap::with_capacity_and_hasher(8, FxBuildHasher),
                trailing: Vec::new(),
            },
            pending_key: None,
            pending_key_span: None,
            pending_key_trivia: Vec::new(),
            at_start: true,
        }
    }

    pub(super) fn new_array() -> Self {
        PFrame::Array {
            table: PArray {
                items: Vec::with_capacity(8),
                trailing: Vec::new(),
            },
            pending_item_trivia: Vec::new(),
            at_start: true,
        }
    }

    pub(super) fn into_value(self) -> PValue {
        match self {
            PFrame::Object { table, .. } => PValue::Object(table),
            PFrame::Array { table, .. } => PValue::Array(table),
        }
    }

    pub(super) fn set_started(&mut self) {
        match self {
            PFrame::Object { at_start, .. } | PFrame::Array { at_start, .. } => *at_start = false,
        }
    }

    pub(super) fn is_at_start(&self) -> bool {
        match self {
            PFrame::Object { at_start, .. } | PFrame::Array { at_start, .. } => *at_start,
        }
    }

    pub(super) fn trailing_mut(&mut self) -> &mut Vec<TriviaLine> {
        match self {
            PFrame::Object { table, .. } => &mut table.trailing,
            PFrame::Array { table, .. } => &mut table.trailing,
        }
    }
}
