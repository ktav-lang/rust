//! Value-writer benchmarks: `emit_canonical`, `render` and
//! `to_string_force_strings`.
//!
//! These three take an owned [`ktav::Value`] and produce text. Until this
//! file they had **no** benchmark coverage at all: `vs_json`'s `render/`
//! group measures `ktav::to_string`, which is the streaming serde text
//! serializer and never touches this code, and `format`'s `format_str`
//! measures the trivia-preserving writer. So the paths that carry the
//! § 5.9 canonical writer — the ones every binding reaches through
//! `ktav_emit_canonical`, and the ones the conformance corpus exercises —
//! were unmeasured.
//!
//! The fixture is parsed once, outside the measured closure, so what is
//! timed is emission only.
//!
//! Run with `cargo bench -p ktav --bench emit`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use ktav::render::render;
use ktav::{emit_canonical, to_string_force_strings, Value};

#[path = "fixtures.rs"]
mod fixtures;

fn sizes() -> Vec<(&'static str, String)> {
    vec![
        ("small_1k", fixtures::small_1k()),
        ("medium_50k", fixtures::medium_50k()),
        ("large_500k", fixtures::large_500k()),
    ]
}

/// Parse a fixture into the `Value` the writers consume. Panics rather
/// than returning, because a fixture that does not parse would make every
/// number below meaningless.
fn value_of(text: &str) -> Value {
    ktav::parse(text).expect("benchmark fixture must parse")
}

fn bench_emit_canonical(c: &mut Criterion) {
    let mut group = c.benchmark_group("emit_canonical");
    for (label, text) in sizes() {
        let value = value_of(&text);
        // Throughput is stated over the emitted bytes, not the source:
        // the canonical form is what this code produces.
        let emitted = emit_canonical(&value).expect("fixture must be representable");
        group.throughput(Throughput::Bytes(emitted.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &value, |b, v| {
            b.iter(|| emit_canonical(black_box(v)).unwrap());
        });
    }
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");
    for (label, text) in sizes() {
        let value = value_of(&text);
        let emitted = render(&value).expect("fixture must be representable");
        group.throughput(Throughput::Bytes(emitted.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &value, |b, v| {
            b.iter(|| render(black_box(v)).unwrap());
        });
    }
    group.finish();
}

fn bench_force_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("force_strings");
    for (label, text) in sizes() {
        let value = value_of(&text);
        let emitted = to_string_force_strings(&value).expect("fixture must be representable");
        group.throughput(Throughput::Bytes(emitted.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &value, |b, v| {
            b.iter(|| to_string_force_strings(black_box(v)).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_emit_canonical,
    bench_render,
    bench_force_strings
);
criterion_main!(benches);
