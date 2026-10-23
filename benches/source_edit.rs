//! Короткий baseline/candidate-прогон; внешний лимит процесса — 15 секунд.
//! Round 3: text-path и segments-путь меряются раздельно; segments
//! готовятся при построении Case, вне таймера.

use std::fmt::Write;
use std::hint::black_box;
use std::time::{Duration, Instant};

use ktav::edit::{edit_value, edit_value_segments, EditError};
use ktav::Value;

// 25 сценариев x (SAMPLE + WARMUP + validate) — с запасом внутри
// SUITE_LIMIT и внешнего cap 15s на процесс.
const SAMPLE: Duration = Duration::from_millis(150);
const WARMUP: Duration = Duration::from_millis(20);
const SUITE_LIMIT: Duration = Duration::from_secs(10);

struct Case {
    name: &'static str,
    source: String,
    path: String,
    replacement: Value,
    expected: Option<String>,
    // Готовые literal-сегменты; None только если кейс не поддерживает
    // segments-путь. Для controls выводится из path через split('.') —
    // у них путь без escapes.
    segments: Option<Vec<String>>,
    // Новые round-3 кейсы: segments-путь тоже измеряется.
    time_segments: bool,
}

impl Case {
    fn run_text(&self) -> Result<String, EditError> {
        edit_value(
            black_box(&self.source),
            black_box(&self.path),
            black_box(&self.replacement),
        )
    }

    fn run_segments(&self) -> Result<String, EditError> {
        edit_value_segments(
            black_box(&self.source),
            black_box(self.segments.as_deref().unwrap()),
            black_box(&self.replacement),
        )
    }

    // Вне таймера: оба результата проверяются против expected и друг друга.
    fn validate(&self) -> (usize, usize) {
        match (&self.expected, self.run_text()) {
            (Some(expected), Ok(actual)) => {
                assert_eq!(&actual, expected, "{}: changed output", self.name);
                if self.segments.is_some() {
                    let by_segments = self.run_segments().unwrap();
                    assert_eq!(by_segments, actual, "{}: changed segment API", self.name);
                }
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

/// Controls (rounds 1-2): только text-путь в таймере.
fn control_cases() -> Vec<Case> {
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
            path: small_path.clone(),
            replacement: Value::Integer("1".into()),
            expected: Some(expected),
            segments: Some(small_path.split('.').map(str::to_owned).collect()),
            time_segments: false,
        },
        Case {
            name: "noop_500k",
            source: large.clone(),
            path: large_path.clone(),
            replacement: Value::String("old".into()),
            expected: Some(large.clone()),
            segments: Some(large_path.split('.').map(str::to_owned).collect()),
            time_segments: false,
        },
        Case {
            name: "missing_500k",
            source: large,
            path: "absent".into(),
            replacement: Value::Null,
            expected: None,
            segments: Some(vec!["absent".into()]),
            time_segments: false,
        },
        Case {
            name: "shrink_500k",
            source: format!("target:: {}\n", "x".repeat(500 * 1024)),
            path: "target".into(),
            replacement: Value::Integer("1".into()),
            expected: Some("target: 1\n".into()),
            segments: Some(vec!["target".into()]),
            time_segments: false,
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
            segments: Some(vec!["target".into(), "0".into()]),
            time_segments: false,
        });
    }
    let inline_source = format!("outer: [0{}]\n", ", 2".repeat(17 * 1024));
    cases.push(Case {
        name: "inline_source_50k",
        expected: Some(inline_source.replacen("[0", "[1", 1)),
        source: inline_source,
        path: "outer.0".into(),
        replacement: Value::Integer("1".into()),
        segments: Some(vec!["outer".into(), "0".into()]),
        time_segments: false,
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
        segments: Some(vec!["target".into()]),
        time_segments: false,
    });
    let payload = "x".repeat(500 * 1024);
    cases.push(Case {
        name: "grow_string_500k",
        source: "target: old\n".into(),
        path: "target".into(),
        expected: Some(format!("target: {payload}\n")),
        replacement: Value::String(payload.into()),
        segments: Some(vec!["target".into()]),
        time_segments: false,
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
        segments: Some(vec!["target".into()]),
        time_segments: false,
    });
    cases
}

/// Round-3: компактные целевые кейсы, каждый меряется в обоих путях.
fn targeted_cases() -> Vec<Case> {
    let mut cases = Vec::new();

    // Маленький документ.
    cases.push(Case {
        name: "tiny_doc",
        source: "alpha: old\nbeta: 2\ngamma: 3\n".into(),
        path: "beta".into(),
        replacement: Value::Integer("42".into()),
        expected: Some("alpha: old\nbeta: 42\ngamma: 3\n".into()),
        segments: Some(vec!["beta".into()]),
        time_segments: true,
    });

    // Глубокий dotted-путь (24 сегмента), text и segments.
    let deep = (0..24).map(|i| format!("k{i}")).collect::<Vec<_>>();
    let deep_line = format!("{}: old\n", deep.join("."));
    cases.push(Case {
        name: "deep_path_d24",
        expected: Some(deep_line.replace(": old", ": 1")),
        source: deep_line,
        path: deep.join("."),
        replacement: Value::Integer("1".into()),
        segments: Some(deep),
        time_segments: true,
    });

    // Escaped-ключ: literal `a.b\c`. Внутри quoted-ключа точка не
    // требует escape (регресс quoted_keys: quoted-сегмент opaque),
    // backslash — `\\`; в bare text-пути точка — сепаратор → `\.`.
    let escaped_key_line = "\"a.b\\\\c\": old\nplain: 2\n";
    cases.push(Case {
        name: "escaped_path",
        expected: Some("\"a.b\\\\c\": 1\nplain: 2\n".into()),
        source: escaped_key_line.into(),
        path: "a\\.b\\\\c".into(),
        replacement: Value::Integer("1".into()),
        segments: Some(vec!["a.b\\c".into()]),
        time_segments: true,
    });

    // Unicode-ключ.
    cases.push(Case {
        name: "unicode_key",
        source: "ключ: old\nтег✓: 2\n".into(),
        path: "ключ".into(),
        replacement: Value::Integer("1".into()),
        expected: Some("ключ: 1\nтег✓: 2\n".into()),
        segments: Some(vec!["ключ".into()]),
        time_segments: true,
    });

    // Рост с переходом маркера: plain-значение → raw `::` (тело с '['
    // требует raw-маркера).
    let raw_payload = format!("[{}", "x".repeat(4 * 1024));
    cases.push(Case {
        name: "raw_transition_grow",
        source: "target: old\n".into(),
        path: "target".into(),
        replacement: Value::String(raw_payload.clone().into()),
        expected: Some(format!("target:: {raw_payload}\n")),
        segments: Some(vec!["target".into()]),
        time_segments: true,
    });

    // Multiline-замена 128 KiB: verbatim-блок `(( ... ))`; измеряет
    // копирование тела. Хвостовой LF замены даёт пустую строку перед
    // `))`; expected — независимый hand-derived шаблон.
    let multiline_body = "line one\nline two\n".repeat(7283); // ~128 KiB
    cases.push(Case {
        name: "multiline_repl_lf",
        source: "target: old\nnext: 0\n".into(),
        path: "target".into(),
        replacement: Value::String(multiline_body.clone().into()),
        expected: Some(format!("target: ((\n{multiline_body}\n))\nnext: 0\n")),
        segments: Some(vec!["target".into()]),
        time_segments: true,
    });

    // То же на CRLF-документе: EOL тела следует EOL документа.
    let crlf_body = multiline_body.replace('\n', "\r\n");
    cases.push(Case {
        name: "multiline_repl_crlf",
        source: "target: old\r\nnext: 0\r\n".into(),
        path: "target".into(),
        replacement: Value::String(multiline_body.into()),
        expected: Some(format!("target: ((\r\n{crlf_body}\r\n))\r\nnext: 0\r\n")),
        segments: Some(vec!["target".into()]),
        time_segments: true,
    });

    cases
}

fn cases() -> Vec<Case> {
    let mut cases = control_cases();
    cases.extend(targeted_cases());
    cases
}

fn bench_path(
    label: &str,
    case_name: &str,
    suffix: &str,
    len: usize,
    capacity: usize,
    run: impl Fn() -> Result<String, EditError>,
) {
    let warmup = Instant::now();
    while warmup.elapsed() < WARMUP {
        drop(black_box(run()));
    }
    let started = Instant::now();
    let mut iterations = 0u64;
    while started.elapsed() < SAMPLE {
        drop(black_box(run()));
        iterations += 1;
    }
    let elapsed = started.elapsed().as_nanos();
    println!(
        "{label},{case_name}_{suffix},{iterations},{elapsed},{:.1},{len},{capacity}",
        elapsed as f64 / iterations as f64,
    );
}

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "unknown".into());
    assert!(matches!(label.as_str(), "baseline" | "candidate"));
    let suite_start = Instant::now();
    let cases = cases();
    println!("label,case,iterations,elapsed_ns,ns_per_op,output_len,output_capacity");
    for case in &cases {
        assert!(
            suite_start.elapsed() < SUITE_LIMIT,
            "benchmark suite deadline"
        );
        let (len, capacity) = case.validate();
        bench_path(&label, case.name, "text", len, capacity, || case.run_text());
        if case.time_segments {
            bench_path(&label, case.name, "segments", len, capacity, || {
                case.run_segments()
            });
        }
    }
}
