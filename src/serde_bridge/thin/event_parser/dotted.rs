use crate::error::{ConflictKind, Error, ErrorKind, Result, Span};
use crate::parser::inline::{decode_key_segment, key_is_single_segment, split_key_path};
use crate::parser::validate::{check_key, KeyValidity};
use crate::whitespace::is_inline_whitespace;

use super::super::{Event, EventSink};

use super::classify::{path_shape_of, BracketKind};
use super::state::{EventParser, Frame, NodeId, ObjectLevel, PathShape};

impl<'a> EventParser<'a> {
    // -----------------------------------------------------------------------
    // Dotted-key reconciliation
    // -----------------------------------------------------------------------

    /// Returns `(leaf_segment, parent_node_id)`: the node under which
    /// the leaf value's segment will be registered (the current frame's
    /// `frame_root` for single-segment keys, or the deepest prefix node
    /// for dotted keys).
    pub(super) fn reconcile_dotted_key<S: EventSink<'a>>(
        &mut self,
        key: &'a str,
        line_num: usize,
        key_span: Span,
        events: &mut S,
    ) -> Result<(&'a str, NodeId)> {
        // Single segment (no UNescaped `.`) fast path. Decode the
        // segment if it contains a `\`; otherwise reuse the source
        // borrow.
        if key_is_single_segment(key) {
            self.close_synthetics_to_real(events);
            // Validate the RAW segment (forbidden bytes must be
            // escaped, quoted segments checked against their own
            // class) before decoding — see `parser::validate`.
            match check_key(key) {
                KeyValidity::Valid => {}
                KeyValidity::Empty => {
                    return Err(Error::Structured(ErrorKind::EmptyKey {
                        line: line_num as u32,
                        span: key_span,
                    }));
                }
                KeyValidity::Invalid => {
                    return Err(Error::Structured(ErrorKind::InvalidKey {
                        line: line_num as u32,
                        key: key.to_string(),
                        span: key_span,
                    }));
                }
            }
            let leaf = self.decode_key_in_arena(key, line_num, key_span)?;
            let frame_root = match self.stack.last() {
                Some(Frame::Object { frame_root, .. }) => *frame_root,
                _ => unreachable!("dispatched as object"),
            };
            return Ok((leaf, frame_root));
        }

        // Multi-segment path — split on UNescaped `.`, decode each
        // segment (arena-allocated if it had `\`).
        let raw_segments = split_key_path(key);
        debug_assert!(raw_segments.len() >= 2);
        let mut decoded_segments: Vec<&'a str> = Vec::with_capacity(raw_segments.len());
        for seg in &raw_segments {
            // `seg` is a `split_key_path` key segment; the twin main engine
            // (parser/insert.rs key-path walk) trims these with the INLINE
            // view — § 3.2 pre-splits lines, so segments are LF/CR-free.
            let trimmed = seg.trim_matches(is_inline_whitespace);
            // Empty segment → EmptyKey (`a..b`, leading/trailing `.`;
            // spec 0.7 § 6.5 names `a..b` explicitly as EmptyKey).
            match check_key(trimmed) {
                KeyValidity::Valid => {}
                KeyValidity::Empty => {
                    return Err(Error::Structured(ErrorKind::EmptyKey {
                        line: line_num as u32,
                        span: key_span,
                    }));
                }
                KeyValidity::Invalid => {
                    return Err(Error::Structured(ErrorKind::InvalidKey {
                        line: line_num as u32,
                        key: key.to_string(),
                        span: key_span,
                    }));
                }
            }
            let decoded = self.decode_key_in_arena(trimmed, line_num, key_span)?;
            decoded_segments.push(decoded);
        }

        let leaf = *decoded_segments.last().unwrap();

        // Spec 0.7 § 5.3.2: a dotted key re-entering an Object that
        // already exists — whether created by an earlier dotted pair or
        // explicitly as `a: { … }` / `a: {}` — MUST merge, regardless
        // of intervening sibling pairs. Descend the shared node arena
        // from the frame's `frame_root`: every proper prefix must
        // either already be an Object node (fine — merge) or be absent
        // (create it); descending through a Leaf is a BlockedByValue
        // conflict (§ 6.3). The arena is NOT per synthetic level, so a
        // prefix closed by an intervening sibling stays visible here and
        // duplicate detection beneath a reopened prefix stays exact.
        // Identity is `(parent node id, decoded segment)` — segments
        // are never joined into a `.`-separated string, because decoded
        // segments may contain literal dots.
        let frame_root = match self.stack.last() {
            Some(Frame::Object { frame_root, .. }) => *frame_root,
            _ => unreachable!("dispatched as object"),
        };
        let mut cur = frame_root;
        let mut prefix_existed = vec![false; decoded_segments.len()];
        for k in 0..decoded_segments.len() - 1 {
            let seg = decoded_segments[k];
            match self.probe(cur, seg) {
                Some(id) if matches!(self.node_shape(id), PathShape::Leaf(_)) => {
                    return Err(Error::Structured(ErrorKind::KeyPathConflict {
                        line: line_num as u32,
                        // RAW key text (escapes intact), matching the
                        // owned parser's `full_path` reporting.
                        path: key.to_string(),
                        kind: ConflictKind::BlockedByValue,
                        span: key_span,
                    }));
                }
                Some(id) => {
                    prefix_existed[k + 1] = true;
                    cur = id;
                }
                None => {
                    cur = self.add_child(cur, seg, PathShape::Object);
                }
            }
        }

        let prefix_segments = &decoded_segments[..decoded_segments.len() - 1];

        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => unreachable!("dispatched as object"),
        };

        let mut lcp_count: usize = 0;
        let mut pending_seg_idx: Option<usize> = None;

        for (i, seg) in prefix_segments.iter().enumerate() {
            if lcp_count + 1 >= cur_levels_len {
                pending_seg_idx = Some(i);
                break;
            }
            let cur_prefix = match self.stack.last().unwrap() {
                Frame::Object { levels, .. } => levels[1 + lcp_count].prefix.unwrap(),
                _ => unreachable!(),
            };
            if *seg != cur_prefix {
                pending_seg_idx = Some(i);
                break;
            }
            lcp_count += 1;
        }

        let pops = cur_levels_len - 1 - lcp_count;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }

        let push_start = pending_seg_idx.unwrap_or(prefix_segments.len());
        for (i, seg) in prefix_segments.iter().enumerate().skip(push_start) {
            if prefix_existed[i + 1] {
                // The prefix was closed by intervening siblings (or was
                // an explicit `a: { … }`) and is being re-opened as a
                // synthetic — a § 5.3.2 merge site the deserializer's
                // merge pass must fold back together.
                self.reopens += 1;
            }
            self.push_synthetic(seg, events);
        }

        Ok((leaf, cur))
    }

    /// Decode a key segment per § 3.7 / § 5.3.3. Bare segments without
    /// a `\` return the source slice as-is (zero-copy fast path), as do
    /// quoted segments (§ 5.3.3) whose interior contains no `\` — the
    /// key is then the source slice between the delimiters, never
    /// trimmed. Any segment containing a `\` decodes via
    /// `decode_key_segment` and is allocated in the bump arena so the
    /// returned `&'a str` outlives the call.
    fn decode_key_in_arena(
        &self,
        seg: &'a str,
        line_num: usize,
        key_span: Span,
    ) -> Result<&'a str> {
        let is_quoted = seg
            .as_bytes()
            .first()
            .is_some_and(|&b| b == b'"' || b == b'\'' || b == b'`');
        if !is_quoted && !seg.as_bytes().contains(&b'\\') {
            return Ok(seg);
        }
        if is_quoted {
            debug_assert!(seg.len() >= 2 && seg.as_bytes()[seg.len() - 1] == seg.as_bytes()[0]);
            let interior = &seg[1..seg.len() - 1];
            if !interior.as_bytes().contains(&b'\\') {
                // Validated unescaped quoted interior: the key IS the source
                // slice between the quotes (§ 5.3.3 — quoted content is never
                // trimmed). No temporary String, no bump copy.
                return Ok(interior);
            }
        }
        let decoded = decode_key_segment(seg, line_num, key_span)?;
        Ok(self.bump.alloc_str(&decoded))
    }

    #[inline]
    fn push_synthetic<S: EventSink<'a>>(&mut self, seg: &'a str, events: &mut S) {
        // Emission-only: key state for the synthetic prefix is already
        // recorded in the shared node arena/index (or about to be by
        // `register_value_path`), so nothing can fail here.
        events.push(Event::Key(seg));
        events.push(Event::BeginObject);
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => levels.push(ObjectLevel { prefix: Some(seg) }),
            _ => unreachable!(),
        }
    }

    pub(super) fn close_synthetics_to_real<S: EventSink<'a>>(&mut self, events: &mut S) {
        let cur_levels_len = match self.stack.last().unwrap() {
            Frame::Object { levels, .. } => levels.len(),
            _ => return,
        };
        let pops = cur_levels_len - 1;
        for _ in 0..pops {
            self.pop_synthetic_level(events);
        }
    }

    pub(crate) fn close_synthetics_until<S: EventSink<'a>>(
        &mut self,
        target_synthetic_count: usize,
        events: &mut S,
    ) {
        loop {
            let cur = match self.stack.last() {
                Some(Frame::Object { levels, .. }) => levels.len() - 1,
                _ => return,
            };
            if cur <= target_synthetic_count {
                return;
            }
            self.pop_synthetic_level(events);
        }
    }

    fn pop_synthetic_level<S: EventSink<'a>>(&mut self, events: &mut S) {
        match self.stack.last_mut().unwrap() {
            Frame::Object { levels, .. } => {
                levels.pop();
                events.push(Event::EndObject);
            }
            _ => unreachable!(),
        }
    }

    /// Register a fully-decoded leaf segment under `parent_node` with
    /// the value shape that now occupies it, implementing the owned
    /// parser's outcome tables (`parser::insert::insert_value` /
    /// `insert_dotted`, § 6.3). The parent node is looked up in the
    /// shared hash index — no path scan. The two entry points differ on
    /// an OCCUPIED slot:
    ///
    /// - Dotted key (`parent_node` deeper than the frame's
    ///   `frame_root`, mirrors `insert_dotted`): the descent was
    ///   already validated per-prefix by `reconcile_dotted_key`, so
    ///   ANY occupied final segment is a `DuplicateKey` —
    ///   unconditionally, regardless of shape.
    /// - Single-segment key (`parent_node == frame_root`, mirrors
    ///   `insert_value`'s four-arm table):
    ///
    ///   - leaf value onto existing Object → `KeyPathConflict`
    ///     `Overwrite { existing: "object", new_kind }`
    ///   - leaf value onto existing leaf → `DuplicateKey`
    ///   - Object onto existing Object → `DuplicateKey`
    ///   - Object onto existing leaf → `KeyPathConflict`
    ///     `Overwrite { existing: <kind>, new_kind: "object" }`
    ///
    /// - absent slot → record it. Returns the registered node's id.
    pub(super) fn register_value_path(
        &mut self,
        parent_node: NodeId,
        leaf: &'a str,
        shape: PathShape,
        raw_key: &str,
        line_num: usize,
        key_span: Span,
    ) -> Result<NodeId> {
        let frame_root = match self.stack.last() {
            Some(Frame::Object { frame_root, .. }) => *frame_root,
            _ => unreachable!("only objects have keys"),
        };
        // Single-segment keys have parent == frame_root; every dotted
        // key and every inline-compound child has a deeper parent.
        let dotted = parent_node != frame_root;
        match self.probe(parent_node, leaf) {
            // `insert_dotted` parity: an occupied final segment
            // of a dotted key is ALWAYS a DuplicateKey — shape
            // conflicts along the way were already raised per-
            // prefix by `reconcile_dotted_key`.
            Some(_) if dotted => Err(Error::Structured(ErrorKind::DuplicateKey {
                line: line_num as u32,
                key: raw_key.to_string(),
                span: key_span,
            })),
            Some(id) => {
                let existing = self.node_shape(id);
                match existing {
                    PathShape::Object => match shape {
                        PathShape::Leaf(label) => {
                            Err(Error::Structured(ErrorKind::KeyPathConflict {
                                line: line_num as u32,
                                path: raw_key.to_string(),
                                kind: ConflictKind::Overwrite {
                                    existing: "object",
                                    new_kind: label,
                                },
                                span: key_span,
                            }))
                        }
                        PathShape::Object => Err(Error::Structured(ErrorKind::DuplicateKey {
                            line: line_num as u32,
                            key: raw_key.to_string(),
                            span: key_span,
                        })),
                    },
                    PathShape::Leaf(existing) => match shape {
                        PathShape::Leaf(_) => Err(Error::Structured(ErrorKind::DuplicateKey {
                            line: line_num as u32,
                            key: raw_key.to_string(),
                            span: key_span,
                        })),
                        PathShape::Object => Err(Error::Structured(ErrorKind::KeyPathConflict {
                            line: line_num as u32,
                            path: raw_key.to_string(),
                            kind: ConflictKind::Overwrite {
                                existing,
                                new_kind: "object",
                            },
                            span: key_span,
                        })),
                    },
                }
            }
            None => Ok(self.add_child(parent_node, leaf, shape)),
        }
    }

    /// Register the INTERNAL key paths of an inline compound value
    /// (`a: {x: 1}` — events staged by the direct inline scanner)
    /// into the shared node arena, under `base_node` (the node just
    /// registered for the compound itself). Single pass (review
    /// R7-F3): a stack of currently-open object `NodeId`s descends
    /// nested objects as their Begin events pass, and a bracket-array
    /// depth excludes array interiors — arrays are leaves, nothing
    /// inside a bracketed array is registered (§ 5.3.2 / § 6.3).
    ///
    /// Every staged event is examined exactly once — expected `O(E)`
    /// over the compound's `E` staged events plus key hashing in
    /// `register_value_path` — where the previous walk re-scanned each
    /// nested object's whole event range per ancestor
    /// (`matching_bracket` + recursion), `Theta(D^2)` for a chain of
    /// `D` dotted-expansion levels.
    ///
    /// Registration is provably collision-free: the inline events were
    /// already validated internally by the shared `insert_value`
    /// tables during the direct scan (each path appears exactly once), and `base_node`
    /// was just inserted absent. Errors are still propagated with `?`
    /// defensively rather than panicking.
    ///
    /// The walk visits `(parent, key)` registration calls in the same
    /// pre-order — event order — as the replaced recursive walk, so
    /// every conflict diagnostic (`ErrorKind` / line / span / payload)
    /// is unchanged.
    pub(super) fn register_inline_child_paths(
        &mut self,
        base_node: NodeId,
        events: &[Event<'a>],
        line_num: usize,
        key_span: Span,
    ) -> Result<()> {
        debug_assert!(matches!(events.first(), Some(Event::BeginObject)));
        debug_assert!(matches!(events.last(), Some(Event::EndObject)));
        let inner = &events[1..events.len() - 1];
        let mut node_stack = std::mem::take(&mut self.path_node_stack);
        node_stack.clear();
        node_stack.push(base_node);
        let result = self.register_inline_child_walk(inner, &mut node_stack, line_num, key_span);
        self.path_node_stack = node_stack;
        result
    }

    /// The [`EventParser::register_inline_child_paths`] walk proper,
    /// split out so the reusable `path_node_stack` buffer is restored
    /// even when a registration conflict errors out mid-walk.
    fn register_inline_child_walk(
        &mut self,
        inner: &[Event<'a>],
        node_stack: &mut Vec<NodeId>,
        line_num: usize,
        key_span: Span,
    ) -> Result<()> {
        // Bracketed-array interiors are leaves in the path model:
        // `array_depth > 0` suppresses registration and node
        // push/pop until the matching closer (§ 5.3.2 / § 6.3).
        let mut array_depth: usize = 0;
        let mut i = 0;
        while i < inner.len() {
            self.dbg_reg_scans += 1;
            if array_depth > 0 {
                match inner[i] {
                    Event::BeginArray => array_depth += 1,
                    Event::EndArray => array_depth -= 1,
                    _ => {}
                }
                i += 1;
                continue;
            }
            match inner[i] {
                Event::Key(k) => {
                    // Each Key is immediately followed by exactly one
                    // value event or a bracketed compound — guaranteed
                    // by the direct inline scanner.
                    self.dbg_reg_scans += 1;
                    let value_ev = &inner[i + 1];
                    let shape = path_shape_of(value_ev);
                    let parent = match node_stack.last() {
                        Some(&p) => p,
                        None => unreachable!("object node stack underflow"),
                    };
                    let child_node =
                        self.register_value_path(parent, k, shape, k, line_num, key_span)?;
                    if matches!(value_ev, Event::BeginObject) {
                        node_stack.push(child_node);
                    } else if matches!(value_ev, Event::BeginArray) {
                        array_depth = 1;
                    }
                    i += 2;
                }
                Event::EndObject => {
                    debug_assert!(node_stack.len() > 1, "unbalanced object node stack");
                    node_stack.pop();
                    i += 1;
                }
                other => unreachable!("pair position must be Key, got {other:?}"),
            }
        }
        debug_assert_eq!(node_stack.len(), 1, "unbalanced object node stack");
        debug_assert_eq!(array_depth, 0, "unterminated inline array");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Frame close
    // -----------------------------------------------------------------------

    pub(super) fn close_frame<S: EventSink<'a>>(
        &mut self,
        expected: BracketKind,
        line_num: usize,
        trimmed_span: Span,
        events: &mut S,
    ) -> Result<()> {
        // Depth-1 close of a lone-`{`/`[`-opened root (§ 5.0.1 rules
        // 4-5): a matching close consumes the root — the frame is
        // popped, its End event emitted, and `root_consumed` set
        // (mirrors owned parser close_frame). Mismatched kind at this
        // depth errors against the FRAME's kind.
        if self.stack.len() == 1 && self.root_is_explicit_compound {
            let frame_kind = match self.stack.last() {
                Some(Frame::Object { .. }) => BracketKind::Object,
                _ => BracketKind::Array,
            };
            if frame_kind as u8 != expected as u8 {
                return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                    line: line_num as u32,
                    span: trimmed_span,
                    expected: frame_kind.to_compound(),
                    found: expected.close(),
                }));
            }
            if matches!(self.stack.last(), Some(Frame::Object { .. })) {
                self.close_synthetics_to_real(events);
            }
            let got = match self.stack.pop().unwrap() {
                Frame::Object { .. } => BracketKind::Object,
                Frame::Array => BracketKind::Array,
            };
            let _ = self.opener_offsets.pop();
            self.root_consumed = true;
            let close_event = match got {
                BracketKind::Object => Event::EndObject,
                BracketKind::Array => Event::EndArray,
            };
            events.push(close_event);
            return Ok(());
        }
        if self.stack.len() <= 1 {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: expected.to_compound(),
                found: expected.close(),
            }));
        }
        if matches!(self.stack.last(), Some(Frame::Object { .. })) {
            self.close_synthetics_to_real(events);
        }
        let popped = self.stack.pop().unwrap();
        let got = match &popped {
            Frame::Object { .. } => BracketKind::Object,
            Frame::Array => BracketKind::Array,
        };
        let _ = self.opener_offsets.pop();
        if got as u8 != expected as u8 {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: got.to_compound(),
                found: expected.close(),
            }));
        }
        // § 5.3.2: no fold is needed on close. The child frame's
        // `frame_root` IS the node of its own key path in the shared
        // global node arena/index; its top-level entries are keyed
        // `(frame_root, segment)` — exactly the key the parent's
        // dotted-key descent would probe. The child's subtree is
        // therefore already linked at the parent's descent point and
        // visible to later `a.x: 2` re-entry with no copying. Any
        // collision the old defensive fold arm claimed to catch is
        // impossible: the shared index enforces `(parent, segment)`
        // node identity at insertion time.
        let close_event = match got {
            BracketKind::Object => Event::EndObject,
            BracketKind::Array => Event::EndArray,
        };
        events.push(close_event);
        Ok(())
    }
}
