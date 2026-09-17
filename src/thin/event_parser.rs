//! Tokenize Ktav text directly into a flat [`EventStream`] — no
//! intermediate tree. Mirrors the validation logic of [`crate::parser`]
//! but emits a linear sequence of `Event`s into a single bump-arena
//! `Vec` instead of a recursive `ThinValue`.
//!
//! Dotted keys are resolved here, at tokenize time, by maintaining a
//! per-object-frame stack of currently-open synthetic prefixes. When a
//! new line's prefix diverges from the stack, the divergence point is
//! emitted as a sequence of `EndObject`s; the new tail is emitted as
//! `Key`+`BeginObject`s.
//!
//! Duplicates and path conflicts are caught the same way the tree-builder
//! catches them (spec 0.7 § 5.3.2 / § 6.3): the parser maintains ONE
//! parse-wide shared node arena of key-path SHAPES — one `PathShape`
//! per real
//! key segment ever seen, labelled `Object` or `Leaf(<kind>)` — plus a
//! two-tier lookup index keyed on `(parent node id, decoded segment)`:
//! small documents scan a contiguous linear list with zero hashing and
//! zero heap allocation, and once entries exceed
//! `LINEAR_INDEX_THRESHOLD` the index spills (once) into a
//! `FxHashMap` for O(1) lookups on large documents. Identity is
//! STRUCTURAL: a node is its `(parent, segment)` pair, never a joined
//! string, because decoded segments may contain literal dots. A child
//! object's frame holds only the `NodeId` of its own key path, so its
//! subtree is visible to the enclosing frame with no copying on close.
//! A dotted key re-entering a path already shaped as an `Object`
//! (whether created by an earlier dotted pair or explicitly as
//! `a: { … }`) MERGES, regardless of intervening sibling pairs; a dotted
//! path descending through a non-Object leaf, or a plain pair naming an
//! earlier-established Object, raises `KeyPathConflict`. The synthetic
//! `ObjectLevel`s hold no key state at all — only the prefix needed for
//! longest-common-prefix comparison and emission bookkeeping.
//!
//! A compound value's INTERNAL key paths — for both inline (`a: {x: 1}`)
//! and explicit multi-line compounds — are registered into the shared
//! parse-wide shared node-shape arena under the compound's own node, recursively for nested
//! objects, and stop at array boundaries (arrays are leaves), so later
//! dotted re-entry sees them (§ 5.3.2 / § 6.3). Event order / first-
//! appearance order lives in the event stream and is independent of the
//! node index.

mod classify;
#[cfg(test)]
mod counter_tests;
mod dotted;
mod lines;
mod state;

pub(crate) use lines::parse_events;
