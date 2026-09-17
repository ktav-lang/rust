use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use rustc_hash::FxHashMap;

use super::super::event::Event;

// ---------------------------------------------------------------------------
// Parser state
// ---------------------------------------------------------------------------

/// Below this many registered `(parent, segment)` entries the parser
/// scans a small contiguous list instead of touching the hash map —
/// small documents (the common case) then pay no hashing and no heap
/// allocation at all. 8 matches the owned parser's per-frame
/// `ObjectMap` capacity precedent (`src/parser/frame.rs`).
pub(super) const LINEAR_INDEX_THRESHOLD: usize = 8;

pub(super) struct LinearEntry<'a> {
    parent: NodeId,
    segment: &'a str,
    id: NodeId,
}

pub(crate) struct EventParser<'a> {
    pub(crate) bump: &'a Bump,
    pub(crate) stack: Vec<Frame<'a>>,
    pub(crate) collecting: Option<Collecting<'a>>,
    /// Byte offset (in original input) of the opener that started each
    /// frame — one entry per open frame. The implicit root records `0`
    /// (it has no opener); an explicit `{`/`[`-opened root records the
    /// byte offset of its opener, mirroring parser.rs lines 167-186.
    pub(crate) opener_offsets: Vec<u32>,
    /// Byte offset of the `(` / `((` line that started a multi-line
    /// string, if one is currently being collected.
    pub(crate) multiline_opener: Option<u32>,
    /// `false` until the first content line is classified and the root
    /// frame pushed. Spec § 5.0.1 determines the root kind lazily, with
    /// no pre-scan — mirroring the owned parser.
    pub(crate) root_initialized: bool,
    /// Set after a top-level inline compound (§ 5.0.1 rules 2-3) or
    /// after the matching close of a lone-`{`/`[`-opened root (rules
    /// 4-5). Any further non-blank, non-comment line is then
    /// `OrphanLineAfterTopLevelInline`.
    pub(crate) root_consumed: bool,
    /// True when the root was opened by a lone `{` or `[` (§ 5.0.1
    /// rules 4-5); a depth-1 close then consumes the root instead of
    /// erroring.
    pub(crate) root_is_explicit_compound: bool,
    /// Number of dotted-key RE-OPENED prefixes emitted as synthetic
    /// `Key`+`BeginObject` pairs this parse — i.e. pushes of a prefix
    /// whose persistent path entry already existed as an Object before
    /// the current line (spec 0.7 § 5.3.2 merge case). Ordinary grouped
    /// dotted keys (`a.b: 1` then `a.c: 2`, no intervening sibling)
    /// re-use the still-open synthetic and do NOT count. `from_str`
    /// uses this to decide whether a reopen-merge pass is needed.
    pub(crate) reopens: usize,
    /// Reusable staging buffer for keyed inline compounds. The inline
    /// scanner emits the compound's events here FIRST (so its internal
    /// errors keep their precedence over dotted-key reconciliation and
    /// path registration), registration walks the staged list, and only
    /// then are the events flushed into the real stream in order. One
    /// buffer per parse, cleared and reused per compound — no per-compound
    /// heap allocation.
    pub(super) staging: Vec<Event<'a>>,
    /// Reusable object-node stack for the single-pass
    /// `register_inline_child_paths` walk (review R7-F3): holds the
    /// `NodeId` of each currently-open nested inline object, mirrored
    /// to the staged event bracket depth. One buffer per parse, taken,
    /// cleared and restored per compound — no per-compound heap
    /// allocation.
    pub(super) path_node_stack: Vec<NodeId>,
    /// Shared parse-wide arena of key-path node SHAPES: slot 0 is a
    /// sentinel
    /// detached root serving the implicit root frame; every other slot
    /// holds one decoded key segment's shape, indexed by `NodeId`.
    /// Identity is the `(parent node id, decoded segment)` pair,
    /// enforced by the shared `index` — segments are NEVER
    /// joined into a `.`-separated string, because decoded segments may
    /// themselves contain literal dots. Ordering lives in the event
    /// stream, not here.
    pub(crate) nodes: BumpVec<'a, PathShape>,
    /// Two-tier lookup index over `nodes`, keyed on `(parent, segment)`
    /// — ONE per parse, shared by all frames. Below
    /// [`LINEAR_INDEX_THRESHOLD`] registered entries it is a contiguous
    /// linear list (`linear`) with zero hashing and zero heap
    /// allocation, which is what small documents (the common case) hit;
    /// the first insert past the threshold spills once into a
    /// `FxHashMap` (`index`) for O(1) lookups on large documents.
    /// Identity is the `(parent node id, decoded segment)` pair —
    /// segments are NEVER joined into a `.`-separated string, because
    /// decoded segments may themselves contain literal dots. Ordering
    /// lives in the event stream, not here.
    pub(super) linear: BumpVec<'a, LinearEntry<'a>>,
    /// `None` until the linear tier spills (see [`LINEAR_INDEX_THRESHOLD`]).
    pub(super) index: Option<FxHashMap<(NodeId, &'a str), NodeId>>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_node_allocs: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_index_probes: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_entry_compares: usize,

    /// Regression control for review round 7 finding R7-F3: total
    /// number of staged `Event`s EXAMINED by
    /// `register_inline_child_paths` — the inline-compound child-path
    /// registration walk (loop positions, Key value peeks, and every
    /// `matching_bracket` scan). Unlike the index counters above, this
    /// measures the registration phase's own event re-scans.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) dbg_reg_scans: usize,
}

impl<'a> EventParser<'a> {
    pub(crate) fn new(bump: &'a Bump) -> Self {
        // The root frame is pushed lazily by `classify_root` once the
        // first content line is classified — no pre-scan. `finish`
        // falls back to an empty Object root if no content line was
        // ever encountered (§ 5.0.1 rule 1).
        let mut p = EventParser {
            bump,
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
            root_initialized: false,
            root_consumed: false,
            root_is_explicit_compound: false,
            reopens: 0,
            staging: Vec::new(),
            path_node_stack: Vec::new(),
            nodes: {
                let mut nodes = BumpVec::with_capacity_in(16, bump);
                // Sentinel detached root: `frame_root` of the implicit
                // root frame.
                nodes.push(PathShape::Object);
                nodes
            },
            linear: BumpVec::with_capacity_in(LINEAR_INDEX_THRESHOLD, bump),
            index: None,
            dbg_node_allocs: 0,
            dbg_index_probes: 0,
            dbg_entry_compares: 0,
            dbg_reg_scans: 0,
        };
        // The sentinel push above counts as a node allocation, so the
        // counter reflects the exact arena size.
        p.dbg_node_allocs += 1;
        p
    }

    pub(super) fn node_shape(&self, id: NodeId) -> PathShape {
        self.nodes[id as usize]
    }

    pub(super) fn probe(&mut self, parent: NodeId, segment: &str) -> Option<NodeId> {
        self.dbg_index_probes += 1;
        if let Some(index) = &self.index {
            self.dbg_entry_compares += 1;
            return index.get(&(parent, segment)).copied();
        }
        for i in 0..self.linear.len() {
            let e = &self.linear[i];
            self.dbg_entry_compares += 1;
            if e.parent == parent && e.segment == segment {
                return Some(e.id);
            }
        }
        None
    }

    pub(super) fn add_child(
        &mut self,
        parent: NodeId,
        segment: &'a str,
        shape: PathShape,
    ) -> NodeId {
        let id = u32::try_from(self.nodes.len()).expect("path node count overflow");
        self.nodes.push(shape);
        self.dbg_node_allocs += 1;
        if let Some(index) = &mut self.index {
            index.insert((parent, segment), id);
        } else if self.linear.len() >= LINEAR_INDEX_THRESHOLD {
            // Spill: build the hash from the retained linear entries.
            // The list itself is left in place — a few hundred bytes of
            // arena, never consulted again once `index` is `Some`.
            let mut map = FxHashMap::default();
            for e in &self.linear {
                map.insert((e.parent, e.segment), e.id);
            }
            map.insert((parent, segment), id);
            self.index = Some(map);
        } else {
            self.linear.push(LinearEntry {
                parent,
                segment,
                id,
            });
        }
        id
    }

    /// Detached root for frames whose key path lives in no enclosing
    /// object — array-element objects (arrays are leaves, so paths never
    /// cross an array boundary).
    pub(super) fn add_detached_root(&mut self) -> NodeId {
        let id = u32::try_from(self.nodes.len()).expect("path node count overflow");
        self.nodes.push(PathShape::Object);
        self.dbg_node_allocs += 1;
        id
    }
}

pub(crate) type NodeId = u32;

pub(crate) enum Frame<'a> {
    /// `levels` is parallel to "real frame + open synthetic prefixes".
    /// Index 0 is always the real object's namespace; subsequent entries
    /// are stacked synthetics. Key state lives in the parse-wide shared
    /// node arena/index, NOT in the frame: the frame only holds the
    /// `NodeId` under which its own entries are keyed, so a dotted
    /// prefix closed by intervening siblings can still be re-entered (it
    /// merges) and duplicate/conflict detection below a reopened prefix
    /// stays exact — with no per-frame table and no copying on close.
    Object {
        levels: BumpVec<'a, ObjectLevel<'a>>,
        /// The node of this object's own key path in the shared node-shape
        /// arena — top-level entries of the frame are keyed
        /// `(frame_root, segment)`. Node `0` (sentinel detached root)
        /// for root frames and for objects opened inside arrays (array
        /// elements are unnamed, and arrays are leaves in the path
        /// model, so paths never cross an array boundary).
        frame_root: NodeId,
    },
    Array,
}

/// Shape a registered key path holds, mirroring the owned parser's
/// `Value` kinds in `parser::insert` (§ 6.3 conflict classification).
#[derive(Clone, Copy)]
pub(crate) enum PathShape {
    /// The path was established as a nested Object.
    Object,
    /// The path was established as a leaf value; the label is the same
    /// kind string the owned parser's `kind_label` produces (used in
    /// `ConflictKind::Overwrite` diagnostics).
    Leaf(&'static str),
}

pub(crate) struct ObjectLevel<'a> {
    /// `None` for the real object level, `Some(prefix_segment)` for a
    /// synthetic dotted-key level. Kept ONLY for the LCP comparison and
    /// emission bookkeeping — all key state lives in the shared node
    /// arena/index.
    pub(super) prefix: Option<&'a str>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new_object(bump: &'a Bump, frame_root: NodeId) -> Self {
        let mut levels = BumpVec::with_capacity_in(2, bump);
        levels.push(ObjectLevel { prefix: None });
        Frame::Object { levels, frame_root }
    }
    pub(crate) fn new_array() -> Self {
        Frame::Array
    }
}

#[derive(Copy, Clone)]
pub(crate) enum MultilineMode {
    Stripped,
    Verbatim,
}

pub(crate) struct Collecting<'a> {
    pub(crate) mode: MultilineMode,
    pub(crate) lines: BumpVec<'a, &'a str>,
}
