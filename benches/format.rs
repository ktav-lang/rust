//! Formatter (`format_str`) and `ErrorEnvelope` benchmarks.
//!
//! `format_str` runs a second full parse (`parser::fmt_parser`) that
//! additionally builds a trivia tree recording every comment and blank-line
//! run, then re-emits the document preserving both — work `ktav::parse`
//! never does. Prior to this file neither that path nor `ErrorEnvelope`
//! construction/rendering had any benchmark coverage at all (see
//! `bench-baseline.md`).
//!
//! Run with `cargo bench -p ktav --bench format`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[path = "fixtures.rs"]
mod fixtures;

// ---------------------------------------------------------------------------
// A representative document for `format_str`: unlike `fixtures::synth`,
// this interleaves blank-line runs with comments and content, since
// trivia (comments + blank lines) is the entire point of this benchmark —
// a formatter bench that never sees a blank line would not exercise the
// code `format_str` adds over `parse`.
// ---------------------------------------------------------------------------

fn synth_with_trivia(target_bytes: usize) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(target_bytes + 256);
    out.push_str("## generated formatter benchmark fixture\n\n");
    let mut i = 0u32;
    while out.len() < target_bytes {
        match i % 8 {
            0 => {
                let _ = writeln!(out, "## section {}", i / 8);
                let _ = writeln!(out, "name_{}: value_{}", i, i);
            }
            1 => {
                let _ = writeln!(out, "port_{}: {}", i, 8000 + (i as u64 % 1000));
                out.push('\n');
            }
            2 => {
                let _ = writeln!(out, "## note on ratio_{}", i);
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
                let _ = writeln!(out, "obj_{}: {{", i);
                let _ = writeln!(out, "    ## inner comment {}", i);
                let _ = writeln!(out, "    inner_a: {}", i);
                out.push('\n');
                let _ = writeln!(out, "    inner_b: {}.5", i % 100);
                let _ = writeln!(out, "}}");
                out.push('\n');
            }
            5 => {
                let _ = writeln!(out, "service.{}.host: 10.0.0.{}", i, i % 256);
                let _ = writeln!(out, "service.{}.port: {}", i, 30000 + (i as u64 % 5000));
            }
            6 => {
                let _ = writeln!(out, "list_{}: [", i);
                let _ = writeln!(out, "    item-a-{}", i);
                let _ = writeln!(out, "    ## comment inside a list");
                let _ = writeln!(out, "    item-b-{}", i);
                let _ = writeln!(out, "]");
            }
            _ => {
                let _ = writeln!(out, "tag_{}: alpha-beta-gamma-{}", i, i);
                out.push('\n');
            }
        }
        i = i.wrapping_add(1);
    }
    out
}

fn bench_format_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_str");
    for (label, target) in [
        ("small_1k", 1_024_usize),
        ("medium_50k", 50 * 1_024),
        ("large_500k", 500 * 1_024),
    ] {
        let text = synth_with_trivia(target);
        // Strict gate, mirroring parse.rs's validate_synth: a formatter
        // benchmark that silently started failing to format would be
        // worse than useless, it would time an early-return error path.
        ktav::format_str(&text)
            .unwrap_or_else(|e| panic!("format_str fixture for {label} must format: {e}"));
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &text, |b, t| {
            b.iter(|| {
                let s = ktav::format_str(black_box(t)).unwrap();
                black_box(s)
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// ErrorEnvelope::from_error + to_json, on a realistic parse failure.
// Cheap to add alongside format_str since it reuses parse.rs's own
// error-injection fixture machinery.
// ---------------------------------------------------------------------------

fn bench_error_envelope(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_envelope");
    let good = fixtures::medium_50k();
    fixtures::validate_synth(&good);
    let (bad, _bad_line) = fixtures::inject_bad_line(&good);
    let err = match ktav::parse(&bad) {
        Err(e) => e,
        Ok(_) => panic!("error_envelope fixture must fail to parse"),
    };

    group.bench_function("from_error_medium_50k", |b| {
        b.iter(|| {
            let envelope = ktav::ErrorEnvelope::from_error(black_box(&err), black_box(&bad));
            black_box(envelope)
        })
    });

    let envelope = ktav::ErrorEnvelope::from_error(&err, &bad);
    group.bench_function("to_json_medium_50k", |b| {
        b.iter(|| {
            let json = black_box(&envelope).to_json();
            black_box(json)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_format_str, bench_error_envelope);
criterion_main!(benches);
