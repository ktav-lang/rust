//! R10-F1 tests: quote-presence prescan cost (deterministic counters).

use super::ix_fixtures;
use crate::value::Value;
// --- R10-F1: quote-presence prescan cost (deterministic counters) ----------

#[test]
fn r10f1_deep_chain_leaf_shapes_are_valid() {
    use crate::Value;

    // The D x M family must have exactly the requested spine depth and
    // leaf length — a shape drift would silently weaken the counter
    // pins below.
    for (depth, leaf_bytes) in [(1usize, 8usize), (4, 64), (32, 512), (100, 16)] {
        let doc = ix_fixtures::deep_chain_leaf(depth, leaf_bytes);
        let mut v = crate::parse(&doc).expect("family document must parse");
        let mut levels = 0usize;
        while let Some(obj) = v.as_object() {
            let key = if levels == 0 { "k" } else { "a" };
            v = obj
                .get(key)
                .unwrap_or_else(|| panic!("spine key {key} missing at level {levels}"))
                .clone();
            levels += 1;
        }
        assert_eq!(levels, depth + 1, "spine depth at depth={depth}");
        match &v {
            Value::String(s) => assert_eq!(s.len(), leaf_bytes, "leaf at depth={depth}"),
            other => panic!("leaf must be a String, got {other:?}"),
        }
    }
    assert!(ix_fixtures::deep_chain_leaf(64, 4096).len() > 4096 + 64 * 5);
}

#[test]
fn r10f1_root_quotes_threaded_over_quotefree_descendants() {
    use crate::Value;

    // The root body carries the quote flag, so quote-free descendant
    // levels run the quote-aware machines; their segmentation must be
    // exactly what the quote-free machines produce (R8-F2
    // byte-identity): the quoted key stays ONE key (its comma is
    // opaque, its dot is content), and the quote-free descendants
    // split normally.
    let doc = "{\"a,b\": {c: {d: 1, e: 2}}, g: [1, [2, 3]]}";
    let v = crate::parse(doc).expect("threaded doc must parse");
    let root = v.as_object().unwrap();
    let quoted = root.get("a,b").expect("quoted key must survive whole");
    let quoted = quoted.as_object().expect("quoted key maps to object");
    let c = quoted.get("c").unwrap().as_object().unwrap();
    assert_eq!(c.get("d"), Some(&Value::Integer("1".into())));
    assert_eq!(c.get("e"), Some(&Value::Integer("2".into())));
    let g = root.get("g").unwrap().as_array().unwrap();
    assert_eq!(g[0], Value::Integer("1".into()));
    assert_eq!(g[1].as_array().unwrap()[0], Value::Integer("2".into()));

    // All three engines accept the same document.
    crate::parse_strict(doc).expect("strict must accept");
    crate::from_str::<serde_json::Value>(doc).expect("serde must accept");
    let mut events = 0usize;
    crate::parse_events(doc, |_| {
        events += 1;
    })
    .expect("events must accept");
    assert!(events > 0);
}

#[test]
fn r10f1_quote_prescan_cost_no_depth_multiplier() {
    // Fixed leaf M=512, varying depth D. Before R10-F1 the two prescan
    // sites (`split_top_level`'s dispatch and the separator scan) saw
    // the whole M-byte leaf at EVERY level, ~2*M*D recorded bytes; the
    // threaded root flag leaves a constant number of root-body scans
    // plus per-level key-prefix work.
    let m = 512usize;
    let measure = |depth: usize| {
        let doc = ix_fixtures::deep_chain_leaf(depth, m);
        crate::parser::inline::ix_probe::reset();
        let v = crate::parse(&doc).expect("family document must parse");
        assert!(v.as_object().is_some());
        crate::parser::inline::ix_probe::snapshot().hq_bytes
    };
    let d1 = measure(1);
    let d32 = measure(32);
    assert!(d1 >= m as u64, "root-body prescan must happen: d1={d1}");
    assert!(
        d32 < 2 * d1 + 32 * 64,
        "prescan bytes must not multiply by depth: d1={d1} d32={d32}"
    );
    assert!(d32 < 8 * m as u64, "absolute bound: d32={d32} m={m}");
}

#[test]
fn r10f1_quote_prescan_cost_linear_in_leaf() {
    // Fixed depth D=8, varying leaf M. The total prescan volume must
    // stay a small constant factor of M (the pre-fix code recorded
    // ~2*M*8 bytes here) and grow roughly linearly in M.
    let measure = |m: usize| {
        let doc = ix_fixtures::deep_chain_leaf(8, m);
        crate::parser::inline::ix_probe::reset();
        let _ = crate::parse(&doc).expect("family document must parse");
        crate::parser::inline::ix_probe::snapshot().hq_bytes
    };
    let small = measure(512);
    let large = measure(8192);
    assert!(small >= 512, "root prescan must happen: {small}");
    assert!(small < 8 * 512, "small bound: {small}");
    assert!(
        large >= 8192,
        "prescan must still scale with the leaf: {large}"
    );
    assert!(large < 8 * 8192, "large bound: {large}");
    let ratio = large as f64 / small as f64;
    assert!(
        ratio > 8.0 && ratio < 32.0,
        "growth must be ~linear in M: ratio={ratio}"
    );
}

/// Deterministic companion to the two scaling pins: prints the
/// quote-prescan counters over the whole D x M family for both parse
/// paths, so a future round can re-derive the before/after table with
/// `cargo test --lib ix_probe_r10f1_quote_prescan_counters -- --nocapture`.
#[test]
fn ix_probe_r10f1_quote_prescan_counters() {
    let shapes = [
        (1usize, 512usize),
        (4, 512),
        (8, 512),
        (32, 512),
        (64, 512),
        (8, 8192),
        (8, 65536),
    ];
    for (depth, leaf) in shapes {
        let doc = ix_fixtures::deep_chain_leaf(depth, leaf);
        for (path, run) in [
            (
                "P",
                Box::new(|doc: &str| {
                    let _ = crate::parse(doc);
                }) as Box<dyn Fn(&str)>,
            ),
            (
                "E",
                Box::new(|doc: &str| {
                    let _ = crate::parse_events(doc, |_| {});
                }) as Box<dyn Fn(&str)>,
            ),
        ] {
            crate::parser::inline::ix_probe::reset();
            run(&doc);
            let s = crate::parser::inline::ix_probe::snapshot();
            println!(
                "HQ\tdeep_chain_leaf\t{path}\tdepth={depth}\tleaf={leaf}\tdoc_bytes={}\thq_calls={}\thq_bytes={}\thq_max={}",
                doc.len(),
                s.hq_calls,
                s.hq_bytes,
                s.hq_max
            );
        }
    }
}

// R11-F1: the ROOT quote flag (R10-F1) routes quote-free descendants
// through the quote-aware split machine, so SplitQ's last-segment
// handling must match SplitFast's on this shape: a `.`-armed segment
// whose § 3.3 whitespace skip runs to EOF has no separator, and the
// pushed raw segment must resolve — through the callers' colon search
// (§ 5.8.2/§ 5.3: separator finding precedes key validation) — to the
// § 6.12 missing-separator MalformedInlineCompound on every entry
// point. The pre-fix SplitQ raised EmptyKey here whenever the ROOT
// carried a quote anywhere (ancestor key, sibling value, sibling
// array item), while a quote-free root took SplitFast and already
// said MalformedInlineCompound for the very same nested body.
#[test]
fn r11f1_splitq_last_segment_missing_separator_matches_fast_machine() {
    use crate::ErrorKind;

    let expect_malformed = |input: &str, res: Result<(), crate::Error>| {
        assert!(
            matches!(
                res,
                Err(crate::Error::Structured(
                    ErrorKind::MalformedInlineCompound { .. }
                ))
            ),
            "input {input:?}: expected MalformedInlineCompound, got {res:?}"
        );
    };
    let all_entry_points = |input: &str| {
        expect_malformed(input, crate::parse(input).map(|_| ()));
        expect_malformed(input, crate::parse_strict(input).map(|_| ()));
        expect_malformed(
            input,
            crate::from_str::<serde_json::Value>(input).map(|_| ()),
        );
        expect_malformed(input, crate::parse_events(input, |_| {}).map(|_| ()));
    };

    // The finding's repro documents (SPACE after the dot). The nested
    // body `b.  ` is quote-free in every case — only the threaded
    // flag selects SplitQ for it.
    for input in [
        r"{a: {b. }}",       // quote-free root: SplitFast already said Malformed
        r"{a: {b. }, q: '}", // quote in a SIBLING VALUE after
        r"{q: ', a: {b. }}", // quote in a SIBLING VALUE before
        r#"{"a": {b. }}"#,   // quote at an ANCESTOR KEY position
        r"[{b. }, ']",       // quote in a SIBLING ARRAY ITEM
    ] {
        all_entry_points(input);
    }

    // SPACE, TAB and U+2000 after the dot trigger the identical path
    // (assembled from pieces so the corpus harvest sees only the
    // trivial fragments).
    for tail in [" ", "\t", "\u{2000}"] {
        let mut doc = String::from("{a: {b.");
        doc.push_str(tail);
        doc.push_str("}, q: '}");
        all_entry_points(&doc);
    }

    // The sibling quote's SPECIES is irrelevant (§ 5.3.3: value-side
    // quotes are content).
    for q in ["'", "\"", "`"] {
        let doc = format!(r"{{a: {{b. }}, q: {q}}}");
        all_entry_points(&doc);
    }

    // Positive controls — unchanged by the fix. No trailing whitespace
    // after the dot: plain Exhausted path, MalformedInlineCompound
    // before and after (SplitFast verdict included via the quote-free
    // root).
    for input in [r"{a: {b.}, q: '}", r"{a: {b.}}"] {
        all_entry_points(input);
    }
    // Valid trailing comma: Ok everywhere, same tree.
    let ok_doc = r"{a: {b: 1, }, q: '}";
    let v = crate::parse(ok_doc).expect("trailing-comma control must parse");
    let root = v.as_object().unwrap();
    let a = root.get("a").unwrap().as_object().unwrap();
    assert_eq!(a.get("b"), Some(&Value::Integer("1".into())));
    assert_eq!(root.get("q"), Some(&Value::String("'".into())));
    crate::parse_strict(ok_doc).expect("strict must accept");
    crate::from_str::<serde_json::Value>(ok_doc).expect("serde must accept");
    crate::parse_events(ok_doc, |_| {}).expect("events must accept");

    // The GENUINE dotted key with an empty final segment HAS a
    // separator, never reaches EofAfterWsSkip, and stays EmptyKey
    // (§ 6.5 via insert_value) — quote-free root and quoted sibling
    // alike.
    for input in [r"{a.: 1}", r#"{"a": 1, b.: 2}"#] {
        for res in [
            crate::parse(input).map(|_| ()),
            crate::parse_strict(input).map(|_| ()),
            crate::from_str::<serde_json::Value>(input).map(|_| ()),
            crate::parse_events(input, |_| {}).map(|_| ()),
        ] {
            assert!(
                matches!(
                    res,
                    Err(crate::Error::Structured(ErrorKind::EmptyKey { .. }))
                ),
                "input {input:?}: expected EmptyKey, got {res:?}"
            );
        }
    }
}
