use super::state::EventParser;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use memchr::memchr;

use super::super::EventStream;

/// Runs a full LF-only parse (duplicating `parse_events`' fast-path
/// line loop, since `parse_events` builds the parser locally) and
/// returns the exact `(dbg_node_allocs, dbg_index_probes,
/// dbg_entry_compares)` counts.
/// All test docs are LF-only, so only the `memchr(b'\r', ..).is_none()`
/// branch is needed.
fn parse_counters(doc: &str) -> (usize, usize, usize) {
    let bump = Bump::new();
    let mut p = EventParser::new(&bump);
    let mut events: EventStream<'_> = BumpVec::with_capacity_in(64, &bump);
    let bytes = doc.as_bytes();
    let mut line_num: usize = 0;
    let mut line_start: usize = 0;
    while line_start <= bytes.len() {
        let end = memchr(b'\n', &bytes[line_start..])
            .map(|p| line_start + p)
            .unwrap_or(bytes.len());
        let line: &str = &doc[line_start..end];
        line_num += 1;
        p.handle_line(line, line_num, line_start as u32, &mut events)
            .unwrap();
        if end == bytes.len() {
            break;
        }
        line_start = end + 1;
    }
    p.finish(bytes.len() as u32, &mut events).unwrap();
    (p.dbg_node_allocs, p.dbg_index_probes, p.dbg_entry_compares)
}

/// Runs a full LF-only parse and returns the exact registration-
/// phase event-view counter (`dbg_reg_scans`): every staged Event
/// the inline-child registration walk examined. Test docs are
/// LF-only, so only the `memchr(b'\r', ..).is_none()` branch is
/// needed (same as `parse_counters` above).
fn registration_scans(doc: &str) -> usize {
    let bump = Bump::new();
    let mut p = EventParser::new(&bump);
    let mut events: EventStream<'_> = BumpVec::with_capacity_in(64, &bump);
    let bytes = doc.as_bytes();
    let mut line_num: usize = 0;
    let mut line_start: usize = 0;
    while line_start <= bytes.len() {
        let end = memchr(b'\n', &bytes[line_start..])
            .map(|p| line_start + p)
            .unwrap_or(bytes.len());
        let line: &str = &doc[line_start..end];
        line_num += 1;
        p.handle_line(line, line_num, line_start as u32, &mut events)
            .unwrap();
        if end == bytes.len() {
            break;
        }
        line_start = end + 1;
    }
    p.finish(bytes.len() as u32, &mut events).unwrap();
    p.dbg_reg_scans
}

/// Keyed inline compound with a D-segment dotted key:
/// `root: {a.a. … .a.x: 1}`. The inline scanner's shared
/// `insert_value`/`descend` tables expand the dotted key into a
/// chain of D nested objects, so the staged event list is
/// `3D + 2` events and registration must descend D levels.
///
/// EXACT single-pass counts, derived from the code paths (not
/// fitted): every staged event is examined EXACTLY once — a Key's
/// read is its loop iteration, its value's read the peek, an
/// EndObject its own iteration, and BeginObject values are peeks —
/// so reads == inner.len() == 3D + 2.
///
/// PRE-FIX (measured with this same counter, commit 5371db7), the
/// walk re-scanned each nested object's full event range per
/// ancestor via `matching_bracket` + recursion: 44 / 134 / 458 /
/// 1,682 event views at D = 4 / 8 / 16 / 32 — ratios 3.05 / 3.42 /
/// 3.67 trending the quadratic 4x, on a `Theta(D)` input with a
/// linear number of index nodes. `MAX_INLINE_DEPTH` does not bound
/// it: the physical inline compound is ONE; the extra levels come
/// from dotted-key expansion, not recursive bracket parsing. The
/// single pass removes the re-scan entirely: scans(2D) ≈ 2·scans(D).
#[test]
fn inline_child_registration_scans_are_linear() {
    for d in [4usize, 8, 16, 32] {
        let key = format!("{}x", "a.".repeat(d));
        let doc = format!("root: {{{key}: 1}}");
        assert_eq!(
            registration_scans(&doc),
            3 * d + 2,
            "registration event views at D={d}"
        );
    }
}

/// The round-7 representative parses with UNCHANGED semantics:
/// D dotted prefixes expand to a chain of D nested objects under
/// the compound's key (§ 5.3.2), and inline children stay visible
/// to later dotted re-entry.
#[test]
fn dotted_inline_compound_expansion_is_unchanged() {
    let doc = "root: {a.a.a.a.x: 1}";
    let v: serde_json::Value = crate::from_str(doc).unwrap();
    assert_eq!(
        serde_json::to_string(&v).unwrap(),
        r#"{"root":{"a":{"a":{"a":{"a":{"x":1}}}}}}"#
    );
    // Dotted re-entry into an inline child merges (§ 5.3.2) ...
    let v: serde_json::Value = crate::from_str("a: {x: 1}\na.y: 2").unwrap();
    assert_eq!(serde_json::to_string(&v).unwrap(), r#"{"a":{"x":1,"y":2}}"#);
    // ... and a full-path duplicate stays a DuplicateKey.
    let err = crate::from_str::<serde_json::Value>("a: {x: 1}\na.x: 2").unwrap_err();
    assert!(err.to_string().contains("duplicate key"), "{err}");
}

/// Deep chain `a: { ... a: { x: 1 } ... }` of depth D.
///
/// EXACT counts, derived from the code paths (not fitted):
/// - `dbg_node_allocs == D + 2`: one node per `a` level (D), one for
///   `x` (leaf), plus the sentinel pushed in `new()` (which counts,
///   so totals stay exact). `x: 1` sits inside D open `a` objects,
///   so there is exactly one leaf node.
/// - `dbg_index_probes == D + 1`: one probe per real key seen — D
///   for the `a` openers, 1 for `x`. `}` closers probe nothing.
///
/// The PRE-FIX representation copied the whole key path as `&str`
/// slots on every frame close, allocating Theta(D^3) path slots
/// overall. Measured slot-copies at D = 4 / 8 / 16 / 32 were
/// 35 / 165 / 969 / 6,545 (doubling ratios 4.7 / 5.9 / 6.8 —
/// clearly super-quadratic, trending cubic). The shape arena makes
/// allocation exactly linear in nodes: allocs(2D) <= 3 * allocs(D).
#[test]
fn deep_chain_path_metadata_is_linear() {
    for d in [4usize, 8, 16, 32] {
        let mut doc = String::new();
        for _ in 0..d {
            doc.push_str("a: {\n");
        }
        doc.push_str("x: 1\n");
        for _ in 0..d {
            doc.push_str("}\n");
        }
        let (allocs, probes, _) = parse_counters(&doc);
        assert_eq!(allocs, d + 2, "node allocs at D={d}");
        assert_eq!(probes, d + 1, "index probes at D={d}");
    }
    // Linear-growth check across consecutive pairs of the sizes above.
    let allocs_of = |d: usize| {
        let mut doc = String::new();
        for _ in 0..d {
            doc.push_str("a: {\n");
        }
        doc.push_str("x: 1\n");
        for _ in 0..d {
            doc.push_str("}\n");
        }
        parse_counters(&doc).0
    };
    let a4 = allocs_of(4);
    let a8 = allocs_of(8);
    let a16 = allocs_of(16);
    let a32 = allocs_of(32);
    assert!(a8 <= 3 * a4);
    assert!(a16 <= 3 * a8);
    assert!(a32 <= 3 * a16);
}

/// Flat object with K scalar keys, one per line.
///
/// EXACT counts: one node per key + the sentinel in `new()`
/// (`dbg_node_allocs == K + 1`), and one index probe per key
/// (`dbg_index_probes == K`). No synthetic prefixes exist, so no
/// extra nodes or probes appear.
///
/// PRE-FIX, every duplicate/conflict check compared full joined
/// key-path strings entry-by-entry against a per-frame table —
/// exactly K(K-1)/2 entry comparisons for K keys: 28 / 120 / 496 /
/// 2,016 at K = 8 / 16 / 32 / 64 — precisely quadratic. The shared
/// index makes probes exactly linear: probes(2K) <= 3*probes(K).
///
/// The hybrid two-tier index restores pre-fix EXACT comparison
/// counts below the spill threshold and stays sub-quadratic above:
/// the i-th key's lookup (0-based) scans i entries while linear, so
/// for K <= 8: compares = K(K-1)/2 (K=8 → 28, exactly the pre-fix
/// count). The 9th insert spills (linear holds 8), so for K >= 9:
/// compares(K) = 28 + 8 + (K-9) → K=16 → 43, K=32 → 59, K=64 → 91.
/// PRE-FIX comparisons were exactly K(K-1)/2 (28 / 120 / 496 / 2,016);
/// the hybrid matches pre-fix below the threshold and is
/// sub-quadratic above it.
#[test]
fn flat_object_index_probes_are_linear() {
    let build = |k: usize| {
        let mut doc = String::new();
        for i in 0..k {
            doc.push_str(&format!("k{i}: {i}\n"));
        }
        doc
    };
    for k in [8usize, 16, 32, 64] {
        let (allocs, probes, compares) = parse_counters(&build(k));
        assert_eq!(allocs, k + 1, "node allocs at K={k}");
        assert_eq!(probes, k, "index probes at K={k}");
        let expected_compares = if k <= 8 {
            k * (k - 1) / 2
        } else {
            28 + 8 + (k - 9)
        };
        assert_eq!(compares, expected_compares, "entry compares at K={k}");
    }
    let counters_of = |k: usize| parse_counters(&build(k));
    let probes_of = |k: usize| parse_counters(&build(k)).1;
    let p8 = probes_of(8);
    let p16 = counters_of(16).1;
    let p32 = counters_of(32).1;
    let p64 = counters_of(64).1;
    assert!(p16 <= 3 * p8);
    assert!(p32 <= 3 * p16);
    assert!(p64 <= 3 * p32);
    // Compares grow sub-quadratically across doubling pairs.
    assert!(counters_of(32).2 <= 3 * counters_of(16).2);
    assert!(counters_of(64).2 <= 3 * counters_of(32).2);
}
