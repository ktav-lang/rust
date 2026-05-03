// Deterministic Ktav fixture generator for crate-level Criterion benches.
//
// Mirrors the synthesizer shape used by `editor/lsp/benches/fixtures.rs`
// (round-robin mix of pairs, dotted keys, typed scalars `:i`/`:f`, raw
// `::` literals, nested objects/arrays, multi-line raw blocks, and
// comments) so the parsing baseline reflects realistic config workloads
// rather than a single artificial pattern. No randomness — the index `i`
// drives every choice and every payload, so successive runs and
// successive machines see byte-identical input.
//
// Each bench file `include!`s this module rather than promoting it to a
// shared crate (Criterion benches can't share a `mod common` cleanly).

// Not every bench binary references every helper here, so silence the
// per-binary unused-warnings rather than scatter `#[allow]`s.
#![allow(dead_code)]

use std::fmt::Write as _;

/// 1 KiB target.
pub fn small_1k() -> String {
    synth(1_024)
}

/// 50 KiB target.
pub fn medium_50k() -> String {
    synth(50 * 1_024)
}

/// 500 KiB target.
pub fn large_500k() -> String {
    synth(500 * 1_024)
}

/// Synthesize a Ktav document at least `target_bytes` long.
pub fn synth(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 256);
    out.push_str("# generated benchmark fixture\n");
    let mut i = 0u32;
    while out.len() < target_bytes {
        match i % 12 {
            0 => {
                let _ = writeln!(out, "name_{}: value_{}", i, i);
            }
            1 => {
                let _ = writeln!(out, "port_{}:i {}", i, 8000 + (i as u64 % 1000));
            }
            2 => {
                let _ = writeln!(out, "ratio_{}:f {}.{}", i, i % 100, i % 1000);
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
                let _ = writeln!(out, "service.{}.port:i {}", i, 30000 + (i as u64 % 5000));
            }
            7 => {
                let _ = writeln!(out, "# section {}", i / 12);
            }
            8 => {
                let _ = writeln!(out, "obj_{}: {{", i);
                let _ = writeln!(out, "    inner_a: {}", i);
                let _ = writeln!(out, "    inner_b:f {}.5", i % 100);
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

/// Produce a copy of `text` with a single bad line injected at roughly
/// the midpoint (snapped to the nearest line boundary). The bad line
/// `"key:value\n"` is invalid Ktav (no space after the colon), so the
/// parser will fail mid-document — exercising the error-construction
/// path on a realistic-shaped input.
pub fn with_bad_line(text: &str) -> String {
    let target = text.len() / 2;
    // Snap to the next newline so we don't split a line.
    let split = text[target..]
        .find('\n')
        .map(|off| target + off + 1)
        .unwrap_or(text.len());
    let mut out = String::with_capacity(text.len() + 16);
    out.push_str(&text[..split]);
    out.push_str("key:value\n");
    out.push_str(&text[split..]);
    out
}
