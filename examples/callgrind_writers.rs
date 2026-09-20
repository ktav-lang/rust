//! Instruction-count harness for the hot paths, in the spirit of `iai`.
//!
//! Wall-clock benchmarking on the development host is noise-dominated: a
//! criterion A/B of the writer paths produced confidence intervals
//! spanning several times the median on the larger fixtures, so a real
//! 40% improvement and no improvement at all were statistically
//! indistinguishable. Callgrind counts *instructions executed*, which is
//! deterministic — the same binary and input give the same number on
//! every run, whatever else the machine is doing.
//!
//! Not an `iai-callgrind` benchmark on purpose: that crate needs its own
//! `iai-callgrind-runner` binary at a matching version, and its MSRV is
//! well above the 1.71 this crate promises, so adding it as a
//! dev-dependency would break the MSRV CI job. This example needs
//! nothing but `valgrind`, and nothing automatically builds or runs it.
//!
//! Usage (from WSL, where valgrind lives):
//!
//! ```sh
//! export CARGO_TARGET_DIR=/tmp/ktav-callgrind   # keep off the Windows target dir
//! cargo build --release --example callgrind_writers
//! BIN="$CARGO_TARGET_DIR/release/examples/callgrind_writers"
//!
//! # total instructions, fixture setup included
//! valgrind --tool=callgrind --callgrind-out-file=/dev/null \
//!   "$BIN" emit_canonical large_500k 50
//!
//! # per-function attribution of the measured loop ALONE: collection is
//! # off until `measured_loop` is entered, so fixture construction is
//! # excluded without having to subtract it
//! valgrind --tool=callgrind --collect-atstart=no \
//!   --toggle-collect='*measured_loop*' --callgrind-out-file=/tmp/cg.out \
//!   "$BIN" to_string medium_50k 20
//! callgrind_annotate --threshold=95 /tmp/cg.out | head -40
//! ```
//!
//! With the whole-run form, pass `0` as the iteration count to measure
//! the fixture setup alone and subtract it: `(refs(n) - refs(0)) / n` is
//! the per-call instruction count.

use serde::Serialize;

use ktav::render::render;
use ktav::{emit_canonical, format_str, to_string_force_strings, Value};

#[path = "../benches/fixtures.rs"]
mod fixtures;

// ---------------------------------------------------------------------------
// The serde workload's document shape, mirroring `benches/vs_json.rs` so
// the numbers line up with that benchmark's `render/ktav` row. `Value`
// does not implement `Serialize`, so reaching the streaming text
// serializer needs a typed value, not a parsed one.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Timeouts {
    read: u32,
    write: u32,
}

#[derive(Serialize)]
struct Upstream {
    host: String,
    port: u16,
    weight: u32,
    enabled: bool,
    timeouts: Timeouts,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct Config {
    service: String,
    port: u16,
    tls: bool,
    ratio: f64,
    banned_patterns: Vec<String>,
    upstreams: Vec<Upstream>,
}

fn make_config(n_upstreams: usize, n_patterns: usize) -> Config {
    Config {
        service: "ktav-bench".into(),
        port: 20082,
        tls: true,
        ratio: 0.75,
        banned_patterns: (0..n_patterns)
            .map(|i| format!(".*pattern{}:\\d+", i))
            .collect(),
        upstreams: (0..n_upstreams)
            .map(|i| Upstream {
                host: format!("h{}.example.internal", i),
                port: 1080,
                weight: 100 + (i as u32 % 7),
                enabled: i % 5 != 0,
                timeouts: Timeouts {
                    read: 30,
                    write: 10,
                },
                tags: vec!["primary".into(), "eu-west-1".into(), "edge".into()],
            })
            .collect(),
    }
}

/// The same (upstreams, patterns) triple `benches/vs_json.rs` uses, so a
/// size label means the same document in both places.
fn config_for(size: &str) -> Config {
    match size {
        "small_1k" => make_config(5, 4),
        "medium_50k" => make_config(100, 50),
        "large_500k" => make_config(1000, 200),
        other => panic!("unknown size {other:?}"),
    }
}

fn fixture_text(size: &str) -> String {
    match size {
        "small_1k" => fixtures::small_1k(),
        "medium_50k" => fixtures::medium_50k(),
        "large_500k" => fixtures::large_500k(),
        other => panic!("unknown size {other:?}; expected small_1k|medium_50k|large_500k"),
    }
}

/// Everything a workload needs, built before collection starts.
struct Setup {
    text: String,
    value: Value,
    config: Config,
}

/// The counted region. `#[inline(never)]` so `--toggle-collect` has a
/// stable symbol to switch collection on and off at; the returned sum is
/// printed by the caller so the loop cannot be optimised away.
#[inline(never)]
fn measured_loop(workload: &str, setup: &Setup, iters: usize) -> usize {
    let mut sink = 0usize;
    for _ in 0..iters {
        sink += match workload {
            "emit_canonical" => emit_canonical(&setup.value).expect("representable").len(),
            "render" => render(&setup.value).expect("representable").len(),
            "force_strings" => to_string_force_strings(&setup.value)
                .expect("representable")
                .len(),
            "format_str" => format_str(&setup.text).expect("formattable").len(),
            "to_string" => ktav::to_string(&setup.config).expect("serializable").len(),
            "parse" => match ktav::parse(&setup.text).expect("fixture must parse") {
                Value::Object(o) => o.len(),
                Value::Array(a) => a.len(),
                _ => 1,
            },
            other => panic!(
                "unknown workload {other:?}; expected \
                 emit_canonical|render|force_strings|format_str|to_string|parse"
            ),
        };
    }
    sink
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let workload = args.get(1).map(String::as_str).unwrap_or("emit_canonical");
    let size = args.get(2).map(String::as_str).unwrap_or("medium_50k");
    let iters: usize = args
        .get(3)
        .map(|s| s.parse().expect("iteration count must be a number"))
        .unwrap_or(50);

    // Built unconditionally so the setup cost is identical for every
    // workload at a given size, which is what makes the
    // `refs(n) - refs(0)` subtraction valid across workloads.
    let text = fixture_text(size);
    let value = ktav::parse(&text).expect("fixture must parse");
    let config = config_for(size);
    let setup = Setup {
        text,
        value,
        config,
    };

    let sink = measured_loop(workload, &setup, iters);
    println!("{workload}/{size} iters={iters} sink={sink}");
}
