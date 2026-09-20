//! Reopen-merge normalization for the deserialization event path.
//!
//! The thin event parser emits a dotted-key Object re-opened after an
//! intervening sibling (spec 0.7 § 5.3.2) as a separate `Key` +
//! `BeginObject` … `EndObject` block at its own document position — the
//! raw stream never re-opens an already-closed compound. That is the
//! documented contract for the public [`parse_events`](super) callback.
//!
//! The serde
//! [`EventDeserializer`](crate::serde_bridge::EventDeserializer), however,
//! is a plain one-pass `MapAccess`: two root-level `Key("a")` object
//! blocks would surface as a duplicate field (structs) or a silent
//! last-win (maps). Spec § 5.3.2 instead requires the VALUE to be the
//! merged object, in first-appearance order. This pass rebuilds the
//! stream so that every re-opened block's content is folded into the
//! buffer of the key's FIRST object block; reopen-free streams are
//! untouched (see [`super::parse_events_merged`] for the zero-copy fast
//! path that skips this module entirely when the parser reports no
//! reopens).
//!
//! The rebuild uses an explicit stack of bump-allocated buffers (no
//! recursion — documents may nest arbitrarily deep).
//!
//! Cost model: one pass over the stream. Per buffer, child-key lookup
//! is a plain linear scan while the buffer holds at most `LINEAR_MAX`
//! keys (the common case — no index is allocated), and an
//! arena-allocated open-addressing hash index (FxHash, linear
//! probing) above that, so lookups are expected O(1) and the whole
//! pass is expected O(E) over the E stream events plus key hashing.
//! FxHash is fast and deterministic but not hash-flood-resistant (the
//! same trade-off the crate already makes for its object maps); a
//! colliding worst case degrades to the linear scan this pass has
//! always done — never worse. One caveat, stated plainly: the reopen
//! trigger is document-global, so a single reopen anywhere makes
//! `parse_events_merged` rebuild the ENTIRE stream here, branches
//! with no reopens included; a more selective rebuild is out of
//! scope.

#[cfg(test)]
mod perf;
mod reopen;
mod table;
#[cfg(test)]
mod tests;

pub(crate) use reopen::merge_reopened;
