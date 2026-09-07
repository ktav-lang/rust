//! Reopen-merge normalization for the deserialization event path.
//!
//! The thin event parser emits a dotted-key Object re-opened after an
//! intervening sibling (spec 0.7 § 5.3.2) as a separate `Key` +
//! `BeginObject` … `EndObject` block at its own document position — the
//! raw stream never re-opens an already-closed compound. That is the
//! documented contract for the public [`parse_events`](super) callback.
//!
//! The serde [`EventDeserializer`](super::event_deserializer), however,
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

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use rustc_hash::FxHasher;

use super::event::{Event, EventStream};

#[cfg(test)]
mod perf {
    use std::cell::Cell;

    thread_local! {
        static KEY_COMPARISONS: Cell<usize> = const { Cell::new(0) };
    }

    pub fn record() {
        KEY_COMPARISONS.with(|c| c.set(c.get() + 1));
    }

    pub fn get() -> usize {
        KEY_COMPARISONS.with(Cell::get)
    }

    pub fn reset() {
        KEY_COMPARISONS.with(|c| c.set(0));
    }
}

/// One flattened item in a rebuild buffer: either a leaf event, or a
/// nested compound whose contents live in buffer `buf` (`open` is the
/// `BeginObject` / `BeginArray`; the matching close is derived at
/// flatten time).
enum Item<'a> {
    Ev(Event<'a>),
    Obj { buf: usize, open: Event<'a> },
}

/// A rebuild buffer. `seen` maps each child key to the buffer of its
/// FIRST object block, so a later `Key(k)` + `BeginObject` with the same
/// key is detected as a § 5.3.2 reopen and merged into that
/// buffer. `seen` stays the order-carrying source of truth
/// (first-appearance order); `index` is a lazily built open-addressing
/// index over it, `None` while the buffer is small.
struct Buf<'a> {
    items: BumpVec<'a, Item<'a>>,
    seen: BumpVec<'a, (&'a str, usize)>,
    index: Option<BumpVec<'a, SeenSlot>>,
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
struct SeenSlot {
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
    fn seen_find(&self, k: &str) -> Option<usize> {
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
    fn seen_insert(&mut self, bump: &'a Bump, k: &'a str, target: usize) {
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

/// Flatten work item: append `Ev` to the output, or splice in buffer
/// `buf` bracketed by `open` / its matching close.
enum Work<'a> {
    Ev(Event<'a>),
    /// Emit `open`, splice buffer `buf`, then emit `close`.
    Enter {
        buf: usize,
        open: Event<'a>,
        close: Event<'a>,
    },
}

fn matching_close(open: Event<'_>) -> Event<'_> {
    match open {
        Event::BeginObject => Event::EndObject,
        Event::BeginArray => Event::EndArray,
        _ => unreachable!("open is always BeginObject/BeginArray"),
    }
}

fn is_open(ev: &Event<'_>) -> bool {
    matches!(ev, Event::BeginObject | Event::BeginArray)
}

/// Rebuild `src` folding re-opened object blocks into the buffer of
/// their first appearance (spec 0.7 § 5.3.2). Merging only removes
/// events, so the output is pre-sized to `src.len()`.
pub(crate) fn merge_reopened<'a>(src: &EventStream<'a>, bump: &'a Bump) -> EventStream<'a> {
    let mut bufs: Vec<Buf<'a>> = vec![Buf {
        items: BumpVec::with_capacity_in(src.len(), bump),
        seen: BumpVec::with_capacity_in(8, bump),
        index: None,
    }];
    // Index 0 is the document root, pre-created above. `root_open`
    // records its kind (implicit or explicit § 5.0.1 root) so the
    // flatten step can re-emit the bracket pair around the root.
    let mut root_open: Option<Event<'a>> = None;
    // Explicit stack of open buffer indices; the root frame is pushed
    // when the bare root opener is seen.
    let mut stack: Vec<usize> = Vec::with_capacity(8);

    let mut i = 0;
    while i < src.len() {
        match src[i] {
            // A bare Begin with no preceding Key is the document root
            // opener (§ 5.0.1). Its matching close pops the root frame.
            ev @ (Event::BeginObject | Event::BeginArray) if stack.is_empty() => {
                root_open = Some(ev);
                stack.push(0);
                i += 1;
            }
            // A bare Begin with a frame already open is an anonymous
            // compound value (an array element — only array elements
            // lack a key). It is never a § 5.3.2 reopen target (the
            // parser's persistent path table only tracks keyed paths),
            // so it always gets a fresh child buffer — same treatment
            // as the keyed array-valued arm below, minus the `seen`
            // entry and the `Key` item.
            ev @ (Event::BeginObject | Event::BeginArray) => {
                let idx = bufs.len();
                bufs.push(new_buf(bump, src.len()));
                bufs[stack_top(&stack)]
                    .items
                    .push(Item::Obj { buf: idx, open: ev });
                stack.push(idx);
                i += 1;
            }
            Event::Key(k) => {
                let value = src.get(i + 1).copied();
                match value {
                    // Key + compound opener.
                    Some(open @ Event::BeginArray) => {
                        // Arrays never merge: keys cannot repeat inside
                        // an array frame and array-valued keys are
                        // rejected as duplicates by the parser (§ 6.3),
                        // so this is always fresh. No `seen` entry.
                        let idx = bufs.len();
                        bufs.push(new_buf(bump, src.len()));
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        bufs[stack_top(&stack)]
                            .items
                            .push(Item::Obj { buf: idx, open });
                        stack.push(idx);
                        i += 2;
                    }
                    Some(open @ Event::BeginObject) => {
                        let top = stack_top(&stack);
                        match bufs[top].seen_find(k) {
                            Some(target) => {
                                // § 5.3.2 reopen: the persistent path
                                // table in the parser guarantees a seen
                                // hit is Object-shaped (reopens of
                                // leaf/scalar/array blocks are rejected
                                // as conflicts before any events are
                                // produced). Splice nothing here — the
                                // block's content walks into the target
                                // buffer, and its EndObject just pops
                                // this frame.
                                stack.push(target);
                            }
                            None => {
                                let idx = bufs.len();
                                bufs.push(new_buf(bump, src.len()));
                                bufs[top].seen_insert(bump, k, idx);
                                bufs[top].items.push(Item::Ev(Event::Key(k)));
                                bufs[top].items.push(Item::Obj { buf: idx, open });
                                stack.push(idx);
                            }
                        }
                        i += 2;
                    }
                    // Key + scalar.
                    Some(scalar) => {
                        debug_assert!(!is_open(&scalar));
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        bufs[stack_top(&stack)].items.push(Item::Ev(scalar));
                        i += 2;
                    }
                    // Malformed stream cannot happen from the parser;
                    // keep the last Key harmlessly in release.
                    None => {
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        i += 1;
                    }
                }
            }
            // Bare closers close one frame; any other bare event (a
            // scalar array item) lands in the top buffer.
            Event::EndObject | Event::EndArray => {
                stack.pop();
                i += 1;
            }
            ev => {
                bufs[stack_top(&stack)].items.push(Item::Ev(ev));
                i += 1;
            }
        }
    }

    // Flatten depth-first with an explicit work stack.
    let mut out: EventStream<'a> = BumpVec::with_capacity_in(src.len(), bump);
    if let Some(open) = root_open {
        out.push(open);
        let mut work: Vec<Work<'a>> = Vec::with_capacity(16);
        push_buf_items(&mut work, &bufs[0]);
        while let Some(w) = work.pop() {
            match w {
                Work::Ev(ev) => out.push(ev),
                Work::Enter { buf, open, close } => {
                    out.push(open);
                    // Children first, then the matching close: push the
                    // close first so it is popped after the (reversed)
                    // children are exhausted.
                    work.push(Work::Ev(close));
                    push_buf_items(&mut work, &bufs[buf]);
                }
            }
        }
        out.push(matching_close(open));
    }
    out
}

fn new_buf<'a>(bump: &'a Bump, hint: usize) -> Buf<'a> {
    Buf {
        items: BumpVec::with_capacity_in(hint.min(64), bump),
        seen: BumpVec::with_capacity_in(8, bump),
        index: None,
    }
}

#[inline]
fn stack_top(stack: &[usize]) -> usize {
    *stack
        .last()
        .expect("stream is balanced; root frame always open")
}

/// One `&str` equality between two child keys. Counts comparisons in
/// test builds so tests can assert lookup-cost growth deterministically.
#[inline]
fn key_eq(a: &str, b: &str) -> bool {
    #[cfg(test)]
    perf::record();
    a == b
}

/// Push `buf`'s items onto the work stack in reverse so they flatten in
/// document order.
fn push_buf_items<'a>(work: &mut Vec<Work<'a>>, b: &Buf<'a>) {
    for item in b.items.iter().rev() {
        work.push(match item {
            Item::Ev(ev) => Work::Ev(*ev),
            Item::Obj { buf, open } => Work::Enter {
                buf: *buf,
                open: *open,
                close: matching_close(*open),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin::event_parser::parse_events;

    /// Post-fix oracle for the key-comparison counter. K distinct
    /// Object-valued siblings plus one reopen: K = 8 never builds the
    /// index, so its cost is EXACTLY the pre-fix linear-mode cost (29);
    /// above `LINEAR_MAX` the index keeps the count linear in K — the
    /// pre-fix counts were quadratic (121 / 497 / 2017 for K = 16 / 32 /
    /// 64). Only str comparisons count; hashing and probing over
    /// occupied-with-different-hash or vacant slots do not.
    #[test]
    fn key_comparison_counts_stay_linear() {
        use std::fmt::Write as _;

        let mut prev = None;
        for k in [8_usize, 16, 32, 64] {
            let mut text = String::new();
            for i in 0..k {
                let _ = writeln!(text, "a{i}.x: 1");
            }
            text.push_str("a0.y: 2\n");

            let bump = Bump::new();
            let (stream, _) = parse_events(&text, &bump).unwrap();
            perf::reset();
            let _ = merge_reopened(&stream, &bump);
            let count = perf::get();
            eprintln!("K = {k}: {count} key comparisons");
            if k == 8 {
                assert_eq!(count, 29, "K = 8 must keep the exact linear-mode cost");
            } else {
                assert!(
                    count <= 4 * k + 16,
                    "K = {k}: {count} comparisons exceed the linear bound"
                );
            }
            if let Some(p) = prev {
                assert!(
                    count < 3 * p,
                    "K = {k}: {count} grows super-linearly vs {p}"
                );
            }
            prev = Some(count);
        }
    }

    fn json_of(text: &str) -> String {
        let v: serde_json::Value = crate::from_str(text).unwrap();
        serde_json::to_string(&v).unwrap()
    }

    #[test]
    fn value_representative_three_siblings_one_reopen() {
        assert_eq!(
            json_of("a0.x: 1\na1.x: 1\na2.x: 1\na0.y: 2\n"),
            r#"{"a0":{"x":1,"y":2},"a1":{"x":1},"a2":{"x":1}}"#
        );
    }

    #[test]
    fn value_twelve_siblings_reopens_crossing_the_index_threshold() {
        let mut text = String::new();
        let mut want = String::from("{");
        for i in 0..12 {
            let _ = std::fmt::Write::write_fmt(&mut text, format_args!("a{i}.x: 1\n"));
            let _ = std::fmt::Write::write_fmt(&mut want, format_args!("\"a{i}\":{{\"x\":1"));
            if i == 3 {
                want.push_str(",\"y\":2");
            }
            if i == 9 {
                want.push_str(",\"z\":3");
            }
            want.push('}');
            if i < 11 {
                want.push(',');
            }
        }
        text.push_str("a3.y: 2\na9.z: 3\n");
        want.push('}');
        assert_eq!(json_of(&text), want);
    }

    #[test]
    fn value_reopen_inside_nested_object() {
        assert_eq!(
            json_of("o: {\na.x: 1\np: 2\na.y: 3\n}\n"),
            r#"{"o":{"a":{"x":1,"y":3},"p":2}}"#
        );
    }

    #[test]
    fn value_anonymous_array_items_with_reopen() {
        assert_eq!(
            json_of("a.x: 1\nitems: [[2], {b: 3}]\na.y: 4\n"),
            r#"{"a":{"x":1,"y":4},"items":[[2],{"b":3}]}"#
        );
    }

    #[test]
    fn event_indexed_merge_twelve_siblings_two_reopens() {
        let mut text = String::new();
        for i in 0..12 {
            text.push_str(&format!("a{i}.x: 1\n"));
        }
        text.push_str("a3.y: 2\na9.z: 3\n");

        let bump = Bump::new();
        let mut want: Vec<Event> = vec![BeginObject];
        for i in 0..12 {
            let key: &'static str = Box::leak(format!("a{i}").into_boxed_str());
            want.push(Key(key));
            want.push(BeginObject);
            want.push(Key("x"));
            want.push(Integer("1"));
            if i == 3 {
                want.push(Key("y"));
                want.push(Integer("2"));
            }
            if i == 9 {
                want.push(Key("z"));
                want.push(Integer("3"));
            }
            want.push(EndObject);
        }
        want.push(EndObject);
        assert_eq!(merged(&text, &bump), want);
    }

    fn raw<'a>(text: &'a str, bump: &'a Bump) -> Vec<Event<'a>> {
        let (stream, _) = parse_events(text, bump).unwrap();
        stream.into_iter().collect()
    }

    fn merged<'a>(text: &'a str, bump: &'a Bump) -> Vec<Event<'a>> {
        let (stream, _) = parse_events(text, bump).unwrap();
        merge_reopened(&stream, bump).into_iter().collect()
    }

    macro_rules! check {
        ($text:expr, $want:expr) => {{
            let bump = Bump::new();
            let want: Vec<Event> = $want;
            assert_eq!(merged($text, &bump), want);
        }};
    }

    use Event::{BeginArray, BeginObject, EndArray, EndObject, Integer, Key};

    /// Anonymous compound array items survive the merge with the
    /// § 5.3.2 reopen (`a.y` after the array) folded into the first
    /// `a` block.
    #[test]
    fn array_in_array_with_reopen() {
        check!(
            "a.x: 1\nitems: [[2]]\na.y: 3\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("items"),
                BeginArray,
                BeginArray,
                Integer("2"),
                EndArray,
                EndArray,
                EndObject,
            ]
        );
    }

    #[test]
    fn object_in_array_with_reopen() {
        check!(
            "a.x: 1\nitems: [{b: 2}]\na.y: 3\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("items"),
                BeginArray,
                BeginObject,
                Key("b"),
                Integer("2"),
                EndObject,
                EndArray,
                EndObject,
            ]
        );
    }

    /// The corruption site (anonymous compound) lives in a different
    /// branch than the reopens.
    #[test]
    fn compound_item_in_other_branch_than_reopen() {
        check!(
            "a.x: 1\ns: 0\na.y: 3\nitems: [[2]]\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("s"),
                Integer("0"),
                Key("items"),
                BeginArray,
                BeginArray,
                Integer("2"),
                EndArray,
                EndArray,
                EndObject,
            ]
        );
    }

    /// Reopened dotted-key object inside a root array.
    #[test]
    fn reopen_inside_root_array() {
        check!(
            "[\n  {\n    a.x: 1\n    s: 2\n    a.y: 3\n  }\n]\n",
            vec![
                BeginArray,
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("s"),
                Integer("2"),
                EndObject,
                EndArray,
            ]
        );
    }

    /// Deeply nested anonymous compounds mixed with a keyed reopen.
    #[test]
    fn deeply_nested_anonymous_compounds_with_reopen() {
        check!(
            "a.x: 1\nt: [[{u: [2]}]]\na.y: 3\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("t"),
                BeginArray,
                BeginArray,
                BeginObject,
                Key("u"),
                BeginArray,
                Integer("2"),
                EndArray,
                EndObject,
                EndArray,
                EndArray,
                EndObject,
            ]
        );
    }

    /// Compound array items without reopens: the zero-copy fast path
    /// still returns the raw stream untouched, and the merge pass is
    /// the identity on it.
    #[test]
    fn compound_items_no_reopen_fast_path() {
        let text = "items: [[1], {b: 2}]\nz: 3\n";
        let bump = Bump::new();
        let (stream, reopens) = parse_events(text, &bump).unwrap();
        assert_eq!(reopens, 0);
        assert_eq!(
            stream,
            crate::thin::parse_events_merged(text, &bump).unwrap()
        );
        assert_eq!(stream, merge_reopened(&stream, &bump));
    }

    #[test]
    fn reopen_after_intervening_sibling_merges() {
        check!(
            "a.b: 1
c: 2
a.d: 3
",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("b"),
                Integer("1"),
                Key("d"),
                Integer("3"),
                EndObject,
                Key("c"),
                Integer("2"),
                EndObject,
            ]
        );
    }

    #[test]
    fn reopen_explicit_object_merges() {
        check!(
            "a: {\n    x: 1\n}\na.y: 2\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("2"),
                EndObject,
                EndObject,
            ]
        );
    }

    #[test]
    fn grouped_dotted_keys_unchanged() {
        check!(
            "a.b: 1\na.c: 2\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("b"),
                Integer("1"),
                Key("c"),
                Integer("2"),
                EndObject,
                EndObject,
            ]
        );
    }

    #[test]
    fn merge_is_identity_on_reopen_free_stream() {
        let text = "port: 8080\nhost: \"example.com\"\nlist: [1, 2, 3]\nnested: {\nx: {y: 1}\n}\n";
        let bump = Bump::new();
        let (stream, reopens) = parse_events(text, &bump).unwrap();
        assert_eq!(reopens, 0);
        assert_eq!(stream, merge_reopened(&stream, &bump));
    }

    #[test]
    fn nested_reopen_merges_innermost() {
        check!(
            "a.b.c: 1\nq: 2\na.b.d: 2\n",
            vec![
                BeginObject,
                Key("a"),
                BeginObject,
                Key("b"),
                BeginObject,
                Key("c"),
                Integer("1"),
                Key("d"),
                Integer("2"),
                EndObject,
                EndObject,
                Key("q"),
                Integer("2"),
                EndObject,
            ]
        );
    }

    #[test]
    fn reopen_inside_explicit_frame_merges() {
        check!(
            "o: {\na.x: 1\np: 2\na.y: 3\n}\n",
            vec![
                BeginObject,
                Key("o"),
                BeginObject,
                Key("a"),
                BeginObject,
                Key("x"),
                Integer("1"),
                Key("y"),
                Integer("3"),
                EndObject,
                Key("p"),
                Integer("2"),
                EndObject,
                EndObject,
            ]
        );
    }

    #[test]
    fn arrays_untouched() {
        let text = "a: [1, 2]\nb: 3\n";
        let bump = Bump::new();
        assert_eq!(merged(text, &bump), raw(text, &bump));
    }

    #[test]
    fn inline_roots_pass_through() {
        for text in ["{a: 1}\n", "[1]\n"] {
            let bump = Bump::new();
            assert_eq!(merged(text, &bump), raw(text, &bump));
        }
    }
}
