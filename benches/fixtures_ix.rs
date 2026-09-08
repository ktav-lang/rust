// R8-F6 measurement fixture family: large C (recorded inline compound
// spans) at small depth — the shape the deep-nesting fixtures do not
// cover. Mounted via #[path] by src/parser/tests.rs (ix_fixtures);
// kept separate from fixtures.rs so a crate never loads one file as
// two modules (clippy::duplicate_mod — arena_probe mounts fixtures.rs
// too).

// Not every consumer references every helper, so silence the
// per-binary unused-warnings rather than scatter `#[allow]`s.
#![allow(dead_code)]

use std::fmt::Write as _;

// --- verbatim copy of benches/fixtures.rs `synth` + `medium_50k` ---
// (deliberate duplication: this crate must not load fixtures.rs as a
// second module — arena_probe already mounts it — so the mainstream
// synth baseline ships with the family; same copy discipline as the
// a2-harness bench_ab binary)

/// 50 KiB target.
pub fn medium_50k() -> String {
    synth(50 * 1_024)
}

/// Synthesize a Ktav document at least `target_bytes` long.
pub fn synth(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 256);
    out.push_str("## generated benchmark fixture\n");
    let mut i = 0u32;
    while out.len() < target_bytes {
        match i % 12 {
            0 => {
                let _ = writeln!(out, "name_{}: value_{}", i, i);
            }
            1 => {
                let _ = writeln!(out, "port_{}: {}", i, 8000 + (i as u64 % 1000));
            }
            2 => {
                let _ = writeln!(out, "ratio_{}: {}.{}", i, i % 100, i % 1000);
            }
            3 => {
                let _ = writeln!(
                    out,
                    "flag_{}: {}",
                    i,
                    if i % 2 == 0 { "true" } else { "false" }
                );
            }
            4 => {
                let _ = writeln!(
                    out,
                    "label_{}:: literal text with spaces and symbols !@#${}",
                    i, i
                );
            }
            5 => {
                let _ = writeln!(out, "service.{}.host: 10.0.0.{}", i, i % 256);
            }
            6 => {
                let _ = writeln!(out, "service.{}.port: {}", i, 30000 + (i as u64 % 5000));
            }
            7 => {
                let _ = writeln!(out, "## section {}", i / 12);
            }
            8 => {
                let _ = writeln!(out, "obj_{}: {{", i);
                let _ = writeln!(out, "    inner_a: {}", i);
                let _ = writeln!(out, "    inner_b: {}.5", i % 100);
                let _ = writeln!(out, "    inner_c:: raw body for {}", i);
                let _ = writeln!(out, "}}");
            }
            9 => {
                let _ = writeln!(out, "list_{}: [", i);
                let _ = writeln!(out, "    item-a-{}", i);
                let _ = writeln!(out, "    item-b-{}", i);
                let _ = writeln!(out, "    :: literal-{}", i);
                let _ = writeln!(out, "]");
            }
            10 => {
                let _ = writeln!(out, "doc_{}: (", i);
                let _ = writeln!(out, "first line of body {}", i);
                let _ = writeln!(out, "second line of body {}", i);
                let _ = writeln!(out, ")");
            }
            _ => {
                let _ = writeln!(out, "tag_{}: alpha-beta-gamma-{}", i, i);
            }
        }
        i = i.wrapping_add(1);
    }
    out
}

/// One line `k: [` + `n` empty-object items `,`-separated + `]`:
/// every item is a recorded compound, so C = n at depth 1 with ~3-byte
/// items — the most lookups per body byte the grammar can spell.
pub fn wide_line_arr_tiny(n: usize) -> String {
    let mut items = String::with_capacity(n * 3);
    for i in 0..n {
        if i > 0 {
            items.push(',');
        }
        items.push_str("{}");
    }
    format!("k: [{items}]\n")
}

/// One line `k: [` + `n` `{a:1}` items + `]`: same C = n, slightly
/// less degenerate items.
pub fn wide_line_arr_small(n: usize) -> String {
    let mut items = String::with_capacity(n * 6);
    for i in 0..n {
        if i > 0 {
            items.push(',');
        }
        items.push_str("{a:1}");
    }
    format!("k: [{items}]\n")
}

/// One line `k: {` + `n` pairs `a<i>: {}` + `}`: the object twin of
/// `wide_line_arr_tiny`.
pub fn wide_line_obj_tiny(n: usize) -> String {
    let mut items = String::with_capacity(n * 8);
    for i in 0..n {
        if i > 0 {
            items.push(',');
        }
        let _ = write!(items, "a{i}: {{}}");
    }
    format!("k: {{{items}}}\n")
}

/// One line `k: [` + `n` `[{}]` items + `]`: C = 2n (every inner
/// array and every innermost object records) at depth 2, the deepest
/// shallow shape allowed by "small depth".
pub fn wide_line_two_level(n: usize) -> String {
    let mut items = String::with_capacity(n * 5);
    for i in 0..n {
        if i > 0 {
            items.push(',');
        }
        items.push_str("[{}]");
    }
    format!("k: [{items}]\n")
}

/// `lines` lines of `k<i>: [{a:1},{a:2}]` — many small inline trees
/// (C = 2 per tree), the mainstream inline-heavy shape.
pub fn many_inline_trees(lines: usize) -> String {
    let mut out = String::with_capacity(lines * 22);
    for i in 0..lines {
        let _ = writeln!(out, "k{i}: [{{a:1}},{{a:2}}]");
    }
    out
}

/// `k: {a: {a: ... 1 ... }}` nested `depth` deep: the opposite shape —
/// one pair per level, O(depth) lookups into C = depth-1 pairs.
pub fn deep_chain(depth: usize) -> String {
    format!("k: {}1{}", "{a: ".repeat(depth), "}".repeat(depth))
}

/// R10-F1 prescan family: `k: {a: {a: … <leaf> …}}` — `depth` nested
/// inline Objects around one quote-free `leaf_bytes`-byte scalar leaf
/// (`a`-`z` repeating). At a fixed leaf length the family isolates
/// depth (D); at a fixed depth it isolates the leaf length (M). The
/// quote-presence prescans the R10-F1 fix removed saw every leaf byte
/// at every level here; the ix_probe `hq_*` counters pin the cost.
pub fn deep_chain_leaf(depth: usize, leaf_bytes: usize) -> String {
    let mut leaf = String::with_capacity(leaf_bytes);
    for i in 0..leaf_bytes {
        leaf.push((b'a' + (i % 26) as u8) as char);
    }
    format!("k: {}{}{}", "{a: ".repeat(depth), leaf, "}".repeat(depth))
}
