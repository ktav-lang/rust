//! Side-by-side benchmark: Ktav vs serde_json.
//!
//! Built around the principle that **the document tree is identical**:
//! we synthesise a `Config` struct once, then ask each format to
//! serialise it into its own canonical text. That guarantees both
//! parsers see semantically equivalent input — only the surface
//! syntax differs.
//!
//! Three axes:
//!   * `parse_to_value`     — untyped, format-native value tree.
//!   * `parse_to_struct`    — serde-driven typed deserialisation.
//!   * `render`             — typed value back to text.
//!
//! Each axis is exercised at three sizes (small / medium / large) so
//! the cross-over cost (per-doc fixed cost vs per-byte cost) is visible.
//!
//! Run:  `cargo bench -p ktav --bench vs_json`
//! HTML: `target/criterion/report/index.html`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared document model
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
    weight: u32,
    enabled: bool,
    timeouts: Timeouts,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
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

// Sizes chosen so the smallest fits in a single L1 line group and the
// largest spills out of L2 — typical config files vs synthetic stress.
const SIZES: &[(&str, usize, usize)] = &[
    ("small", 5, 4),
    ("medium", 100, 50),
    ("large", 1000, 200),
];

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_parse_to_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_value");

    for &(label, upstreams, patterns) in SIZES {
        let cfg = make_config(upstreams, patterns);
        let ktav_text = ktav::to_string(&cfg).unwrap();
        let json_text = serde_json::to_string(&cfg).unwrap();

        // Throughput is keyed off the Ktav size, so percent-deltas line
        // up; the ratio between the two formats stays in the report.
        group.throughput(Throughput::Bytes(ktav_text.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("ktav", label),
            &ktav_text,
            |b, t| {
                b.iter(|| {
                    let v = ktav::parse(black_box(t)).unwrap();
                    black_box(v)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("json", label),
            &json_text,
            |b, t| {
                b.iter(|| {
                    let v: serde_json::Value =
                        serde_json::from_str(black_box(t)).unwrap();
                    black_box(v)
                })
            },
        );
    }

    group.finish();
}

fn bench_parse_to_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_struct");

    for &(label, upstreams, patterns) in SIZES {
        let cfg = make_config(upstreams, patterns);
        let ktav_text = ktav::to_string(&cfg).unwrap();
        let json_text = serde_json::to_string(&cfg).unwrap();

        group.throughput(Throughput::Bytes(ktav_text.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("ktav", label),
            &ktav_text,
            |b, t| {
                b.iter(|| {
                    let v: Config = ktav::from_str(black_box(t)).unwrap();
                    black_box(v)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("json", label),
            &json_text,
            |b, t| {
                b.iter(|| {
                    let v: Config = serde_json::from_str(black_box(t)).unwrap();
                    black_box(v)
                })
            },
        );
    }

    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");

    for &(label, upstreams, patterns) in SIZES {
        let cfg = make_config(upstreams, patterns);
        let ktav_text = ktav::to_string(&cfg).unwrap();
        group.throughput(Throughput::Bytes(ktav_text.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("ktav", label),
            &cfg,
            |b, c| {
                b.iter(|| {
                    let s = ktav::to_string(black_box(c)).unwrap();
                    black_box(s)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("json", label),
            &cfg,
            |b, c| {
                b.iter(|| {
                    let s = serde_json::to_string(black_box(c)).unwrap();
                    black_box(s)
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// One-time printout: the size differential between the two formats.
// Criterion captures stdout, so this lands at the head of the report.
// ---------------------------------------------------------------------------

fn print_size_table() {
    eprintln!("\n=== Document size: Ktav vs JSON ===");
    eprintln!(
        "{:<8} {:>10} {:>10} {:>8}",
        "size", "ktav (B)", "json (B)", "ratio"
    );
    for &(label, upstreams, patterns) in SIZES {
        let cfg = make_config(upstreams, patterns);
        let kt = ktav::to_string(&cfg).unwrap();
        let js = serde_json::to_string(&cfg).unwrap();
        eprintln!(
            "{:<8} {:>10} {:>10} {:>7.2}x",
            label,
            kt.len(),
            js.len(),
            (kt.len() as f64) / (js.len() as f64),
        );
    }
    eprintln!();
}

fn all(c: &mut Criterion) {
    print_size_table();
    bench_parse_to_value(c);
    bench_parse_to_struct(c);
    bench_render(c);
}

criterion_group!(benches, all);
criterion_main!(benches);
