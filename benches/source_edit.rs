//! Короткий baseline/candidate-прогон; внешний лимит процесса — 15 секунд.

use std::fmt::Write;
use std::hint::black_box;
use std::time::{Duration, Instant};

use ktav::edit::{edit_value, edit_value_segments, EditError};
use ktav::Value;

const SAMPLE: Duration = Duration::from_millis(300);
const WARMUP: Duration = Duration::from_millis(20);
const SUITE_LIMIT: Duration = Duration::from_secs(10);

struct Case {
    name: &'static str,
    source: String,
    path: String,
    replacement: Value,
    expected: Option<String>,
}

impl Case {
    fn run(&self) -> Result<String, EditError> {
        edit_value(
            black_box(&self.source),
            black_box(&self.path),
            black_box(&self.replacement),
        )
    }

    fn validate(&self) -> (usize, usize) {
        match (&self.expected, self.run()) {
            (Some(expected), Ok(actual)) => {
                assert_eq!(&actual, expected, "{}: changed output", self.name);
                // Оба публичных варианта пути обязаны дать одинаковый результат.
                let segments: Vec<&str> = self.path.split('.').collect();
                let by_segments =
                    edit_value_segments(&self.source, &segments, &self.replacement).unwrap();
                assert_eq!(by_segments, actual, "{}: changed segment API", self.name);
                (actual.len(), actual.capacity())
            }
            (None, Err(EditError::NotFound)) => (0, 0),
            (_, outcome) => panic!("{}: unexpected outcome: {outcome:?}", self.name),
        }
    }
}

fn flat_document(bytes: usize) -> (String, String) {
    let mut text = String::with_capacity(bytes + 64);
    let mut count = 0;
    while text.len() < bytes {
        writeln!(text, "key{count}: old").unwrap();
        count += 1;
    }
    (text, format!("key{}", count / 2))
}

fn cases() -> Vec<Case> {
    let (small, small_path) = flat_document(50 * 1024);
    let (large, large_path) = flat_document(500 * 1024);
    let expected = small.replacen(
        &format!("{small_path}: old\n"),
        &format!("{small_path}: 1\n"),
        1,
    );
    let mut cases = vec![
        Case {
            name: "scalar_50k",
            source: small,
            path: small_path,
            replacement: Value::Integer("1".into()),
            expected: Some(expected),
        },
        Case {
            name: "noop_500k",
            source: large.clone(),
            path: large_path,
            replacement: Value::String("old".into()),
            expected: Some(large.clone()),
        },
        Case {
            name: "missing_500k",
            source: large,
            path: "absent".into(),
            replacement: Value::Null,
            expected: None,
        },
        Case {
            name: "shrink_500k",
            source: format!("target:: {}\n", "x".repeat(500 * 1024)),
            path: "target".into(),
            replacement: Value::Integer("1".into()),
            expected: Some("target: 1\n".into()),
        },
    ];
    for (depth, name) in [
        (1, "inline_d1_8k"),
        (16, "inline_d16_8k"),
        (64, "inline_d64_8k"),
    ] {
        let payload = "x".repeat(8 * 1024);
        let expected = format!(
            "target: [{}{}{}]\n",
            "[".repeat(depth),
            payload,
            "]".repeat(depth)
        );
        let mut replacement = Value::String(payload.into());
        for _ in 0..depth {
            replacement = Value::Array(vec![replacement]);
        }
        cases.push(Case {
            name,
            source: "target: [old]\n".into(),
            path: "target.0".into(),
            replacement,
            expected: Some(expected),
        });
    }
    let inline_source = format!("outer: [0{}]\n", ", 2".repeat(17 * 1024));
    cases.push(Case {
        name: "inline_source_50k",
        expected: Some(inline_source.replacen("[0", "[1", 1)),
        source: inline_source,
        path: "outer.0".into(),
        replacement: Value::Integer("1".into()),
    });
    let multiline_source = format!(
        "blob: (\n{})\ntarget: 0\n",
        "  body line\n".repeat(47 * 1024)
    );
    cases.push(Case {
        name: "multiline_source_500k",
        expected: Some(multiline_source.replacen("target: 0", "target: 1", 1)),
        source: multiline_source,
        path: "target".into(),
        replacement: Value::Integer("1".into()),
    });
    let payload = "x".repeat(500 * 1024);
    cases.push(Case {
        name: "grow_string_500k",
        source: "target: old\n".into(),
        path: "target".into(),
        expected: Some(format!("target: {payload}\n")),
        replacement: Value::String(payload.into()),
    });
    let payload = "x".repeat(128 * 1024);
    let mut object = ktav::ObjectMap::default();
    object.insert("payload".into(), Value::String(payload.clone().into()));
    cases.push(Case {
        name: "compound_block_128k",
        source: "target: old\n".into(),
        path: "target".into(),
        expected: Some(format!("target: {{\n    payload: {payload}\n}}\n")),
        replacement: Value::Object(object),
    });
    cases
}

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "unknown".into());
    assert!(matches!(label.as_str(), "baseline" | "candidate"));
    let suite_start = Instant::now();
    let cases = cases();
    println!("label,case,iterations,elapsed_ns,ns_per_op,output_len,output_capacity");
    for case in cases {
        assert!(
            suite_start.elapsed() < SUITE_LIMIT,
            "benchmark suite deadline"
        );
        let (len, capacity) = case.validate();
        let warmup = Instant::now();
        while warmup.elapsed() < WARMUP {
            drop(black_box(case.run()));
        }
        let started = Instant::now();
        let mut iterations = 0u64;
        while started.elapsed() < SAMPLE {
            drop(black_box(case.run()));
            iterations += 1;
        }
        let elapsed = started.elapsed().as_nanos();
        println!(
            "{label},{},{iterations},{elapsed},{:.1},{len},{capacity}",
            case.name,
            elapsed as f64 / iterations as f64,
        );
    }
}
