//! Instruction-count harness for the writer paths, in the spirit of `iai`.
//!
//! Wall-clock benchmarking on the development host is noise-dominated: a
//! criterion A/B of these same paths produced confidence intervals
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
//! valgrind --tool=callgrind --callgrind-out-file=/dev/null \
//!   "$CARGO_TARGET_DIR/release/examples/callgrind_writers" emit_canonical large_500k 50
//! ```
//!
//! The number to read is valgrind's `I   refs:` line on stderr. Pass
//! `0` as the iteration count to measure the fixture setup alone, and
//! subtract it: `(refs(n) - refs(0)) / n` is the per-call instruction
//! count, free of the parse that builds the input.

use ktav::render::render;
use ktav::{emit_canonical, format_str, to_string_force_strings, Value};

#[path = "../benches/fixtures.rs"]
mod fixtures;

fn fixture_text(size: &str) -> String {
    match size {
        "small_1k" => fixtures::small_1k(),
        "medium_50k" => fixtures::medium_50k(),
        "large_500k" => fixtures::large_500k(),
        other => panic!("unknown size {other:?}; expected small_1k|medium_50k|large_500k"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let workload = args.get(1).map(String::as_str).unwrap_or("emit_canonical");
    let size = args.get(2).map(String::as_str).unwrap_or("medium_50k");
    let iters: usize = args
        .get(3)
        .map(|s| s.parse().expect("iteration count must be a number"))
        .unwrap_or(50);

    let text = fixture_text(size);

    // `format_str` consumes source text; the other three consume a
    // parsed Value. Both setups happen before the counted loop, and the
    // `0`-iteration run measures exactly this much, so whichever one
    // applies cancels out of the subtraction.
    let value: Value = if workload == "format_str" {
        Value::Null
    } else {
        ktav::parse(&text).expect("fixture must parse")
    };

    // `black_box` is unavailable on the 1.71 floor as a stable import
    // path for this use, so the result is consumed by summing its length
    // into a value printed at the end: the optimiser cannot drop the
    // call, and the arithmetic is two instructions per iteration.
    let mut sink = 0usize;
    for _ in 0..iters {
        sink += match workload {
            "emit_canonical" => emit_canonical(&value).expect("representable").len(),
            "render" => render(&value).expect("representable").len(),
            "force_strings" => to_string_force_strings(&value)
                .expect("representable")
                .len(),
            "format_str" => format_str(&text).expect("formattable").len(),
            other => panic!(
                "unknown workload {other:?}; expected \
                 emit_canonical|render|force_strings|format_str"
            ),
        };
    }

    // Printed so the loop cannot be optimised away, and so a run that
    // silently did nothing is visible.
    println!("{workload}/{size} iters={iters} sink={sink}");
}
