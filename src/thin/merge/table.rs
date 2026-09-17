use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use rustc_hash::FxHasher;

use super::super::event::Event;
use super::reopen::key_eq;

/// One flattened item in a rebuild buffer: either a leaf event, or a
/// nested compound whose contents live in buffer `buf` (`open` is the
/// `BeginObject` / `BeginArray`; the matching close is derived at
/// flatten time).
pub(super) enum Item<'a> {
    Ev(Event<'a>),
    Obj { buf: usize, open: Event<'a> },
}

/// A rebuild buffer. `seen` maps each child key to the buffer of its
/// FIRST object block, so a later `Key(k)` + `BeginObject` with the same
/// key is detected as a § 5.3.2 reopen and merged into that
/// buffer. `seen` stays the order-carrying source of truth
/// (first-appearance order); `index` is a lazily built open-addressing
/// index over it, `None` while the buffer is small.
pub(super) struct Buf<'a> {
    pub(super) items: BumpVec<'a, Item<'a>>,
    pub(super) seen: BumpVec<'a, (&'a str, usize)>,
    pub(super) index: Option<BumpVec<'a, SeenSlot>>,
}

/// Buffers at or below this many keyed children never build the hash
/// index: the linear scan over `seen` (whose preallocation is exactly
/// 8 slots) is cheaper than hashing, the worst case is a bounded 28
/// comparisons for all misses, and small buffers keep exactly the
/// pre-index behavior.
const LINEAR_MAX: usize = 8;

/// Sentinel `pos` of an empty index slot. Kept on `pos` (not `h`) so a
/// real entry whose hash happens to be 0 is still distinguishable.
const VACANT: usize = usize::MAX;

const INDEX_MIN_SLOTS: usize = 16;

/// One slot of the open-addressing index over `seen`: `pos` is an
/// offset into `Buf::seen`, `h` the cached FxHash of that key (reused
/// on rehash, so growing never re-hashes keys).
pub(super) struct SeenSlot {
    h: u32,
    pos: usize,
}

fn key_hash(k: &str) -> u32 {
    use std::hash::{Hash, Hasher};

    let mut h = FxHasher::default();
    k.hash(&mut h);
    h.finish() as u32
}

impl<'a> Buf<'a> {
    /// Buffer of `k`'s first object block, if `k` was already seen in
    /// this buffer. Linear scan below `LINEAR_MAX`, hash probe above.
    pub(super) fn seen_find(&self, k: &str) -> Option<usize> {
        match self.index.as_ref() {
            None => self
                .seen
                .iter()
                .find(|(key, _)| key_eq(key, k))
                .map(|(_, idx)| *idx),
            Some(table) => {
                let mask = table.len() - 1;
                let h = key_hash(k);
                let mut i = (h as usize) & mask;
                loop {
                    let slot = &table[i];
                    if slot.pos == VACANT {
                        return None;
                    }
                    if slot.h == h && key_eq(self.seen[slot.pos].0, k) {
                        return Some(self.seen[slot.pos].1);
                    }
                    i = (i + 1) & mask;
                }
            }
        }
    }

    /// Record a fresh (never-seen-here) child key. Builds the index
    /// lazily the first time `seen` outgrows `LINEAR_MAX`.
    pub(super) fn seen_insert(&mut self, bump: &'a Bump, k: &'a str, target: usize) {
        self.seen.push((k, target));
        match self.index.as_mut() {
            Some(table) => {
                if self.seen.len() * 4 > table.len() * 3 {
                    Self::index_grow(table, bump);
                }
                Self::index_place(table, key_hash(k), self.seen.len() - 1);
            }
            None => {
                if self.seen.len() > LINEAR_MAX {
                    let mut table = BumpVec::with_capacity_in(INDEX_MIN_SLOTS, bump);
                    for _ in 0..INDEX_MIN_SLOTS {
                        table.push(SeenSlot { h: 0, pos: VACANT });
                    }
                    for pos in 0..self.seen.len() {
                        Self::index_place(&mut table, key_hash(self.seen[pos].0), pos);
                    }
                    self.index = Some(table);
                }
            }
        }
    }

    /// Place `(h, pos)` at the first vacant slot of its probe sequence.
    /// Always terminates: `seen_insert` keeps load at or under 3/4, so
    /// a vacant slot always exists.
    fn index_place(table: &mut BumpVec<'_, SeenSlot>, h: u32, pos: usize) {
        let mask = table.len() - 1;
        let mut i = (h as usize) & mask;
        while table[i].pos != VACANT {
            i = (i + 1) & mask;
        }
        table[i] = SeenSlot { h, pos };
    }

    /// Double the table, re-probing existing slots with their cached
    /// hashes (no re-hashing of keys). The old table's bump memory is
    /// not reclaimed — standard bumpalo growth, bounded at 2x the
    /// final table.
    fn index_grow(table: &mut BumpVec<'a, SeenSlot>, bump: &'a Bump) {
        let mut fresh = BumpVec::with_capacity_in(table.len() * 2, bump);
        for _ in 0..table.len() * 2 {
            fresh.push(SeenSlot { h: 0, pos: VACANT });
        }
        for slot in table.iter() {
            if slot.pos != VACANT {
                Self::index_place(&mut fresh, slot.h, slot.pos);
            }
        }
        *table = fresh;
    }
}
