//! Parse / render / round-trip benchmarks.
//!
//! Run with `cargo bench -p ktav`. Criterion stores baselines per machine
//! under `target/criterion/` — the first run establishes a baseline; every
//! subsequent run reports `+/- X%` against it.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};

// Synthesized realistic-mix fixtures (1k / 50k / 500k targets) +
// `inject_bad_line` for the error-path bench. Sibling file mounted as a
// module — Cargo treats each `benches/*.rs` as its own crate root, so
// the explicit `#[path]` is needed to share `fixtures.rs`.
#[path = "fixtures.rs"]
mod fixtures;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const SMALL: &str = include_str!("../tests/fixtures/upstreams.conf");

fn synthesize(n_upstreams: usize, n_patterns: usize) -> String {
    let mut s = String::with_capacity(n_upstreams * 200 + n_patterns * 30);
    s.push_str("port: 20082\n\n");

    s.push_str("banned_patterns: [\n");
    for i in 0..n_patterns {
        s.push_str("    .*pattern");
        s.push_str(&i.to_string());
        s.push_str(":\\d+\n");
    }
    s.push_str("]\n\n");

    s.push_str("upstreams: [\n");
    for i in 0..n_upstreams {
        s.push_str("    {\n        host: h");
        s.push_str(&i.to_string());
        s.push_str(".example\n        port: 1080\n        timeouts: {\n");
        s.push_str("            read: 30\n            write: 10\n");
        s.push_str("        }\n        tags: [\n            primary\n            eu\n");
        s.push_str("        ]\n    }\n");
    }
    s.push_str("]\n");
    s
}

// Multi-line payload — exercises the dedent path.
fn synthesize_multiline(n_lines: usize) -> String {
    let mut s = String::with_capacity(n_lines * 40);
    s.push_str("doc: (\n");
    for i in 0..n_lines {
        s.push_str("    line ");
        s.push_str(&i.to_string());
        s.push('\n');
    }
    s.push_str(")\n");
    s
}

// ---------------------------------------------------------------------------
// Typed target (for serde de/ser benches)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct Timeouts {
    read: u32,
    write: u32,
}

#[derive(Serialize, Deserialize)]
struct Upstream {
    host: String,
    port: u16,
    timeouts: Option<Timeouts>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    port: u16,
    banned_patterns: Vec<String>,
    upstreams: Vec<Upstream>,
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_parse_to_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_value");

    group.bench_function("small_real_config", |b| {
        b.iter(|| {
            let v = ktav::parse(black_box(SMALL)).unwrap();
            black_box(v)
        })
    });

    for (label, upstreams, patterns) in [
        ("10_upstreams", 10_usize, 20_usize),
        ("100_upstreams", 100, 200),
        ("1000_upstreams", 1000, 500),
    ] {
        let text = synthesize(upstreams, patterns);
        let size_kb = text.len() / 1024;
        group.bench_with_input(
            BenchmarkId::new(label, format!("{}KB", size_kb)),
            &text,
            |b, t| {
                b.iter(|| {
                    let v = ktav::parse(black_box(t)).unwrap();
                    black_box(v)
                })
            },
        );
    }

    group.finish();
}

fn bench_parse_to_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_struct");
    let text = synthesize(100, 50);
    group.bench_function("100_upstreams_typed", |b| {
        b.iter(|| {
            let cfg: Config = ktav::from_str(black_box(&text)).unwrap();
            black_box(cfg)
        })
    });
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");
    let text = synthesize(100, 50);
    let cfg: Config = ktav::from_str(&text).unwrap();
    group.bench_function("100_upstreams_typed", |b| {
        b.iter(|| {
            let s = ktav::to_string(black_box(&cfg)).unwrap();
            black_box(s)
        })
    });
    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    let text = synthesize(100, 50);
    group.bench_function("100_upstreams_typed", |b| {
        b.iter(|| {
            let cfg: Config = ktav::from_str(black_box(&text)).unwrap();
            let s = ktav::to_string(&cfg).unwrap();
            black_box(s)
        })
    });
    group.finish();
}

fn bench_multiline_dedent(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiline_dedent");
    for n in [10_usize, 100, 1000] {
        let text = synthesize_multiline(n);
        group.bench_with_input(BenchmarkId::new("lines", n), &text, |b, t| {
            b.iter(|| {
                let v = ktav::parse(black_box(t)).unwrap();
                black_box(v)
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Synthesised size-keyed workloads (1k / 50k / 500k)
//
// These benches feed the realistic-mix synthesiser into both the success
// path (`parse_synth`) and the error path (`parse_synth_error`). The
// success path is gated by `validate_synth` so we only ever time a doc
// that matches the synthesiser's contract. The error path injects one
// bad line at a known root-Object position and asserts the exact
// error category (`MissingSeparatorSpace`), line, and span — so we
// know we're timing a realistic, deterministic error construction.
// ---------------------------------------------------------------------------

type SynthFn = fn() -> String;
const SYNTH_SIZES: &[(&str, SynthFn)] = &[
    ("small_1k", fixtures::small_1k),
    ("medium_50k", fixtures::medium_50k),
    ("large_500k", fixtures::large_500k),
];

fn bench_parse_synth(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_synth");
    for &(label, gen) in SYNTH_SIZES {
        let text = gen();
        // Strict gate: refuse to time anything but the expected Object.
        fixtures::validate_synth(&text);
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &text, |b, t| {
            b.iter(|| {
                let v = ktav::parse(black_box(t)).unwrap();
                black_box(v)
            })
        });
    }
    group.finish();
}

fn bench_parse_synth_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_synth_error");
    for &(label, gen) in SYNTH_SIZES {
        let good = gen();
        fixtures::validate_synth(&good);
        let (bad, bad_line) = fixtures::inject_bad_line(&good);
        // Strict negative control: the injected line must fail with exactly
        // MissingSeparatorSpace at the predicted line and span, otherwise we
        // would be timing whatever accident the input happens to trigger.
        match ktav::parse(&bad) {
            Err(ktav::Error::Structured(k)) => {
                assert!(
                    matches!(k, ktav::ErrorKind::MissingSeparatorSpace { marker: ':', .. }),
                    "fixture for {label}: unexpected error category: {k:?}"
                );
                assert_eq!(k.line(), Some(bad_line), "fixture for {label}: unexpected error line");
                assert_eq!(
                    k.span().slice(&bad),
                    Some("value"),
                    "fixture for {label}: unexpected error span"
                );
            }
            other => panic!(
                "fixture for {label}: expected MissingSeparatorSpace at line {bad_line}, got {other:?}"
            ),
        }
        group.throughput(Throughput::Bytes(bad.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &bad, |b, t| {
            b.iter(|| {
                let r = ktav::parse(black_box(t));
                black_box(r.is_err())
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_to_value,
    bench_parse_to_struct,
    bench_render,
    bench_roundtrip,
    bench_multiline_dedent,
    bench_parse_synth,
    bench_parse_synth_error,
);
criterion_main!(benches);
