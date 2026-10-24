//! Парный прогон; общий внешний caps 15 охватывает все процессы.
use std::hint::black_box;
use std::time::{Duration, Instant};

use ktav::{Error, ErrorEnvelope, ErrorKind, Span};
use serde_json::Value as Json;

const SAMPLE: Duration = Duration::from_millis(50);
const WARMUP: Duration = Duration::from_millis(5);

enum Work {
    Line {
        source: String,
        error: Error,
        line: u32,
    },
    Json {
        envelope: ErrorEnvelope,
        expected: String,
    },
    Parse {
        source: String,
        expected: Json,
    },
}

struct Case {
    name: &'static str,
    work: Work,
}

impl Case {
    fn validate(&self) {
        match &self.work {
            Work::Line {
                source,
                error,
                line,
            } => {
                let envelope = ErrorEnvelope::from_error(error, source);
                assert_eq!(envelope.line, Some(*line));
                assert_eq!(envelope.line_text.as_deref(), Some(": bad"));
                assert_eq!(envelope.span, error.span());
            }
            Work::Json { envelope, expected } => assert_eq!(&envelope.to_json(), expected),
            Work::Parse { source, expected } => {
                assert_eq!(&ktav::from_str::<Json>(source).unwrap(), expected);
            }
        }
    }

    fn run(&self) {
        match &self.work {
            Work::Line { source, error, .. } => {
                drop(black_box(ErrorEnvelope::from_error(
                    black_box(error),
                    black_box(source),
                )));
            }
            Work::Json { envelope, .. } => {
                drop(black_box(black_box(envelope).to_json()));
            }
            Work::Parse { source, .. } => {
                drop(black_box(
                    ktav::from_str::<Json>(black_box(source)).unwrap(),
                ));
            }
        }
    }
}

fn line_case(name: &'static str, tail: bool) -> Case {
    let mut prefix = String::from("anchor: ok\n");
    if tail {
        prefix.push_str(&"## note\n".repeat(200_000));
    }
    let start = u32::try_from(prefix.len()).unwrap();
    let line = if tail { 200_002 } else { 2 };
    let source = format!("{prefix}: bad\n");
    let error = Error::Structured(ErrorKind::EmptyKey {
        line,
        span: Span::new(start, start + 1),
    });
    Case {
        name,
        work: Work::Line {
            source,
            error,
            line,
        },
    }
}

fn json_case(name: &'static str, payload: String) -> Case {
    let quoted = serde_json::to_string(&payload).unwrap();
    let expected = format!(
        "{{\"error\":\"Message\",\"reason\":null,\"line\":null,\"line_text\":null,\"span\":null,\"path\":null,\"body\":{quoted},\"canonical\":null,\"spec_section\":null,\"message\":{quoted}}}"
    );
    let envelope = ErrorEnvelope::from_error(&Error::Message(payload), "");
    Case {
        name,
        work: Work::Json { envelope, expected },
    }
}

fn parse_case(name: &'static str, reopen: bool) -> Case {
    let count = 2048;
    let mut source = String::from("rows: [\n");
    for _ in 0..count {
        source.push_str("[1]\n");
    }
    source.push_str("]\n");
    if reopen {
        source.push_str("a: {x: 1}\nb: 0\na.y: 2\n");
    } else {
        source.push_str("a: {x: 1, y: 2}\nb: 0\n");
    }
    let expected = serde_json::json!({"rows": vec![vec![1]; count], "a": {"x": 1, "y": 2}, "b": 0});
    Case {
        name,
        work: Work::Parse { source, expected },
    }
}

fn main() {
    let label = std::env::args().nth(1).unwrap_or_default();
    assert!(matches!(label.as_str(), "baseline" | "candidate"));
    let cases = [
        line_case("line_tail_1600k", true),
        line_case("line_head_control", false),
        json_case("json_plain_16k", "ordinary text 123".repeat(1024)),
        json_case("json_unicode_16k", "שלום café 中文 😀".repeat(512)),
        json_case("json_escaped_control", "\"\\\n\r\t\u{0001}".repeat(2048)),
        parse_case("one_reopen_tiny_arrays", true),
        parse_case("no_reopen_control", false),
    ];
    println!("label,case,iterations,elapsed_ns,ns_per_op");
    for case in &cases {
        case.validate();
        let warmup = Instant::now();
        while warmup.elapsed() < WARMUP {
            case.run();
        }
        let start = Instant::now();
        let mut iterations = 0u64;
        while start.elapsed() < SAMPLE {
            case.run();
            iterations += 1;
        }
        let elapsed = start.elapsed().as_nanos();
        println!(
            "{label},{},{iterations},{elapsed},{:.1}",
            case.name,
            elapsed as f64 / iterations as f64
        );
    }
}
